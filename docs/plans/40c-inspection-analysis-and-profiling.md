---
title: Inspection, analysis and profiling
status: done
date: 2026-10-14
adrs: []
review_sources: [docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md]
scenario_sources: [docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md, docs/design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md]
doc_owner: docs/plans/40-simulation-performance-diagnostics.md
doc_topics: [design, environment, validation]
---

# Inspection, analysis and profiling

This packet makes collected observations usable: one Rust-owned inspection and explanation
operation answers what an actual operation is doing, what it has cost, what changed recently,
and which observation would distinguish remaining causal hypotheses. It applies before a run
handle exists, while work is active or draining, and after completion or interruption. Python
and CLI consumers project the same typed answer. Explicit deeper capture adds evidence when
requested; ordinary inspection remains a passive read of already published state.

The [coordinator](40-simulation-performance-diagnostics.md) owns combined scope, decisions and
current finding dispositions. This packet contributes to
[principal F05](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f05)
and consumes shared corrections for principal
[F01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f01),
[F02](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f02),
[F03](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f03) and
[F04](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f04).
It neither duplicates their status nor closes F05 through a formatter or tool recommendation.

## Baseline and foundation assessment

The examined baseline is `a08f5fa09` plus the concurrent dirty checkout on 2026-10-14.
Plan authoring is documentation-only: no probe, build, test, formatter, dependency change or
product edit establishes new execution evidence here. The selected standard is Core 3.4,
Heuristics 1.0 and Process Simulator 1.5 with the pse-arrow binding.

Existing suppliers are valuable but serve different lifetimes. `pse-runtime::workflow::run`
exposes live handle progress and joined reports; `workflow::progress` reads admitted terminal
history. `python/pse/_runs.py` retains that distinction. `pse-runtime::inspection_documents`
and `pse-py::inspection` provide typed reflection/observation boundaries, rather than a complete
operation explanation. Terminal reports, native callback metrics, engine observation and
benchmark phase aggregation must remain suppliers, not independent causal authorities.

The [instrumentation evidence](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/instrumentation-and-live-inspection.md)
establishes the relevant gaps: preparation precedes handles, retained prefixes cannot answer
late recent-window questions, and occupied actors/native calls cannot safely service arbitrary
introspection. The narrow source inspection confirms existing public progress/report consumers;
it does not establish absence of every possible internal reader or native getter.

The [tool evidence](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/tooling-and-probes.md)
records exact pins `tracing =0.1.44`, `tracing-subscriber =0.3.23`, DataFusion `=55.1.0` and
`datafusion-tracing =55.0.0`; the checkout already has a full-debug `profiling` profile.
Candidate exporters and Samply are not selected dependencies. P02 collected late-attach sample
timestamps but no resolved instruction/call-chain frames; decoding success is insufficient.
Useful Rust/native capture remains an explicit investigation and delivery obligation.

AP-03/AP-06 favor a bounded composition over observation snapshots, independently testable
without starting storage or a solver merely to explain supplied facts. AP-04 keeps numerical
interpretation with its context owner; AP-07/H5/H6/H16/H26 favor selected interval/detail reads,
compact references and bounded copying. Do not scan every retained study member or hydrate every
coordinate to answer a current-child question. A mechanism substitution should change its
capture adapter, while a new experiment supplies context detail through existing navigation.
Neither change should teach Python/CLI consumers how native getters work.

## Accepted rule change and boundaries

The operator explicitly accepted principal-review RC01 on **2026-10-14**: lightweight current,
cumulative and bounded recent collection begins with ordinary operations; opt-out reports its
coverage limit; all supported contexts support live insight, including the preceding minute.
Deep tracing, sampling, allocation and off-CPU/wait capture remain optional. The coordinator
owns the decision/blueprint §23.1 amendment route before dependent implementation; acceptance
of the direction is not evidence that the amendment or implementation is complete.

Blueprint §19, §20.6 and PS-10 retain scientific result/assessment authority. An active diagnostic
view grants no access to unsealed scientific results. Native termination, original assessment,
scientific partiality, publication, diagnostic writer outcome, cancellation request and actual
drain remain separate. Inspection cannot quietly evaluate residuals, properties, derivatives,
queries or original checks to fill missing fields, or call mutable native getters from another
thread. Capture does not replace cancellation/drain ownership or numerical iteration policy.

