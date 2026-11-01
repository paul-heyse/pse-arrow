---
title: "Plan59f: Simulation observability and diagnostics"
status: complete
date: 2026-10-31
adrs: [ADR-0233]
review_sources: [docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md, docs/functional_reviews/cli-test-and-runtime-workflows-2026-10-30.md, docs/functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md]
doc_role: plan
doc_retention: while-dependent
doc_retention_reason: "Retained complete series at maintainer request; Plan58, Plan60, proposed ADRs and qualification references consume its evidence."
doc_retirement_trigger: owner-release
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [architecture, design, libraries, environment, validation]
---

# Simulation observability and diagnostics

**Complete:** selected delivery is closed; the [final Plan59 conclusion](59-agent-cli.md#current-state-and-next-step)
records the worktree-removal deferral and qualification limits. Earlier targets and dated
checkpoints below retain their original context; package tables and Outcomes record delivery.

## Search extension impact assessment — 2026-10-31

The [coordinator](59-agent-cli.md) owns the operator-confirmed [search-review](../functional_reviews/rust-centered-search-and-investigation-cli-2026-10-31.md)
extension; [59k](59k-query-and-result-foundations.md) owns the new typed search/result views,
[B06](59b-runs-and-continuity.md#b06) their stream transport and
[D10](59d-knowledge-and-workflow-verbs.md#d10) passive evidence-locator integration.
**Proposed assessment:** no new F package is selected. Search-result compaction and delivery
can reuse existing source/evidence and output owners without changing F's product-reader
protocol, scientific context, observation continuation or capture admission. F01–F06 remain
completed at their recorded acceptance scope; neither new search behavior nor broader
scientific qualification is inferred from that evidence.

The preserved contract includes original scientific and invocation identity, operation outcome
separate from observation/delivery success, units and nonfinite/unavailable categories, native
association limits, retained-history versus current-checkpoint distinctions, source/coverage
gaps, expiry, explicit investigation effects and observer cancellation without scientific control.
`inspect` evidence locators do not reconstruct or republish these facts, start a product reader
implicitly or acquire control. B06 owns search delivery; F03's existing product follow remains
its tested owner rather than another dependency of ordinary source discovery.

If K01/A10 later select an actual change to a shared view consumed by F, first add a bounded
F07 with that exact changed contract, governing publication dependency and affected consumers.
Its acceptance must challenge preserved scientific/observation distinctions and reuse the
relevant actual product-observation controls; delete only the replaced shared-view plumbing,
callers and tests in the proving slice. This condition does not reopen F01–F06, authorize
production changes or introduce a recurrence gate. No F07 or fresh product test is needed for
this documented impact assessment.

## Purpose and assessed foundation

**Proposed:** let an agent discover active simulation work, inspect the performing owner's
evidence, follow meaningful updates and explicitly escalate into a diagnostic investigation.
This companion implements the selected parts of the new review's
[O01–O04](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O01)
and [I06](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I06), with
conditional interfaces from O08. The coordinator owns all review dispositions; F owns packages.

Source baseline is review commit `e907d8a1f` and its inspected production baseline
`890aa07f76fac5739cfc9811a47e823c326be663`. That amendment was planning-only. Authorized
execution resumed on 2026-10-30; package state and Outcome below distinguish delivered slices
from remaining acceptance. Existing completed evidence retains its original scope.
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
| CLI invocation and actual executed tool generation | A/B invocation contract and actual foreground owner or detached wrapper |
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
source/generated navigation, and source navigation can consume it without moving scientific interpretation into analysis.
Expose current-source/binary mismatches. Missing scientific-model-to-code mapping stays
unavailable. F06 includes this composed case when delivered, but base inspection/follow/capture
does not wait on G–I. Static calls prove neither runtime causal edges, native settlement nor
memory release; no build telemetry or automatic semantic enrichment is introduced.

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| [F01](#f01) | completed | 59f | Actual discovery/schema/direct/delegated/refusal and unprepared-help consumers pass; optional candidate association checks full kernel identity and original foreground scope or detached service membership without acquiring control |
| [F02](#f02) | completed | 59f | Actual catalogue/related/shared-waiter/identity/stale-page consumers pass; units/categories/loss and non-atomic native aggregate accounting remain distinct from scientific evidence |
| [F03](#f03) | completed | 59f | Actual filled-pipe slow reading, broken/full renderer, cancellation and deadline consumers preserve original native progress; output uses cancellable readiness and bounded terminal delivery without a writer task |
| [F04](#f04) | completed | 59f | Actual separate-process canonical pages preserve failed history, retry identities/cursors and unavailable live numerics; checkpoint reuse closes the additional-publication inquiry without another bridge |
| [F05](#f05) | completed | 59f | Actual finalized-member→frozen→portable reopen, partial/source-failed capture, unavailable tools/clocks/symbols, changed-validator and bounded raw window/tail consumers pass; external query execution and remote export remain conditional and unselected |
| [F06](#f06) | completed | 59f | Selected shell consumers and unavailable-OTLP backend case pass; guidance records optional backend and RMCP triggers. Coordinator-owned scope-end series qualification remains separate |

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
The owner view may link available invocation/source/API detail directly;
missing enrichments remain explicit and do not require all G/H/I packages to finish. Typed next
actions declare arguments/effects and never run a capture or check merely to explain a snapshot.
They never capture, attach a profiler, grant result access or cancel work implicitly.

**Native process facts (review O13).** Link the exact selected systemd unit and native
status/journal facilities. Show `ActiveState`/`SubState`, available CPU/memory/task accounting,
and applicable service `MainPID`/`Result`/`InvocationID` separately from capture completeness.
Do not invent service properties for foreground scopes or label a mixed read atomic. sd-notify
`Ready` follows actual initialization and `Status` names a current wrapper phase; notification
transport alone is not readiness. Calculation values, units, source ages and gaps remain with
scientific owners. Low CPU or silent foreign execution does not prove a hang.

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

`MultiProgress`/draw targets and optional tracing-indicatif consume existing facts only;
non-TTY/NDJSON output requires no animation task. Tokio console/metrics, Nextest timelines/USDT
and profilers are explicit diagnostic escalation for an actual question, not always-on collectors,
watchdogs or restart policy. Inspect selected tool/version support before enabling an attachment.

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

F05 returns original diagnostic/evidence references, selected statement pages, clocks/coverage
and typed next observations directly; no generic investigation-packet composer is required. Address selected raw
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
the already available owner views readable and does not launch replacement collection.

<a id="f06"></a>
## F06 — Integration, conditional adapters and publication

**Consumes:** F01–F05 selected capabilities and E03's per-verb guidance integration.
No D01 packet composer or full G/H/I readiness is a prerequisite to observation/capture.

**Delivers:** actual shell-consumer journeys across checkpoint discovery, explanation, follow,
retained study history and selected investigation, including failure/coverage cases. Update current
diagnostics/environment guidance with these consumers; preserve original APIs and retire only
mechanisms demonstrably replaced. Qualify a useful F01→F02 inspection slice and its identity/
coverage failure case first, then follow/history/capture slices on their actual prerequisites;
do not defer every assembled CLI journey to F06. The coordinator owns scope-end static and
assembled series qualification timing.

Local private control data remains filesystem-owned. Existing object_store `GetOptions`,
`GetRange` and streaming reads can serve immutable result/checkpoint data where their backend
semantics fit; object storage does not become process ownership, report finality or remote-log
coordination. An actual remote-export consumer must justify credentials/transfer/abort lifetime
and preserve the one resolved dependency family before adoption.

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

Performance and resource benefits remain unmeasured. Selected F01–F06 functional acceptance is
complete. The coordinator owns review dispositions, scope-end static and assembled series
qualification, and document retirement; these remain separate from the F consumer results.

## Outcome — 2026-10-30 bounded implementation

**Implemented:** the existing prepared-reader transport now carries an opaque product-issued
next request. Product follow reads one copied checkpoint and preserves the original inspection,
source/process identity, publication coordinates and producer generations/sequences. It reports
unchanged/updated/resynchronized observations, skipped publications, native gaps and terminal
outcomes. The host continuously polls one outstanding passive helper through Tokio/Futures,
emits whole records and joins observer cancellation without acquiring scientific control.
`observe follow` exposes observing/stopped state separately from the original product outcome.

**Implemented:** `observe investigate` explicitly admits diagnostic effects. The product refuses
capture/export on the passive route before attachment or publication. Investigation delegates to
the existing capture, frozen-inspection, analysis, maintained-query-specification and review-pack
owners. The admitted analysis allocation remains held through encoding. Query specifications
say `executed: false`; no external-query execution is claimed. Capture is bounded below the
transport deadline and joins its collector before responding. No new crate, mathematical service,
database startup or generic scientific controller was introduced.

**Tested:** `scripts/pse test pse-host:product_observation` selected and completed one integration
test with zero failures against the zero baseline, complete required capture and complete native
settlement. Its existing C04 fixture declarations source-build/capture the product reader and
runtime publication helper only for the selected target. The journey exercises actual captured
discovery/schema agreement, direct/delegated checkpoint inspection, distinct CLI observers,
whole numerical records with units/nonfinite/loss, bounded NDJSON follow, passive capture refusal,
explicit timeline capture, supplied frozen-analysis/export, original failed terminal outcome and
expiry/unavailability. The cooperating producer remains alive after follow and capture; this is
not a solver or assembled scientific qualification. Retained workflow report:
`01a18e8e-a015-771b-a94d-6c9be60246ea`, with one selected/completed test, no failed or incomplete
records, no pending obligations in that invocation, and native child exit zero.

**Tested:** the focused host command
`scripts/cargo-bin/cargo-pse-correctness test -p pse-host --lib --locked product_reader --no-default-features`
passed12 controls with zero failures against the zero baseline, including lexical opaque
continuations and cancellation that leaves an independent original native process alive.
The CLI request lexical control separately passed1 test. The force-validating focused command
`scripts/cargo-bin/cargo-pse-correctness nextest run -p pse-runtime --lib --locked --features product-reader -E 'test(tooling::reader::tests)'`
passed10 controls with zero failures against the zero baseline, covering source
continuity/reset/gaps, refusal, immutable supplied evidence, selected packs and explicit collector
settlement. Earlier shared-source compilation interruptions established no product acceptance.

**Tested decision, F04:** the bounded cross-process publication inquiry and the actual prepared
CLI journey show that existing native checkpoints already carry the useful current iteration,
objective/residual values and original units, nonfinite/unavailable categories, performing producer
generation/sequence, dropped history, terminal summary and expiry. Reuse those checkpoints. A
second latest-snapshot publisher would duplicate those facts with another write/identity/expiry
contract; a separate event stream would add history/storage obligations without an unanswered
selected consumer question. No additional bridge is selected. Canonical long-term history remains
with its current owner, and active canonical reads retain `LiveNumericsUnavailable` where no live
checkpoint is supplied. This closes the additional-publication inquiry. The separate captured CLI
history acceptance below exercises the canonical owner; a remote event-history service remains
unselected.

Mistake corrected: the initial integration shape put effectful requests under the passive opaque
route. Explicit product `Access` admission and the separate new-work CLI verb now refuse that
combination before effects. Deliberate limits: optional animated human rendering was not adopted;
maintained SQL remains specification-only; existing source-bound pages and native collectors were
extended instead of replacing their ownership. The final selected consumer journeys below close
the remaining F acceptance. External-query execution is conditional on a selected execution
consumer, not an automatically authorized backlog item. Functional package completion does not
establish whole-series static, scientific, remote or performance qualification.

### F05 finalized-member and portable consumer follow-up

**Tested:** `scripts/pse test pse-host:product_observation` completed the extended actual public
journey in invocation `01a18ec7-c6ed-77c7-bd52-5aa58b541222`: one selected/completed test,
zero failures against the zero baseline, complete required capture, native child exit zero and
complete settlement. The runtime-owned fixture and host consumer use existing APIs only; no
production protocol, mathematical service, database, dependency or pack-composition layer was
added for this acceptance.

The consumer adds the actual terminal `CaptureDocument` and its explicitly selected finalized
Perfetto prefix to the supplied frozen evidence and review-pack request. It then relocates the
pack and makes the original checkpoint, frozen-input file and capture output paths unavailable.
The existing `PackReader::open`, `read_frozen`, `read_analysis` and `open_member` reopen only
selected contained members. Original operation identity, two original capture documents and
the raw member's acknowledged byte count and SHA-256 match after relocation. Original paths
are not resolution authority; missing input paths do not trigger reconstruction or collection.

A second actual timeline capture deliberately exhausts a 32-byte fixture output allowance.
Its native `Partial`/`Failed` status and `output_exhausted` loss survive in the frozen set. The
performing producer remains alive and its published checkpoint bytes remain unchanged through
capture and portable reopening; only the fixture's later explicit finish request produces the
original `Failed` operation outcome. This establishes the selected finalized-member,
portable-read and partial-capture independence cases. It does not qualify remote export, real
solver execution or all capture backends; the final expanded consumer below exercises the other
selected F05 negative and bounded-read cases.

### Native association, output settlement and final public consumers

**Implemented:** the product checkpoint and response transport share one
`pse_diagnostics::process_facts::ManagedInvocationCandidate` declaration. The launch marker is
only a routing candidate, paired with the publisher's actual boot, PID namespace, PID and start
ticks. The product emits it only when it matches the selected checkpoint's process/start locator.
The host checks original invocation/controller records, the exact native unit incarnation and
actual current process membership before adding native facts. Absent, stale or rejected
association preserves the original opaque product view. No association grants cancellation,
admission or scientific result authority.

Native service/scope/slice CPU nanoseconds, current memory bytes and current task counts retain
their unit aggregate scope and native unavailable values. Service `Result` is read only for a
service. Original `ActiveState`/`SubState`, applicable `MainPID` and manager `InvocationID` remain
separate from capture completeness and scientific outcome. Accounting failures retain original
diagnostics alongside the readable base unit observation. These independent reads are explicitly
non-atomic. Proven associations offer passive native activity and journal follow-ups with literal
arguments and the original manager incarnation; suggestions do not run those external facilities
or establish journal delivery/completeness.

**Implemented:** follow rendering awaits one owned descriptor without a background writer or
optional queue. Pipes and terminals use an independently reopened nonblocking description;
sockets use per-call nonblocking sends. Regular-file writes preserve the original offset and
append semantics, yield between bounded chunks and retain ordinary kernel file-write completion
as their boundary. Stderr, final summaries and failure delivery are bounded too. A cancelled or
failed partial delivery is a nonzero observation exit, never a successful complete frame or a
new frame appended to an incomplete one. Performing native work retains its original owner.

**Tested:** the force-validating runtime reader/checkpoint selection completed12 controls with
zero failures against the zero baseline under `product-reader,canonical-tests`, using
`scripts/cargo-bin/cargo-pse-correctness nextest run -p pse-runtime --lib --locked --features product-reader,canonical-tests -E 'test(tooling::reader::tests) | test(checkpoint_actual_published_wire_keeps_fixed_version_and_refuses_old_headers)'`.
The host command
`scripts/cargo-bin/cargo-pse-correctness nextest run -p pse-host --lib --locked -E 'test(product_reader::) | test(fixtures::) | test(cli::product::)'`
completed27 controls with zero failures and zero compiler warnings, including actual cancelled
socket writes and original regular-file offset/append behavior. The expanded
`pse-host:product_observation` target also compiles with zero warnings.

**Implemented and Tested by the final public consumer below:** the existing C04 publication fixture publishes
an original preparation owner, a distinct waiting consumer/shared-producer relationship and a
later native progress record on an explicit fixture rendezvous. Its four host tests exercise
checked original native association and truthful rejected/absent enrichment; unprepared passive
help; process/related/record pages and stale identity/cursors; a demonstrably full test-owned
pipe before slow reading, observer cancellation or deadline; broken/full renderers; optional
backend outage; and original terminal/expiry outcomes. Selected capture consumers additionally
exercise missing optional tools/symbol checks, unknown clocks, mid-stream source loss, exact raw
windows/tails and a changed native ETag. Original evidence and the performing owner remain
independent of failed enrichments. No manual public fixture environment input, database owner
or production path was added.

**Implemented and Tested by the final canonical consumer below:** the two-attempt canonical history test
uses a source-captured native CLI and product reader supplied by its scoped runner-borrowed C04
declaration. Each retained page runs in another process, preserves the failed original attempt's
events and cursors, distinguishes retry generations and exposes unavailable live numerics.
The original producer owns the existing canonical fixture; passive readers only connect and
drain their own clients. This replaces the test's in-process delegation route.

**Implemented conditional dispositions:** local shell observation and selected Perfetto
investigation need no required Collector/OpenObserve service. Optional OTLP export remains
diagnostic correlation, with loss/shutdown facts separate from delivery. RMCP is explicitly
unselected until an actual client requires typed tools/resources/subscriptions beyond shell
access; a future adapter must preserve native/canonical authority and demonstrate actual host
attachment. No MCP configuration or new required service is selected. The environment guide
documents actual reader preparation, discovery/schema, query/follow, retained history and
explicit investigation routes. The final public journeys below establish selected functional
acceptance; optional remote adapters and full-series qualification remain outside that claim.

**Tested, first expanded public consumer:** `scripts/pse test pse-host:product_observation`
completed all four selected tests in workflow `01a18f00-5e10-703e-9714-dcbd955eea71`, with two
failures against the zero baseline, complete required capture and complete native settlement.
The actual filled-pipe slow-reader and broken-pipe/full-renderer/cancel/deadline consumers passed.
The association fixture assumed a managed marker that correctness intentionally removes; the
optional-backend fixture omitted the required telemetry resource identity and also attempted a
second installation of the existing host subscriber. The original failure report is retained.

**Implemented correction:** C04 now passes its typed original `Input.invocation` to fixture
projection and publishes a bounded fixture-owned routing hint for the cooperating producer.
Full kernel identity and original native membership still establish the association; a copied
hint supplies no authority. The optional telemetry fixture supplies the original decoder's
required identity and uses its existing one-time host installation.

The corrected routing exposed a production distinction: ordinary foreground workflows retain
v4 foreground scope custody, while detached services retain v5 service/controller custody. The
passive association now opens the original custody kind and validates its full current process
identity and original native membership before and after reading. Foreground scope views omit
service-only properties rather than inventing a service launch record. The focused host command
`scripts/cargo-bin/cargo-pse-correctness nextest run -p pse-host --lib --locked -E 'test(product_reader::) | test(fixtures::) | test(cli::product::) | test(passive_scope_wire_omits_service_properties_and_uses_its_declared_view_schema)'`
completed28 controls with zero failures against the zero baseline. Its only compiler warning was
the then-unconsumed foreground-drain prerequisite owned by the separate source workflow slice.

**Tested, final expanded public consumer:** `scripts/pse test pse-host:product_observation`
selected and completed all four tests with zero failures against the zero baseline in workflow
`01a18f23-d66a-7590-9b99-763a3c049284`. Required capture and native settlement completed,
the original child exited zero, and no native lifetime remained pending. The retained report
SHA-256 is `1ca2f8d23fbab3bc84a4c02b29c3be2fd7dca7ab0dbf1c632416d6025f292345`.

This actual prepared CLI consumer proves the foreground association, original native accounting
and passive status/journal suggestions. Absent, wrong-invocation, wrong-boot and wrong-start
candidates leave the original scientific view readable. The slow consumer fills a test-owned
4096-byte pipe, observes the original producer advance while output is stalled, then receives
eight complete samples and a clean terminal NDJSON frame. Broken pipe, full renderer, observer
SIGTERM and output deadline similarly leave the producer alive and advancing; cancelled and
stalled observers exit130 and124 respectively. A real optional OTLP endpoint outage leaves local
inspection usable, with loss/shutdown evidence separate from unobserved remote delivery.

The same consumer exercises explicit capture with missing optional tools, unchecked symbols and
unknown clocks, mid-stream source disappearance, changed native ETag and exact bounded raw
windows/tails. It admits actual finalized raw members into supplied frozen evidence, reopens a
portable selected pack after original paths disappear and preserves original identities, byte
counts and hashes. Partial/failed capture and failed enrichment leave scientific outcome and
the existing readable owner evidence independent.

**Tested, final canonical history consumer:**
`scripts/pse test pse-runtime --lib --features canonical-tests -E 'test(product_retained_study_and_attempt_pages_preserve_original_history_and_join_read_custody)'`
selected and completed one test with zero failures against the zero baseline in workflow
`01a18f26-1569-75cb-9eff-1a1e1eb3ca97`. Required capture, fixture settlement and native
settlement completed, the original child exited zero, and no native lifetime remained pending.
The retained report SHA-256 is
`0d0cde725508a1f648a558ece792ecf9b262c84039bb7ad4eb4b411649b23828`.
Each canonical page uses the separately captured CLI/reader. Original failed-attempt events,
retry attempt/generation separation, bound continuation refusal and unavailable active live
numerics survive the process boundary. Two observer invocations retain distinct invocation IDs
and the same original scientific context. The original canonical fixture owns its service;
readers drain their own connection custody, and isolated fixture removal succeeds afterward.

**Completion boundary:** these actual consumers and focused owner controls complete selected
F01–F06 functional acceptance. No solver execution, broad numerical reference, remote export,
external-query execution, required telemetry backend, MCP adapter or performance benefit is
claimed. The original failed expanded run remains retained alongside the corrected passing
results. Coordinator-owned integrated series qualification and document lifecycle remain open.

**Scope-end allocation correction:** strict Clippy exposed the oversized frozen-inspection
document inside the investigation result. Boxing that one owner document removes both nested
enum disparities; the outer payload remains unchanged. A direct owner/delegated serialization
comparison and schema-reference control preserve the wire contract. Force-validating selected
reader/frozen-inspection units passed 18/18 with zero failures or warnings. The post-integration
main `scripts/pse test pse-host:product_observation` passed 4/4 in invocation
`01a18f67-b740-73f4-a6bd-7d6fd84328fd`, with successful native setup, complete capture and
settlement and no incomplete records or retries. Final default/no-default/native workspace Clippy
is accounted for by P01; no performance benefit is inferred from this allocation change.
