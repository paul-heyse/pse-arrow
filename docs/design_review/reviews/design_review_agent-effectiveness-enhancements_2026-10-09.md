---
title: Agent effectiveness enhancements
date: 2026-10-09
tier: design
purpose: target
baseline: d0f2c41818a34539a910654dfea4760771603f45
reviewer: independent design reviewer
---

# Agent effectiveness enhancements

## 1. Integrated assessment and scope

**Decision: Revise the agent workspace's command and observation boundaries.** The existing environment owner, native tools, host lifecycle ownership and assessment runner provide a useful foundation. Five bounded defects prevent the assembled surface from consistently delivering the target: an advertised test-selection entry point cannot import its runner; some recipes reinterpret native argument values as shell syntax; a forwarded code-generation check flag leaves mutating stages enabled; failed host observations become empty successful observations; and the recommended activity route needs workload admission before it can explain that admission.

These findings do not justify replacing the workspace with a generic agent workbench, command registry or job service. Correct the existing owners, retain native tool access and select additional capabilities for a concrete question. Optional Rust semantic navigation and the active SurrealDB instance's native MCP endpoint deserve consideration; neither is necessary to repair the five defects, and neither has demonstrated an agent-quality or productivity improvement here.

The functional target is less effort forming commands, fewer environment complications, parameterized related actions, readable truthful feedback and freedom to use native tools. This is a Linux-host-specific personal hobby workspace, primarily operated through Codex, with limited Claude use. The review covers the broad agent-workspace subsystem: instructions and runtime configuration, recipe discovery and composition, environment selection, existing admission/recovery/visibility, test selection, runner evidence and useful code/database navigation. It includes the neighboring scientific APIs needed to distinguish generic administrative tools from typed PSE operations.

The selected standard is **Core 3.4**, **Heuristics for Efficient Architecture 1.0**, the **pse-arrow binding**, and **ProcessSimulator 1.5 where applicable**. This is a design-tier, target-purpose review. All seven foundations are assessed. Current repository rules are the design being evaluated, rather than independent acceptance criteria; recommendation-dependent rule changes appear in §11.

The production baseline is clean `main` at `d0f2c41818a34539a910654dfea4760771603f45`. Concurrent work during the review adds evidence documents only. Plans 29 and 30 supply completed historical context and named qualification receipts. Their completion does not qualify this review's newly identified paths. Broader Plan 28 campaigns remain paused and are not resumed by this assessment.

No new resource, concurrency or runtime caps are proposed. Existing allocation, ownership, recovery and observational behavior are assessable. This is not a whole-simulator architecture review, numerical certification, full product qualification, fresh runtime adoption, configuration change or implementation authorization.

The independent reviewer read the shared worker/reviewer contracts, selected standard and skills, relevant contract/source owners, capability evidence and coordinator-produced raw interface controls. The sibling corpus-intelligence review was read as ideas and methods. Its runner defects, product boundaries and recommendations are not transferred findings in this repository. This review's findings have local causes and evidence.

**Disposition owner:** this review pending operator selection. Once adopted, the selected owning plan takes current disposition status. Completed Plans 29/30 and paused Plan 28 do not acquire new work through this document.

## 2. Responsibilities and the governing workspace model

The domain is executing and understanding an operation in a selected checkout and context. Its consequential distinctions do not require a universal workflow language:

- A **command invocation** carries an executable, an ordered argument vector, explicit choices and intended effects. Shell source is a different input.
- A **checkout environment** supplies tool/configuration defaults; **native capability preparation** is requested separately.
- An **allocation** identifies a live host owner and its actual process lifetime. An environment marker is not ownership by itself.
- A **test selection** identifies requested work; an enumerated inventory, terminal report, cleanup disposition and qualification claim answer different questions.
- A **current observation** may be complete, partial or unavailable. Empty observed state is meaningful only after successful observation.
- An **application operation** carries PSE identities, codecs, revision and publication meaning. Generic database access does not acquire those semantics.
- A capability can be **installed**, **configured**, **enabled**, **exposed to a session**, or **exercised**. These states are not interchangeable.

These distinctions mostly govern the current implementation. The findings concern specific places where their realization is incomplete.