There is no hard timing threshold, statistical campaign, environmental/provenance equivalence
proof, mandatory collector, global subscriber takeover or root-cause inference engine. Actual
target/process identity and owned cleanup are operating requirements, not proof campaigns.

## Read and explanation contract

The reader consumes [40a](40a-operation-context-and-live-history.md)'s operation identity,
relationships, published snapshots, bounded history and coverage, plus
[40b](40b-context-and-native-diagnostics.md)'s safe context/native projections. It does not own
a second observation store, reset policy or transport. Its request selects operation/child,
optional detail and a time window. The default asks for the preceding **60 seconds**, clamped
to elapsed operation time. Report requested versus actual observed interval and partiality;
a thirty-second operation has thirty seconds of potential history, not sixty seconds of zeros.

One typed response carries:

- Actual operation/attempt/producer and selected context; current state or last observation,
  phase age, freshness and source; active child/shared producer navigation where known.
- Cumulative work and recent changes with applicable clock, units, reset/segment identity,
  inclusiveness and coverage. No cross-attempt subtraction or invented zero for missing data.
- Observed scientific/context outcomes, produced or withheld members and original causes,
  separately from native checkpoint, publication, capture and drain state.
- Established explanatory statements linked to supporting observations, unresolved hypotheses
  labeled as such, and the next discriminating observation or explicit capture action.

For example, “the selected experiment is last observed inside derivative construction; callback
count has not advanced in the observed interval” establishes a location and observed change,
not a hang or derivative defect. If a stack could distinguish active symbolic work from a blocked
native dependency, say so. When available evidence already establishes unrelated-coordinate
traversal, explanation can identify that structural amplification without claiming measured
dominant latency; numerical correction remains [40d](40d-affine-preparation-locality.md)'s scope.

Busy owners cannot block inspection behind their own work. Read owner-published immutable state
through A APIs; return freshness/coverage when a checkpoint is stale. Do not synchronously ask
an occupied surrogate actor, native worker or publication task to reconstruct its state.
Cross-process unavailable/stale transport is an observation failure, not an inferred numerical
failure. Terminal explanation consumes retained diagnostics and existing result permissions.

## Causal analysis by context

Use deterministic composition and plain explanations. Each recipe first checks scope, freshness,
coverage and reset validity, then relates actual work to requested demand. These are supported
questions, not a new universal cost unit or a requirement to invent unavailable measurements.

| Question | Consumed observations and meaningful distinction | Next discriminating observation when unresolved |
| --- | --- | --- |
| Preparation/reuse | Selected policy and body/row/output/coordinate/derivative extent; actual construction, rebind, reuse, eviction or shared-flight observations | Actual producer construction or examined dependency scope; a request count or faster repeat cannot establish reuse |
| Expensive evaluations | Inclusive callback duration and calls, nested provider/implicit work and demand order | Separate high per-call cost from high evaluation count; collect a scoped sample when the callback interior is opaque |
| Numerical progress | Accepted/rejected trials, restoration/main phase, original versus normalized quantities, method-specific work | Selected rejection/restoration or original qualification observation; falling objective is insufficient |
| Dynamics | IC, native mesh and returned physical time, accepted/rejected steps, resets, sensitivities, checkpoint/replay work | Owner checkpoint or selected segment detail; output samples and proposed steps are not accepted native progress |
| Studies | Selected occurrence, dependency readiness, queue/admission and actual producer/worker work | Follow failed predecessor or occupied child; caller-local zero is not worker reuse or idle time |
| Fits/shooting/horizons | Outer request to experiment, derivative upgrade, window integration, profile chain or controller role | Follow expensive nested owner and original assessment; do not blindly sum overlapping child times |
| Memory | Reservation/current retained capacity, RSS/lifetime high-water and optional allocation/live-byte observations | Selected allocation/retention capture; reservation is not RSS, and allocation churn is not retained memory |
| CPU/wait | Wall interval, scoped CPU deltas, owner state and optional scheduling/blocking evidence | Active computation versus runnable delay versus blocking; low CPU or an idle task leaves the wait source unproven |

