---
title: CLI, test and runtime workflow opportunities
status: proposed
date: 2026-10-30
doc_role: review
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [design, libraries, environment, diagnostics, validation]
---

# CLI, test and runtime workflow opportunities

## Judgment and scope

The most valuable target is a smaller CLI over native execution, test and scientific owners.
It should make a selected operation easy to run, understand and investigate. It should shed
optional continuity, collection and informational features whose implementation costs exceed
their usefulness. The principal opportunity is removing responsibilities, followed by better
delegation to libraries; adding libraries to the present topology would leave much of its
complexity intact.

This is a **combined functional inquiry** into developer and operator workflows: CLI behavior,
test architecture and results, process/resource ownership, preparation, compilation feedback
and useful diagnostics. The operator explicitly permits dropping low-value briefs and basic
informational wrappers. It is neither a broad diagnostic audit nor a completion assessment.
Recommendations can change the target, including previously planned guarantees.

**Proposed:** all opportunities and target interactions below. **Interface-checked:** focused
source and library contracts cited below. No production implementation, configuration change,
dependency installation, build, test or benchmark was performed for this review. Historical
closeout records supply observations under their original conditions, not fresh qualification.
Plan59 remains paused; this review does not revise its status or authorize product changes.

The inspected source baseline is `26ac0b893a1659656f08bcfb5e4f92dc701ef4c5`. The checkout had
six untracked library documents, preserved throughout. The applicable standard is Core 3.4,
Efficient Architecture Heuristics 1.0, Test Assessment 1.2, Process Simulator 1.5 and the
pse-arrow binding, selected by
[`standard.toml`](../design_review/design_principles/standard.toml). AP-01–AP-07 and the
test-assessment companion shape the proposals; PS-10/PS-13 retain independent scientific
outcome and reference requirements. No universal new test framework follows from this review.

## Evidence and present strengths

The investigation read the supplied Nextest and testing compendium documents end to end,
and the runtime, shutdown, progress and caching references listed below. Focused current code
and consequential pinned APIs were investigated in parallel. Context7 supplied initial library
lookups; primary documentation, exact release sources and installed sources resolved important
details. An overview's version or capability claim is not automatically true of the selected
toolchain.

| Area | Current evidence and implication |
|---|---|
| Test front door | `cli/testing.rs`, `test_workflow`, `discovery.rs` and `selection.rs` already separate intent, selection, capture, admission and execution. Official `nextest-metadata` is already used. Preserve these useful boundaries; do not invent another test controller. |
| Test interpretation | `extraction.rs::NextestExtractor` reconstructs suite/test lifecycle, retries and selected population from experimental libtest JSON, including a qualified-runner ignored-terminal workaround. This assurance has a real maintenance cost; removing it changes the evidence contract. |
| Report placement | `.config/nextest.toml` configures a profile-shared JUnit location. Concurrent runs need producer-side isolation before JUnit can serve as an invocation's evidence. |
| Physical ownership | `manager.rs` already uses typed zbus proxies, systemd signals, pidfds, unit dependencies and cgroup state. `OwnedUnit::wait` already combines notifications with uncached state reads. The opportunity is simpler topology, not a blanket replacement of polling with signals. |
| Application shutdown | `managed/lifecycle.rs` creates one graceful-shutdown subsystem; `managed/supervisor.rs` separately spawns and joins watchers and passes a payload report through a oneshot. Normal joins exist. Partial delegation is visible; a leaked task is not established. |
| CPU work and readiness | `source_command/batch.rs` already uses tokio-rayon with cooperative cancellation and retained completion. `managed/readiness.rs` already uses sd-notify. Library absence is not the explanation for all friction. |
| Test reuse | `NativeTestCommand::prepare` builds/lists; execution reuses retained binaries and Cargo metadata. No duplicate runtime compilation was established in that list/run handoff. The native fixture route and current CLI route are distinct consumers, not proven duplicate stages of every CLI invocation. |
| Slow feedback | The retained `.local/plan59-closeout-native-reader-tests.log` records a runtime test compile of 3m56s followed by 18 tests in 6.454s. The command selected `pse-runtime --lib` with `canonical-tests,product-reader`, then filtered reader/observation identities. Filtering execution did not narrow the library test compilation unit. |
| Cache and width | Current configuration selects ordinary unbounded placement, 32 Cargo jobs, 16 physical test lanes, mold, incremental workspace builds and nonincremental registry builds. The cache wrapper is configured. Historical cache hit rates, Cargo dirty reasons and critical-path timings were not captured, so cache failure and CPU starvation are unproven. |
| CLI tests | Existing `tests/cli_output.rs`, `catalog.rs`, `source_commands.rs` and `artifact_commands.rs` run real binaries. Existing insta snapshots cover help/response presentation. Improve these owners selectively rather than replace them wholesale. |