| Owner | Responsibility and consumed contract | Independent reason for change |
|---|---|---|
| `justfile` | Discoverable thin bindings and related-action compositions; native arguments, selected modes and prerequisites. | Useful binding, command grammar or composition change. |
| `scripts/pse_env.py`, `build_environment.py`, `native_operation` | Compose environment with explicit precedence; prepare requested capabilities; preserve command exits and separate boundary failure. | Environment/tool installation or native preparation behavior. |
| `scripts/host_admission.py` | Existing allocation selection, ownership, short metadata decisions and actual drain/recovery. | Existing host policy or lifecycle realization. |
| `scripts/activity.py` | Read current units, checkout build processes and limits; disclose observational uncertainty. | Host observation or presentation behavior. |
| `scripts/select.py`, `affected.py` | Native nextest command construction and package/change selection. | Selection policy and argument handling. |
| `validation.py`, `validation_scope.py`, `validation_receipts.py` | Existing gate composition, retained logs, selected/terminal reconciliation and scoped evidence/reuse. | Qualification scope, receipt semantics or result presentation. |
| `test_run.py`, `native_tests.py`, `test_resources.py` | Bind runner selection to fixtures, separate preparation from observer execution and retain incomplete/failed resources. | Test lifecycle and actual artifact association. |
| `xtask/src/main.rs`, `codegen.rs`, producer identity | Rust-owned commands, generation comparisons and deployment/artifact identity. | Rust API or declaration/generator behavior. |
| Shared roles, `.codex`, `.claude`, skill selection | Assignment boundaries, runtime defaults and optional capabilities. | Concrete runtime or coordination need. |
| Typed Rust/Python scientific APIs | Inspect authored models, run/attempt records, result manifests and diagnostic/Arrow streams. | Scientific operation or boundary meaning. |

Dependencies are generally coherent. Recipes compose executing owners; environment defaults have one implementation owner; host ownership is not inferred from a caller's claim; receipts retain their original evidence rather than becoming a new execution; generated outputs remain derived from declarations. A new related action should bind existing capabilities at these owners. A genuine new scientific operation belongs to its scientific owner, not to recipe string manipulation.

## 3. Contracts worth preserving

**Implemented, source-inspected:** `pse_env.compose` honors caller/local/default precedence, reports selected refusals and keeps secrets out of explanatory/export projections. Native preparation is explicit. `host_admission.inherit` checks boot, process/cgroup membership and unit invocation identity; nested commands cannot enlarge an enclosing role merely by carrying an environment variable. Allocation reconciliation and release distinguish launcher death from actual kernel drain. These are useful boundaries, not interchangeable with an arbitrary command prefix.

**Implemented, source-inspected:** validation writes directly to an identified run log, supports concise output and optional live following, continues independent gates after failure and retains per-gate status. `validation_receipts.reconcile` refuses mismatches between selected and executed identities. Reuse identifies the original evidence and validates its premises; reviewed transfer is explicit rather than reported as a new test. `test_resources` retains incomplete, failed, referenced or manually pinned resources. A bare command without a runner association does not infer successful cleanup.

These are scoped artifact/runner guarantees. They do not establish generic detached-process ownership, reconnect delivery, absence of every descendant after arbitrary process escape, uninterrupted reboot availability, scientific correctness or physical power-loss survival. Managed service/cgroup guarantees and application scientific outcomes retain their own owners and evidence.

**Implemented, source-inspected:** the Python extension routes distinguish installed kind, actual imported path, linked bytes and stronger deployment association. `doctor.check_extension` checks import/lockfile observations; it does not establish that every Rust source edit is included in the installed extension. Ordinary Python operation after a Rust edit still needs the documented rebuild. Stronger producer/deployment controls are separately selected. That distinction is honest; it is an opportunity for clearer prerequisite feedback, not a finding that all Python test results are invalid.

**Implemented, source-inspected:** `ModelingPackage.inspect/source_tables`, `Runtime.resource_usage/run_record/attempt_record/result_manifest`, run diagnostics/table access and `registry_table/TableStream` already expose useful typed tasks. Streams have explicit consumption/cancel/close semantics. These APIs are the simplest existing route when the question concerns scientific source, lineage or outcome meaning.

## 4. Scenarios that distinguish alternatives

| ID | Stimulus and kind of change | Expected boundary and observed consequence |
|---|---|---|
| S01 | Execute an ordinary targeted Rust selection from a normal checkout environment; binding/composition. | Recipe reaches shared selector/runner without caller `PYTHONPATH` repair. Current direct-script entry fails before command construction, F01. |
| S02 | Pass a state/output path containing spaces or punctuation, or a multiword native filter; instance/binding. | One input argument reaches the native owner unchanged. Current interpolation splits a legitimate state path, F02. |
| S03 | Request generation comparison rather than publication; policy/mode. | Compare the selected generated outputs or reject an unsupported composition before effects. Current `codegen --check` retains mutating bootstrap/final stages, F03. |
| S04 | Explain busy admission or unavailable user-manager transport; observation/failure. | Obtain useful observations independently of the unavailable workload slot, preserving unavailable versus empty. F04/F05 obstruct this. |
| S05 | Add a related analysis/inspection action or substitute a tool implementation. | Reuse native CLI/API capability; bind only additional semantic meaning. Existing typed inspection, nextest and just discovery are preferable to copying tool grammars or a second catalog. |
| S06 | Resolve an alias, trait method or same-spelling Rust symbol, or inspect the deployed database schema/context; optional capability. | Select the fitting semantic tool and context explicitly. Analyzer preparation and raw database operations have different scopes from text matches and scientific results. |
| S07 | Interrupt a selected test/assessment or retain evidence across changes; lifecycle. | Preserve terminal incompleteness, actual drain and evidence identity. Existing managed runner and resource owners supply credible routes; unexamined generic detachment is not claimed. |

