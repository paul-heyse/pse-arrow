---
title: Operation context and live diagnostic history
status: done
date: 2026-10-14
adrs: []
review_sources:
  - docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
scenario_sources:
  - docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
  - docs/design_review/evidence/simulation-performance-diagnostics-2026-10-14/contexts-and-analysis.md
doc_owner: docs/plans/40-simulation-performance-diagnostics.md
doc_topics: [design, preparation, environment, validation]
---

# Plan 40a: Operation context and live history

## Purpose and boundary

This companion develops the shared observation contract for [Plan 40](40-simulation-performance-diagnostics.md). It addresses review [F01](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f01), [F02](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f02) and [F03](../design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md#f03), supplying the relationships and coverage consumed by [native/context projections](40b-context-and-native-diagnostics.md) and [inspection](40c-inspection-analysis-and-profiling.md).

The operator accepted RC01 on 2026-10-14. Ordinary operations collect lightweight current/cumulative observations and bounded recent context from operation start. Explicit opt-out is reported as absent coverage. Detailed callback histories, expensive engine capture and profilers remain opt-in. This does not change native eligibility, scientific acceptance, publication admission or resource ownership.

**Interface-checked baseline:** `a08f5fa09` and the existing working tree. **Proposed:** the target, packages and future verification here. No new implementation, product test or measurement is established by authoring this plan. Architectural context is blueprint §14, §19–§21 and §23.1; the coordinator owns the accepted-rule route and finding disposition.

## Foundations and selected direction

`pse-backend-native::solve::Progress` owns bounded copied events and a dropped count, but retains the earliest prefix. `RunHandle` exposes it only after preparation/start. `workflow::progress::ProgressStream` deliberately opens closed terminal evidence. These are useful suppliers with different lifetimes; they cannot be repurposed by opening provisional scientific result manifests.

`pse-engine::session::assurance::TaskTracer` propagates both span and subscriber. Math admission/submission supervisors, native threads and fitting lanes propagate other ownership without an equivalent continuous diagnostic context. `pse-columnar::flight` already owns shared production and independent waiter cancellation. Diagnostic relationships must follow that existing ownership rather than introducing another scheduler or flight cache.

`CallbackState`, `PreparationCounts`, native work evidence and engine metrics already define different observations. Preserve their owners. Shared pool peak is since the last observer reset; process VmHWM is lifetime high-water. Neither is automatically an operation peak. `ExecutionTrace` silently declines observations at its collection cap; this needs explicit coverage, not a larger implicit cap.

Current manifests provide a viable dependency direction. Compiler, math, engine, columnar and native owners already depend on `pse-diagnostics`; it has no runtime, Arrow or native dependency. Its existing `DiagnosticFacts`/`DiagnosticLocation` use low-coupling source references. `pse-ids` itself depends on it, so adding a reverse `pse-ids` dependency would create a cycle. Extend the existing low diagnostic boundary with a separate observation module rather than adding a crate or making lower owners depend on runtime. Runtime owns composition, retention and access; native/context owners retain their particular payload meanings. A0 checks this placement and the public boundary before code, without turning the low module into a generic workflow engine or another tracing framework.

Use established tracing for span/event collection and library-owned async context. The simulator owns the additional meanings tracing does not supply: request/attempt attribution, a shared producer and its waiters, native/context interpretation, observation coverage and a current/recent read. Core operation state must remain available without taking over an embedding application's global subscriber.

## Shared observation contract

These meanings have one definition here during implementation and move to blueprint §23.1 through the accepted decision route. They are not a demand for one class or service per row.

| Meaning | Required behavior and boundary |
| --- | --- |
| Diagnostic operation | Exists before the first expensive preparation/admission effect. It can reference model/run/attempt/occurrence identity as available, but does not fabricate a scientific run or alter semantic/cache keys. Minted diagnostic identity and process-instance association prevent accidental association with another operation; they are not source/environment attestations. |
| Operation relationships | Distinguish execution nesting, dependency wait, shared production, reuse and sequence/continuation. A shared producer can have multiple waiters. Neither a parent span nor a process ancestry tree alone defines scientific dependency or the critical path. |
| Current activity | Phase entry, active children, latest safe native checkpoint, known wait/drain reason, last observation and freshness. Distinguish current observed state from a stale last checkpoint. No event is not evidence of inactivity. |
| Cumulative facts | Actual producer-owned work since a declared operation/attempt/segment boundary, with unit, clock, availability and reset semantics. Requests, constructions, reused products and executed results remain distinct. |
| Recent interval | Default inspection asks for the preceding 60 seconds, shortened to actual operation/observation duration. Retention supports that ordinary request through bounded recent observations or interval aggregates. Record dropped detail and incomplete ranges; never silently answer a different interval or zero-fill missing work. Longer requested intervals can be partial and must say so. |
| Coverage | Disabled, available, sampled, curtailed, stale, interrupted and unavailable observations remain distinguishable where their interpretation differs. Queue/refusal/loss counts and observed versus acknowledged prefixes are retained without inventing complete history. |
| Terminal explanation | Keeps scientific outcome, publication outcome, optional diagnostic-writer outcome and actual join/drain separate. A live observation is not a closed result, resumable native checkpoint or scientific permission. |
| Inspection lease | Non-owning with respect to scientific cancellation and native resources. Closing an inspector must neither cancel computation nor keep unrelated scientific resources alive indefinitely. Existing last-run-handle cancellation semantics are not transferred to diagnostic readers. |

Observation fields that carry model quantities retain the original or transformed coordinate authority and references. Lower producers publish facts they already possess, not additional residual/property/derivative evaluations. Native immutable snapshots are published at owner-safe checkpoints; another thread cannot query an executing mutable foreign handle merely because a getter exists.

### Time and resource interpretation

Keep wall latency, active callback time, native CPU/system time, queue/wait intervals and sampled CPU distinct. Parent/child intervals may overlap. Exclusive time requires the union of relevant child intervals and enough coverage; an uninstrumented remainder is not automatically wait time. Shared production is counted once while each waiter's latency remains visible. Critical-path statements require actual dependency/completion relationships; report unavailable attribution when those are missing.

Snapshots used for deltas belong to the same owner/attempt/segment and declare resets. A retry, event reset, native counter restart or worker replacement must not produce a negative or misleading cross-generation delta. Fixed-width counter overflow is explicit. Retained bytes, pool reservations, allocator work, process/cgroup samples and external contention are different scopes. Do not reset a shared observer to manufacture a private per-run measurement.

### Collection and access

The default light route maintains current state, cumulative facts and recent context, rather than storing every callback. Capture/retention parameters remain diagnostic configuration, not solver profile identity or execution caps. A0 selects bounds and cadence using burst, nested-work and busy-owner cases; no hard simulator latency or overhead threshold is introduced. Optional capture has separately accounted bounded storage and visible refusal/loss.

An inspector must not synchronously ask an occupied solver, surrogate actor or preparation owner to compute its reply. It reads independently published immutable state. Native callbacks must not block on console, filesystem, collector, canonical I/O or an observer queue. Avoid copying large points/models or formatting plans in an inner loop. The existing nonblocking durable tap and separate diagnostic pool are positive examples.

Local inspection has no mandatory store or collector dependency. Managed inspection identifies the actual primary/worker/native child and its parent operation. A0 compares bounded local checkpoint publication and existing owned control/IPC routes, selecting the simplest one that supports independent late inspection and busy-owner access. Do not introduce a mandatory new daemon or preserve several equivalent production transports. Existing managed transport can differ from local storage when it serves an actual boundary, with the same diagnostic meaning.

Hard death cannot guarantee an unflushed final observation. State the latest readable prefix and freshness. Normal cancellation/drop must preserve stop-request, checkpoint, native drain and observer-writer completion distinctions. Optional diagnostic failures never supply or revoke scientific permission. Collection overhead can affect scheduling/time-limit exits within the declared numerical contract; no bitwise or identical-trajectory promise is added.

## Investigations and decisions

| Inquiry | Settling evidence and required decision | Constrained work |
| --- | --- | --- |
| I-A1: placement and producer surface | Check existing crate dependencies and a small compiler/native/runtime producer composition. Select a narrow observation interface in the existing low diagnostic boundary; specify how high-level typed source references map without an `ids -> diagnostics -> ids` cycle. No new crate is presumed. | A1 and all B producers. If a new crate or changed architectural decision is genuinely necessary, prepare the concrete rule impact before that change. |
| I-A2: recent retention and live access | Compare current/cumulative snapshots with recent aggregates/detail under a busy owner, a burst exceeding detail storage and an operation inspected late. Choose interval/cadence/bounds and a local/managed access realization. Verify that an occupied actor cannot stall inspection and an opt-out cannot masquerade as zero work. | A2/A4 and C0/C2. A decision must include loss/freshness and hard-death limits, not only a transport name. |
| I-A3: diagnostic lifecycle and costs | Trace watcher/writer tasks, per-process versus per-operation ownership, shutdown and resource samples. Select bounded retention and cleanup tied to diagnostic owner obligations; no age/size-based scientific cleanup. | A5/A6 and C4. If a useful native/private resource is unobservable, mark its exact scope unknown rather than widen a generic accounting claim. |

These are bounded design/functional investigations in the execution scope, not completed research claims. Record their decisions and any changed consumers here. A source-inspected interface is insufficient evidence for cross-process live readiness; a local type test is insufficient for enclosing integration. Do not start a statistical benchmark, full producer capture or unrelated Plan 28/39 campaign to decide them.

## Execution decisions and checkpoint — 2026-10-15

I-A1 selects the existing `pse-diagnostics::observation` owner. Its identities, values,
coverage, bounded hub and passive readers depend on neither Runtime, Arrow nor scientific
IDs. High owners copy source references without adding them to scientific hashes.

I-A2 selects current/cumulative facts, 60-second recent requests, one-second aggregates
and point history with a predecessor. Point history preserves first/latest observations,
actual timestamps, coalescing, absence/nonfinite values and resets. Defaults are 128 metric
slots (effective ceiling 256), 256 detail events and a 16 MiB process diagnostic allowance;
actual storage is charged lazily. Runtime reserves 2 MiB plus 4 KiB of that allowance for
its bounded serialization buffers. The allowance covers internal retained state and a
bounded read copy, not arbitrarily many caller-retained snapshots. Refusal never curtails
scientific work.

I-A3 selects one process-owned publisher and atomic checkpoint image, at most 1 MiB and
normally refreshed once per second, shared by local, managed and private native-child
placements. No store/daemon or new scientific IPC channel is required. Active/recent
operations precede completed history; external image omission and compacted history stay
explicit. Producer callbacks never perform file I/O. Last runtime registration requests
an asynchronous final publication; it does not join/write under scientific admission.
Explicit diagnostic release joins the process publisher and removes only its exact files.
A hard death leaves at most the last image. Readers own no native/cancellation resources.

The [Rust consumer guide](../dev/simulation-diagnostics.md) describes hub pagination, exact operation readers, process checkpoint
reads and RunHandle/retained-surrogate readers. Managed claims copy their actual canonical
run/attempt/occurrence references. Private native children exchange a diagnostic-only
operation locator and parent link outside scientific hashes. Process RSS/high-water and
CPU ticks retain process lifetime scope; pool bytes retain shared-runtime current scope.

Pure hub/history, runtime/flight and real process/adapter controls have passed the selected
acceptance linked in the Outcome. This companion is complete at that boundary, retained for
the active series' consumers; its packet state is not assembled Plan 40 qualification.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| A0 | Coordinator G0 accepted-rule route; source review | Settle I-A1–I-A3 sufficiently for the shared contract: placement, light policy, live access, interval/loss semantics and owner lifetimes. Exercise only discriminating small controls where source is insufficient. Publish decisions, remaining native-specific gaps and required generated/consumer changes. | Complete — selected targeted acceptance |
| A1 | A0 shared meanings and dependency decision | Implement narrow observation identities, scope/clock/unit/reset/coverage and producer/read contracts in existing owners. Supply pure local tests for unavailable/nonfinite/count/time/reset interpretation and shared relationships. No global subscriber takeover, runtime dependency in a low owner or new scientific identity. | Complete — selected targeted acceptance |
| A2 | A1; A0 retention decision | Install operation state before preparation; collect current/cumulative and bounded recent facts from ordinary start. Migrate generic prefix-only production retention and its consumers/tests to the selected target; retain historical scientific/native records only for their separate real contracts. Test late inspection, bounded bursts, shortened windows, opt-out and open intervals. | Complete — selected targeted acceptance |
| A3 | A1/A2; existing flights/native ownership | Propagate diagnostic context through submission/admission tasks, OS/experiment lanes, nested work and shared-flight producer/waiter links. Preserve caller dispatch where supplied and explicit foreign-hook limits. Test two waiters/one producer, one waiter cancellation, unrelated concurrent operations and nested non-additive work. | Complete — selected targeted acceptance |
| A4 | A0 access decision; A1/A2; A3 relevant boundaries | Deliver local independent live reads and actual managed/native-child/transport correlation. Do not weaken canonical closed-result admission. Coordinate C2 consuming interfaces and B projections in the same functional slice. Test inspection while the original caller/worker is occupied, lookup before a scientific run exists and distinct worker/process instances. | Complete — selected targeted acceptance |
| A5 | A2/A4; current nonblocking tap/terminal guards | Complete coverage, explicit collector-curtailment, writer/refusal/failure and cancellation/drop/death behavior. Join owned diagnostic tasks without taking native ownership. Remove silent-limit and completion-only explanation paths when replaced. Test detail saturation, failed optional writer, interrupted prefix and actual stop/join distinctions. | Complete — selected targeted acceptance |
| A6 | A1/A3; selected producer/resource facts in B1/B5/B10 | Integrate truthful cost/resource/reuse snapshots: producer versus waiter, actual construction versus request, per-operation versus shared/process scope and valid interval deltas. Keep unknowns and existing known-subtotal controls. Test nested/shared overlap, resets and competing operations without resetting global peaks. | Complete — selected targeted acceptance |
| A7 | A2–A6 implemented for their consumers | Finish targeted foundation/consumer migration and deletion. Confirm lower observation responsibilities test with their actual dependencies, generic progress is not a compatibility path, terminal evidence remains readable under its own contract, and B/C can consume the delivered slices. Hand focused results and limitations to 40e. | Complete — selected targeted acceptance |

A2 and producer integration may progress in coherent slices; a settled interface alone is not a delivered prerequisite. A4's managed portion needs the relevant actual child/worker producer, not every native adapter. A5 applies incrementally as each read/write boundary lands rather than postponing necessary safety to a final sweep.

## Verification and handoff

**Proposed verification:** compile the selected affected packages through repository routes and run targeted tests of the new mechanism. Pure observation/interval/loss tests must not start a store or native solver. Integration slices retain force-validation where they actually consume Arrow. Use a supplied clock/snapshots for deterministic interval tests and real owned workers only for boundaries that need them.

Principal review S01/S03/S07–S12 and supporting S01–S04/S13–S16/S21–S25 are revealing routes; identifiers are qualified by their source document. [40e](40e-qualification-and-closeout.md) owns assembled scope-end acceptance, hygiene/manual checks and final finding closure. Do not run those suites per packet or infer all-context adequacy from one passing local observer.

Completion requires implemented consumers, replaced paths removed, and honest active/recent/cumulative/terminal reads under the selected interruption limits. It does not require exact timing criteria, complete source/environment proof or a universal explanation of every opaque native instruction.

## Outcome — selected implementation, 2026-10-15

**Implemented:** the low bounded observation owner, early runtime activities, passive Rust
readers, task/thread/shared-flight propagation, rolling progress, recent counter and point
history, loss/reset/freshness semantics, scoped resources and independently published process
checkpoints. Actual managed and private-child correlation uses diagnostic identities and
existing scientific references. Explicit publisher shutdown follows scientific drain; dropping
a runtime registration never joins optional I/O under admission locks. Superseded prefix-only
generic progress was replaced; durable scientific progress retains its separate owner.

A mistake made and corrected was wrapping large preparation futures in an async context
adapter that stored multiple inline copies. The shared adapter now uses pinned projection
and stores the input future once. Fitting's large preparation has its own charged heap
boundary; no general heap allocation or larger stack limit was imposed. A separate review
caught delayed continuation capture after spawning; scope construction now captures the
originating lane before task-local context is lost.

Deliberate deviations: one process checkpoint realization serves local/managed/native-child
placements; there is no new daemon or mandatory store. The internal budget includes bounded
serialization/read copies, while caller-retained copies remain caller memory. Unsupported
native internals and lost transport history stay explicit. These choices do not change
scientific acceptance, resource ownership, numerical identity or the permanently eliminated
environment/provenance proof scope.

Focused **Tested** results and their zero-failure baseline are recorded at the
[coordinator's selected verification](40-simulation-performance-diagnostics.md#selected-implementation-verification).
They do not establish the assembled 40e or remaining 40c/40d target.
