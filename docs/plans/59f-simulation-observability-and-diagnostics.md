---
title: "Plan59f: Simulation observability and diagnostics"
status: active
date: 2026-10-29
adrs: [ADR-0233]
review_sources: [docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md]
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [architecture, design, libraries, environment, validation]
---

# Simulation observability and diagnostics

## Purpose and assessed foundation

**Proposed:** let an agent discover active simulation work, inspect the performing owner's
evidence, follow meaningful updates and explicitly escalate into a diagnostic investigation.
This companion implements the selected parts of the new review's
[O01–O04](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O01)
and [I06](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I06), with
conditional interfaces from O08. The coordinator owns all review dispositions; F owns packages.

Source baseline is review commit `e907d8a1f` and its inspected production baseline
`890aa07f76fac5739cfc9811a47e823c326be663`. No production implementation is authorized by this
document amendment. Existing A01/E05 work and the D02 trial are retained; execution remains stopped.
Core3.4, heuristics1.0, test assessment1.2 and process-simulator1.5 apply through the binding.

The foundation already includes `pse-diagnostics::ObservationHub`, coherent producers, coverage,
`inspection::Statement` and `NextObservation`; runtime process checkpoints, typed inspection,
`ExecutionContext`/`Carrier`, capture, frozen analysis, query specifications and selected review
packs; and `RetainedDataServices`/`StudyObserver`. Source inspection supports reuse, not new
qualification. See the [diagnostics guide](../dev/simulation-diagnostics.md) and the review's
[library research](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#library-mechanisms-and-research-coverage).

The important gap is consumer composition. A separate CLI process cannot obtain the original
`RunHandle`'s `Arc<Progress>` by running on the same host. Published process checkpoints and
retained canonical progress can be read now; additional live study data requires a deliberate
publication contract. The target preserves that distinction rather than opening provisional results.

## Target and authority boundaries

The lean host owns invocation selection, control and outer output handling. An explicitly prepared
product executable owns product request decoding, scientific identities, inspection semantics,
canonical access and response schemas. Extend an existing appropriate product composition root;
do not add a crate merely to share a parser or import runtime/Arrow/database types into the host.
Basic help, host status and invocation control remain usable without product preparation.

F01 defines the versioned delegated product protocol and its discovery/schema binding. The host
forwards selected payload bytes under time/extent bounds and interprets only the agreed transport
metadata. The product validates requests before effects and issues typed versioned responses.
Unknown versions/missing preparation produce actionable unavailability; passive reads do not
build tools, initialize a database or construct mathematical services. Explicit preparation is
separate from reading. New-work admission follows A05; existing-work control follows B03.

Keep these meanings independently observable:

| Meaning | Authority |
| --- | --- |
| CLI invocation and actual executed tool generation | A/B invocation contract and original supervisor |
| Scientific RunId/AttemptId and canonical lookup keys | Existing product run/attempt owners |
| Diagnostic OperationId, producer, source sequence and reset generation | Performing observation owner |
| Process/unit identity, invocation incarnation and resource facts | Native host manager and process facts |
| Scientific outcome and accepted result | Existing canonical/local scientific owner, never a progress renderer |
| Coverage, freshness, transport loss and retained history | Original source/reader, with explicit unavailable categories |

Use existing `RunHandle` source references and `invocation::InvocationEvidenceDocument` to link
these facts. Associations are potentially many-to-many and absent associations remain absent.
The CLI ledger does not become a scientific correlation recorder or a database policy mirror.

## Package state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [F01](#f01) | planned | 59f | Product delegation/schema binding and existing-identity association |
| [F02](#f02) | planned | 59f | Passive process/operation inspection and evidence-based explanations |
| [F03](#f03) | planned | 59f | Bounded follow with truthful continuity and optional human progress |
| [F04](#f04) | planned | 59f | Canonical study/history consumer and gated cross-process live-publication decision |
| [F05](#f05) | planned | 59f | Explicit capture, frozen analysis/query and selected evidence export |
| [F06](#f06) | planned | 59f | Integrated consumer acceptance and conditional backend/interface dispositions |

<a id="f01"></a>
## F01 — Product delegation and correlation

**Consumes:** A03's invocation/output contracts, A06's narrow owner-dispatch slice and existing
product identity/inspection APIs. The working host dispatch is required for CLI integration;
product handler/schema design can proceed before it exists. No whole-D prerequisite applies.

**Delivers:** prepared product-reader selection and a small request/result protocol over the
existing typed product operations. Discovery describes supported operations and schema versions;
the owning parser/decoder remains authoritative. Reuse existing scientific/observation source
references, adding CLI invocation association at cooperating launch/publication boundaries.

Separate transport failure from a valid response describing unavailable evidence. Preserve
original result/reference identities and units. Schema generation uses the product's Serde and
directional Schemars declarations, not a host-owned copy. Keep opaque payload forwarding opaque.

**Acceptance:** direct and delegated reads agree for the same supplied checkpoint; version/schema
mismatch refuses before effects; an unprepared product reader leaves host-only help/control
usable; two observer invocations of one study retain distinct invocation and original scientific IDs.
Retire any replaced standalone consumer only with its actual caller migration, not by pathname.

<a id="f02"></a>
## F02 — Passive inspection and why-state navigation

**Consumes:** F01 and existing `tooling::inspection::{list_processes,inspect_checkpoint,inspect_image}`.
B01 supplies invocation-linked navigation when present; explicit checkpoint inspection need not
wait for the whole invocation-history implementation.

**Delivers:** bounded process/operation discovery, selected inspection, related-operation navigation
and source-linked explanations rendered from existing `Statement {kind,evidence,next}`. Display
actual phases, their ages, source availability and coverage. Add owner-published wait facts only
where existing phases/relationships cannot distinguish a useful question. Do not derive a hang
from low CPU, stale checkpoints or silent foreign execution. Inclusive phase time is not additive.

Candidate command families are `pse observe list|inspect`; settle final spelling with A06's actual
tree, keeping operation semantics here. Snapshot reads carry limits and continuation metadata.
They never capture, attach a profiler, grant result access or cancel work implicitly.

**Acceptance:** shared producer/waiter observations stay distinct; reused PID/start identity is
rejected; missing/curtailed/stale evidence is not empty success; a preparation/publication/native
phase returns the owner's evidence and useful next observation without starting science.

<a id="f03"></a>
## F03 — Follow and presentation

**Consumes:** F02 and A03's bounded output/NDJSON contract. Follow is initially bounded polling
of copied checkpoint pages. No push transport or new daemon is required for initial delivery.

**Delivers:** snapshot and follow over the same explicit selection, optional resume coordinates
where the source supports them, and an observer-owned stop token. Preserve source generation,
sequence, publication timestamp, durable cursor and gap/current-resynchronization distinctions.
Closing a view stops observation only. A follow wait ending is distinct from workload completion.

Use existing Tokio ownership with Futures stream composition; `FuturesUnordered` needs an actual
population bound and continuous polling, and fail-fast dropping is unsuitable for independent
settlement. Coalesce/drop optional updates visibly; never await renderer capacity in a solver
callback. Required test-verdict evidence retains B02/C02's separate completeness path.

Human presentation may use indicatif `MultiProgress` and rate-limited stderr draw targets after
exact-pin adoption. Its hidden printing is not a durable error channel. Use unknown-total
indicators for solver progress; percentages/ETA require a meaningful finite count. NDJSON stays
clean and independent of terminal rendering, with explicit nonterminal/terminal observation state.

**Acceptance:** slow reader, broken pipe, renderer failure and observer cancellation cannot own
scientific cancellation or block native progress. Resets/gaps are reported; compact JSON is not
cut at a byte budget; closing follow leaves the original run alive under its intended owner.

<a id="f04"></a>
## F04 — Studies, retained history and live-publication investigation

**Consumes:** F01/F02, existing math-free `RetainedDataServices` and `StudyObserver` cursors.
F03 supplies streaming presentation; the initial study snapshot/history reader can precede it.

**Delivers:** explicit selected canonical study/attempt reads, original lineage and retained
terminal progress with existing protection/continuity semantics. No read starts a configured
database. Local in-process live attachment, another local process and a remote worker remain
different capability cases. Expose `LiveNumericsUnavailable` rather than synthesizing live data.

**Investigation before any extra publication:** determine which useful live numerical fields are
already present in published process checkpoints for a cooperating worker. Compare using those
facts with an owner-produced bounded latest snapshot and with an event-history stream. Exercise
one cross-process local/worker consumer, not a full performance campaign. Record coverage,
identity/sequence, expiry, loss, storage/write burden and what the added bridge makes possible.

**Exit decision:** if current checkpoints meet the intended consumer, retain them and close the
investigation without another bridge. If a bridge is justified, define product-owned publication,
selection, gaps and retention before scheduling its producer/reader implementation as additional
F work through the coordinator. Do not open provisional scientific result rows or imply that OTel
sampling is exact replay. A full remote event-history service is not automatically selected.

**Acceptance:** a separate CLI reads original retained history and reports unavailable live samples
honestly; retries retain distinct attempts/cursors. Investigation closes with a reasoned selection
and concrete downstream work or explicit non-selection, not an unresolved implementation TODO.

<a id="f05"></a>
## F05 — Explicit investigation and selected evidence

**Consumes:** F01/F02, existing capture, frozen-inspection, review-analysis/query/pack/export APIs
and A03's output/error semantics. It does not depend on optional remote live publication.

**Delivers:** explicit request paths to bounded capture, read/analyze supplied evidence, maintained
query specifications and portable selected evidence export. Use original target identity, units,
clock/coverage, source sequence and losses. `NextObservation` describes an action without running
it. Passive inspection, explicit attachment/recording and mathematical diagnostics stay distinct.

Reuse `CaptureSession` collector settlement, `review_analysis::analyze`, `review_query::specification`
and review packs. Where a selected external query is actually executed, its caller owns the exact
prepared tool, timeout, output bound, schema interpretation, cancellation and original child join.
A pure SQL specification alone is not execution qualification. Do not introduce arbitrary SQL or
rebuild retired generic debugger/campaign controllers to achieve superficial interface parity.

**Acceptance:** capture failure/partial output leaves scientific outcome unchanged; export reopens
only supplied finalized members without original paths; absent symbols/tools/clocks remain explicit.
Compare retained invocations under original applicability conditions without claiming causal speedup.
At least one actual selected capture→frozen analysis/export consumer journey establishes delivery.

<a id="f06"></a>
## F06 — Integration, conditional adapters and publication

**Consumes:** F01–F05 selected capabilities and E03's per-verb guidance integration.

**Delivers:** actual shell-consumer journeys across checkpoint discovery, explanation, follow,
retained study history and selected investigation, including failure/coverage cases. Update current
diagnostics/environment guidance with these consumers; preserve original APIs and retire only
mechanisms demonstrably replaced. The coordinator owns assembled series qualification timing.

Record the following conditional dispositions, without requiring adoption for core completion:

- Existing OTLP/Collector/OpenObserve and Perfetto can supply optional correlation/investigation.
  Preserve `ExecutionContext`/`Carrier` and shared-producer links; backend absence does not remove
  local inspection. Query integration needs an explicit consumer and bounded contract first.
- RMCP remains deferred until an actual client needs typed tools/resources/subscriptions beyond
  shell access. Then compare a thin `ServerHandler` adapter with no MCP; protocol cancellation and
  tasks do not become native/canonical execution authority. Actual host attachment is required
  before a delivered-adapter claim; no Codex MCP configuration change is selected here.
- A new crate or required service must earn its dependency/lifecycle benefit and existing decision
  route. Neither is a prerequisite to this plan's initial observation delivery.

**Acceptance:** two invocations can inspect the same lineage; inactive/partial evidence and failed
science remain separate; backend outage leaves local reads usable; ordinary shell use needs no
MCP. Deferred options have their consumer trigger and implementation boundary recorded.

## Verification and handoff

**Proposed verification:** targeted parser/reader/cursor controls during implementation, followed
by actual prepared-product delegation and selected native observation/capture journeys at scope
end. Tests must challenge a wrong identity, unavailable source and incomplete stream, not merely
snapshot the renderer. Preservation of native/scientific authority is independent of human polish.
No broad numerical reference, remote deployment or timing/capacity campaign is implied.

**Interface-checked:** existing source/review integration paths. **Proposed:** all F packages;
no product execution or new diagnostic collection ran to author this document. Performance and
resource benefits remain unmeasured. The coordinator owns disposition and the current next work.
