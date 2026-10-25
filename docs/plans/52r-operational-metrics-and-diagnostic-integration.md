---
title: "Plan 52r: Operational metrics and diagnostic integration"
status: done
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, diagnostics, performance, preparation, validation, environment, libraries]
---

# Plan 52r: Operational metrics and diagnostic integration

## Target and scope

**Proposed:** answer operational population, distribution, preparation/reuse, database and engineering
questions through owner-issued metrics and correlated native references. Ordinary metrics remain
independent of selected trace retention. Develop the [review's O05–O09, O14 and O19](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md#O05),
with metric-semantics contributions to O10/O20. This companion owns TEL09–TEL12's design and
acceptance; the [coordinator](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-opportunity-dispositions)
owns all package progress and opportunity dispositions. There is no second completion ledger here.

The maintainer accepted review RC01/RC02 for planning on 2026-10-24; their full decisions and
routes live at [the coordinator](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-rule-decisions).
The selected authoring target is native Collector plus OpenObserve OSS and one exact structured
FastMCP interface, with no Docker requirement. The original planning pass did not authorize
service installation or instrumentation. The subsequent maintainer-authorized implementation
and its bounded acceptance are recorded in the Outcome below. [52p](52p-shared-telemetry-and-native-propagation.md)
owns SDK/configuration/identity/propagation; [52q](52q-local-telemetry-pipeline-and-health.md)
owns collection/deployment/health/retention; [52s](52s-telemetry-queries-and-agent-investigations.md)
owns typed backend reads, exemplar presentation, profiling/offline/watches. Existing 52g/52h own
consumer journeys and assembled acceptance. [52f F02](52f-evidence-analysis-and-trace-correlation.md#F02)
retains the SDK request-task/header seam and actual Rust→database→Surrealism continuity/I11.

## Foundation assessment

The inspected authoring baseline is main `6e93c07c1` plus concurrent working-tree changes. The
unrelated Plan54 build/cache work is preserved and supplies future build-observation consumers,
not an implementation responsibility here. Core3.4, Heuristics1.0, Test Assessment1.2 and relevant
Process Simulator1.5 constraints guide this focused assessment; no broad audit is initiated.

Native observation is already richer than sampled spans. `pse-diagnostics::observation` retains
coverage, absolute-counter baselines/generations/resets, additive intervals, unavailable/nonfinite
values and producer/operation relationships. `pse-runtime::math::{counted,counted_sync}` scopes
preparation counts to enclosing operations; service totals and nested operation counts have
different populations. `PreparationCounts` distinguishes structural views, observation programs,
value rebuilds and shared products. Keep those meanings instead of replacing them with one
cache-hit ratio or subtracting concurrent service snapshots to invent per-request work.

[Simulation diagnostics](../dev/simulation-diagnostics.md#preparation-work-and-waiting) already
explains compiler/modeling, math/native preparation, worker reuse and publication facts. Native
join, compiler authority wait and actual execution can overlap. Its current counter/phase readers
remain useful when export is disabled, sampling drops spans or the backend is unavailable.
Metrics must not erase their source-clock, coverage and retained-interval limits.

`scripts/build_observation.py` consumes actual Cargo JSON and observed descendant process identity;
Cargo freshness is explicitly different from sccache hits. `validation_result.py` consumes exact
assessment scope, checks, provenance, baseline failures and unattempted populations. Existing
activity/pressure readers retain process/cgroup incarnation and unavailable fields. These owners
are the foundations to project, not evidence for another run database or telemetry outcome ledger.

The review's Python1.45.1 SDK inquiry tested attribute filtering and a small histogram, recording/
sampling differences and a refusing exporter. Its backend inquiry used synthetic OTLP JSON and
OpenObserve1.1.2 without a Collector or application SDK network export. Retain those observations
at their original scope; actual native metrics, SDK/Collector ingress, restart/reset, backend
queries and FastMCP consumers require new qualification during authorized execution.

AP-04 requires a metric's population, unit and reset semantics to govern instrumentation and reads.
AP-06 supports small independent population tests without launching unrelated science/backend work.
AP-07/H4/H7/H17/H26 favor pre-aggregation at owners and SDK Views, compact bounded dimensions and
selected backend queries. Exporting every native checkpoint, model identity or callback event would
multiply retained state without supplying a needed ordinary operational answer.

## Population and physical design

A metric definition belongs with its performing owner and declares event/state population, unit,
instrument/aggregation, allowed dimensions and reset/start scope. Query adapters consume that
meaning alongside per-signal backend normalization; they do not infer it from a name. No universal
telemetry vocabulary, registry framework or query language is required to express these contracts.
A small shared declaration is appropriate only where actual producers/consumers share its meaning.

Use direct instruments when the owner performs the counted event or observes a duration. Use an
observable instrument when collection can cheaply read an already-owned cumulative total or state.
Both routes consume source facts; neither polls native callbacks or executes new scientific work.
The following distinctions guide TEL09 rather than prescribing one metric for every row:

| Owned population | Appropriate observation | Interpretation constraint |
| --- | --- | --- |
| Actual admission/dispatch/completion transitions | `Counter` increments or owned cumulative total via `ObservableCounter` | Declare attempts versus accepted work versus terminal outcomes separately |
| Current queued/running/reserved/draining state | `UpDownCounter`, `ObservableUpDownCounter` or observable gauge according to owner semantics | Caller completion does not mean physical resource release |
| Completed wait/preparation/execution interval | `Histogram` recorded once by the duration owner | Name inclusive wall time; active CPU requires different source evidence |
| Retained cumulative compiler/native totals | Observable counter over the actual owner total | Reading the same total again must not add it again |
| Process/cgroup/engine state | Observable gauge or selected native scrape | RSS, managed bytes, reservations and host pressure are distinct |

A polled total must never be repeatedly `add`ed to a synchronous counter. A cumulative reset starts
another declared generation/incarnation; absent baseline or missing collection is not zero work.
Do not reconstruct an exact histogram from aggregate count/sum/current values or treat a retained
native point sample as a complete duration population. A late observer cannot recover pre-activation
work unless its source actually supplies the cumulative total and start semantics.

Metric labels use bounded declared classes such as operation family, stage, provider/package/suite,
mode and owned outcome category. Exact invocation, test/case/model IDs, filesystem paths, trace
and span IDs stay in spans/logs/evidence references. Resource promotion is deliberate: service/
process incarnation must remain queryable without blindly multiplying every metric by all resource
attributes. Parent/nested scope totals and shared producers/waiters are not additive disjoint work.

Start with explicit cumulative export and explicit histogram buckets appropriate to the actual
units/ranges. Keep histogram sample count/sum/bounds, collection interval, source start/reset and
coverage accessible to typed consumers. Estimated quantiles interpolate bucket populations;
PromQL `increase` extrapolates. Neither replaces an exact source count or raw-observation percentile.
Exponential histograms remain eligible when an actual SDK→Collector→backend consumer qualifies
their representation; they are not assumed equivalent to this explicit-histogram baseline.

## Work packages

<a id="TEL09"></a>
### TEL09 — Population definitions, instruments and SDK Views

At selected native/host owners, declare and instrument the actual operational populations required
by admission, preparation, execution, drain and diagnostic questions. Use Rust API `Meter` counter,
up/down, histogram and observable builders; Python `Meter.create_counter`,
`create_up_down_counter`, `create_histogram` and `create_observable_*` supply corresponding roles.
Reuse existing authoritative counts rather than counting wrappers and requests as scientific work.

Compose aggregation policy with TEL01's process providers. Rust `SdkMeterProvider::builder()` and
`PeriodicReader` supply collection/export; Python `MeterProvider` and
`PeriodicExportingMetricReader` supply the host equivalent. Configure Python
`View(attribute_keys=..., aggregation=ExplicitBucketHistogramAggregation(...))` before collection
and use `DropAggregation` for unselected streams when useful. Apply Views early enough to avoid
unnecessary aggregation state, not only backend-side label deletion after that state exists.

Python1.45.1's inspected `View` has no `cardinality_limit`; the review found no corresponding
metrics overflow implementation in the inspected source. Bound producer dimensions explicitly and
test actual selected series. Do not promise automatic overflow protection from the overview.
Rust stream `with_cardinality_limit`/overflow behavior is different. Its allowed-attribute Views
API (`with_allowed_attribute_keys`) is feature-gated by `spec_unstable_metrics_views`; the stable
baseline bounds attributes at producers. Select that experimental feature only if its concrete
reduction benefit warrants qualification of the exact closure. Experimental bound instruments are
an optimization candidate for stable high-frequency dimensions, never a reason for scalar tracing.

Observable callbacks read cheap published state without scientific locks held for network work,
service startup or mutable preparation. Expensive readers publish their own snapshot at their owner
rather than moving IO into periodic SDK collection. Preserve unavailable/stale source evidence;
do not emit a fabricated current gauge value when the source is gone. Reader/provider shutdown
and exporter failures consume TEL02/TEL03 lifecycle rather than adding another flush owner.

Acceptance uses independently enumerated transitions and observations: repeated reads of a fixed
cumulative total, generation reset, missing first baseline, overlapping nested operations, one
shared producer/two waiters, refused admission, cancellation/drain and an absent source. Assert
count/sum/buckets and allowed series before export and through the selected typed metric reader.
Include values near bucket boundaries and a label-variation case that would otherwise expand the
population. Delete superseded ad hoc metric aggregators only after their actual consumers migrate;
preserve native diagnostic intervals/checkpoints whose contracts are not replaced.

<a id="TEL10"></a>
### TEL10 — Preparation/reuse and database metrics at native owners

The authorized 52p/52q refinement supplies a bounded prerequisite: actual registered-executor and
selected-future snapshots shared by native checkpoints and OTel callbacks. Its outcome remains
with 52p; general metric populations/views, scientific preparation/cache counts and the database
families below remain this package's work. Tokio queue/poll measurements neither establish those
scientific populations nor supply equivalent Rayon scheduler measurements.

Project the preparation populations from actual modeling/compiler/math/engine owners. Distinguish
cold structural construction, valid retained reuse, value rebind/rebuild, artifact/compiler misses,
shared-flight production and waiters. Source-issued artifact/build/dependency references belong in
selected detail. Telemetry neither decides cache validity nor invents invalidation causes from a
miss. Salsa execution events do not establish a miss cause or paired wait duration on their own.

Reuse `PreparationCounts`, performing-owner phases and existing compiler/native counters described
in the diagnostics guide. Scope each export as owner/service or operation population deliberately;
nested operation summaries can include the same underlying work. Count production once in the
operational metric stream and attach waiter associations separately. Do not add process-global
snapshot deltas to operation-local counts or equate fewer preparations with fewer fresh solves.

Cover actual value-only and structural edits, concurrent study/fitting demand, preparation
cancellation and existing native worker reuse where these supported owners supply meaningful facts.
Retain fresh domain/value/physical checks and result verification. A cached symbolic layout, reused
worker or compiled code image does not restore a previous scientific result, native IVP history,
seed/clock authority or a settled execution permission.

For database questions, combine selected application/SDK admission/session/transfer/decode metrics
with the exact custom server's exposed query/storage/process families and current `storage_health`
readers. Inspect selected SurrealDB3.3.4 build/revision, enabled features and endpoint coverage before
advertising families; enterprise audit/slow-query capabilities in documentation are not community
build evidence. F02 supplies actual causal transport continuity; missing F02 continuity does not
prevent truthful metrics, but those metrics must not pretend to prove a parent chain.

TEL06 selects one acquisition path per database metric family: supported OTLP or `/metrics` scrape,
with TEL09 population meaning retained. Avoid double counting the same family through both paths.
Query-template/class identity replaces full SQL/bindings by default. Engine-managed bytes, process
RSS, cgroup usage/pressure and reserved scientific capacity remain separate. `EXPLAIN`, maintenance
and profile capture stay explicit actions; collecting a metric must not execute them implicitly.

Acceptance follows real edit/re-solve and shared preparation with independently expected producer/
waiter counts and source references, including valid reuse and required rebuild. Add a selected
real database operation with application/server metrics and service incarnation, restart, absent
family and unavailable-read cases. Compare exact counts with native readers where their populations
match; preserve scope differences otherwise. TEL13/TEL14 and 52g consume this behavior rather than
requiring agents to interpret raw metric names. No speed or cache-effectiveness claim follows.

<a id="TEL11"></a>
### TEL11 — Build, test, qualification and host observation projections

Extend existing command owners to project actual build/discovery/setup/execution/publication/
teardown stages, source/build/mode references and failure classes. `build_observation.py`,
`validation.py`/`validation_result.py`, exact Rust test readers and invocation/result evidence remain
the sources. Preserve the distinction between a Cargo fresh artifact, compiler-cache hit/miss,
build-script execution, lock wait, test collection and actual test execution.

Coordinate with Plan54 at its public observation boundary rather than modifying compiler/cache
policy or interpreting global cache-server deltas as one invocation's work. Consumers retain the
observation scope, build/environment identity, missing stages and uncertainty actually supplied.
A diagnostic `/proc` stage observation is sampled evidence, not a complete compiler timeline.
Do not change the ordinary unbounded build selection or scientific/reference admission to gather
telemetry; lifetime collection belongs outside individual workload allocation under 52q.

Aggregate useful repeated questions by bounded package, suite, stage, mode and failure class.
Exact test names, source paths and individual cases remain in structured logs/traces and authentic
assessment/evidence readers. Declared, selected, executed, passed/failed, cancelled and unattempted
populations stay distinct. Retries and reused compatible qualification evidence retain their source
conditions; telemetry cannot relabel them as newly executed passes or change zero failure baselines.

Use existing process/service/cgroup and `pressure_observation.py` source identity and interval
semantics for selected host signals. Keep boot/process/cgroup incarnation, observed time and
unavailable/partial fields. Never combine before/after resource deltas across changed incarnations
or confuse allocation allowance with measured RSS/native bytes. TEL06 owns actual collection;
TEL13 owns normalized query access; this package owns the engineering interpretation/migration.

Acceptance exercises actual targeted build/test consumers with a failed check and unattempted scope,
then follows authentic source evidence through the canonical interface. Include compile versus test
failure, setup versus teardown, interruption and stale/restarted process observations. Verify a
fixture's independent population expectations locally, then a real invocation without introducing
failures into the maintained test suite. Existing recorded failing evidence can serve the failure
consumer where its provenance fits. 52g's engineering journey and 52h qualify actual assembled reads;
sampled telemetry never replaces exact assessment records or F01's comparison semantics.

<a id="TEL12"></a>
### TEL12 — Sampling, exemplar semantics and metric qualification

Keep owner metrics independent of head trace sampling and retained trace availability. Parent-based
sampling preserves a coherent propagated decision; explicit finite full capture supplies selected
dense detail. Head-dropped spans cannot be recovered by later filtering. Long-running native work
makes default tail sampling an unsuitable silent assumption: it needs all related spans at one
Collector, retained decision state and a policy for traces longer than its window. Start without
long-window tail sampling; reconsider only for a real slow/error investigation requiring it.

Python `AlwaysRecordSampler` can serve a genuine local processor consumer, but retains recording
cost and RECORD_ONLY does not supply ordinary trace export or trace-based exemplars. Prefer direct
owner metrics to a custom processor counting every request/provider/wrapper span. Collector
`spanmetrics` is an alpha candidate only for explicitly selected transport RED populations, before
sampling and with correct span kinds. It does not replace native completion/scientific metrics.
Rust `MetricsLayer` remains eligible for already-structured metric events; new owner metrics use
direct Meter instruments where their population is clearer.

Python trace-based exemplars may link to retained sampled executions. Rust0.33.1 lacks the same
native exemplar capability; expose that support difference. Never insert unique trace IDs into
metric labels to simulate parity. OpenObserve's inquiry retained exemplar pointers on all buckets,
including one below the exemplar value; its fixture exemplar times preceded the referenced trace.
Initially present pointers as related examples. Strong contributing-execution/bucket attribution
requires matching instrument/population, measurement time, source interval, trace retention and
bucket membership. Missing/expired/unsampled exemplars are honest query outcomes under TEL14.

Qualify cumulative points and reset provenance independently of a nonempty PromQL result. The
inquiry's SQL retained point start timestamps, while historical PromQL labels exposed the latest
`start_time`; that label is not historical point provenance. TEL13 must retain a route to original
point/reset facts or explicitly decline the stronger query. Do not use extrapolated `increase` as
an exact native operation count or bucket quantiles as raw-sample percentiles.

Acceptance changes sampling while holding independently scripted owner work fixed; operational
counts stay population-correct while spans/exemplars differ as predicted. Test reset/restart,
histogram boundaries and partial interval, exemplar with missing trace, loose attribution and the
absence of Rust parity. Combine a tiny independent expected count/sum/bucket/reset oracle with
actual SDK→Collector→OpenObserve ingress and FastMCP reads; neither synthetic OTLP nor a shared
normalizer on both actual/expected sides settles the complete contract. 52h owns O20's assembled
campaign; this package supplies explicit metric/sampling assertions and source obligations.

## Dependencies, execution fit and verification

TEL01/TEL02/TEL03 supply implemented identity/provider and lifecycle seams; TEL09 can first develop
against local SDK readers, without requiring persistent service deployment or all owner stages.
TEL10 consumes TEL04's actual stage/lifecycle facts and existing native counters. TEL11 can migrate
engineering observations alongside Plan54's delivered observation contract without waiting for its
cache-policy conclusions. TEL12's end-to-end assertions need TEL06/TEL07 collection/health and
TEL13/TEL14 query behavior; agreeing their interfaces is not delivery evidence.

Coordinate shared edits at process bootstrap, runtime preparation, database telemetry selection,
`build_observation.py` and validation/read families. Reuse the existing provider composition and
source readers. Keep callbacks cheap and aggregate at the performing owner, filter dimensions before
SDK state grows, then query compact selected backend intervals. Combined SDK aggregation, exporter
queues, Collector queues and retained backend cardinality need consideration together; independent
per-queue bounds do not establish a workstation-wide memory bound. 52q owns pipeline capacity/
retention policy. This plan introduces no new global governor or arbitrary memory cap.

**Proposed verification:** implementation packages use touched-package `just check-package <pkg>`
and `just unit-package <pkg> <filter>`, plus focused Python/script tests through `scripts/pse-env`.
Actual Arrow-consuming correctness checks retain explicit force-validation. Local instrument/reader
fixtures require no science or backend startup; actual native/backend/canonical journeys remain
necessary for their separate claims. Scope-end integration/component/native/Python journeys and
`just hygiene`, governance/docs and applicable manual native/powerset checks are coordinated in 52h,
after all selected functional scope. `just ready` follows dependency/skill-selection changes;
the coordinator owns `just turn-end` and the shared index. The original authoring pass ran no
product tests, new services or performance probes; subsequent authorized execution is recorded below.

All commands reported during execution name mode, scope, zero failure baseline and actual result.
The original test/assessment baseline semantics remain untouched. A useful distribution or trace
does not certify scientific usability, full-plan completion or improvement. Quantitative overhead,
latency, throughput or capacity claims require representative ordinary versus selected-detail
measurements across relevant edit/re-solve, study and engineering workflows. Avoid a mandatory
benchmark matrix when making no quantitative claim; source reasoning can establish removed
structural duplication without establishing elapsed benefit.

## Concrete sources and implementation decisions

[Review S02/S03/S05–S08](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md#sources-and-concrete-implementation-leads)
retain exact release/source leads and the bounded inquiry's evidence limits. Use Rust
[SDK0.33.1 metrics](https://docs.rs/opentelemetry_sdk/0.33.1/opentelemetry_sdk/metrics/index.html),
Python [v1.45.1 Views](https://github.com/open-telemetry/opentelemetry-python/blob/v1.45.1/opentelemetry-sdk/src/opentelemetry/sdk/metrics/_internal/view.py),
the [OTel metric model](https://opentelemetry.io/docs/specs/otel/metrics/data-model/) and
[OpenObserve1.1.2 OTLP metrics](https://github.com/openobserve/openobserve/blob/v1.1.2/src/core/src/metrics/otlp.rs)
to settle consequential behavior. Authoring Context7 results corroborated attribute filtering and
observable cumulative semantics, but their main-branch examples do not establish Python1.45.1
cardinality support or exact-release Rust features.

The baseline choices are explicit cumulative export, useful explicit histogram buckets, bounded
producer dimensions, direct/observable instruments fitted to actual source ownership and selective
trace retention. Implementers choose concrete instrument names/buckets and exact source-family
selection with real consumer questions, retaining their one owner. Unsupported server families,
strong exemplar attribution and experimental Views/histograms stay visibly unsupported until
qualified; a material requirement may reopen that affected SDK/backend choice through the
coordinator. No broad investigation, alternative stack or second query path is pre-authorized.


<a id="execution-outcome"></a>
## Outcome — authorized execution, 2026-10-24

**Implemented baseline:** execution started from main `85638ebed`, preserving unrelated library
references and concurrent work. The coordinator owns integrated source/checkpoint admission.

**Implemented:** TEL09–TEL12 now project performing-owner populations through the shared providers.
Native frontier/basis single-flight acquisition, retained reuse, production and waiters remain
separate; actual native supervisor attempt/admission/dispatch/join/control facts and owner intervals
are exported without sampled spans being their counter source. Modeling value and structural edits
retain the original preparation and scientific-result checks. Canonical query statistics use the
SDK's `QueryStatisticsObserver`; builder/request, statement success/failure and timed/untimed
populations are not interchangeable. Generated native metric declarations and the host declarations
supply the query adapter's meanings. Existing native checkpoints/readers remain available.

**Implemented:** `telemetry_metrics.py` applies explicit cumulative export, bounded attribute Views
and explicit histogram buckets to authentic build, assessment and pressure owners. Exact test,
invocation, trace and model identifiers are not new metric dimensions. Pressure collection uses
published source snapshots and omits unavailable/expired fields; periodic SDK callbacks do not run
host queries or science. Build freshness, process samples, assessment stages and exact qualification
results retain their original owners and units.

**Implemented:** the custom SurrealDB3.3.4/MX1.0 server exports native metrics over OTLP/gRPC to the
existing Collector ingress. Its per-launch `service.instance.id` differs from deployment/configuration
identity and is associated with the owned process/status readers. Collector0.163.2 exports its own
internal metrics through a periodic OTLP reader into that same pipeline; port8888 remains passive.
There is no duplicate database scrape. Explicit immutable configuration replacement validates before
stopping services, preserves credentials/data/seven-day retention and retains recoverable prior
configuration. Finite trace capture redirects only traces; ordinary metrics keep their selected route.

### Acceptance and limits

All results below use a zero failure baseline. Local receipt paths in this Outcome are relative to
`~/.local/state/pse-arrow/telemetry-qualification/`; explicit `/tmp` paths are separate retained local
probe receipts. These are acceptance evidence, not a second package-progress ledger.

| Evidence | Tested scope and result |
|---|---|
| `scripts.tests.telemetry_operational_acceptance`, through `scripts/pse-env -- .venv/bin/python -m`, receipt `plan52rs-operational-20261024-corrected/acceptance.json` | **Tested, passed:** actual Python1.45.1 SDK→Collector→OpenObserve1.1.2→canonical STDIO reads under both `always_on` and `always_off`; independently specified three-event counters and histogram count/sum/buckets, original collection/start clocks, process reset/incarnation and filtered dimensions. Sampling changes span availability without changing those metric populations. |
| `scripts.tests.telemetry_native_operational_acceptance`, through `scripts/pse-env --native -- .venv/bin/python -m`, receipt `plan52rs-native-20261024-final/acceptance.json` | **Tested, passed:** two real KINSOL solves, shared preparation demand, retained reuse, value edit and structural edit; exact retained native checkpoint totals compared with same-incarnation SDK/backend points. Source counts and histogram populations remain distinct; count/sum are not reconstructed duration distributions. |
| `scripts.tests.telemetry_database_acceptance --telemetry-state ~/.local/state/pse/telemetry --output <fresh-output>`, through `scripts/pse-env --native -- .venv/bin/python -m`, receipt `plan52rs-database-20261024/current/receipt.json` | **Tested, passed:** fresh owned canonical database, actual MX/query/process OTLP populations, independent server snapshots, new server incarnation after an owned restart, and20 captured server spans with a metric point delivered during finite capture. Unknown-family and stopped-provider reads remain unavailable rather than zero. `current/native/acceptance.json` independently validates actual application canonical query statistics against SDK/backend points. `current/retirement/passed.json` records all contexts/kernel scopes drained and fixture retirement. |
| `scripts.tests.telemetry_semantics_acceptance`, through `scripts/pse-env -- .venv/bin/python -m`, receipt `plan52rs-semantics-20261024-source-bounded/acceptance.json` | **Tested, passed:** original SDK cumulative collections3/6/9, explicit bucket counts/sums, actual `rate`, `increase`, p95 and fraction-below reads, plus canonical STDIO rate/exemplar consumers. The four-second estimator comparison uses interior evaluations after the second original source point and no later than the last. Rate/increase use the independently computed source slope with5% tolerance; p95 is bucket interpolation0.0865 and the fraction at0.01 is2/3, not a raw-sample percentile or exact operation count. |
| `scripts.tests.telemetry_service` and actual replacement/recovery probe `/tmp/pse-reconfigure-20261024.json` | **Tested, passed:** candidate validation refusal before effects, sealed-generation preservation, failed-activation rollback, interrupted-pointer recovery and actual installed-service replacement/recovery preserving credentials, seven-day retention and previously queryable metrics. Both probe generations were drained and their authenticated unit files removed. |
| Selected Python component `python/pse/tests/test_capability_adapters.py::test_capability_project_selection_metadata_and_retained_revision` | **Tested, passed:** `.venv/bin/python -m pytest -n0 <selected-test> --basetemp <owned-fixture>/pytest`, with explicit fresh `PSE_SURREAL_STATE`;1 passed in26.91s. `plan52rs-component-20261024/{receipt.json,component.log,retirement/passed.json}` records exit0 and complete fixture retirement. |

**Tested:** the targeted service/alert/native-reader command `scripts/pse-env -- python -m unittest
scripts.tests.test_telemetry_service scripts.tests.test_telemetry_alerts
scripts.tests.test_surreal_mcp scripts.tests.test_surreal_diagnostics
scripts.tests.test_surreal_server.SurrealSupervisorTests.test_native_metrics_keep_selected_route_during_finite_trace_capture`
passed57 tests, zero failures. Performing-owner tests `operational_metrics_shared_flight_counts_one_producer_two_waiters` and
`operational_metrics_cumulative_population_and_histogram_ignore_head_sampling`, plus canonical
transport statistics-observer tests, separately exercise owner sharing, sampling independence and
statement-statistics classifications. Their complete scope-end
command accounting belongs to the coordinator, not this supporting document.

**Tested with limits:** Python exemplar retrieval preserves the original sampled SDK trace/span IDs,
value, filtered attributes and source seconds through backend and canonical reads. Its meaning remains
`related_examples`: the observed OpenObserve bucket duplication and absent attached native target do
not establish bucket membership, complete population coverage or causal contribution. Rust0.33.1
exemplars remain explicitly unsupported. SQL point start/reset provenance is preserved separately
from PromQL's latest-start labels. No representative speed, cache-effectiveness, overhead, capacity,
crash-durability or productivity measurement is claimed.

### Mistakes corrected and deliberate boundaries

Initial host acceptance assumed a trace schema existed before sampled stimuli and failed visibility;
the corrected journey publishes real sampled stimuli before asserting sampling-dependent absence.
Initial native fixture decoding assumed JSON arrays without the native codec, a retained checkpoint
exceeded the reader's finite bound, and a requested completed solve was assumed still retained.
The driver now decodes with the native codec, retains a bounded normalized copy of the same admitted
checkpoint, and distinguishes the requested solve from the actual retained terminal target. These
failed receipts remain distinct from the passing native receipt.

The first database setup used the older1GiB fixture budget, which the current module-pool contract
correctly refused; the fixture then selected the current minimum2GiB server allocation without changing
host policy. A subsequent initializer failed because the current `xtask` binary lacked `canonical-tools`.
The exact initializer was built through `scripts.arrow_validation` with explicit force-validation;
no new Python extension was built. Both failed closed fixtures/receipts remain retained. The ordinary
scientific provider's stale/absent allocation was not repaired or adopted; fresh owned providers supplied
both database and component acceptance instead. A physical-versus-logical CPU assumption in the MCP
fixture was corrected to use the actual `host_admission.cpu_set`, without changing admission policy.

An earlier semantic run exposed rounded range-edge extrapolation in the fixture's constant-slope
oracle. The final source-bounded receipt records the original SDK clocks and compares only the actual
interior stimulus interval, rather than weakening the estimator meaning or relabeling extrapolation
as exact work. Earlier partial/failed semantic receipts remain separate evidence.

**Scope-end verification:** the coordinator's [targeted outcome](52-agent-effectiveness-and-engineering-workflows.md#52r52s-targeted-execution-outcome--2026-10-24)
records the corrected Python/generated gates, passing default/no-default and full-workspace native
Clippy, governance and documentation publication. The final concurrency review led to one
endpoint-lifetime shared HTTP pool with request-local configuration and unchanged source fences;
the overlapping-selection regression and actual SDK/backend/STDIO semantic rerun passed
(`plan52rs-semantics-20261024-pool-fixed/acceptance.json`). These results do not close F02/I11's
SDK request-task/header and actual Rust→database→Surrealism parent continuity, broader G/H
assembled journeys, or quantified benefit qualification.

The [coordinator](52-agent-effectiveness-and-engineering-workflows.md) alone owns TEL09–TEL12 progress,
opportunity/inquiry dispositions and remaining work. This supporting plan retains the design and
acceptance definitions consumed by [52g](52g-agent-and-operator-workflows.md) and
[52h](52h-qualification-and-publication.md). It remains in the working corpus until those dependencies
no longer need it and enduring meaning/reference migration is complete under ADR-0096; its retention
is not another backlog or an assertion that assembled qualification and measured benefits are done.