Support method-specific detail from 40b, including unavailable/inapplicable fields, physical
units/basis/normalization, original/native mappings and rejected or partial work. Do not flatten
roots, optimization, global search and integration into one convergence score. Provider, analysis,
surrogate, continuation and publication contexts use the same navigation with their own facts.
The [context evidence](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md)
owns the richer recipes and independently selectable controls.

## Bounded mechanism investigations

Every investigation below must return its evidence, selected decision, downstream consequences
and stop boundary. Refresh exact documentation through capability skills/current Context7 when
selecting APIs; source availability from the
[library catalog](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/library-capabilities.md)
does not establish safe attachment or live compatibility. All eligible libraries remain candidates.

| Investigation/question | Evidence and decision required | Dependents and stop boundary |
| --- | --- | --- |
| Useful Rust/native samples: why did P02 yield timestamps without frames; which route actually localizes owned work? | One bounded owned Rust-plus-native representative; inspect symbols, unwind information, target/thread selection, attach interval, decoded frames and losses. Compare perf/Samply or fitting qualified alternative, with exact enabled build/API contracts | C4 sampling waits for a working route. Stop after settling useful frames and scope; no cold benchmark campaign or host-wide permission change. An unavailable first tool triggers a bounded alternative, or a concrete coordinator decision with the functional target still unmet |
| Timeline/export: can the chosen library show ongoing work and causal flows? | Compare existing structured layer, fitting Chrome/Perfetto-compatible exporter and other relevant library alternatives on an open long interval, shared producer/two waiters, process boundary and interrupted writer. Decide live-readable representation, flush and loss behavior | C4 export waits for the chosen lifecycle/format. Stop at a readable current/partial timeline with declared clock/flow limits; a valid final-only file cannot satisfy live inspection |
| Async scheduling: is optional Tokio Console useful for a specific unresolved question? | Check exact instrumentation/features/profile and `tokio_unstable` requirements, caller subscriber coexistence, task/resource semantics and native-thread blind spots | Only selected scheduling capture depends on this. Stop with supported question or explicit inapplicability; no routine instrumented rebuild, default daemon or universal wait attribution |
| Allocation/native heap: can requested memory detail distinguish churn and retention? | Compare fitting library/external mechanisms; known Rust and native allocation controls, allocator/custom-pool/interposition compatibility, stack coverage, attachment history and observer copying/perturbation | Only selected allocation capture depends on this. Stop with real coverage and activation cost; optional absent native/custom-pool attribution remains explicit, never equated to zero |
| Off-CPU/wait: what is runnable delay, blocking and a known resource cause? | Known-wait control; exact process/thread/event eligibility and permissions, stack/native interoperability and collection effects; compare fitting scheduling/blocking mechanisms | Only selected wait capture depends on this. Stop at supported distinctions; blocking stacks alone do not establish the full wait-resource cause or deadlock |

No candidate is selected merely because it emits a file. Sampling is complementary to semantic
history and starts at actual attachment; it cannot recover the preceding minute. Expensive memory
or wait mechanisms may remain unavailable on a particular operation with honest reasons. That
per-operation partiality must not be used to close the required working deeper-profile capability
without a qualified route or an explicit unresolved coordinator decision.

## Work packages