Current selected runtime pins include Tokio 1.53.3, tokio-util 0.7.19,
tokio-graceful-shutdown 0.20.0, tokio-rayon 2.1.0, sd-notify 0.5.0 and zbus 6.1.0.
Nextest is selected at 0.9.146 in `.config/diagnostic-tools.toml`; the supplied compendium
discusses 0.9.150. The caching overview's 0.18 baseline requires qualification against current
sccache 0.19 source. Dependency declarations remain authoritative; these are inspected
conditions, not another pin registry.

## O01 — Keep the commands an agent would actually use

I would use precise test selection, original failures and rerun actions; active calculation
state; bounded result/source reads; concrete symbol/dependency answers; and explicit process
inspection/cancellation. I would rarely choose a generic delegation brief, a wrapper around
reading a document, or an automatically assembled inventory of conventions already available
in the working context.

**Recommendation:** remove the planned generic `pse brief`/delegation-packet product and
mandatory session-start brief from the near-term target. Keep `changes` only where it adds
actionable selected-path facts beyond native Git output. Keep owner routing where it locates
the actual generator or contract. Drop/defer citation/scaffold wrappers whose only benefit is
combining a few existing commands. Direct files, native Git, Clap help and current skills are
credible alternatives. Do not turn every capability in Plans59g–i into another summary layer.

Preserve useful focused source intelligence and failure explanations; they answer specific
questions. Remove D01 packet composition dependencies from those producers. Prefer one
summary plus addresses to existing detail, without repeated collection or automatic probes.

Defer D08's bespoke shared-index commit transaction unless actual use justifies its maintenance.
Its concurrency guarantee is substantive: ordinary Git does not become equivalent merely
because this feature is deferred. Keep explicit path/hunk ownership and known publication
limits. Existing D09 evidence remains evidence for its investigated strategy, not a reason to
implement an unused command. Proposed losses are automatic delegation assembly, some
navigation convenience and an integrated concurrency-safe commit interface.

**Delivery example:** after a failed test, I can open its original diagnostic and selected
source, then choose a runnable next action. I need not generate a repository brief first.
Planning owners: [59d](../plans/59d-knowledge-and-workflow-verbs.md),
[59e](../plans/59e-environment-and-adoption.md), affected F/G/H/I composition dependencies.

## O02 — Consume completed native test results instead of reconstructing the runner

Developers need trustworthy selected outcomes, setup failures, retries and useful original
output. Prefer **native Nextest exit + typed inventory + invocation-owned JUnit** as the
completed-run source. Keep a small domain fold for selected identities, setup policy, runner
exit, required output and fixture settlement. Avoid rebuilding the current event machine in
XML.

