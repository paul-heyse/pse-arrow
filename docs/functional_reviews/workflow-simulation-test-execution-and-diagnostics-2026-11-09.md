# Faster workflow and simulation tests with actionable run evidence

## Judgment and scope

The highest-value change is to make a selected run explain its progress and preserve its
first useful failure. Pair that with two ways to shorten the feedback loop: reuse the work
that a source change did not invalidate, and enter a scarce fixture scope only for work that
needs it. Use Nextest for native test selection, scheduling and runner evidence; keep the
scientific and fixture facts with the owners that can actually observe them. A faster runner
cannot report why a model was refused, whether a source edit committed, or whether a native
worker drained.

This review covers the Plan 66 workflow and simulation cases that prompted the inquiry, the
standalone modeling conformance campaign, and the shared Rust test machinery that materially
affects them. Source baseline: `d35debf5a58bc5da793702f0172bfc17b2484896`, with in-flight
Plan 66 working-tree changes inspected where relevant (runtime modeling and evidence, native
workflow tests and authored reference inputs). Those changes are not represented as tested
by an earlier run. The [selected standard](../design_review/design_principles/standard.toml)
is core 3.4, its efficient architecture and test-assessment companions, process-simulator 1.5
and the repository binding. Key distinctions are DP-09/10 (reuse and preparation),
DP-19–23 (owned execution, diagnostics and evidence), AP-07 (workload fit), and PS-02,
PS-09–13 (property and solver contracts, scientific checks and independent reference).

The review builds on [Plan 70](../plans/70-test-assurance-architecture.md), the
[CLI testing review](cli-test-and-runtime-workflows-2026-10-30.md), the
[storage and build reuse review](storage-lifecycle-and-build-reuse-2026-11-06.md), and
[simulation diagnostics](../dev/simulation-diagnostics.md). Those already account for many
of the candidate facilities. Keep [Plan 66 assembled acceptance](../plans/66l-required-population-and-assembled-acceptance.md)
separate: better test evidence and a quicker selected rerun cannot close the required
scientific population. This review changes no acceptance scope.

### What the original runs establish

| Original managed invocation | Retained evidence and limit |
|---|---|
| `01a1c1b6-43e7-765d-ba43-ae708e99e77a`, selected native workflow tests | Cargo reported **6m14s** for the test build; the host recorded **376.75s** for discovery build and **0.053s** for listing. Nextest reported **672.769s** for its run; the host's enclosing test phase recorded **680.18s**. Of 48 selected: 39 passed, six completed with failures, one was interrupted, and two never ran. Nextest counted the interrupted attempt in its seven failures. The 2,778 unselected tests are outside that denominator. The outer launcher also reported a separate 1m07s dev build. These intervals describe distinct layers and should not be added indiscriminately. |
| Same run, original failure details | Nextest's `restore-cargo` setup passed. Five **test bodies** failed around 90 seconds in canonical fixture setup; the FeedFlash test failed at 102.321 seconds with an unknown staged-edit outcome after a connection failure. The log does not identify the cause of the shared server delay or establish a solver bottleneck. The outer managed scope settled; that does not retroactively resolve the staged edit. |
| `01a1c1c3-5b87-73e7-b7cf-907db4d74e5f`, three-case follow-up | Cargo reported **2m48s**; Nextest ran for **36.561s**. Two valve cases passed. The scalar cohort case failed on fixture **drain** at 36.550 seconds. A shorter selection therefore still had an owner-lifecycle failure; the two invocations are not a controlled before/after performance comparison. |
| `01a1c1bd-e327-77b1-b346-75e028579957`, `seed-conformance` | Cargo reported **5m33s** for the native `xtask` dev build. The campaign then aborted with a main-thread stack overflow and exit 134 before publishing a report. The separate GDB invocation `01a1c1c4-aa6e-7491-8250-da0e504d8390` locates the signal near `RuntimeActivity::scope` in case resolution. That narrows the diagnostic boundary; it does not establish the underlying cause or validate an in-flight repair. |

Evidence was read through `pse inspect invocation`, `pse inspect tests --failures`,
`pse inspect log`, and the original phase files referenced by the workflow report. These
are historical observations under the original invocation conditions. Current mechanisms
below are source-inspected, library contracts are *Interface-checked*, and changes are
*Proposed*. There were no new builds, tests, benchmarks or simulation runs for this review.