Growth is relevant principally in source-read volume, repeated command discovery, unit/process counts, test inventory and repeated log retrieval. Existing filtered native discovery and identified logs avoid a need to materialize another full command or result inventory. No workload premise here demands a new resource limit, universal trace service or cost model.

## 5. Local evidence and recent use

The [supporting evidence](../evidence/agent-effectiveness-enhancements-2026-10-09/README.md) separates capabilities, live host observations, bounded interface controls and retrospective activity. The independent reviewer inspected [`probe-interfaces.py`](../evidence/agent-effectiveness-enhancements-2026-10-09/probe-interfaces.py) and [`interface-observations.json`](../evidence/agent-effectiveness-enhancements-2026-10-09/interface-observations.json), rather than deriving verdicts from a worker's conclusion.

**Tested as negative interface controls**, coordinator execution on this baseline: direct selection fails importing `scripts`; a quoted Surreal state path splits and produces native parser exit 2; synthetic failed `systemctl` produces activity exit 0 with empty units; synthetic unavailable admission reaches acquisition once and executes no observer. These reproduce the stated defects, not passing production acceptance. No product tests, service failure/saturation experiment or generator execution was performed for them. The generator dry-run is **Interface-checked** command-composition evidence; actual hidden mutation was not executed.

The first synthetic admission control stopped at a compiler-cache prerequisite and did not reach admission. The corrected control preserves unrelated environment prerequisites and removes only allocation/selection inputs. The retained development note prevents the earlier attempt from being counted as admission evidence.

**Interface-checked:** installed Codex 0.161.0 and just 1.58.0, the capability worker's native help/source checks, and the reviewer's own Context7 resolution/query of just and rust-analyzer. Current documentation is reconciled with installed/pinned contracts; it does not supersede exact source compatibility. Just already supplies usage, show and a structured dump. Its positional-argument facility is already used correctly by several repository recipes.

**Tested, narrow endpoint exercise:** authenticated HTTP MCP initialization and tool listing succeeded against the existing 3.3.0 server at port 18240, followed by constant `RETURN 1`. The schema-list call returned an explicit `NotFound` tool error for `pse/canonical`; outer HTTP 200 and absence of a JSON-RPC error did not make that call successful. This establishes the deployed MCP route and that error distinction, not canonical schema presence or scientific data access. Server liveness/ownership, native WebSocket application readiness and agent-facing HTTP MCP are separate observations.

The fixed [retrospective](../evidence/agent-effectiveness-enhancements-2026-10-09/retrospective/retrospective.md) window is October 6 through October 9, 06:52:57 America/New_York, excluding this review's root and descendants. It covers 108 attributable Codex sessions in six root trees and records 24,377 shell launches. These are clustered activity, not independent task or test denominators. Native exits were recoverable for 16,204 launches; wrapper completion, yields, polls, native exits and explicit test outcomes remain distinct. The extraction's parsing/attribution diagnostics and dynamic-command gaps prevent converting its classifications into effectiveness or failure rates. Repeated output call IDs retain the first payload without establishing equality; later stream/notify evidence may be omitted. The finalized portable/gzip extractor was interface-checked but not rescanned; retained counts identify their original extraction provenance.

Checked traces show recipe/selector recovery, native-library/store prerequisite refusals, repeated assessment-log retrieval and some real Cargo waiting. They support targeted convenience questions. They also show successful native selection and actual targeted passes after appropriate setup. Changing runtime/store code and fixture state are competing explanations for refusals. Repeated reads may be preservation or progress checks. Tool-mention absence does not prove semantic navigation was unavailable or needed. The older Plan 29 comparator has different parser/window/policy composition and cannot establish a causal before/after result.

The sibling corpus review usefully suggests distinguishing native exit from wrapper completion, testing error paths, using filtered native discovery and keeping live tool exposure distinct from installed capability. Its specific cleanup/terminal-backpressure findings were not reproduced or inferred here. No claim is made that stale documentation caused retries or that these recommendations will yield a measured speedup.

## 6. Architectural foundations and independent gates