The exact [0.9.146 JUnit aggregator](https://raw.githubusercontent.com/nextest-rs/nextest/cargo-nextest-0.9.146/nextest-runner/src/reporter/aggregator/junit.rs)
records setup suites and retries and writes at `RunFinished`. A crash can leave no complete
report. The consumer must treat missing, malformed, mismatched or incomplete evidence as
incomplete, never pass. Compare inventory by compiler context, binary ID and test name;
classify setup/skipped populations separately. Native retry information cannot override the
runner's failure policy. Physical fixture drain and scientific acceptance remain separate.

`quick-junit`'s `Report`, `TestCaseStatus`, `TestRerun` and deserialization are a promising
typed reader, preferable to handwritten XML interpretation. Context7 had no matching entry;
primary pages confirmed the mechanism but disagreed about the latest release. An exact
candidate pin and round-trip compatibility remain unresolved, not qualified integration.

**Required isolation:** each invocation/compiler group must produce its report at a private
destination. Copying the current shared report after completion cannot establish who wrote it.
[`junit.path` configuration](https://raw.githubusercontent.com/nextest-rs/nextest/cargo-nextest-0.9.146/nextest-runner/src/config/elements/junit.rs)
is relative to the profile store directory. A captured derived configuration changing only
placement is credible; precedence against repository/tool configuration needs resolution.

Alternatives are retaining the reconciler or embedding `nextest-runner`. The former preserves
independent lifecycle/byte-reference checks at their present cost; the latter replaces JSON
with internal API coupling, configuration and process ownership. Prefer the binary's reporting
boundary under the [upstream stability policy](https://nexte.st/docs/stability/).

Delete replaced `NextestExtractor` event ordering, retry-name reconstruction, ignored-tail
compatibility and mechanism-specific tests. Preserve Cargo diagnostics and required raw bytes;
JUnit's XML-safe output is not a byte-preserving replacement. **Explicit loss:** no independent
proof of every start/terminal transition or event byte address. This requires a contract
decision under blueprint §24.1/§24.2 and Plan59c, not just a parser refactor.

## O03 — Make failure investigation native, addressable and progressive

The useful result view answers: what failed, under which selection, what original evidence
exists, and what can run next. Extend the existing result owner with a bounded failed-case
page, setup/runner failures, native attempt/flaky data, original stdout/stderr addresses and
an explicit rerun selection. Distinguish observer failure from the subject's failure. A
successful result read must not obscure a failed performing operation.

Use `nextest-metadata` for native identities and native filters for execution. Do not infer a
runnable test from a source name or silently rerun to explain an old failure. Stale source,
changed features and changed external fixture inputs are visible mismatches. Retained binaries
preserve one executable population; they do not freeze all scientific inputs.

[Native record/replay/rerun](https://nexte.st/docs/features/record-replay-rerun/), recordings and
timeline links are better candidates than a new test-history engine. Some mechanisms are
experimental or newer than the current selected runner: compare a deliberate version move
with simply linking existing artifacts. Replay explains an old execution; it is not fresh
validation of edited source. No test-results database is justified by an inspected consumer.

**Deletion:** parallel summary/history formats and failure briefs that restate the same source.
**Delivery example:** select one failed case, read its original assertion, inspect its test
identity and choose an explicit rerun with current fixture requirements. Depends on O02's
result contract, not on completing semantic intelligence or all observation work.

## O04 — Put test fixtures in test owners and prepare only selected requirements

A framing/schema test should run without a database; a canonical history test should obtain
its service through the existing fixture owner. The useful sequence is selected build/list,
exact fixture demand, preparation of required inputs, execution and settlement. These are
responsibilities, not a requirement for five executables or repeated metadata passes.

Complete the selected-fixture direction in Plan59c C04/C05 using the existing native fixture
and service owners. Preserve constructed-owned versus borrowed-service modes, selected
provider requirements, setup failure before payload effects, and actual teardown. Native
[Nextest setup scripts](https://nexte.st/docs/configuration/setup-scripts/) can export prepared
inputs where that lifecycle fits; they are not automatically a service teardown owner.
`rstest` fixtures can organize in-test inputs without creating a second external controller.

Compare reusing `NativeTestCommand`/canonical service owners against projecting their actual
capabilities into the selected workflow. Do not duplicate them in the host or rewrite ordinary
test bodies merely to make the CLI understand tests. Retire replaced recipe/controller bodies,
duplicate requirement discovery and obsolete fixture declarations with migrated consumers.
Scientific test changes remain justified by their own assertion/fixture requirements.

**Delivery example:** a mixed selection prepares a canonical service only when an actual
selected test requires one; a support-absent control still exercises absence. No missing
prerequisite is converted to a skip. This integrates current responsibilities from Plan53
without reviving its historical Python controllers.

## O05 — Let Nextest own scheduling and expose meaningful cases

Use native groups, threads, retries, timeouts and fail-fast policy inside one compatible
Nextest invocation. The unresolved Plan59c C07 issue is coordination across independently
split runners: a native named group does not become a global semaphore across processes.
Prefer avoiding unnecessary splits. Where incompatible compiler contexts truly require them,
sequence the affected groups explicitly before designing another scheduler.

Keep 16 physical lanes and ordinary unbounded host placement. A real exclusive fixture or
scientific timing requirement is different from a general CPU cap. Compare orchestration cost
with the value of cross-runner concurrent execution; no performance gain is established here.

Use `rstest` for a useful static case matrix, `datatest-stable` for a file corpus needing
individual selection/timing, and `libtest-mimic` only for genuinely dynamic harness cases.
Do not split every cheap framing loop into another process or replace economical recursive
Clap help snapshots. Delete any custom scheduling/case registry that native mechanisms replace.

**Delivery example:** two tests sharing an exclusive service cannot overlap, while independent
tests retain native parallelism. A corpus failure names the particular fixture without requiring
a custom CLI test taxonomy. See [Nextest test groups](https://nexte.st/docs/configuration/test-groups/).

## O06 — Strengthen actual CLI behavior with established test utilities

Keep existing real-binary tests and semantic JSON/schema assertions. Adopt `assert_cmd` and
`predicates` selectively where repeated status/stdout/stderr code obscures failures;
[`Assert::new(Output)`](https://docs.rs/assert_cmd/latest/assert_cmd/assert/struct.Assert.html)
can consume output from an existing fixture process. Use existing insta for stable views.
Consider `snapbox` for commands with meaningful filesystem effects and `trycmd` if executable
documentation examples become a maintained corpus.

This is a library substitution at the assertion boundary, not a new command-test framework.
Process utility timeouts do not by themselves prove descendant termination. Keep actual
kernel/service lifetime tests where those are the subject. Retire replaced ad hoc assertion
helpers and one-off journey scripts once durable tests cover their behavior; preserve unique
evidence until its owning migration is settled.

**Delivery examples:** malformed invocation has the correct structured error, exit and stream
placement with no effects; reading an incomplete result retains both observation and original
subject failure. Snapshot agreement alone is insufficient for those semantics.

## O07 — Narrow the compilation boundary for reader feedback

The recorded slow reader run compiled the whole runtime library test executable to run a
small reader/observation selection. A name filter cannot remove unrelated test bodies from
that compilation unit. Give public reader protocol contracts a coherent named integration
target using `tooling::reader::{encode,serve,schema}`. Keep genuinely private invariants local;
do not publish internals solely for tests or recompile production source with `#[path]`.

This removes unrelated unit-test bodies, not the ordinary runtime dependency closure.
[Cargo also builds eligible same-package binaries for integration tests](https://doc.rust-lang.org/cargo/commands/cargo-test.html#target-selection).
Inspect that consequence before selecting the target. Runtime compiler, mathematics,
DataFusion and telemetry dependencies have real consumers; this review establishes none as
unused. If dependency weight remains important, compare a meaningful retained-observation
module/feature boundary with a production extraction. A new crate requires the decision route.

Migrate public assertions through real encoding/serving and independently decode expected
results. Avoid calculating an expected identity with the same private helper under test.
Delete migrated duplicates and old fixture-selection entries. Whole-runtime qualification
remains available; a narrower pass establishes only its narrower scope.

**Delivery example:** changing framing runs a named public-contract suite without a canonical
service or unrelated runtime unit-test population. No numerical compile-time reduction is
claimed. This is structural work removal, not evidence that caching failed.

## O08 — Compare Cargo-native helper composition without universal preparation

Current native fixture preparation builds service helpers separately, then builds/lists tests.
Two Cargo invocations do not establish two expensive recompilations; compatible artifacts are
already reusable. Retaining selective construction is a valid low-burden option.

Reopen Plan59c C06 as a bounded comparison with Cargo artifact dependencies: `-Z bindeps`,
`artifact = "bin:pse-canonical-service"`, and `lib = true` if library use is also required.
**Correction for subsequent planning:** cross-package artifact paths use
`CARGO_BIN_FILE_<DEP>_<NAME>`; C06's `CARGO_BIN_EXE_pse-canonical-service` “artifact form” is
the wrong interface. `CARGO_BIN_EXE_*` serves same-package integration tests. See the
[Cargo artifact-dependency contract](https://doc.rust-lang.org/cargo/reference/unstable.html#artifact-dependencies).

An artifact edge can let Cargo schedule the helper in one DAG, but package-wide dev
dependencies can eagerly build it for unrelated tests. Compare a feature-selected optional
edge in the real canonical context, a dedicated harness package, or retaining selective
construction. Do not build a custom Cargo-equivalence planner. Adding two `-p` flags cannot
combine arbitrary helper-bin and runtime-lib selectors without affecting selected roots.

Preserve immutable captured executable identity, source-built fixture provenance, actual
Arrow validation across host/target/artifact units and support-absent variants. Delete only
the helper build/path handoff actually replaced by the selected Cargo edge.

**Decision condition:** select single-DAG construction only if it preserves exact target and
fixture demand without expanding pure reader preparation. No measured saving or elimination
of a repeated Arrow build is established.

## O09 — Separate synchronous ownership from explicitly persistent work

**Preferred target:** ordinary foreground operations use the existing native foreground owner;
explicitly detached operations use one transient systemd service containing a fixed Rust
wrapper, payload and ordinary descendants. The CLI remains with foreground work through
child wait, required output drain and settlement. Closing a detached client leaves its
explicitly persistent job running. These are two lifetime modes, not two generic supervisors.

Use `foreground::run_owned`/native workload ownership for the first. For the second, use
`ManagerProxy::start_transient_unit` and the existing private `UnixCommand` transport, whose
byte-vector program/argv/cwd/environment can feed the wrapper. The fixed wrapper's D-Bus
control arguments need not encode the raw child command. Systemd establishes placement before
the wrapper starts; descendants inherit it. Remove external payload relocation where that
new topology establishes the required containment before effects.

Nested ordinary calls inherit the enclosing owner. Remove independently cancellable generic
nested subtrees as a default feature; truly independent work is an explicit detached launch.
Detachment does not authorize escape from an enclosing finite allocation: independent lifetime
requires an admitted placement/lifetime contract, otherwise inherit the constraints or refuse.
Product scientific cancellation remains with its operation owner. Shared finite envelopes for
separately owned residents still need coherent accounting, including wrapper overhead.

Alternatives are all invocations manager-owned, or the present service-plus-external-scope
topology. The first unexpectedly continues foreground work after client death unless origin
watchers return; the second retains reconstruction to support the full continuity promise.
Direct systemd payload execution is simpler for commands needing only unit status, but does
not replace raw-byte private input and required report interpretation.

**Retire:** foreground generic services, OriginLease watchers, external-payload relocation
gates/recovery, and nested generic subgroup/ancestry machinery whose only purpose disappears.
**Explicit loss:** generic foreground reconnect/history/cancel continuity and crash-written
foreground finalizer documents. Required test/scientific artifacts remain operation-owned. Existing
foreground code needs capture/result integration; the proposed single-service topology is
not qualified. Blueprint §21.7, Plan59 R3/59b and the environment guide need a decision.

## O10 — Give cooperative tasks and forced process termination clear owners

Systemd should own containment and forced termination; Rust should own cooperative
cancellation, child reaping, required stream drain and truthful results. Configure service
`TimeoutStartUSec`/`TimeoutStopUSec` deliberately; consider `KillMode=mixed` when the wrapper
must orchestrate cleanup before remaining descendants are killed. Keep `Restart=no` for
one-shot effectful work. An observation timeout must not silently become a workload deadline.
[Systemd timeout semantics](https://www.freedesktop.org/software/systemd/man/latest/systemd.service.html)
apply to stages, not one universal elapsed-time budget.

Use a retained Tokio `Child` and joined futures for a small fixed flow. If multiple subsystems
really have ordered lifetimes, let `SubsystemHandle::start`/`NestedSubsystem::join` be the
actual owner instead of combining a ceremonial hierarchy with a parallel task registry.
Tokio-util `CancellationToken` signals intent; `TaskTracker::close` does not prohibit new
admission. Neither establishes process-tree completion. Preserve explicit admission cutoff
and joining where required.

Replace `foreground.rs`'s short `std::process::Child::try_wait` polling loops with async child
waiting or pidfd readiness where compatible with exec handoff, raw bytes and reaping. Retain
existing signal/error semantics: graceful-shutdown 0.20's convenience signal installation
logs registration failure, whereas the present route propagates it. Do not weaken that by
blind substitution. Rayon work remains cooperatively cancellable and joined; dropping its
bridge future does not stop running CPU code.

Delete replaced timers/manual watchers/registries with their callers. A minimal `ExecStopPost`
observation can say crash/incomplete, not recreate a missing original child result. O09's
topology determines how much shutdown machinery remains worth improving.

## O11 — Remove automatic host-generation collection from this scope

Keep immutable prepared generations and atomic current selection; remove automatic pruning
from the near-term CLI target. This trades automatic space reclamation for an explicit
administrative cleanup boundary. It can retire collection scheduling and per-entry/finalizer
claims used solely to prove artifact deletion safety. Preserve actual admission/resource
ownership and provenance. Existing unresolved claims/artifacts remain untouched by this review
or by a future migration until their disposition is explicitly settled.

This is preferable to another recovery daemon or systemd FD-store protocol for occasional
developer artifacts. Kernel reference custody is not general artifact-deletion authority.
Do not extend disposable development recovery until it becomes a platform obligation.

Automatic stale-host refresh is a **separate decision**. Explicit `scripts/bootstrap-host`
could remove refresh/re-exec machinery, but adds an operator step and a stale-tool risk.
Removing collection does not imply selecting explicit-only refresh. Compare that optional
change on its own user experience, retaining source identity where correctness needs it.

**Delivery example:** prepare a new host while an old detached operation finishes; its
immutable generation remains available without automated retirement reconstruction. This
requires amendments to blueprint §21.7/§24.1, proposed ADR-0233/ADR-0234 and Plan59a/59e.
It authorizes no present deletion.

## O12 — Make compilation explanations useful without a cache platform

Expose concise existing phase durations and links to original Cargo output, effective
profile/features/targets and optional native timing artifacts. On demand, Cargo `--timings`
can explain dependency waits/critical paths. Distinguish metadata checking, executable
codegen/link, preparation and test execution. A successful check/Clippy run did not necessarily
produce the executable a test run needs.

Keep the current mixed profile pending an actual decision: incremental workspace builds help
edits; nonincremental dependencies can benefit from sccache; final linked executables remain
outside its Rust output-cache path. The exact [sccache 0.19 limitations](https://raw.githubusercontent.com/mozilla/sccache/v0.19.0/docs/Rust.md)
do not justify globally disabling incremental compilation. Shared daemon statistics cannot
attribute misses to one invocation. No persistent cache ledger, smaller CPU partition or
pre-test Clippy warming stage is recommended.

Compare an optional lower-optimization feedback profile only where compile latency dominates
and runtime fidelity permits it. It creates another artifact population and cannot replace
production-shaped scientific/performance evidence. Keep established freshness, graph validation,
one feature context and independent checkout target directories.

There is a smaller bootstrap simplification: `scripts/bootstrap-host` builds binaries and
`generation/bootstrap.rs` invokes construction again. Compare one authoritative producer
that still captures source coherence and publishes complete artifacts. The observed second
build was largely fresh; removing it does not explain the separate 236-second runtime compile.
Delete duplicate invocation only with its coherence duties preserved. See
[Cargo timings](https://doc.rust-lang.org/cargo/reference/timings.html).

## O13 — Show live calculation and process facts through native observations

For active work, show the exact selected unit, phase, current calculation facts and links to
its outputs. Combine systemd `ActiveState`/`SubState` with applicable service/process properties
and available CPU/memory/task accounting, separately labeling capture completeness. Service
`Result`/`MainPID` and invocation identity must not be synthesized for foreground scopes with
different native semantics. Link native status/journal facilities rather than create another
resource model.

Use sd-notify `Ready` after application initialization and `Status` for wrapper phase.
Use actual scientific progress already emitted by the operation; solver iterations are not
necessarily a percentage of completion. Gaps and unavailable cross-process observations stay
explicit. Keep tracing/miette typed diagnostics and source correlation. Optional selected
Tokio console/metrics, Nextest timeline/USDT or profiler artifacts can deepen a particular
investigation without default permanent collectors, watchdogs or restart loops.

Indicatif `MultiProgress`, draw targets and tracing integration can present concurrent work
and suspend rendering around diagnostics. They manage display, not processes, resource
admission or cancellation. Non-TTY/machine output should consume the same facts without
animation or a mandatory renderer task. No fabricated ETA or blanket high-volume tracing.

`object_store` is useful for existing immutable remote/checkpoint/result artifacts, including
range reads and backend operations where supported. It is not a process owner or completion
protocol. Retain local filesystem custody for local private control data; do not introduce
remote log storage, conditional-write coordination or a new backend without an actual
consumer. Capability reuse must respect the one resolved object-store family.

**Delivery example:** inspect a slow calculation, distinguish active solver progress from
capture lag, then open native unit/journal or a selected diagnostic attachment. Build phase
links from O12 do not become a separate build-observability product.

## O14 — Add high-value adversarial assurance at small boundaries

The compendium offers valuable tools, but not a reason to rebuild all tests. Extend existing
Proptest use at byte framing boundaries: generate chunks, EOF placement, malformed data and
limits, then check byte conservation and explicit truncation against independent expectations.
Preserve useful counterexamples as ordinary tests. Do not invest in property infrastructure
for the event parser O02 proposes deleting.

Use independent scientific expectations/reference routes for scientific behavior; runner
reports and same-helper expected values cannot replace them. `trybuild`/`ui_test` are candidates
when compiler diagnostics are the API. Loom/Shuttle deserve a small modeled synchronization
question, not a claim to explore unmodified Tokio/systemd/FFI orchestration. Miri, mutation,
coverage, allocation and CPU tools remain selected inquiries with their own applicability
limits. They do not become universal CLI commands or new default gates.

**Delivery example:** a minimized truncation case has a stable regression test and a useful
byte-level failure. A process cancellation assertion still runs against the actual physical
owner; a model checker does not substitute for that journey.

## Library capability dispositions

These are selection judgments, not an adoption checklist. The supplied overviews remain
repository-agnostic awareness material; current declarations and exact APIs govern adoption.

| Capability family | Disposition and role |
|---|---|
| Nextest metadata/filtering/reports | Prefer native inventory, filters, scheduling and completed reports; O02–O05. Keep exact selection and separate physical settlement. |
| `nextest-runner`, experimental libtest JSON | Internal embedding not preferred. Retain event reconstruction only if its additional assurance has a real retained consumer. |
| `quick-junit` | Candidate typed completed-report reader; exact pin and selected-runner compatibility unresolved. |
| `assert_cmd`, predicates, insta, expect-test, snapbox, trycmd | Select at assertion/presentation/filesystem boundaries; retain meaningful existing tests. No wholesale conversion. |
| `rstest`, datatest-stable, libtest-mimic | Case identity and fixtures where useful; no universal case registry. |
| Proptest/QuickCheck | Existing Proptest is the natural extension point; no second generator needed for the same responsibility. |
| Tokio/tokio-util/tokio-rayon/Rayon | Async wait, cancellation signals and retained joins; CPU work uses the existing execution fit. No second governor or dropped-future completion claim. |
| tokio-graceful-shutdown | Actual subsystem ownership where needed, or remove the wrapper for a small fixed flow. Preserve signal error behavior. |
| zbus/systemd, sd-notify | Native containment, state/signals and readiness; simplify topology before adding manager features. No default restart/watchdog/FD-store platform. |
| Indicatif/tracing-indicatif | Optional rendering over owned observations. Never supervision or result authority. |
| tracing, tracing-error, miette, tracing assertions | Reuse structured diagnostics; assert events only where contractual. No new `anyhow`/eyre error universe in product crates. |
| Tokio console/metrics, OTel/Prometheus | Selected diagnosis or existing application telemetry integration. No always-on backend prerequisite for local tests. |
| Nextest recordings/Perfetto/USDT, debuggers/profilers | Prefer native attachments and explicit escalation; version/support boundaries remain visible. |
| Criterion/iai-callgrind, pprof/flamegraph, dhat/Valgrind | Use when a concrete performance/allocation question needs them. No mandatory campaign follows from a reasoned work-removal proposal. |
| Loom/Shuttle, Miri, coverage/mutation, compiler UI tests | Targeted assurance with explicit model/platform scope; not broad proof or default gates. |
| `object_store` | Existing data/checkpoint roles; no speculative control-plane migration. |
| Serde/serde_json/Schemars | One typed domain result and derived schema/view adapters. Avoid separate handwritten status, report and presentation meanings. They cannot make external process/report failure impossible at compile time. |
| cargo_metadata/gix/source intelligence | Reuse exact selected context and native Git facts for a focused next action. No generic briefing subsystem is needed. |
| Result databases/analytics frameworks | Defer: no inspected use justifies another result store or analytical authority. |

Primary leads supplementing the exact sources above include
[quick-junit](https://docs.rs/quick-junit/latest/quick_junit/),
[Tokio child ownership](https://docs.rs/tokio/latest/tokio/process/struct.Child.html),
[tokio-util task tracking](https://docs.rs/tokio-util/latest/tokio_util/task/struct.TaskTracker.html),
[graceful-shutdown subsystem ownership](https://docs.rs/tokio-graceful-shutdown/latest/tokio_graceful_shutdown/),
[indicatif](https://docs.rs/indicatif/latest/indicatif/), and
[object_store](https://docs.rs/object_store/latest/object_store/).

Reviewed local capability sources:
[Nextest](../libraries/nextest.md),
[testing compendium](../libraries/rust_testing_library_compendium.md),
[zbus/systemd](../libraries/zbus_and_zbus_systemd.md),
[sd-notify](../libraries/sd-notify.md),
[Tokio](../library_leverage/libraries/tokio/tokio.md),
[tokio-util](../libraries/tokio-util.md),
[tokio-rayon](../libraries/tokio-rayon.md),
[graceful shutdown](../libraries/tokio-graceful-shutdown.md),
[progress](../libraries/indicateif.md),
[caching](../libraries/rust_caching.md), and
[object storage](../libraries/object_store.md).
The earlier supplied [Schemars](../libraries/schemars.md) and
[Serde/JSON](../libraries/serde_serde-json.md) material informs typed result/view composition.

## Integration, order and proposed rule changes

First decide which guarantees are worth keeping. O01/O09/O11 can remove substantial work
before further implementation. O02 then fixes the result source; O04/O05 place selected setup
and scheduling with the native/test owners. O07/O08 improve feedback without another test
platform. O03/O06/O12–O14 add focused usability and assurance at those settled boundaries.
O10 follows the actual chosen topology rather than polishing machinery O09 removes.

Do not require all opportunities to ship together. Native completed reports can improve the
current execution owner before the topology changes; synchronous and detached consumers must
still share the same performing-result meaning. Scientific workflow ownership, Python
references and database product state are not redesigned here.

| Proposed decision | Authority/plan impact | Preserved obligation or explicit loss |
|---|---|---|
| Narrow runner reconciliation to native completed evidence, O02 | Blueprint §24.1/§24.2; Plan59c C02/C03; affected Plan53 result consumers | Keep actual selection, original failure, required setup/capture/drain. Lose independent event lifecycle/byte-address assurance. |
| Synchronous foreground and single-service explicit detach, O09/O10 | Blueprint §21.7; Plan59 R3/59b; applicable host ADRs and environment guide | Preserve physical containment/cancellation/settlement. Lose default generic foreground reconnection and independent nested subgroup control. |
| Remove automatic generation collection, O11 | Blueprint §21.7/§24.1; proposed ADR-0233/ADR-0234; Plan59a A05/59e E01 and retirement consumers | Keep immutable artifacts/provenance and explicit administration; accept disk retention. Existing claims are not disposal authorization. |
| Reduce command portfolio, O01 | Plan59d/e and dependent F/G/H/I composition requirements | Focused diagnostic/source capabilities remain useful. Explicitly defer convenience and specialized commit guarantees. |
| Narrow production/test boundary or Cargo artifact edge, O07/O08 | Plan59c C04–C07; relevant Plan53/54 owners | Preserve selected requirements, support-absent variants, exact feature context and Arrow validation. New crate/boundary changes use the existing ADR/design route. |
| Add selected libraries and diagnostic attachments | Existing exact-pin/dependency and test routes | No blanket ADR for ordinary library adoption; no new default gates or installation authority from this review. |

The proposals use AP-01/AP-06/AP-07 and H4/H5/H12/H18–H20/H24–H28 to reduce semantic owners,
optional participants and auxiliary work. They do not waive the zero failure baseline,
force-validation, independent science, one type universe, exact pins or operator-selected
finite/scientific placement. Current authorities still govern until amended through the
existing plan/decision routes.

## Scenarios that would demonstrate delivery

These are proposed consumer examples, not a newly mandated qualification campaign.

| Journey | What must be observable |
|---|---|
| Pass, assertion failure, setup failure, retry/flaky, ignored/default filter and zero match | Native policy and exact selected identity remain visible; setup failure and absence never become a test pass. |
| Two concurrent runner groups | Each report belongs to its producer; a shared stale JUnit file cannot supply another run's result. |
| Runner crash, malformed report or cancellation | Partial evidence stays useful; missing terminal data remains incomplete. No synthetic successful finalizer. |
| Read failure then rerun | Original command/outcome is prominent; rerun is explicit and detects changed source/context/fixture assumptions. |
| Reader-only versus canonical selection | Pure public protocol tests avoid service/provider preparation; canonical demand reaches the existing fixture owner with owned/borrowed modes intact. |
| Repeat a selected build/run | Native retained metadata avoids redundant list/build within the invocation. Cache eligibility is distinguished from actual reuse; no promised hit rate. |
| Kill foreground client versus close detached observer | Foreground descendants settle; explicitly detached work continues and remains inspectable/cancellable. Missing output remains incomplete. |
| Raw-byte command and nested finite allocation | Exact argv/cwd/private environment survive; aggregate placement includes wrapper costs. Lost independent nested cancellation is explicit. |
| Prepare newer tools during an older operation | Old immutable artifacts remain usable. No automatic generation collection runs. |
| Diagnose slow build or calculation | Existing phase/timing/native unit facts identify the next useful investigation without a generic brief, mandatory collector or another governor. |

Planning should settle report isolation/reader compatibility, foreground capture integration,
single-service containment and finite-sharing behavior, and target-specific Cargo artifact
composition before selecting those implementations. No timing improvement, cache-hit increase,
process-safety proof or assembled product qualification is claimed by this review.
