---
title: "Plan59f: Simulation observability and diagnostics"
status: active
date: 2026-10-29
adrs: [ADR-0233]
review_sources: [docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md]
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
document amendment. Existing completed evidence is retained; execution remains paused at the
coordinator's 2026-10-30 checkpoint.
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

This revision integrates the library-integration review's
[I07](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I07),
[I10](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I10),
[I11](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I11),
[I12](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I12),
[I13](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I13) and
[I14](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I14).
Their review-qualified identities supplement O01–O04 rather than replacing those dispositions;
the coordinator owns the operator-confirmed 2026-10-30 RC01–RC04 decision routes.

## Target and authority boundaries

The lean host owns invocation selection, control and outer output handling. An explicitly prepared
product executable owns product request decoding, scientific identities, inspection semantics,
canonical access and response schemas. Extend an existing appropriate product composition root;
do not add a crate merely to share a parser or import runtime/Arrow/database types into the host.
Basic help, host status and invocation control remain usable without product preparation.

F01 defines the versioned delegated product protocol and its discovery/schema binding. The host
forwards selected payload bytes under time/extent bounds and interprets only the agreed transport
metadata. The product decodes into its owned request types and checked domain constructors
before effects, and emits valid typed results through directional Serde/Schemars views. Input
schema validity, decode success and scientific/domain validity remain distinct. Unknown versions,
invalid identity/selection combinations and malformed input refuse at that boundary; host output
projection cannot establish scientific permission or turn an observation into an accepted result.
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

Product readers own useful detail modes and pages: selected operation facts/statements first,
then requested related operations, record windows or numerical detail. Preserve `Statement.kind`
(`Established`, `Hypothesis`, `Limitation`), original `evidence` and `next` rather than inferring
strength from prose. Process/unit facts, invocation metadata and scientific observation form
distinct sections. Preserve units, clock domains, timestamp ages, reset generations, gaps,
coverage and unknown causal relationships. A finite numerical value, unavailable value and
nonfinite category use the owner's typed policy; serde_json's conversion of nonfinite numbers
to null is not a faithful numerical contract.

Page selected records at their semantic boundary; do not recursively truncate arbitrary nested
records after serialization. Continuations bind source generation/digest and reader context,
and refuse changed/expired input unless deliberately retained immutable evidence serves the
original page. Unknown totals and omitted detail remain explicit. Report examined source scope
separately from returned record count: a small page may require a broader read. A03's generic
fields/budgets only project these producer-owned results mechanically.

## Package state