| Foundation | Verdict | Scoped reason |
|---|---|---|
| AP-01 Separation of concerns | **violated at diagnostic launch** | F05 makes host observation depend on unrelated workload/compiler preparation; otherwise owners separate environment, lifecycle, selection and evidence coherently. |
| AP-02 Stable contracts | **violated** | F01/F02/F03 break the consumed entry-point, argument and check-mode contracts. Native interfaces remain good replacement boundaries. |
| AP-03 Composition | **violated** | F03 composes mutating and checking operations without a coherent overall mode; F05 confuses observation with admitted work. |
| AP-04 Domain model/authority | **violated in invocation/observation realization** | Ordered argv versus shell source, checking versus publishing, and unavailable versus empty are consequential distinctions not consistently preserved, F02–F04. Existing allocation/selection/evidence meanings are adequate and actively enforced. |
| AP-05 Explicit structure/constraints | **violated** | F02/F03 leave argument effects/mode implicit; F04 erases observation completeness. Ownership and explicit native/test capability constraints remain strengths. |
| AP-06 Local reasoning/testability | **satisfied for inspected responsibilities** | Local functions and isolated controls exercise the relevant contracts without unrelated scientific startup. Corrections fit existing owners. This is not an assertion that all integration paths are independently unit-testable. |
| AP-07 Execution fits workload | **violated in S04** | A diagnostic query cannot execute through the recommended route when admission is unavailable, F05. Incidental shell presentation also changes existing class selection. Existing logs, retained inventories and scoped native reuse otherwise have credible physical routes; no unmeasured throughput claim follows. |

| Gate | Verdict | Evidence or scope reason |
|---|---|---|
| G1 Authority | **pass, scoped** | Environment, allocation, selection, generated declaration and evidence owners are identifiable; no independently writable competing authority was established in inspected paths. |
| G2 Semantic fidelity | **fail** | F02 drops argument boundaries; F04 turns unavailable observation into empty state. |
| G3 Validity | **pass, scoped** | Inspected allocation/native/receipt paths reject invalid ownership, unsupported choices and incomplete evidence. F01 is an unavailable promised entry point; this pass does not qualify unexamined application validity. |
| G4 Hidden behavior | **fail** | F03 allows checking intent to retain publication effects. |
| G5 Consistency/recovery | **fail at check composition** | F03 may rewrite drift before the comparison, so a later check cannot establish that the original outputs matched. Inspected actual-drain and incomplete-evidence recovery remain preservation constraints. |
| G6 Transformation/reuse | **fail at invocation lowering** | F02 changes native argument meaning. Existing retained-evidence reuse is separately sound at inspected scope. |
| G7 Truthful capability claims | **fail** | F01's advertised route is unavailable; F04 cannot support an empty-host assertion after failed observation. |
| G8 Library leverage | **pass, scoped** | Native just/nextest, standard subprocess/files/signals, systemd and existing SDK/API routes were compared. No clearly superior complete replacement requiring a wholesale new workbench was established. Existing library capabilities can repair the specific seams. |
| G9 Architectural fitness | **fail** | Individual violated foundations are not offset by the workspace's positive ownership and recovery design. |
| PS-G1 Physical consistency | **not applicable to subsystem acceptance** | No physical quantities, model conventions, property envelopes, balances or scientific boundary conversion is changed or certified. |
| PS-G2 Well-posedness | **not applicable to subsystem acceptance** | No variable-role or structural solver admission is reviewed. Command/test selection does not establish those checks. |
| PS-G3 Numerical integrity | **not applicable to numerical acceptance** | No solver algorithm, derivative, convergence or new scientific result is qualified. PS-12's honest-scope concern informs preservation of separate test/qualification claims through core G7. |

Physical-semantics, well-posedness and numerical-stage tables have no scoped numerical operation to describe. Existing simulator conformance suites and Plan 30 scientific receipts remain historical at their named scope; none was rerun here.

## 7. Findings and correction boundaries

### <a id="f01"></a>F01 — The advertised targeted selector fails before selection

**Cause and evidence — Implemented; negatively Tested in S01:** `scripts/select.py` imports `scripts.test_run` before its direct-script `sys.path` bootstrap. `unit-package`, `unit-libraries` and native selection recipes invoke that file directly. The probe's ordinary `.venv/bin/python scripts/select.py --help` exits 1 with `ModuleNotFoundError`; module invocation reaches the selector's usage response. The absence of a usable help option is separate from the import failure. No test binary was built or run.

**Consequence:** an agent using the documented targeted route must diagnose Python import context before it can form or execute the nextest selection. Historical Plan 29 success is not current route evidence. AP-02, DP-15/DP-24, G7.

**Proposed correction:** the selector/recipe owner provides one valid entry-point contract: establish package context before imports, or consistently use module invocation. Preserve bare-word/filterset handling, explicit force-validation, package intersection, command printing and the existing shared terminal runner. Do not repair callers by requiring ambient `PYTHONPATH` or add another selector.

**Closure:** the normal recipe entry reaches command construction from a clean checkout import context, and the relevant selection reaches the intended package/tests. An isolated subprocess/import control can settle startup without compiling the workspace; targeted execution settles selection separately. A script-only import check that bypasses the actual recipe is insufficient.

### <a id="f02"></a>F02 — Some thin recipes turn native argument data into shell source