C0–C5 execution is authorized; the packet table records the current boundary. Prerequisites below identify usable slices, not completion of whole
40a/40b documents. Common file ownership and concurrent edits are coordinated by the root.

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| C0 — Read/consumer boundary | Accepted RC01 route; A0 dependency placement and A1 semantic contract settled | Define Rust-owned request/response and evidence/hypothesis/next-observation semantics; inventory actual Rust, PyO3, generated Python and CLI consumers. Test request/window/coverage interpretation with explicit fixtures. Replace duplicate formatter interpretation where found; identify actual obsolete callers/tests for deletion rather than assuming terminal result APIs are obsolete | Done — focused consumer/capture acceptance |
| C1 — Capture decisions | C0 semantics; relevant pinned sources/tool evidence | Complete bounded sample/export investigations above; decide optional scheduling/allocation/wait support where material. Preserve P02's negative result. Consumer is C4 capture integration; remove rejected scratch mechanisms and duplicate proposed capture paths. Targeted owned controls establish useful frames and open/interrupted export, or return a concrete unresolved functional decision | Done — focused consumer/capture acceptance |
| C2 — Passive inspection and typed consumers | C0; implemented A1/A2 snapshot/history read slice, A5 coverage; A4 only for remote read; 40b safe current projection for each migrated context | Implement pre-handle/live/terminal Rust inspection and generated typed Python/CLI projection from its declaration. Current/cumulative/recent views remain responsive with occupied owner; default 60-second request clamps honestly. Migrate actual consumers together, regenerate through generator, delete displaced duplicate assembly/formatters and their obsolete tests. Targeted Rust/Python tests exercise busy owner, short/partial interval, saturation and terminal permission separation | Done — focused consumer/capture acceptance |
| C3 — Context explanations | C2; A3 shared/nested relationships where needed; corresponding 40b actual work, native/scientific meaning and reset projections | Implement the context recipes with source-linked facts, qualified hypotheses and next observations, including study dependencies and nested fits/shooting. Migrate benchmark/CLI/Python explanations using these meanings; remove replaced local inference and fixtures. Targeted tests distinguish cost-per-call/count, rejected/accepted/restoration, dynamic time/reset, memory scope and unproven waits without new scientific evaluations | Done — focused consumer/capture acceptance |
| C4 — Explicit deeper capture | C1 selected working route; C2 identity/read contract; implemented A3/A4 target identity/clock/transport slices for chosen process scope | Expose explicit safe capture separately from getter; compose caller-owned tracing/export and optional sampling with interval, target/thread, frames/symbols and losses. Stop/join only owned capture resources; operation stop remains its owner. Move actual profile/export consumers, delete superseded collectors/adapters/tests once replaced. Targeted controls cover useful Rust/native frames, open flows, partial writer and capture failure without scientific failure | Done — focused consumer/capture acceptance |
| C5 — Packet handoff | C2–C4 deliveries and every adopted context projection used by them | Complete targeted consumer migration and deletion audit, reconcile unsupported optional fields and unresolved decisions, and hand exact functional evidence/limits to 40e. Delete remaining replaced report/probe glue and obsolete fixtures; retain distinct scientific result APIs. Targeted tests cover pre-handle refusal, nested busy work, remote stale coverage and completed partial operation; no packet-level full campaign | Done — focused consumer/capture acceptance |

C0 can settle the read contract while producer implementation proceeds; C1 can investigate useful
unwinding/export independently of the full projection inventory. C2 can implement local reads
when A1/A2/A5 and its specific projection are available; remote adoption additionally needs A4.
C3 adopts each actual context projection with its consumers instead of waiting for all native
families. C4 needs only the identity/transport slices required by its selected target. Partial
adoption leaves the remaining supported contexts visible at the coordinator and does not claim
the all-context target complete.

## Verification and handoff

**Proposed:** local implementation uses compile checks and meaningful targeted tests in touched
packages (`just check-package`, `just unit-package`) and selected Python units, with explicit
force-validation for Arrow-consuming correctness closures. Generator changes require `just codegen`;
generated contracts/stubs are not edited directly. Tests challenge interpretations using known
typed observations and independently controlled occupied/failing producers, rather than deriving
expected explanations through the production explanation function.