F02/F05 may offer source navigation when original observation/capture identifies an actual
location/build; [59j J01/J04](59j-source-queries-and-change-proposals.md) supplies addressed
source/generated navigation, and D01/G09 compose it without moving scientific interpretation into analysis.
Expose current-source/binary mismatches. Missing scientific-model-to-code mapping stays
unavailable. F06 includes this composed case when delivered, but base inspection/follow/capture
does not wait on G–I. Static calls prove neither runtime causal edges, native settlement nor
memory release; no build telemetry or automatic semantic enrichment is introduced.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [F01](#f01) | in-progress | 59f | Product owns passive protocol and directional schemas; corrected native capture passes10 controls, actual prepared discovery/schema/delegation matches direct product evidence and preserves refusal. Broader identity integration remains |
| [F02](#f02) | in-progress | 59f | Checkpoint catalogue, exact inspection and source-bound pages pass7 controls with original outcome/coverage; actual prepared CLI inspection agrees with direct native output. Broader composed views remain |
| [F03](#f03) | planned | 59f | Bounded follow with truthful continuity and optional human progress |
| [F04](#f04) | in-progress | 59f | Product-owned canonical study/attempt history and cursors pass17 focused controls and an actual source-built two-attempt canonical fixture test; captured CLI history integration remains. Existing checkpoints selected, no extra live-numerics publisher; unavailable live evidence stays explicit |
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
directional Schemars declarations, not a host-owned copy. Borrow typed result/page views for
owned data; preserve `RawValue` for genuinely opaque upstream evidence whose original numeric
lexemes or duplicate keys matter. Do not normalize that evidence through a generic JSON model
or generate the same schema twice. Keep opaque payload forwarding opaque.

**Acceptance:** direct and delegated reads agree for the same supplied checkpoint; version/schema
mismatch refuses before effects; an unprepared product reader leaves host-only help/control
usable; two observer invocations of one study retain distinct invocation and original scientific IDs.
Challenge malformed input, unknown protocol/schema versions and invalid domain combinations at
the owner constructor. A schema-valid response cannot override a failed scientific outcome;
finite/unknown/nonfinite numerical cases preserve their original distinctions in emitted views.
Retire any replaced standalone consumer only with its actual caller migration, not by pathname.

<a id="f02"></a>
## F02 — Passive inspection and why-state navigation

**Consumes:** F01 and existing `tooling::inspection::{list_processes,inspect_checkpoint,inspect_image}`.
B01 supplies invocation-linked navigation when present; explicit checkpoint inspection need not
wait for the whole invocation-history implementation.

**Delivers:** bounded process/operation discovery, selected inspection, related-operation navigation
and source-linked explanations rendered from existing `Statement {kind,evidence,next}`. Display
actual phases, their ages, source availability and coverage, with established facts, hypotheses
and limitations intact. Add owner-published wait facts only
where existing phases/relationships cannot distinguish a useful question. Do not derive a hang
from low CPU, stale checkpoints or silent foreign execution. Inclusive phase time is not additive.

Candidate command families are `pse observe list|inspect`; settle final spelling with A06's actual
tree, keeping operation semantics here. Snapshot reads use the product-owned pages above.
D01 may compose this base answer with available invocation/source/API sections immediately;
missing enrichments remain explicit and do not require all G/H/I packages to finish. Typed next
actions declare arguments/effects and never run a capture or check merely to explain a snapshot.
They never capture, attach a profiler, grant result access or cancel work implicitly.

**Acceptance:** shared producer/waiter observations stay distinct; reused PID/start identity is
rejected; missing/curtailed/stale evidence is not empty success; a preparation/publication/native
phase returns the owner's evidence and useful next observation without starting science.
Include nested related-operation/record pages, unknown totals and a failed/stale continuation;
mandatory identity, outcome and coverage survive every detail mode. Two readers and an observer
timeout preserve the original run; publication reset/gap and missing live numerics stay explicit.

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
The trigger is an actual unanswered consumer question; checkpoint reuse and explicit missing
live numerics are the initial delivery, not a provisional commitment to build another bridge.

**Exit decision:** if current checkpoints meet the intended consumer, retain them and close the
investigation without another bridge. If a bridge is justified, define product-owned publication,
selection, gaps and retention before scheduling its producer/reader implementation as additional
F work through the coordinator. Do not open provisional scientific result rows or imply that OTel
sampling is exact replay. A full remote event-history service is not automatically selected.
Canonical transitions and long-term study records remain with their existing SurrealDB/product
owner. An addressed object-store artifact never becomes the canonical scientific state model.

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

F05 contributes original diagnostic/evidence references, selected statement pages, clocks/coverage
and typed next observations to D01's incremental investigation packets. Address selected raw
windows/tails through A04's immutable-artifact resolver and native `object_store::GetOptions`,
`GetRange` and streaming payloads where applicable. Preserve byte offsets, encoding and validators;
ETag/version/hash are distinct and local mutable reads do not acquire snapshot isolation. Keep
private local publication/no-follow/UID protections and canonical publication with their owners.
Classify retention by role: sole original capture/diagnostic evidence, authored investigation
inputs and scientific data require their own retention authority; regenerable renderings/query
pages do not justify persisting a whole result for every read. A query is regenerable only while
its complete source bytes and context remain available. Explicit remote export remains a separate
selected capability, with transfer/abort/credential lifecycle established if actually needed.

**Acceptance:** capture failure/partial output leaves scientific outcome unchanged; export reopens
only supplied finalized members without original paths; absent symbols/tools/clocks remain explicit.
Compare retained invocations under original applicability conditions without claiming causal speedup.
At least one actual selected capture→frozen analysis/export consumer journey establishes delivery.
Add mid-stream failure, changed validator and bounded tail/window cases; retained original evidence
and authored inputs remain usable after regenerable pages are removed. A failed enrichment leaves
the already available packet sections readable and does not launch replacement collection.

<a id="f06"></a>
## F06 — Integration, conditional adapters and publication

**Consumes:** F01–F05 selected capabilities and E03's per-verb guidance integration. D01 packet
composition consumes each available F producer slice as it ships; full G/H/I readiness is not
a prerequisite to this base observation/capture journey.

**Delivers:** actual shell-consumer journeys across checkpoint discovery, explanation, follow,
retained study history and selected investigation, including failure/coverage cases. Update current
diagnostics/environment guidance with these consumers; preserve original APIs and retire only
mechanisms demonstrably replaced. Qualify a useful F01→F02 inspection slice and its identity/
coverage failure case first, then follow/history/capture slices on their actual prerequisites;
do not defer every assembled CLI journey to F06. The coordinator owns scope-end static and
assembled series qualification timing.

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