**Cause and evidence — Implemented; negatively Tested in S02:** `surreal command *args` quotes the command but interpolates `{{ args }}` into shell source. `worktree`, `activity`, bundle wrappers and several other inspected recipes use the same pattern. The Surreal probe passes `/tmp/pse review absent-state` as one argv item; the dry-run emits it unquoted and the native parser rejects the split remainder. Source establishes that shell-significant punctuation may also be reinterpreted; no shell-effect payload was executed.

**Consequence:** valid state/output/filter values need a second quoting interpretation, and intended argument data can become unintended effects. This contradicts the target's related-action parameterization and native tool freedom. AP-02/AP-04/AP-05, DP-15, G2/G6. The claim covers the inspected interpolation paths, not an exhaustive proof that every recipe loses arguments.

**Proposed correction:** use native just positional arguments and quoted forwarding at the recipe boundary, as `unit-package` and `py-test` already do. Keep native parsers authoritative. A fixed semantic parameter can be bound deliberately; arbitrary forwarded arguments retain order and bytes. If a recipe deliberately accepts shell source, name that distinct contract explicitly rather than silently imposing it on `*args`.

**Closure:** a harmless argv witness receives exact values containing spaces, quotes and shell punctuation through the real bindings; native multiword filter expressions remain one argument. Native invalid arguments still reach native rejection. No bespoke quoting parser or duplicate CLI schema is required.

### <a id="f03"></a>F03 — Codegen checking mode leaves mutating stages enabled

**Cause and evidence — Implemented; Interface-checked in S03:** `just codegen --check` forwards `--check` only to the full generator. Its preceding Rust-contract and Surreal generators still write; subsequent `cargo hakari generate` and `manage-deps --yes` still mutate. The dry-run reproduces that composition. `xtask::Cmd::Codegen` and `codegen::run` define checking as scratch comparison and rejection of drift/untracked outputs. No generation or mutation was executed in this review.

**Consequence:** an agent can reasonably request native checking semantics and receive publication effects, including changes before a later comparison. A successful later comparison cannot establish that the original generated outputs matched. AP-02/AP-03/AP-05, DP-18/DP-23, G4/G5.

**Proposed correction:** the existing generation composition owner decides the whole operation's mode before any side effect. Either honor comparison consistently or reject an unsupported mode immediately and point to the existing `codegen-check` route. Keep self-ordering bootstrap for an explicitly mutating generation operation, and preserve native/bespoke schema checks and hakari ownership. Renaming a stage or checking only the last command does not repair the composition.

**Closure:** a check request leaves generated outputs, manifests and dependency declarations unchanged, and detects relevant stale output rather than refreshing it before comparison; unsupported combinations refuse before writes. An ordinary generation request still produces the complete replacement outputs. Existing native check tools are preferable to a new publication framework.

### <a id="f04"></a>F04 — Unavailable host observation is reported as empty success

**Cause and evidence — Implemented; negatively Tested in S04:** `activity.systemctl` discards return code and stderr; `units` interprets empty stdout as `[]`. The isolated failed-systemctl control returns activity exit 0 and an empty unit list. The observation contains no indication that unit retrieval was unavailable.

**Consequence:** an agent cannot distinguish an idle host from failed inspection and may treat the report as evidence for a next action. AP-04/AP-05, DP-21, G2/G7. No actual user-manager outage was induced, and no claim is made that a currently running unit was missed in the live healthy query.

**Proposed correction:** the activity owner preserves each observation's availability/completeness and failure cause. A failed unit query does not become successful absence; available `/proc` or limit observations can still be reported as partial information. Keep concise human feedback and structured JSON over one observational meaning. Do not turn the observer into a second lock/admission ledger or infer ownership of bare Cargo locks.

**Closure:** healthy empty, healthy populated and failed/partial queries are distinguishable in JSON, human feedback and the command outcome. Native errors retain a useful cause; partial output never claims complete emptiness.

### <a id="f05"></a>F05 — Diagnostic launch depends on the work admission it diagnoses

**Cause and evidence — Implemented; negatively Tested in S04:** the documented `pse-env` route composes compiler defaults and requires allocation before invoking activity. `host_admission.classify` calls activity functional; the corrected synthetic unavailable-admission control reaches acquisition once, returns 125 and never executes the observer. The same source classifies direct `cargo check` as compile and shell-wrapped `cargo check` as functional because classification consumes incidental argv positions rather than the operation's meaning.

**Consequence:** when an agent needs to explain admission pressure, the recommended activity route cannot observe it. The agent must already know a bypass or unrelated compiler repair to get the diagnosis. Shell presentation also changes existing resource selection for equivalent work. AP-01/AP-03/AP-07, DP-18/DP-21, G9. This is an existing routing/lifecycle defect; it is not evidence for new caps, proof of current live saturation, or a measured slowdown.