<a id="O01"></a>
## O01 — Tell the developer what is waiting, working, or failing

**Recommend.** Put a compact, owner-attributed progress view on the existing invocation and
scientific inspection paths. The developer should be able to tell whether an operation is
waiting on the checkout build permit, compiling or linking, capturing binaries, listing,
waiting on a Nextest slot, setting up a fixture, waiting on native capacity, preparing or
solving a case, publishing, or draining. Show a phase only when its owner can identify it;
where Nextest exposes only eligibility or start/finish, report that boundary and leave the
reason for a particular wait unknown. Include scope, original identity, elapsed or
last-observed time, and whether the observation is stale, omitted, or incomplete. Cross
process clocks require alignment; inclusive intervals overlap and are not CPU accounting.

The host already has a build queue, `inspect invocation`, and a passive activity observer;
the runtime already publishes bounded observations with source identities before solve
admission. Join through existing invocation, Nextest run/binary/test/attempt, fixture
owner and scientific operation identities where available.
[`TargetEffect::begin`](../../crates/pse-host/src/fixtures/target_settlement.rs) already
records the Nextest identifiers and physical owner; a separate Nextest list is not the
same run. For multi-group invocations, the host waits for a predecessor sharing a
named group before its test phase timer starts. Expose that known predecessor separately
from any slot wait inside Nextest. Avoid a second polling ledger
or a universal event vocabulary. An absent attachment does not make a process idle.
Use existing `tracing` spans and [layer filtering](https://docs.rs/tracing-subscriber/0.3.23/tracing_subscriber/layer/index.html)
for selected depth. The installed `tokio-metrics` 0.5.2
[`TaskMonitor`](https://docs.rs/tokio-metrics/0.5.2/tokio_metrics/struct.TaskMonitor.html)
aggregates poll and scheduling observations by instrumented class. A selected
[`FutureMonitor`](https://docs.rs/tokio-metrics/latest/tokio_metrics/struct.FutureMonitor.html)
can add per-future poll/idle detail, for example around a named native admission wait.
Scheduling measurements require `TaskMonitor::builder().publish_scheduling_delay()`
on the containing task, which the default monitors do not enable. They can include
other branches of that task, and a dropped or stuck future does not return a final
sample. Neither monitor observes a blocking solver's own work or proves settlement.
Prefer the owner's phase and join facts for those.

**Delivery example:** for the same selected suite, an agent can distinguish a request
queued behind another writer from a case already running inside fixture setup. A live read
and a later historical read agree on the original attempt and show gaps rather than
inventing progress. Start with phase facts already available; add capture only where
the two example failures cannot otherwise be distinguished.

<a id="O02"></a>
## O02 — Keep useful conformance evidence before export and cleanup

**Recommend.** Wire cooperative interruption to the existing cancellation and drain
contract, and let an operator read the resolved campaign selection and the last known
state of each attempted case without first producing six Arrow relations. The native
conformance owner should expose a bounded projection of admitted run and case identities,
last phase, the first actionable finding per failed case, coverage, and unresolved effects.
Keep the full checks and optional Arrow export for scientific inspection and downstream
consumers. A small run-scoped checkpoint should survive abrupt child exit when the owner
has already published it; a managed supervisor can record the exit separately. It cannot
invent a verdict, rollback, report, or complete denominator after the producer is gone.

In [`conformance_command`](../../xtask/src/main.rs), manifest selection precedes the
executor, but its `CancelSource` is not connected to process signals, and the summary
and export follow the run and native-lane shutdown. Reuse the signal-to-cancel pattern
from [runtime binary support](../../crates/pse-runtime/src/bin/support/signals.rs).
On cooperative interruption stop admission, let the original owner drain, and expose
the returned incomplete report with a nonzero result. [Plan 70f](../plans/70f-conformance-and-benchmark-execution.md)
already records a focused runtime cancellation control with cancelled and unattempted
fixtures; the missing link is in the standalone command. A cleanup error can suppress
the summary even if case execution returned a report. [`RunReport::export`](../../xtask/src/development/conformance.rs)
materializes the tables and publishes each completed IPC file without clobbering; it
does not publish an atomic six-file set. `ModelingPackage::conform` already has bounded
case scheduling and discovery-order merge. Extend the existing
[runtime observation boundary](../dev/simulation-diagnostics.md) from the real case
owners; do not promote its test-only phase probe or parse human progress as a contract.
Publish at meaningful transitions, with coalescing or explicit omission and distinct
run-owned retention. Keep scientific verdict, projection completeness, export outcome
and native settlement separate. An operator-requested export remains caller-owned.

Compared with writing every check into an append-only journal, a compact snapshot
avoids storing repeated scientific payloads. Compared with relying on final Arrow only,
it helps when cancellation, a stack overflow, or failed cleanup prevents export. Durable
resume is a separate capability: replaying observed facts cannot safely reissue a staged
edit or resume a native solver. Release diagnostic state through existing custody when
eligible, preserving enough identity to regenerate a new attempt.

**Delivery example:** an abort during case resolution leaves an identified selection,
last published phase, and an original process exit. A cleanup failure after a scientific
finding exposes both without presenting either as success. An export collision cannot
overwrite a prior result or silently mark an incomplete file set complete.

<a id="O03"></a>
## O03 — Separate readiness, preparation, and current-source validation

**Recommend, starting with the standalone campaign.** Give the person changing a model a
cheap read of the requested manifest run, selected authored fixtures, and required provider
profile before spending minutes compiling. Derive this from the admitted manifest/case and
provider declarations; distinguish what source alone can establish from what needs a
prepared executable, runtime load, or scientific admission. Have the same `Campaign`
owner assess manifest/run selection, paths and explicit provider requirements before
constructing `SharedRuntime`. Where that owner is not prepared, a passive read can say
so; do not introduce a second manifest parser to promise full readiness. The selected
Nextest path already assesses exact fixture requirements before building helpers, and
coalesces helper builds within the invocation.

The [recipes](../../justfile) currently couple build and execution in
`cargo-pse-correctness run --bin xtask -p xtask --features native-solvers,force-validate`.
`seed-conformance` also selects a fixed output directory. The
[`xtask` feature graph](../../xtask/Cargo.toml) pulls `package-fixtures` and
`product-schemas` under native solvers. `--provider coolprop8-water` is an execution
selection; `property-coolprop` is the build capability. Report unsupported or unprepared
combinations before selecting effects, without implicitly installing or changing solver
selection. A prepared executable can be reused for a model-only change only if that model
is loaded at execution and all build inputs remain valid. The new attempt must still bind
the new authored inputs and run the required scientific checks.

For ordinary tests, [the native host guide](../dev/agent-environment.md#builds-in-one-checkout)
and [discovery owner](../../crates/pse-host/src/discovery.rs) already provide one writer,
Cargo freshness, an authenticated executable capture and Nextest execution from captured
metadata. Keep this path for current-source evidence. Offer explicit prepared-image
execution only where it removes a concrete repeat and the caller can see exactly which
code and inputs it validates. Changes to Rust/build-script inputs, generated projections,
the toolchain, provider linkage or features require the corresponding freshness/admission
decision. A hash proves image identity, not that it reflects the current checkout.
Reuse must preserve helper paths, native ABI/loader context and force-validation. Any
material build unit that always recompiles on a small unrelated edit should be investigated
at its actual dependency boundary before proposing a thinner executable or crate split.

Use existing Cargo [`--timings`](https://doc.rust-lang.org/cargo/reference/timings.html)
through `pse test --timings` for a named build question, and the selected
[`sccache` 0.19.0 Rust support](https://github.com/mozilla/sccache/blob/v0.19.0/docs/Rust.md)
when explaining cacheable compiler work. Distinguish Cargo freshness, incremental reuse,
cache hit or ineligibility, link work, and post-build capture. A warm no-op Cargo check
can still hash binaries and verify pooled captures. If that cost matters on a warm run,
compare repeated verification under one held immutable lease with the current checks;
retain the same protection against replacement. Do not bypass Cargo or share mutable
target directories. The historical timings establish neither a cache miss nor wasted
compilation, and do not justify disabling incremental or reducing host capacity.

**Delivery example:** the developer gets a direct explanation of an unavailable
provider without a Cargo job; a model-only edit reuses a valid image but resolves the new
model and generates new evidence. A Rust or feature change re-enters build admission.
A historical image is visibly historical, even when used to reproduce a failure.

<a id="O04"></a>
## O04 — Let Nextest schedule tests and put exclusion around the actual effect

**Recommend the 0.9.151 baseline and narrower fixture occupancy; defer new admission machinery.**
The checkout captured 0.9.146; the operator selected the current release, 0.9.151, as the
target. Its [release notes](https://nexte.st/changelog/#09151-2026-11-02) fix the
weighted-group wakeup when an out-of-group test frees capacity, and a reuse panic for
certain custom targets. The group fix is relevant to the repository's weighted configuration;
the interrupted historical run does not diagnose that bug. Qualify the update using an
exact version capture, effective profiles, an admitted list/reuse/run round trip and a
representative weighted group interaction. Verify the adapter with the 0.9.150 change
to joined Cargo argument spelling (`--flag=value`), not only a version read. The
[tagged manifest](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/Cargo.toml)
retains `nextest-metadata` 0.15.0. No compatibility route for the old runner is proposed.

The [configured scientific group](../../.config/nextest.toml) has two tokens and named
cases take both; that is a per-run exclusion budget, not a two-CPU cap. Store cases use
separate database identities but share a server; some require server-wide exclusion.
Use [groups and `threads-required`](https://nexte.st/docs/configuration/test-groups/) for
the actual protected effects. `show-config test-groups` helps explain effective assignment
to a person, but is not a machine contract. Priorities can make useful failures visible
earlier; they neither enforce dependencies nor confer a global permit. Keep cross-invocation
physical ownership and settlement with the fixture/host. Investigate waits before expanding
concurrency, and check whether a case occupies a scientific slot while it performs a
standalone independent reference or immutable input preparation. Move such work outside
the scarce region only when the evidence joins to the exact authored case and the integrated
journey remains covered.

Nextest [setup scripts](https://nexte.st/docs/configuration/setup-scripts/) are suitable for
run-scoped environment (already used to restore Cargo); wrappers and target runners are
candidates for a concrete execution boundary, not a new place to start borrowed services.
Keep native deadline, process timeout, pipe-leak detection, cancel acknowledgement, and
actual drain separate. Stress runs and one-test debugger/tracer attachments are opt-in
diagnostics. Blanket retry, extending all deadlines, or suspending a process whose lease
or external deadline keeps running would give weaker evidence. Doctests keep their separate
route. Reconsider archives and partitions only for a real second executor or transfer need,
including native runtime paths and artifact size; they do not remove a local build by default.

**Delivery example:** run two independent cases against legitimate resources and retain
the original server-wide exclusive control. The resulting observations separate queue,
fixture work and solver time and show that canceled cases release only after their owners
settle. A missed or unattempted case remains incomplete.

<a id="O05"></a>
## O05 — Expose exact attempts and compare original runs

**Recommend extending the existing report consumer; defer a native recording adapter.**
The host already uses Nextest `list --list-type=binaries-only --message-format=json`
through correctness Cargo, then one full list per admitted group from captured
`--binaries-metadata` and `--cargo-metadata`. The run uses those same captured inputs;
it still [enumerates the harnesses](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/cargo-nextest/src/dispatch/core/run.rs).
[`nextest-metadata` 0.15.0](https://docs.rs/nextest-metadata/0.15.0/nextest_metadata/)
owns inventory and mismatch reasons; [`quick-junit` 0.8.0](https://docs.rs/quick-junit/0.8.0/quick_junit/)
already deserializes the private completed report, including retry/flaky data. The
[completed-result adapter](../../crates/pse-host/src/extraction/completed.rs)
reconciles report, native exit, selection and fixture settlement. Give an agent bounded
attempt detail by original identity, and compare two retained invocations with a typed
view of what was selected, attempted, completed, or unavailable. Keep the original
failure, retry history, separately captured output and explicit omissions. In the
[0.9.151 JUnit writer](https://github.com/nextest-rs/nextest/blob/cargo-nextest-0.9.151/nextest-runner/src/reporter/aggregator/junit.rs#L149),
the primary case is the first failure for consistently failing tests and the last
attempt for a flaky pass. Normalize those roles explicitly; runner policy can fail
an eventually passing test even when XML represents a success. Match runs using
authenticated package/target/feature and case identity, not group ordinal or XML
class name alone. An XML report is neither a live event source nor proof of completeness;
no-report and unknown attempt states must remain visible. Retain nothing merely to
make a historical diff look complete.

Native filtersets, configuration precedence and list mismatch reasons remain authoritative.
The local host supplements listing for skipped binaries that need fixture-declaration
correspondence and for effective named group membership across compiler groups. Do not
remove those checks or implement a local DSL merely to avoid subprocesses. Keep
`nextest-filtering` conditional on a specific semantic duplication with a measurable
consumer cost. For a multi-group run, time those repeated lists separately before trading
target pruning for one broader inventory. Use stable
[machine-readable list](https://nexte.st/docs/machine-readable/list/) and
[JUnit](https://nexte.st/docs/machine-readable/junit/) interfaces where they suffice.

Compare [native rerun](https://nexte.st/docs/features/record-replay-rerun/rerun/) and
[`store export-chrome-trace`](https://nexte.st/docs/features/record-replay-rerun/perfetto-chrome-traces/)
only for the extra capabilities they supply. Nextest rerun handles outstanding failed or
interrupted work and can admit newly listed tests; it is not the exact completed-failure
selection of `pse test --rerun-failed`. A replay reads a previous record. A new attempt
uses admitted current inputs and does not rewrite the original. A trace grouped by
`slot` or `binary` can answer scheduling questions but excludes Cargo and cannot see
inside a solver. An optional attachment should be linked to the exact original run, use
private bounded retention and explicit release, and avoid ambient `latest`. The
[0.9.150 advisory](https://github.com/nextest-rs/nextest/security/advisories/GHSA-vc25-qg58-w57c)
and recording format change make portable recording provenance and version material.
Reject replacing completed-report reconciliation with experimental
`libtest-json-plus` or internal `nextest-runner` APIs: neither removes the owner joins,
and a parallel event ledger adds a second interpretation. Consider USDT only for a
specific runner blind spot after stored traces.

**Delivery example:** compare the 48-case and three-case originals without treating an
unselected case as fixed. The scalar cohort row shows setup failure in one and drain
failure in the other; canceled and never-started cases stay separate. A changed or
expired result is reported as unavailable, not reconstructed from a newer run.

<a id="O06"></a>
## O06 — Share immutable preparation and make the meaningful case selectable

**Recommend reviewing the boundary one representative family at a time.** Separate
reusable authored-source capture, parsed/compiled scientific structure and provider
preparation from mutable initialization, solve state, transaction, and protected result
custody. Use the runtime's existing preparation/cache owners rather than a test-only
second cache. Start with the exchanger design/rating transfer and one fixture-backed
workflow; trace which inputs truly change, which phase waits on a shared producer,
and which work is independently needed for parity. Bind reuse to authored content,
provider/revision, representation, model envelope, numerical controls where they affect
the artifact, and the actual owner lease. Standalone conformance already captures the
common physical root once and reuses its owned bundle for modeling admission. Its
reference manifest has one native and one pure entry; wider runtime/pool reuse has
no demonstrated consumer there. Reuse the qualified design artifact only when
rating explicitly consumes that artifact; rerun the reference and solve when their
independence or changed input requires it. Do not infer unnecessary work from repeated
test names or repeated observations of a shared preparation flight.

Use a hand-named native test for a small set of stable domain cases. For homogeneous
immutable or admitted file-driven cases, assess
[`datatest-stable` 0.3.3 `harness!`](https://docs.rs/datatest-stable/0.3.3/datatest_stable/)
against a real consumer. Its [tagged runner](https://github.com/nextest-rs/datatest-stable/blob/datatest-stable-0.3.3/src/runner.rs)
does recursive regex discovery for inventory; the exact path avoids a repeated scan and
does not reapply the regex. Keep original membership and byte identity with the fixture
owner. [`rstest` 0.28.0 named cases](https://docs.rs/rstest/0.28.0/rstest/attr.rstest.html#optional-case-description)
still include ordinals, so insertion can change a durable exact identity. It fits a
small matrix when that identity is not part of the workflow. A cheap DSL matrix is a
better first identity trial than the ten-file modeling refusal corpus, which also
checks exact membership and pairs each refusal with a corrected positive control.
Defer `libtest-mimic` until
a non-file dynamic population justifies custom discovery. Keep integrated campaigns in
their scientific owner. Batch cheap assertions that share a meaningful preparation if
that removes repeated process startup and the failing input remains identifiable; do not
batch unrelated effects to conceal which case failed.

**Delivery example:** a single stable case can be listed and rerun without rediscovering
or running a whole corpus. The integrated exchanger still proves qualified transfer and
independent reference agreement. A changed provider or revision cannot use old preparation
to claim current qualification.

<a id="O07"></a>
## O07 — Keep richer diagnosis selected and proportional

**Recommend targeted improvements to existing evidence; reject a second default
observability or testing platform.** Use typed errors and existing `pse-testkit` capture
first. `tracing-mock` is useful for explicitly contracted span/event fields and ordering;
it does not provide the existing attribution, coverage, or closure checks. Tokio
future-local dispatch does not automatically cover separately spawned tasks. Use existing
[`assert_cmd`](https://docs.rs/assert_cmd/latest/assert_cmd/cmd/struct.Command.html)
and [Insta](https://docs.rs/insta/latest/insta/struct.Settings.html) for selected CLI output
contracts; defer Snapbox or trycmd unless
an actual bounded presentation/command transcript is materially simpler. Compare typed
outcomes before rendering text diffs. Reject snapshots of unbounded logs, generated
payloads or solver output as a replacement for meaningful assertions.

[`proptest` 1.11.0](https://docs.rs/proptest/1.11.0/proptest/test_runner/struct.Config.html)
already supports seeded generation and failure persistence. For a selected parser or
receipt classification property, add a concrete minimized witness only if a developer
needs to replay it through the public boundary independently of strategy evolution.
A state-machine generator can exercise an independently specified sequential lifecycle;
it cannot witness native commit or drain. Keep Tokio paused-time tests for pure timer
policy. For a named async stall, consider
[`tokio-console`](https://docs.rs/console-subscriber/latest/console_subscriber/) only if
passive observations are insufficient: Tokio tracing and `tokio_unstable` change the
build, and live attachment has its own lifecycle. Use an explicit GDB or
[`rr` 5.10.0](https://github.com/rr-debugger/rr/releases/tag/5.10.0) capture for a selected
native fault, not as a timing baseline. Existing Criterion, scoped CPU/allocation
profiling, fuzzing, mutation, coverage, and modeled concurrency remain investigation
routes for named questions. None belongs in the ordinary path or substitutes for a
real process, native deadline, or independent scientific oracle.

## Order and evidence for adoption

1. **First:** O01 and O02 with the Nextest baseline update in O04. Distinguish observed
   progress, scientific result and settlement before altering scheduling. Demonstrate
   a setup refusal, a drain failure, and an abort before first export using the
   same inspection path as a normal run.
2. **Then:** O03 and O05. Compare unchanged, model-only, Rust-only and provider-change
   scenarios. Preserve exact selection, default filters, ignored/platform cases,
   zero-match semantics and force-validation. Check partial reports, stale or foreign
   capture, and exact rerun with changed current listing. A faster path must do less
   unnecessary work while retaining the evidence for the same claim.
3. **Apply selectively:** O04 fixture scope changes and O06 on a concrete family, then
   O07 or optional Nextest attachments if the new facts leave a specific gap.
   When claiming a performance improvement, measure matched inputs and selection,
   build/profile, cache and service state, and the whole path through settlement.
   Use one small comparison that can change the decision before running a campaign.

The selected directions need no new general-purpose service, history database, or
default profiler. Defer an executable split, wider fixture concurrency, extra caches
or experimental runner integrations until the named boundary can demonstrate value.
An adoption plan would own any contract changes and retire any superseded capture or
selection path in the same change. This review is opportunity evidence; it leaves
existing plans, code, tooling configuration, and scientific qualification unchanged.