**Proposed scenario selection:** principal
[S01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s01),
[S08](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s08),
[S11](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s11) and
[S12](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#s12)
anchor the live-to-terminal journey. Select supporting-context
[S16](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s16),
[S19](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s19) and
[S21–S25](../design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md#s21)
for nested work, typed consumers, saturation, pre-handle access, drain and writer/publication
separation. These IDs are document-qualified; the two documents' S numbers are not interchangeable.

**Proposed:** compare inspection disabled/enabled actual callback/evaluation requests and scientific
guarantees without silently adding science. Allowed scheduling nondeterminism and wall-time-limit
effects do not require identical trajectories or termination; identify collection work honestly.
Sampling/export controls establish functional coverage, not simulator speedup or overhead bounds.

[40e](40e-qualification-and-closeout.md) owns assembled integration, component/native/Python
journeys and full scope-end hygiene/manual checks. It receives actual commands, scope, mode,
zero-baseline failures, meaningful omissions and unresolved decisions. This planned document
claims no newly Tested/Measured behavior. Packet completion requires the migrated read/explanation
and working deeper-profile capability with its limits, not a promise to investigate or an
unavailable-tool message; overall F05 disposition remains with the coordinator.

## Selected Rust consumer slice — 2026-10-15

The authorized 40a/40b execution includes the minimum C0/C2 Rust consumer needed for
independent live inspection: typed hub/operation reads, RunHandle and retained-surrogate
readers, plus bounded process checkpoint reads. The [consumer guide](../dev/simulation-diagnostics.md)
describes these APIs and their coverage/lifetime limits. Focused **Tested** acceptance is
linked from the [coordinator](40-simulation-performance-diagnostics.md#selected-implementation-verification).
That earlier boundary was a partial C0/C2 delivery. The subsequent Outcome below records
the Python/CLI surfaces, causal explanation recipes and selected profiler/export delivery.


## Execution checkpoint — 2026-10-15

The operator authorized all remaining design changes, including C0–C5 and 40d D0–D4,
with targeted acceptance and documentation. 40e assembled qualification, hygiene/manual
checks, final dispositions and retirement remain a separate step.

Shared inspection documents live with their Rust semantic owners; generated Python and CLI
consumers project the same response. Supporting checkpoint version two adds actual process
attachment identity, clock mapping and native-thread execution scopes. Relationship times
and exact phase entry/exit detail use existing bounded observation retention. Additive
interval copies allow passive requested-window reads without reconstructing counters from
point samples. Unknown/unavailable transport remains distinct from scientific failure.

Operational vocabularies are source-owned leaf declarations projected through the registry,
including capture choices without capture I/O dependencies. Closed duration schema projection
has one leaf owner, re-exported by the model document facade. Observation values use the
generator-supported adjacent `kind`/`value` tags. No open-object or unregistered-enum fallback
is added. Passive reads retain the producer's freshness threshold; a requested interval after
a frozen checkpoint is unavailable/partial, rather than zero work. A matching in-flight phase
withholds a completed-time/started-call mean. CLI human output summarizes typed fields and
Rust statements; `--json` retains the full document, including bounded interval detail.

C1 selects installed userspace perf sampling plus Samply save-only import. The earlier P02
negative result is preserved; useful Rust/native frames remain a functional acceptance
requirement. Source inspection rejected direct tracing-perfetto-file callbacks: overflow can
perform sink I/O and its API cannot project known remote links. Standard Perfetto packets
are encoded by exact tracing-perfetto-sdk-schema 0.13.1/prost 0.14.4 and written by an owned
bounded worker. Caller-composed tracing is optional and enqueues bounded records only.
No default subscriber/daemon, native SDK or hand-written protobuf protocol is selected.

Installed perf does not support thread-only inheritance. Default inheritance covers late
worker threads and forked descendants; separately attached descendant recordings may overlap.
Capture declares inherited-task scope and possible overlap instead of adding their samples
as exclusive operation CPU. File accounting reserves all tool outputs independently of the
public inventory; decoder refusal never marks a raw recording readable. Timeline metric
history coalescing/refusal remains visible in both per-metric annotations and aggregate loss.

Console, allocation/native-heap and off-CPU integrations are not selected. Their limits
remain visible next observations, not zero work or required deployment changes. Scope
includes capture-only budgets, explicit stop/join and partial artifacts; it does not revive
environment/provenance proofs, cold benchmark repetition or numerical performance gates.

The optional scheduling choice remains outside ordinary collection: Tokio Console needs
an instrumented `tokio_unstable` build and cannot account for arbitrary foreign native
threads. No unresolved acceptance question requires that deployment here. Allocation
interposition or an allocator switch would add activation and native/custom-pool coverage
limits; current reservation/RSS scope stays explicit without claiming allocation causality.
Off-CPU capture would require eligible scheduling events and permissions, and a blocking
stack alone cannot identify a complete resource cause. Those adapters are not delivered
or required for this selected CPU/timeline surface; no known-wait or allocation control was
run. The context explanations retain those distinctions as possible next observations.

Useful-frame decoding uses `perf script --no-inline`: default inline expansion through
addr2line exhausted the bounded finalization interval on a useful raw recording. Decoding
that same recording without inline expansion completed and retained Rust/HiGHS names;
Samply save-only import also completed. The correction changes only bounded decoding,
not sampling input or scientific execution. CPU setup refuses a terminal selected operation
before creating output; timeline history remains available. Tool exit receipts distinguish
the owner's expected sampler SIGINT from an unexpected exit or hard kill, and failed
decode/import files keep actual partial byte counts without claiming format readability.

## Outcome (recorded after implementation)

### What was built

**Implemented — 2026-10-15:** Rust-owned passive inspection and source-linked explanatory
statements consume existing facts across preparation, callbacks/providers/implicit work,
native methods, conditioning/factors, initialization/recycles/continuation, dynamics,
studies/fitting/shooting/horizons, surrogate, engine, publication and native drain contexts.
They distinguish facts, hypotheses and next observations without initiating scientific work.
The default preceding-minute window preserves actual interval, reset, freshness and loss
limits. Unsupported optional information is unavailable rather than an inferred zero.

**Implemented:** generated closed documents and operational vocabularies project those
same meanings through PyO3, Python and the thin `just diagnostics` CLI. The runtime and
exact-operation readers hold diagnostic authority only. Checkpoint v2 supplies actual
process/thread identity and same-host clock mapping; observed links and phase entry/exit
retain actual times. Counter-window transport carries bounded actual intervals, including
boundary/coverage uncertainty. There is no second observer store, consumer inference engine
or global tracing subscriber. Existing scientific reports/results keep their distinct role.

**Implemented:** explicit owned CPU and standard Perfetto timeline capture are separate
from passive reads and target cancellation. Timeline prefixes remain decodable while
operations are open or after writer interruption. Caller-composed tracing enqueues only
bounded lifecycle records; its spans represent creation-to-close lifetime, not task scheduling.
CPU capture authenticates current process identity, follows known operation links and
retains raw samples, bounded decoded frames and save-only Samply output. Sampling begins
at attachment; inherited tasks and potentially overlapping descendant recordings remain
inclusive. Kernel sample loss is explicitly unknown. Tool errors, partial files and capture
outcomes never change scientific outcomes. Replaced selected-row machinery is documented
in 40d; no preexisting scientific result API was deleted merely because diagnostics exist.

**Tested — 2026-10-15, zero-failure baseline:** the joint 101-test selection recorded in
[40d's Outcome](40d-affine-preparation-locality.md#outcome-recorded-after-implementation)
also exercises passive window/coverage interpretation, context-specific explanations,
actual private-child publication, clock/process identity and sourced vocabularies.
The pure leaf command `CARGO_BUILD_JOBS=1 scripts/pse-env --resource-class compile --
cargo test -p pse-diagnostics --lib` passed **38 tests, zero failures and zero warnings**.
It used `PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw`; this pure
closure does not select unrelated Arrow targets. The log is `/tmp/pse40c-pure-final.log`.

**Tested — 2026-10-15, zero-failure baseline:** after the final capture corrections,
the targeted command below passed **20 tests, zero failures and zero warnings**, with
886 tests not selected. It covers open/ended intervals, two shared waiters, remote clock/flow
alignment, caller layer, live-readable prefixes, writer/output failure, existing-output
preservation, file/graph/process bounds, metric loss, decoder refusal, partial tool inventory,
owned exit classification and terminal-CPU refusal with retained timeline history.

```bash
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw \
CARGO_BUILD_JOBS=2 scripts/pse-env --native -- just unit-package pse-runtime \
'test(capture::tests)' --features pse-runtime/native-solvers \
--execution-effects native-local --ignore-default-filter --test-threads 2
```

Log: `/tmp/pse40cd-capture-final.log`, Nextest `7d7d39a5-746e-45d7-8855-3de1039101b0`.

**Tested — 2026-10-15, zero-failure baseline:** one explicitly selected linked native
capture control passed, with no failures or warnings and 905 tests unselected:

```bash
PSE_CAPTURE_CONTROL_OUTPUT=/tmp/pse40-native-profile-final-uF5BhRo5/capture \
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw \
CARGO_BUILD_JOBS=2 scripts/pse-env --native -- just unit-package pse-runtime \
capture_owned_rust_and_linked_native_profile_control \
--features pse-runtime/native-solvers --execution-effects native-local \
--ignore-default-filter --run-ignored ignored-only --test-threads 1
```

Installed perf 7.0.14 and Samply 0.13.1 sampled six seconds of owned Rust plus repeated
HiGHS work, including a native worker thread born after attachment. Actual decoded text
contains 160 recognizable Rust-control frames and 32 HiGHS API/class frames; the symbol
document reports 1,182 resolved and 162 unresolved frames. The 22,210-byte Samply gzip
decodes to 20 threads, 581 samples, 1,115 stack rows and five library mappings. Capture
completed with no errors/refused items; kernel sampling loss remains unknown and inherited
task scope remains declared. These counts characterize this control only. Raw data,
decoded frames, logs, profile and `capture-document.json` remain under the new output
directory. Log: `/tmp/pse40c-native-profile-final.log`, Nextest
`a572442b-a576-46b1-8631-dcde0413e367`. This is useful-frame evidence, not a latency benchmark,
exclusive operation CPU attribution or numerical simulator qualification.

Independent source readback then identified that timeline loss accumulation followed
fallible packet projection. Known source loss could disappear when output failed first.
Loss is now accumulated before any packet output, including selected history/detail loss.
The revealing control covers ordinary and one-byte output allowances: the latter preserves
both source loss and output exhaustion with an honestly empty, unreadable prefix. Its first
assertion incorrectly required readable output even when no packet fit (20 passes/one
failure); that assertion was corrected. The final capture selection passed **21 tests,
zero failures and zero warnings**, with 886 unselected, using the same command above.
Log: `/tmp/pse40cd-capture-scope-final.log`, Nextest
`bf1008c1-faaf-491f-9d61-adaa1579fdb4`. This supersedes the earlier 20-control selection
for current timeline/source-loss behavior. The useful-frame control remains evidence for
the unchanged CPU sampling/decoding route.

**Interface-checked — 2026-10-15, zero-failure baseline:** default-feature, force-validated
all-target compilation passed for `pse-runtime`, `pse-math`, `pse-compiler` and `pse-py`,
with zero errors/warnings:

```bash
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw \
CARGO_BUILD_JOBS=2 scripts/pse-env -- just check-package pse-runtime \
-p pse-math -p pse-compiler -p pse-py
```

Log: `/tmp/pse40cd-default-check.log`. Native-feature libraries/tests were compiled by
the linked targeted selections; the current native correctness extension was built and
installed by `just py-sync-native-correctness` with actual API stubs generated successfully
(`/tmp/pse40cd-python-sync-current.log`, zero errors/warnings). `just codegen` published
the source-owned schemas, contracts and deterministic Rust boundary fixture
(`/tmp/pse40cd-codegen-current.log`). `just ready` passed after the dependency changes
and again after correcting the temporary-directory environment issue; the current extension
imports without the earlier stale-boundary warning (`/tmp/pse40cd-ready-final.log`).

**Tested — 2026-10-15:** root `just turn-end` completed ADR indexing and formatting with
safe fixes, exit zero (`/tmp/pse40cd-turn-end.log`). Its safe Ruff pass uses `--exit-zero`
and emitted **70 findings before final formatting** against the zero target: D105 2,
E501 38, T201 24, TC003 2, TRY003 3 and TRY300 1. The following formatter changed two
Python files. This is not a clean strict-lint result; current strict counts and remaining
repairs belong to 40e E5. No quality baseline is introduced. `git diff --check` reported
no whitespace errors. These checks do not constitute hygiene or assembled qualification.
No additional product checks ran after documentation-only closeout.


**Tested — 2026-10-15, zero-failure baseline:** current-source Python diagnostic and
existing native-boundary controls passed **32 tests, zero failures/errors/warnings** with
the refreshed force-validated native extension. The existing runner and a fresh owned
temporary base kept fixture setup and native import association explicit:

```bash
PSE_DIAGNOSTICS_DIR=/tmp/pse40cd-python-diagnostics \
PSE_TEST_RESOURCE_STATE=/tmp/pse-friction-review-resources-uMrtOxGw \
scripts/pse-env --resource-class wide --native -- .venv/bin/python \
-m scripts.python_tests --unit-only python/pse/tests/test_diagnostics.py \
python/pse/tests/test_native_boundary_contracts.py -n 0 -q \
--basetemp /tmp/pse40cd-pytest-owned-RkKMpHSP/tests-current-wide
```

Log: `/tmp/pse40cd-python-tests-current-wide.log`. Controls cover exact integers beyond
binary64, explicit nonfinite/unavailable values, generated closed document round-trips,
diagnostic outcome versus scientific permission, retained readers after runtime/handle
release, standalone bounded discovery, checkpoint reads, concurrent capture wait/stop
with released Python execution, continued runtime use and CLI errors/limits. The preceding
exclusive-admission attempt was **not_run** because another heavy owner occupied the lane;
the normal wide profile matches the fixture's 64 GiB pool and completed without changing
host policy or interrupting another owner.

**Implemented:** the consumer audit retained canonical scientific inspection, terminal
progress/result APIs and benchmark-local counts because they answer distinct questions.
New operation inspection/CLI explanations use the single Rust owner; no second explanation
formatter, legacy collector or compatibility production path is retained. Root integration
used the existing main checkout, preserving concurrent dirty work; no worktree was created.
All delegated implementation/review assignments have returned. Source readback found and
resolved the loss-ordering issue above; it was not an additional scientific qualification.

C0–C5 are complete at this focused boundary. Current all-context/local-managed/native/Python
assembly, end-of-scope static/manual checks and finding closure remain with 40e. The
managed-worker refresh was **not_run** in this selected packet; the coordinator retains the
earlier 40a/40b managed pass at its original source/scope, while the joint current selection
exercised actual private-child publication and remote prefix/clock/coverage controls.

### A mistake made and corrected

The first native control failed because perf requires a unit suffix on `--max-size`;
the byte bound now uses `B`. The next control recorded useful data but failed bounded
finalization because default inline decoding invoked expensive addr2line work, and the
owner's normal SIGINT was incorrectly treated as a collector failure. Same-recording
decoder/import probes isolated those issues. The integrated correction uses `--no-inline`,
retains actual owned exit evidence and refreshes failed partial output metadata. Both
attempts remain failed evidence; the fresh final control above is the accepted route.

The first Python-extension build reported four missing `ReportSource` adapters and one
future-compatibility warning for nested workflow `Send` proof depth. The new observation
and capture errors now use the existing typed diagnostic projection/reporting adapter.
The Python crate uses the runtime's existing finite depth of 512; no auto-trait requirement
is suppressed and no runtime boxing is introduced. The failed build log is
`/tmp/pse40cd-python-sync.log`; the successful final build and consumer results are recorded in this Outcome.

The first Python test invocation passed 26 tests and had six fixture setup errors because
the shared `/tmp/pytest-of-paul` belonged to root. It was left unchanged; a fresh session-owned
base directory resolved setup. The next run passed 31 tests and failed an overstrict assertion
that every explicit stop must report `Stopped`. Bounded source history correctly yielded
`Partial`. The corrected assertion accepts that exact source-loss condition while requiring
no tool errors, unknown clock, output exhaustion, refused items or queued loss. All 32 tests
then passed. Readback also found timeline export omitted explicit registry/operation coverage
flags; the shared capture scope-loss check now covers both adapters, with a revealing partial
timeline control. The current-source results above distinguish those repaired attempts.


### Deviations from the plan, deliberate

**Implemented:** exact `tracing-perfetto-sdk-schema =0.13.1` and `prost =0.14.4` encode
standard packets in a bounded owned worker. A direct tracing-perfetto-file callback path
was rejected because overflow can perform sink I/O and its interface cannot express known
remote links. Existing rustix 1.1.5 features supply clock/thread/process identity. The lock
diff adds the required protobuf family and resolves an existing sprs/hashbrown/rayon edge;
no wholesale dependency update was requested. Generator contracts were preserved: closed
source declarations, adjacent value tags, one duration projection and registered source-owned
enums replaced unsupported input shapes rather than adding fallback generation.

**Not selected:** Tokio Console, allocator/native-heap and off-CPU adapters, with the explicit
coverage/activation reasons recorded in the checkpoint. No future return to environment or
provenance proofs is assumed. The 32-block cold benchmark, comparative timings, whole-context
assembled campaigns, hygiene/manual checks, final F05 disposition and retirement were
**not_run** in this packet and remain outside this authorization at 40e/the coordinator.