**Proposed correction:** separate read-only environment/admission diagnosis from workload admission at the existing launch/observer owner. Supply the environment facts that observation actually needs and retain honest unavailable results under F04. Bind the existing operation class at an owner that knows the operation, preserving explicit caller choices and verified nested ownership; do not infer a universal command grammar by parsing arbitrary shell source. The simplest diagnostic alternative is the existing activity program invoked directly under a documented observation contract. The simplest work alternative is explicit existing class binding, not a new scheduler.

**Closure:** unavailable work admission does not prevent the observer from attempting its read-only queries; compiler prerequisites not consumed by observation cannot block it. Direct and recipe-bound forms of the same work select the intended existing class, while genuine native/managed/exclusive operations retain their existing admission and actual-drain guarantees. No profile values need change.

F02/F03 both concern invocation composition, but differ: argument fidelity does not establish whole-operation effect mode. F04/F05 both concern observation, but differ: bypassing workload admission does not make failed transport truthful, and truthful transport errors do not make the observer reachable during admission failure. F01 is a localized entry-point regression. These corrections fit together at existing owners without a new authority or universal runner.

## 8. Native/library fit and optional opportunities

All opportunities below are **Proposed**, unscheduled and independent of defect closure. Their benefits are hypotheses about removed effort, not measured gains.

| Opportunity | Intended consumer and simplest direction | Machinery, benefit and reopening/closure condition |
|---|---|---|
| Filtered command discovery | An agent needs a recipe's native argument/default/body contract. Use just usage/show or filter its transient JSON dump. | Already installed and exercised discovery; no persistent second command catalog. Improve a concrete help/example gap rather than all recipes indiscriminately. |
| Assessment option ergonomics | An agent wants flags without an output path. The leading optional `output` currently consumes `--group`; native validation and bundle routes already work. | Prefer flag forwarding or clear existing examples over a new dispatcher. The failed flag-shaped call is real; it is not proof that the documented grammar is unusable. Closure is a predictable grammar/help pair and exact native selection. |
| Identified result retrieval | An agent resumes a long run or wants only failed selections/next actions. Bind the existing run path, JSON and logs; use selective reads. | Repeated tails are a convenience lead, not demonstrated waste. A small read command may help; a service/database is unnecessary for a few owned files. Preserve exact run/receipt identity and incomplete status. |
| Selected-run fixture feedback | An agent uses affected/raw nextest paths and wonders which resources remain. Offer the existing runner association where useful, or make bare-run retention expectations visible. | Raw tools remain supported. Missing runner association is intentionally conservative, not an automatic validity defect. Avoid forcing every native command through a generic wrapper. |
| Python extension freshness | An agent changed Rust and needs a Python journey. Link installed kind/path observations to the owning rebuild and stronger qualification route where required. | Use existing producer/deployment controls when their guarantee is needed. Do not equate lockfile match with source freshness or automatically rebuild before every Python query. Closure is truthful prerequisite guidance and a consumed guarantee, not another freshness cache. |
| Typed PSE task examples/bindings | An agent inspects model topology, run/attempt state, result manifest or diagnostics. Compose the existing typed API and export/stream contracts. | A small owner-linked example or parameterized binding can remove repeated script formation. A custom MCP adapter is justified only by an actual consumed semantic task and must preserve typed identities/results. |
| Optional Rust semantic navigation | An agent needs definition/reference/type resolution across aliases, macros or trait dispatch. Evaluate the installed analyzer bridge or editor LSP beside `rg`/focused reads. | Bridge 0.4.0 is installed and registered, but project-disabled and absent from this session's tool exposure. It starts analyzer/Cargo preparation; effective features, target directory and native environment need verification. Include correct and ordinary cases; examine omissions/startup cost before adoption. No new HIR service or whole-workspace extraction is necessary by default. |
| Existing SurrealDB HTTP MCP | An agent needs database context/schema or a bounded administrative diagnostic. Direct registration can use the active `/mcp` endpoint. | No new datastore/bridge process is required; authentication and selection are real obligations. The constant query was exercised, canonical schema inspection was not successful. Tool names/allowlists do not make generic query read-only; use an appropriate principal and exact context. Compare native CLI/HTTP and typed application inspection for the actual question. |
| Compiler feedback/profiling | An agent needs structured compiler spans or explains a slow build. Use Cargo JSON diagnostics and scoped native timings/self-profile. | Existing native tool contracts avoid a duplicate diagnostic authority. Compilation profiling is distinct from scientific runtime measurements; no profile or claimed speedup was produced here. |
| Capability/coordination routing | A worker needs its owner, baseline, effects and current status. Reuse existing role briefs and owner links. | The retrospective shows effective native routes as well as mistakes. No additional standing startup tour, mandatory trace, agent ledger or injected workflow is established as necessary. |
| Task-focused runtime exposure | A coding session repeatedly sees unrelated capabilities or needs a currently disabled tool. Select relevant optional capabilities at the existing runtime configuration boundary. | The root's 360 exposed metadata entries are an inventory, not measured token overhead or selection harm. Compare a real coding task before narrowing defaults; preserve native shell tools and explicit session choices. No new runtime cap or forced plugin policy follows. |

