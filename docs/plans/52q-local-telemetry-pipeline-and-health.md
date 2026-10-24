---
title: "Plan 52q: Local telemetry pipeline and health"
status: draft
date: 2026-10-24
adrs: []
review_sources: [docs/functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md]
doc_role: plan
doc_owner: docs/plans/52-agent-effectiveness-and-engineering-workflows.md
doc_topics: [current-work, diagnostics, environment, storage, libraries]
---

# Plan 52q: Local telemetry pipeline and health

## Purpose and boundary

**Proposed:** provide persistent native collection, retained local signal history and an
independent explanation of telemetry availability. This develops the
[OTel review's O10/O11/O18](../functional_reviews/opentelemetry-and-agent-observability-2026-10-24.md#O10)
and the pipeline part of O16/O19/O20. The
[coordinator](52-agent-effectiveness-and-engineering-workflows.md) owns package state,
[rule decisions](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-rule-decisions),
[opportunity dispositions](52-agent-effectiveness-and-engineering-workflows.md#opentelemetry-opportunity-dispositions)
and inquiries TQ01–TQ05. The operator accepted RC01/RC02 for planning on 2026-10-24;
this document does not authorize installation, activation or production edits.

Collector plus OpenObserve OSS is the selected initial topology: native binaries, loopback
ingress/query endpoints, user-level supervision and separately owned state. No Docker,
external paid service, second developer MCP or independent execution controller is selected.
Telemetry is a derived diagnostic projection. Scientific results, admission, native work,
cancellation/drain, required recording and immutable evidence retain their existing owners.

[52p](52p-shared-telemetry-and-native-propagation.md) owns process identity,
programmatic SDK bootstrap, propagation and native instrumentation; [52r](52r-operational-metrics-and-diagnostic-integration.md)
owns metric populations/views and sampling. [52s](52s-telemetry-queries-and-agent-investigations.md)
owns query adaptation and agent exposure. Existing [52f F02](52f-evidence-analysis-and-trace-correlation.md#F02)
alone owns the actual application/SDK→HTTP-header→server→Surrealism seam and I11.
This plan supplies ingress and health evidence without duplicating that inquiry.
FND00 classifies an actual new backend binding or protected-contract change and supplies its
required decision/declaration route before dependent implementation. Ordinary dependency
selection alone does not create an ADR obligation.

## Baseline and focused foundation assessment

The authoring baseline is main `6e93c07c1` plus concurrent dirty Plan54/build-tooling work.
Those changes remain owned there. Integration must respect their actual producer/configuration
contracts; telemetry delivery does not require unimplemented Plan54 completion.
Core3.4, Efficient Architecture Heuristics1.0, Process Simulator1.5 and the binding inform this
focused assessment. It is plan authoring, not a new broad audit or formal solution review.

The existing foundations are useful because they separate optional observation from execution:

| Existing owner | Judgment and planned use |
|---|---|
| [blueprint §23.1](../authoritative_design/sections/operations-and-validation.md#section-23-1) and [diagnostic guide](../dev/simulation-diagnostics.md) | Preserve process/operation identity, independent clocks, missing coverage, bounded detail and optional observation failure. Standard delivery does not replace required source acknowledgement. |
| [native service supervisor](../../scripts/surreal_server.py) and [substrate guide](../dev/surreal-substrate.md) | Reuse its ownership patterns: selected state, exact child/generation identity, lifecycle reservation and explicit stop. Do not copy its scientific worker pool or consume its allocation for lifetime telemetry. |
| [environment guide](../dev/agent-environment.md#placement-and-memory) and [build environment](../../scripts/build_environment.py) | Keep ordinary unbounded build selection and scientific/reference admission intact. Service queue/disk controls are separate from host-wide workload policy. |
| [finite telemetry capture](../../scripts/surreal_telemetry.py) | Its explicit selected-service restoration and bounded span-identity capture are real consumers. A standard receiver is attractive, but cannot replace these semantics merely by accepting OTLP. |
| [capability observations](../../scripts/capabilities/observation.py) | Existing safe stderr and request/native references work without a backend. Extend local availability rather than force cheap reads through network telemetry. |

The [saved backend inquiry](../functional_reviews/probes/opentelemetry-query-2026-10-24.json)
and [retained driver](../../scripts/tests/opentelemetry_query_inquiry.py) cover OpenObserve
OSS1.1.2, direct synthetic OTLP JSON ingest and useful SQL/PromQL retrieval. They did not run
Collector, application exporters, restart/crash recovery or retention. The
[SDK inquiry](../functional_reviews/probes/opentelemetry-sdk-2026-10-24.json) shows that processed
span counts and successful `force_flush()` can coexist with exporter rejection. Neither
inquiry establishes end-to-end durability, application continuity or loss-free delivery.

AP-01/03 favor one pipeline integration over backend policy in every tool. AP-04/05 require
actual delivery stages and source identity, rather than a single healthy boolean. AP-06 favors
small component fixtures before assembled operation. AP-07 favors standard batching/queues,
bounded retained history and selective signals over full-detail permanent collection. PS-09/10
keep numerical control and scientific result interpretation out of this service.

## Target composition and concrete mechanisms

SDKs asynchronously emit OTLP to Collector; selected signal pipelines apply mechanical policy
and export to OpenObserve. The backend indexes diagnostic history and serves exact selected
queries. Local health readers compose process-local facts, supervised service identity and
available pipeline/query observations, retaining the source and time of each observation.
Reading availability does not start services, initialize an SDK or exercise a scientific path.

Use Collector's `receivers`, `processors`, `exporters`, `extensions` and
`service.pipelines` composition. Explicit OTLP receiver endpoints admit only the selected local
producers. Standard batching and exporter `sending_queue`/`retry_on_failure` own ordinary
delivery retries; `file_storage` supplies the selected persistent exporter queue. A bounded
mechanical transform/filter may normalize names or remove sensitive data. It never interprets
a solver return or reconstructs missing operation identity. Pipeline ordering must preserve
52r's intended metric and sampling populations before any lossy reduction.

The [Collector resiliency guide](https://opentelemetry.io/docs/collector/resiliency/),
[internal telemetry guide](https://opentelemetry.io/docs/collector/internal-telemetry/) and
[v0.163.0 transform processor](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/v0.163.0/processor/transformprocessor/README.md)
are implementation leads. Review discovery found distribution0.163.2 while component contracts
were inspected at0.163.0. TEL05 must select the actual distribution/version/artifact digest and
TEL06 must verify its included components and syntax; these references are not an install pin.
Current Context7 configuration/exporterhelper documentation also points to main and cannot
establish exact-release behavior. Do not copy a moving queue API into the selected deployment.

Select OpenObserve native local mode with local disk and SQLite metadata, as exercised by the
driver's `ZO_LOCAL_MODE`, `ZO_LOCAL_MODE_STORAGE`, `ZO_META_STORE` and `ZO_DATA_DIR` settings.
Its [v1.1.2 configuration source](https://github.com/openobserve/openobserve/blob/v1.1.2/src/config/src/config.rs),
[native deployment guide](https://openobserve.ai/docs/getting-started/) and
[stream settings](https://openobserve.ai/docs/reference/api/stream/setting/)
ground installation and retention selection. Explicit local addresses, selected organization/
streams and authenticated access belong to this integration. Fixture credentials and temporary
paths from the inquiry are not a deployment configuration. Any bundled backend MCP surface is
outside the selected exposure; 52s binds ordinary reads through the existing endpoint.

Configuration has one declaration owner for each meaning: 52p owns producer semantics and
process bootstrap, 52r owns metric/sampling policy, this plan owns pipeline/service settings,
and 52s owns backend query mappings. The effective view names the declaration sources, selected
versions, enabled components, endpoint/stream selection and redacted configuration identity.
Secrets remain in owned protected material, excluded from logs, exported attributes and MCP
responses. Explain environment overrides and precedence through the existing environment guide.

## Lifecycle, resource and recovery semantics

Installation is explicit and verifies the selected artifact bytes before publishing a usable
installation. Setup selects owned state and credentials; start selects an exact configured
generation; inspect is passive; stop/restart targets that same owner. Existing native service
patterns can guide a telemetry-specific supervisor without pretending the database supervisor
owns every service. A stale PID or service name alone cannot authorize stopping a process.

Keep Collector queue storage and OpenObserve data/metadata separately identifiable beneath the
telemetry state owner. They are not `.store` scientific data, build scratch or pytest workspaces.
Removing a checkout or cleaning scientific scratch must not remove persistent telemetry state.
State replacement/cleanup names the exact owner and retention consequences. An upgrade makes
binary/configuration and persistent-state compatibility explicit before switching generations.

User-level supervision owns serving lifetime across agent attachments. It coexists with native
scientific allocations and ordinary builds; no arbitrary workstation memory ceiling or blanket
root permission is introduced. Choose SDK queue, Collector queue, batch, disk and query bounds
for their distinct units and lifetimes. Do not combine request counts, bytes and disk retention
into a fictional single capacity. Disk full and permission failures remain inspectable failures.

Normal shutdown first stops the owned serving process and lets its documented flush/queue
behavior settle within selected actual deadlines. It does not wait for every producer or cancel
science. Producer shutdown stays in 52p. A stop acknowledgement does not prove that queued
telemetry reached query visibility; forced termination has a separate observed result. Ordinary
restart policy belongs to the service supervisor, not a new task/retry controller.

Persistent exporter queues improve recovery from backend interruption, subject to queue
admission, capacity, expiry, disk integrity and export acknowledgement. Backend ingest acceptance,
query visibility and survival across abrupt loss are different claims. Replayed batches may
duplicate observations. No exactly-once, transactional cross-signal snapshot or loss-free promise
is selected. TQ02 settles the supported restart/retention envelope before TEL08 closes.

## Work packages and dependencies

Package state lives only in the coordinator. These descriptions define delivery and acceptance,
not another progress table. Each package uses compile checks where relevant and targeted tests;
52h owns assembled qualification and scope-end checks.

<a id="TEL05"></a>
### TEL05 — Native installation, supervision and owned state

Select exact Collector distribution and OpenObserve OSS releases plus artifact digests, platform
and installation identity through the existing environment/tooling conventions. Use the review's
versions as evidence leads, then verify the chosen bytes and component closure. Add explicit
install/setup/start/inspect/stop routes, protected credentials, loopback listeners and independently
owned service/state identities. No installer or dependency is run while authoring this plan.

TEL05 consumes TEL01's identity/configuration contract; service fixtures may proceed before all
instrumentation is implemented. The actual producer path later consumes a tested start/inspect
contract, not merely an agreed endpoint string. Coordinate supervisor and recipe edits centrally.

Acceptance uses an isolated owned state and exact children: wrong digest refuses publication;
listener collision and malformed configuration produce a safe diagnosis; passive inspection
starts nothing; explicit start/query/stop identify the same generation; unrelated service/state
survives. A second agent attachment can read retained history without owning serving lifetime.
Retire installation stubs or ad hoc launch paths once their consumers use this one owner.

<a id="TEL06"></a>
### TEL06 — Signal processing, routing, queues and effective configuration

Compose actual Collector OTLP ingress and authenticated OpenObserve export for selected traces,
logs and metrics. Verify exact-release transport paths/headers, batch/queue/retry units and
`file_storage` support. Publish effective redacted configuration with included component/version
identity, enabled signals, endpoints and retained limits. Preserve 52p's process identity and
52r's temporality, histogram and sampling premises across mechanical processing.

TEL06 needs TEL05's installed component closure and TEL01's resource/configuration meaning.
The first real ingress acceptance also needs an implemented TEL02/TEL03 producer slice.
52r supplies metric policy before its production metrics route is selected. Service self-metrics
must have a distinct source identity and not duplicate an existing host/scrape population.

Acceptance independently follows one actual SDK-produced record of each selected signal to
backend query visibility, checking IDs/units/fields before and after transforms. Exercise refused
ingress, exporter rejection, bounded queue saturation and bad credentials. Show the actual
observations/losses and preserved native outcome. Do not substitute synthetic direct-backend
ingest for the producer→Collector→backend route. Delete per-tool backend configuration/retry
logic that this path actually replaces; no custom durable export queue is introduced.

<a id="TEL07"></a>
### TEL07 — Independent multi-stage health and local fallback

Provide a source-attributed health read covering configuration/recording, SDK queue/drop facts,
export attempt/result, Collector receipt/refusal, queue admission, backend acceptance and query
visibility. Each stage keeps its observation time, source/process/configuration identity and
known limitations. Disabled, unsupported, sampled, refused, failed, expired, stale and unknown
are distinct when their actual evidence exists. Empty results alone select none of these causes.

Consume 52p's local processor/exporter observations, Collector internal telemetry and selected
backend/service reads. Feature-gated SDK internal metrics are supporting evidence only:
processed is not accepted; successful flush is not remote visibility. A useful supported local
fallback reads safe stderr/current observation/checkpoint information when Collector or backend
is down. Avoid exporter-failure recursion and MCP stdout contamination.

TEL07 can implement local contracts alongside TEL05; assembled stage claims require TEL06 and
implemented producer observations. 52s TEL14 exposes the same typed meaning, while 52g consumes
it in recovery guidance. No separate mutable health ledger is needed.

Acceptance injects recording-disabled, dropped/export-rejected, Collector-stopped and backend-down
cases independently. Local explanation remains usable and identifies which later stages are
unknown. A deliberately lost record must not become delivered through a successful flush or a
healthy supervisor. A successful scientific operation remains successful through telemetry
failure, and the health read never starts capture or repairs the service implicitly.

<a id="TEL08"></a>
### TEL08 — Retention, restart/recovery and finite capture substitution

Set explicit backend/stream retention and queue retention/capacity behavior; document eviction,
expired lookups, process reincarnation and counter reset consequences. TQ02 tests actual selected
versions through graceful stop, abrupt child loss, backend outage, queue replay and disk failure.
Query known records before/after restart and after controlled expiry, separating observed
acceptance, visibility and survival. Do not infer general durability from a successful replay.

TQ05 separately compares a standard finite OTLP receiver/export route with
`surreal_telemetry.capture_operation`. Preserve finite bytes/time, actual span identity,
selected-service ownership, explicit activation/restoration, exact child drain and retained
reader compatibility. Collector's
[v0.163.0 file exporter](https://github.com/open-telemetry/opentelemetry-collector-contrib/blob/v0.163.0/exporter/fileexporter/README.md)
is an alpha interchange candidate: inspected default truncation, no queue/retry and no fsync
cannot stand in for sealed evidence publication. 52s TEL16 owns custody integration.

TEL08 consumes tested TEL05/06 lifecycle/storage and TEL13 expired/partial query semantics.
Finite substitution additionally coordinates F02's actual seam consumer and TEL16. If TQ05
establishes equivalence, migrate callers/readers and delete the handwritten HTTP/2/protobuf sink,
its obsolete tests and fixtures together. If it does not, retain the bounded existing capability
for its distinct contract and record the precise unmet requirement at TQ05; do not grow it into
a production Collector or create a second delivery path for ordinary signals.

## Verification, benefit and handoff

**Proposed verification:** TEL05–08 supply discriminating isolated lifecycle, routing and failure
tests. 52h H01–H04 assemble actual producer/Collector/backend/FastMCP consumers with independent
IDs and expected stage outcomes, then run relevant integration/static/manual checks once at
scope end. F02 retains application/database continuity acceptance. No checks, installation,
service startup or new probe were run for this document; retained inquiry evidence keeps its
original limited scope. Failure baseline for future named checks is zero.

**Proposed benefit:** one pipeline removes repeated exporter policy and enables post-exit reads;
independent health makes missing telemetry actionable. Overhead, throughput, retained capacity
and productivity claims require 52h's selected measurements under ordinary and selected-detail
workloads; no quantitative benefit follows from this architecture or the isolated query probe.

At completion, route enduring service/configuration/health meaning to blueprint §23.1 through
the accepted decision route and the environment/diagnostic/capability guides. The coordinator
owns closure, actual evidence, mistakes corrected, deliberate deviations and retirement. The
supporting plan does not create a second status or evidence ledger.