The comparative priority is to use **existing filtered discovery and identified result reads first**, because they require no new process or schema and address observed command/progress questions. For example, obtain the parameter/body contract for `unit-package` and `native-python`, or retrieve only failed gates and artifact paths from one printed assessment directory. A thin typed PSE binding is next when an actual repeated question concerns a named run, attempt, manifest or authored model. Its evaluation query should identify that exact scientific object and expected typed answer rather than enumerate the entire store.

**Semantic navigation is the next optional experiment**, particularly for an alias/trait/macro question that text search cannot settle. A useful local query is which resolved implementation a method call in `crates/pse-runtime/src/session_factory.rs` refers to and where that symbol is referenced; pair it with an ordinary direct-call case and compare correctness/omissions against `rg` plus focused reads. Verify effective analyzer preparation/features/environment before interpreting an empty answer. Broader HIR or rustdoc extraction should follow only if this existing route cannot serve the selected query.

**Direct SurrealDB HTTP MCP is the preferred database-tool experiment** over a new remote bridge or plugin package. Ask for the schema/indexes of an explicitly selected existing context and a bounded diagnostic result, compare with native CLI/HTTP inspection, and include a missing-context/error interpretation case. The currently absent `canonical` context must remain an error; initializing it is a separate operation. Use the typed PSE route first when the question is a scientific outcome or publication identity. Compiler profiling and narrower runtime exposure have lower priority unless a specific build delay or tool-selection problem motivates them. This order compares added machinery and consumed meaning; it is not a mandatory evaluation program or an assertion of measured benefit.

For Rust facts, lexical search, syntax matching, rustdoc API shape, Cargo metadata and semantic HIR answer different questions. The loaded pinned Rust-code-model/ast-grep-ripgrep skills explain those distinctions; their historical pins/probes are not proof of the current analyzer or nightly's behavior. Do not infer name resolution from a text match or function behavior from rustdoc JSON. Conversely, ordinary text search does not need semantic service startup when it already settles the question.

For SurrealDB, the existing native WebSocket SDK integration and agent-facing HTTP MCP are different consumers. The native MCP route uses the running datastore; the pinned stdio `surreal mcp` route is not a demonstrated remote attachment to it. A separate external bridge adds SDK/tool-schema/session/lifecycle maintenance; embedding the server's core-based MCP implementation would not automatically reuse the lightweight application SDK. The official local plugin adds packaging/guidance over the same endpoint. Select those alternatives only for a capability they supply beyond direct registration, CLI or typed inspection. No upgrade or installation is recommended merely because a tool exists.

## 9. Alternatives and total machinery

The preferred direction is **the current native toolbox with corrected bindings and observation contracts**. It preserves existing scientific and lifecycle ownership while removing caller-side repair and misleading feedback. Just, quoted positional argv, existing checking tools, standard-library subprocess/error handling and typed PSE APIs already supply the needed mechanisms.

A **documentation-only workaround** can temporarily name module invocation, literal argument caveats, the separate codegen-check route and a direct observer invocation. It does not settle the advertised entry point, argument fidelity or failed-observation semantics; narrowing the contract must be explicit. It is therefore insufficient for accepting the current broad surface.

A **generic command/result MCP workbench** might centralize discovery, but would introduce another command schema, execution adapter, credential/environment boundary, lifecycle owner and upgrade surface. No inspected consumer requires that total machinery. A thin adapter over a selected typed task remains eligible when it removes concrete repeated interpretation rather than copying native interfaces.

A **bespoke semantic index/HIR service** could serve repeated programmatic consumers, but adds project loading, feature/sysroot/proc-macro handling, freshness and unstable API maintenance. The existing analyzer/editor and focused source/native metadata routes are simpler first alternatives for current agent questions. Eligibility is not restricted to an existing consumer; integration still needs a credible task and benefit.

A **new supervisor or cap regime** is neither requested nor a remedy for these findings. Existing user-manager/cgroup and managed runner lifetimes already own their supported work. The live exec/session interface is sufficient for ordinary attached progress; retained run artifacts do not promise generic detachment. No additional concurrency limit, runtime budget, dashboard or mandatory measurement campaign is introduced.

## 10. Verification and uncertainty

| Evidence | Result and limitation |
|---|---|
| Independent source/contract inspection | **passed** read-only inspection at the stated baseline; establishes Implemented routes and structural causes. |
| Coordinator interface probe and raw JSON, independently inspected | **passed as negative reproductions**, zero unexpected control failures; native commands deliberately return the recorded failures. Isolated admission/systemctl controls are not live service failures or product tests. |
| Codegen dry-run | **passed**, Interface-checked composition only; no actual generator, hakari operation or mutation executed. |
| Native help/source/Context7 evidence | **passed**, version-scoped interfaces; analyzer launch, feature graph and effective TOML application remain unexercised. |
| Existing HTTP MCP initialization/list/constant query | **passed for those calls**; schema listing **failed with explicit NotFound**. No scientific records or write-admission control exercised. |
| Fixed-window retrospective extraction | **passed** read-only extraction at its documented cutoff, with input/parser diagnostics retained. Historical/descriptive activity, not causal effectiveness. |
| Plans 29/30 qualification | **historical**, named scopes and repaired composite outcomes retained at their owners; not rerun or widened. |
| Product/integration/scientific tests, fresh agent sessions, service restart/saturation, configuration/tool adoption and performance/quality measurements | **not_run** in this review. Remedies remain Proposed. |

The unresolved analyzer initialization/environment question affects optional adoption, not the five findings. Missing canonical database selection affects database-inspection evaluation, not the demonstrated `/mcp` transport. Longitudinal benefit remains unmeasured. These are decisions to settle if those options are selected; unexamined simulator breadth is not another defect.

## 11. Rule impacts and disposition

### <a id="rc01"></a>RC01 — Permit diagnosis independently of workload admission

**Current rule:** AGENTS.md's environment route and `.claude/rules/tooling.md` require every command/recipe to pass through the capped environment boundary; `docs/dev/agent-environment.md` describes that route for activity. `pse_env` also composes compiler configuration before observation.

**Proposed change:** retain one environment/placement owner but give read-only environment/admission diagnosis an explicit route independent of workload admission and irrelevant compiler prerequisites. Existing workload caps, verified nested ownership and managed drain remain unchanged. The rule change is the observational exception, not a new resource policy.

**Depends on:** F05. **If kept:** the recommended route cannot diagnose unavailable admission; direct invocation remains an operator workaround outside the prescribed route, and F05's target capability remains unresolved. Sanctioning that invocation is itself the observational rule exception. A hidden bypass in agent memory is not closure.

### <a id="rc02"></a>RC02 — Define overall recipe effects before forwarding mode flags

**Current rule:** `.claude/rules/tooling.md` says thin recipes pass `*args` through to real tools; the codegen recipe does so only for its final generator while unconditionally running mutating bootstrap/hakari stages. Native xtask checking semantics are scratch comparison.

**Proposed change:** a composed recipe declares which native modes it supports and honors them across the whole operation, or rejects them before effects and routes to the existing checking operation. Arbitrary native values remain argv under F02. This refines the composition contract; it does not move generated-output authority or weaken comparison.

**Depends on:** F03. **If kept:** remove any implication that codegen's forwarded `--check` represents a check-only composition and refuse it before effects; otherwise the finding remains. This review does not apply the rule change.

F01, F02 and F04 otherwise restore the stated entry-point/argument/observation contracts and require no new governance rule. Existing command-token class inference is an implementation choice to replace for F05, not a reason to change profile allocations. Optional analyzer/MCP evaluation can remain session opt-in; always-on adoption, a new scientific adapter or a genuinely changed support contract would need its own selected scope. No such rule change is pre-approved here.

| Finding | Current disposition owner | Adoption/closure condition |
|---|---|---|
| F01–F05 | [Plan 31](../../plans/31-agent-effectiveness-enhancements.md#finding-dispositions) | The plan owns adopted scope, confirmed RC decisions, optional enhancement choices and correction evidence. This review retains its original observations and recommendations. |

Priority follows consequence: prevent data-as-shell interpretation and hidden check-mode writes, restore the central targeted-selection entry point, then make failure diagnosis reachable and truthful. F04/F05 can be designed together but retain separate closure controls. Existing mechanisms allow narrow corrections; broader tool adoption is not a prerequisite. This is prioritization, not a staffing/migration plan or newly scheduled work.

## 12. Decision and next consequential choice

**Behavioral/semantic adequacy: not accepted**, because native argument fidelity, check effects and observation truthfulness fail their applicable gates. **Architectural fitness: not accepted**, because the invocation/observation seams violate the named foundations and G9. **Overall: Revise**, bounded to the agent-workspace subsystem.

Retain the useful environment, actual ownership, artifact preparation/observer separation, selected/terminal reconciliation, conservative retention and typed scientific interfaces. Repair the five bounded seams at their existing owners. The proposed directions are suitable inputs to planning; they are not implemented or qualified replacements.

The operator's next decision is whether to adopt that correction scope and RC01/RC02, then separately select any optional task/navigation/database capability whose full benefit and machinery fit the intended use. Completed plans remain historical, paused scientific campaigns remain paused, and no simulator or measured agent-effectiveness certification follows from this review.

Principal artifact: `docs/design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md`.
