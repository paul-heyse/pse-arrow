# Agent workspace effectiveness

**Date:** 2026-10-07

**Tier / purpose:** DESIGN / TARGET

**Boundary:** Plan 29’s development environment, command selection, feedback, agent configuration, test scheduling and process isolation, including their existing harness suppliers and consumers.

**Standard:** Core 3.4, Heuristics for Efficient Architecture 1.0 and the [pse-arrow binding](../design_principles/binding/pse-arrow.md). Process Simulator 1.5 was consulted; its scientific gates are outside this harness-only assessment.

**Baseline:** HEAD `dacc9c33225990984ddbd1d356e797389f02fef3`, with concurrent dirty Plan 28 documents/index and untracked Plan 29 and Plan 28 review preserved.

**Method:** Independent principal review, source inspection, existing study artifacts, Context7 research for general tool contracts, exact-version local/tagged-source fallback for decisive behavior, and bounded coordinator probes. No implementation, full study rerun or product qualification.

**Disposition owner:** [Plan 29](../../plans/29-agent-workspace-effectiveness.md), after maintainer adoption. Plan 28e owns canonical fixture qualification; Plan 28h owns native installation and artifact identity. This review recommends changes and creates no additional tracking ledger.

## Assessment and scope

**Architectural fitness: Revise. Scoped behavioral adequacy: unresolved for the proposed implementation, with specific capability and selection defects established below. Overall decision: Revise.**

Plan 29 identifies useful improvements and chooses mostly suitable foundations. A common environment adapter, composable selectors, quieter output, explicit runtime capability selection and systemd isolation can remove actual friction without creating a new workflow framework. Stale cleanup, argument pass-through, instruction headroom and reuse of existing assessment facilities are particularly direct changes.

The material weaknesses are in how those remedies compose. A persistent session environment is different from an operation-owned native environment. A scheduler slot identifies capacity within one run, not a globally owned server. An aggregate slice limit is different from a command’s failure boundary. A dependency selector is different from exhaustive change-impact discovery. Those distinctions need to govern the proposed behavior before implementation can satisfy the plan’s own claims.

The maintainer has confirmed Plan 29’s assumptions, rubric and standing preferences. They are settled requirements: one workstation; highly intelligent agents; Claude and Codex valued equally; useful capabilities and removal of actual friction; thin tooling; first-class bare commands and overrides; low parallel test load; no alignment lints, disk guards or fd-inheriting locks. This review does not reconsider those choices.

The reviewed scenarios are Plan 29 S1–S7. The consequential variation axes are selecting another package or feature set, changing installed Python extension kind, starting a second test run, creating a worktree, enabling an optional runtime capability and interrupting a command. Production numerical behavior, solver performance and whole-system scientific qualification are excluded.

## Responsibilities and preservation constraints

The current harness already contains important owners worth composing:

| Responsibility | Existing owner and contract |
|---|---|
| Compiler environment | `scripts/build_environment.py::configure` selects compiler/cache defaults and handles inherited build-directory contamination. |
| Native preparation and lifetime | `scripts/native_operation.py::{Operation, environment, owner_record, current, cancel_children}` owns admitted capabilities, generation protection, authentic scope identity and cancellation. |
| Command conveniences | `justfile` supplies selections and arguments; `scripts/build-shell.sh` configures ordinary recipe shells. |
| Assessment execution and feedback | `scripts/validation.py::{main, run_gates, execute, checkpoint}` already provides selected scopes, logs, structured results, interruption handling and constrained reuse/transfer. |
| Canonical deployment | `scripts/surreal_server.py::{setup, start, stop, worker, worker_scope_command}` owns state, listener identity, service/worker processes and their budgets. |
| Database fixture lifetime | `crates/pse-operations/src/testing.rs::{canonical_fixture_store, FixtureLifetime}` creates isolated databases and retains their owners through cleanup. |
| Server-level worker journeys | `crates/pse-runtime/tests/worker.rs::ManagedWorkerCase` owns a state-scoped lock and the deployment’s finite worker slots. |
| Artifact observation | `scripts/doctor.py::{EXTENSION_PROBE, check_extension}` observes installed extension kind; `scripts/producer_deployment.py::bind_python_artifact` owns stronger artifact association. |

The common environment should consume these decisions rather than replace them. Ordinary commands need cheap environment composition; native commands additionally need the existing operation lifetime. Selection should remain visible to Cargo/nextest. Logging should reuse the current assessment machinery where it fits. Database isolation must remain distinct from server control.

This is an adequate domain vocabulary for the harness; no universal registry, new daemon or elaborate instruction model is needed. The proposed operations nevertheless need explicit meanings for caller override, run identity, selected coverage, readiness observation and cancellation.

## Findings

### <a id="f01"></a>F01 — Environment composition does not yet preserve its promised precedence and lifetime

**Principles:** AP-03, AP-04, AP-05, AP-06; DP-01, DP-09, DP-18, DP-19.

**Scenarios:** S1, S4, S6.

The ordinary recipe gap is real. `build-shell.sh` sources `build-env.sh`, whose owner configures compiler settings but does not supply the venv or `.envrc.local`. The coordinator’s clean-shell observation confirmed absent license, venv path and native prefix.

P2 is the right integration direction, but its promise that caller values win does not match all existing suppliers. `native_operation.py::environment` assigns all three native thread variables to `1`, even when the requested capability list is empty. The coordinator reproduced replacement of explicit values `3`, `4` and `5`. `build_environment.configure` also deliberately changes some inherited inputs to preserve compiler supervision and checkout isolation.

Define precedence once in the common adapter: command options, explicitly supported caller overrides, local defaults and repository defaults. Preserve meaningful empty/off values. Retain necessary artifact and checkout correctness boundaries, but make any rejected override legible rather than silently treating it as a default.

Keep `--print` separate from command execution. The ordinary export path should not prepare native assets, create a native operation or start a resource scope. Native command execution should compose `Operation` and its existing admission/drain behavior. A session-start export must not turn temporary native admission or generation paths into persistent session authority.

Claude’s documented `CLAUDE_ENV_FILE` facility supplies subsequent Bash commands; it does not establish an environment for every parent, hook, MCP or LSP process. Those consumers need their own launch configuration or the common executable entry point. [Claude environment persistence](https://code.claude.com/docs/en/hooks#persist-environment-variables).

**Acceptance:** One small environment matrix should compare recipe, common-entry and export paths from a clean shell, with explicit values and empty/off overrides. Confirm requested native capabilities remain lazy, nested execution preserves the existing operation owner, command status/signals propagate, and export output handles quoting and unsets without publishing secrets.

### <a id="f02"></a>F02 — Shared-checkout locking is claimed beyond the capability demonstrated

**Principles:** AP-05, AP-07; DP-15, DP-21, DP-22.

**Scenario:** S7.

AE-06 establishes contention, but its `4.7 agent-h` is the wall duration of commands showing lock messages, not separately measured lock-wait time. Compilation and test execution are included. It supports prioritizing the problem, not a quantitative claim about recoverable waiting.

The small concurrent-check probe supports enabling fine-grained locking for checks. It does not establish P3’s broader acceptance that two agents *building* disjoint crates never block. Cargo’s tracking issue explicitly limits current parallelism to non-artifact-producing commands such as `check`; shared dirty units also retain important waiting limitations. [Cargo fine-grained locking](https://github.com/rust-lang/cargo/issues/4282).

Enable the available capability with an accurate scope. Continue to offer worktrees for genuinely concurrent artifact-producing work. A holder file may improve diagnostics, but it must remain observational: no new exclusion lock, no false claim to identify a bare Cargo owner, and no single mutable “current holder” pretending that several unit locks have one owner.

**Acceptance:** Distinguish disjoint checks, shared-unit checks and artifact-producing commands. Confirm the first gains supported concurrency and the others report remaining contention honestly. If holder diagnostics are added, exercise two active invocations and a stale PID. Do not use this probe as build/test throughput qualification.

### <a id="f03"></a>F03 — Canonical scheduling needs run ownership as well as slot ownership

**Principles:** AP-01, AP-04, AP-05, AP-07; DP-19, DP-20.

**Scenarios:** S5, S7.

AE-07’s broad serialization diagnosis is sound: the `store` group has four available threads and its selected tests each require four.

The underlying constraints differ. `canonical_fixture_store` creates a UUID database and retains cleanup ownership. `ManagedWorkerCase::acquire` instead locks a deployment state and owns its finite worker slots; its journeys can kill workers. Supervisor start/stop/reconfigure operations likewise affect a whole deployment. No inspected source establishes a universal server-installation exclusion requirement for every database-only test.

P5 should preserve those distinctions. Independent database consumers may share a suitably bounded server. Server-lifecycle and managed-worker journeys require exclusive ownership of their selected server resources. Per-slot servers are a valid remedy for the latter; cloning full server installations for every database-only consumer is not yet justified.

Slot-only naming is insufficient under S7. Nextest slots are reused and unique within a run’s scheduling interval. Separate runs can simultaneously use group slot zero. Use a run-owned namespace plus group/slot, with unique state, ports and units. `NEXTEST_RUN_ID` already provides run identity; setup scripts execute before tests, and their environment output can route matched tests. [Nextest slot and run identity](https://nexte.st/docs/glossary/#slot-numbers), [run environment](https://nexte.st/docs/configuration/env-vars/), [setup execution](https://nexte.st/docs/configuration/setup-scripts/#script-execution).

Reuse the supervisor and immutable server binary installation. Own only disposable states and processes at the required lifetime. Override-selected external state must not become eligible for automatic destruction.

**Acceptance:** At low concurrency, run a small database-only pair, a worker/server-control pair and two overlapping nextest invocations. Observe distinct run resources, retained within-server exclusion, awaited fixture cleanup, and cancellation that removes only the run’s owned processes. Measure the store subset afterward; “roughly divided by slot count” remains Proposed until measured.

### <a id="f04"></a>F04 — Selection conveniences can silently contradict their advertised coverage

**Principles:** AP-02, AP-04, AP-05; DP-08, DP-15, DP-24.

**Scenarios:** S1–S3.

P6 promises that `unit-package` intersects a user filter with the named package while an additional `-p` still widens selection. A fixed `package(named)` intersection excludes tests from the additional package. Construct any package restriction from the final selected package set, or explicitly expose independent package and test selectors.

A bare word convenience is reasonable, but only unmistakably simple words should be rewritten. Existing filter expressions, regexes and explicit tool arguments must pass through unchanged. Preserve nextest’s empty-selection failure and print the effective selection.

`affected` can be a useful crate/dependency selector. Cargo reverse dependencies do not prove that every affected test has been found. Feature-gated tests, Python consumers, changed shared fixtures, generators and root configuration require visible treatment. Define the default comparison against HEAD, including staged and unstaged changes; inspect untracked relevant files rather than silently excluding them. Explain what a supplied base changes.

Cargo aliases also do not change plain `cargo check` or `cargo nextest run`. P2/P8 should accurately distinguish configuration shared by bare tools from flags supplied by aliases and recipes. Explicit force-validation on correctness tests remains necessary.

The generation gap is established in `scripts/validation_scope.py::GROUPS` and `justfile`. A complete check should delegate each output to its owner. However, `xtask/src/codegen/python_stubs.rs::{run, installed_extension}` checks metadata from an installed compiled extension. Merely adding `python-stubs --check` cannot establish current Rust API freshness when that extension is stale. Reuse Plan 28h’s artifact preparation/admission boundary, or state the installed-artifact scope and report the missing prerequisite.

**Acceptance:** Use a compact selected-test list to demonstrate a simple word, an existing expression, multiple packages, an empty match, a feature-gated consumer and a root/shared-input change. Expose list-only/structured output through existing nextest facilities. Demonstrate all generated-output owners are reached and an absent/stale extension produces an honest prerequisite result.

### <a id="f05"></a>F05 — Readiness caching and extension adaptation must consume observations, not create new authority

**Principles:** AP-01, AP-04, AP-06; DP-01, DP-09, DP-18.

**Scenarios:** S1, S6.

AE-02 should be narrowed. The installed extension is not “visible nowhere”: `doctor.py::EXTENSION_PROBE` and `check_extension` already observe native linking. The real gap is that `py-unit` does not use suitable routing before conftest imports the extension.

Use artifact-derived observation and the selected interpreter. A missing native library should produce a useful preparation/rebuild instruction; it must not silently replace the extension or weaken tests. Preserve checkout-origin and artifact association owned by `producer_deployment` and Plan 28h. Do not add an independently maintained “native installed” marker.

The proposed doctor cache also needs a narrower contract. Repository input hashes do not detect removal of a tool, mutation of a venv, replacement of an extension, or a server becoming unreachable. A cached healthy result could conceal the failure P4 is intended to diagnose.

The lightest option is cheap environment composition on entry and explicit `ready`/doctor when needed. If a cached status display is useful, label it as cached observation and permit refresh; do not use it as current admission authority.

**Acceptance:** Route `py-unit` with both installed extension kinds and one missing-library case, preserving explicit caller selections. For any retained readiness cache, remove or replace an observed external prerequisite without changing repository files and confirm stale success is not treated as current readiness.

### <a id="f06"></a>F06 — Proposed isolation does not yet guarantee either sibling survival or effective overrides

**Principles:** AP-03, AP-05, AP-07; DP-19, DP-20, DP-21.

**Scenarios:** S5, S7.

P9’s separate slice is a useful direction. Existing `memory-cap.sh` already supplies individual scopes, and the study reports that capped failures remained contained.

A slice’s `MemoryMax` constrains aggregate descendants. Raising a child’s `PSE_MEMORY_MAX` cannot defeat a stricter ancestor. Aggregate pressure can affect a sibling workload even when neither command exceeds its individual limit. Under installed systemd 255, oomd selects eligible descendant cgroups and kills all processes in the selected group; configuring a monitored slice does not establish that only the runaway command will be selected. The decisive contracts are in the installed `systemd.resource-control(5)` and `systemd-oomd.service(8)` manuals.

Define separately the command cap, aggregate workstation protection and intended victim/failure scope. Low parallel defaults and CPU weighting should reduce contention without implying guaranteed sibling survival under aggregate exhaustion. Provide an explicit route to change or leave aggregate limits from inside a session; report effective ancestor limits.

Compose new placement with `native_operation.py::scope_owner`. That function recognizes particular unit names and authentic invocation identity. Renaming scopes or nesting them carelessly can break native reuse, generation protection and cancellation. Native supervised workers may also be launched into separate units; ordinary parent placement does not automatically cover them.

**Acceptance:** Observe actual command and worker placement, then use a small synthetic cap with a harmless sibling. Confirm individual over-cap failure remains isolated and descendants drain. Separately inspect aggregate behavior and the documented override/escape route. No machine-wide stress run is needed.

### <a id="f07"></a>F07 — Runtime and worktree conveniences need version-supported capability boundaries

**Principles:** AP-02, AP-05; DP-15, DP-21, DP-24.

**Scenarios:** S4, S6, S7.

P8’s opt-in direction is appropriate, but “disabled by default for subagents” is not established for installed Codex 0.160. Its exact `AgentRoleOverrides` and `apply_role_to_config_inner` accept a bounded role override set that does not include MCP server configuration. Newer documentation cannot establish that behavior in this version. Use supported project/session selection and state the subagent-specific gap, or deliberately change the runtime and verify it. [Codex 0.160 role override source](https://github.com/openai/codex/blob/rust-v0.160.0/codex-rs/core/src/agent/role.rs).

The pyrefly direction similarly requires an actual LSP plugin binding; a checker replacement is not delivered by renaming the cached pyright plugin. Preserve opt-in access to useful code intelligence and unrelated capabilities when deliberately requested.

P3’s worktree helper must name its source baseline. `git worktree add` does not transport current uncommitted implementation. Print the selected ref and keep dirty-work transfer explicit and scoped. Do not copy another checkout’s venv or extension as though it belonged to the new checkout. Optional Python preparation should construct the new checkout’s own artifact.

**Acceptance:** Start one fresh session in each runtime with default and explicitly enabled capabilities. Inspect actual MCP children and one pyrefly diagnostic. Exercise a small worktree with declared ref and optional Python preparation; confirm its license/environment and import origin belong to that checkout.

## Packet dispositions and remedy interactions

These are review recommendations, not adopted or implemented status.

| Packet | Disposition | Required refinement |
|---|---|---|
| P1 | Support | Delete only genuinely unwired artifacts and their consumers; scope user-config cleanup to identified stale entries. Existing `lint-agents`/`setup-test` suffice. |
| P2 | Revise, then support | Establish precedence, cheap export behavior and native operation lifetime under F01; describe bare-tool parity accurately. |
| P3 | Support scoped capability | Limit fine-grained-locking claims under F02; make worktree baseline and diagnostics honest under F07. |
| P4 | Revise, then support | Reuse extension observation; choose explicit readiness or honestly cached status under F05. Make `.design-edit` a visible, removable checkout override. |
| P5 | Revise, then support | Classify database and server/worker effects; use run-owned state plus slots under F03. Coordinate with 28e. |
| P6 | Revise, then support | Preserve composable selection and truthful coverage; use existing output owners and artifact prerequisites under F04. |
| P7 | Support with refinements | Unique run directories, transparent output options, selective preflight and existing assessment execution. |
| P8 | Support directions; revise unsupported mechanism | Keep compact always-loaded guidance and links; implement runtime capability selection supported by the installed version under F07. |
| P9 | Revise, then support | Separate aggregate protection, command failure and effective override semantics under F06. |

P7 should not introduce a second job runner. `validation.main/run_gates` already supports functional scopes, constrained reuse/transfer, structured selection and durable results; `execute` already handles interruption. Extend that behavior where bundles need it. Timestamp-only directories can collide during concurrent invocation: add a unique run component, print immutable paths, and treat `latest` as convenience. Offer live/full output on request and preserve original exit status.

Preflight can catch missing infrastructure and known fixture prerequisites. It cannot eliminate late code or scientific failures. Classify errors from the operation that knows the cause; avoid labeling arbitrary conftest failures as environment errors.

AGENTS.md currently occupies 31,021 bytes against Codex’s default 32,768-byte project-document budget, leaving 1,747 bytes for additional project instructions. Supported `project_doc_max_bytes = 65536` supplies headroom; the proposed ≤20 KB core is a compactness target, not independent evidence of improved effectiveness. Preserve the task routes, consequential constraints and explicit instructions to load relevant owners. Move detailed mechanics behind usable links, while retaining short deliberate overlap where it helps an isolated worker act correctly. Acceptance should inspect the instructions actually loaded in fresh Claude and Codex sessions and exercise a representative task route, alongside the byte count. [Codex 0.160 project-document configuration](https://github.com/openai/codex/blob/rust-v0.160.0/codex-rs/core/src/config/mod.rs).

Consequences favor daily environment repair, selection repair and containment before speculative throughput improvements. Dependencies nevertheless matter: settle P2/P9 scope composition before rewriting every recipe; settle P3’s claims before relying on shared-checkout builds; establish P5 resource identity before parallelizing server control. P1 cleanup and supported P8 context reductions can proceed independently. A strict P1–P9 sequence is unnecessary.

## Additional capabilities

These opportunities extend the approved direction without adding a required registry, daemon, audit or procedure. Each can remain optional and independently useful.

1. **Cheap environment explanation.** Show the selected interpreter, compiler/cache defaults, installed extension kind, requested native capabilities and effective scope limits without preparing assets or exposing secrets. Reuse `scripts/doctor.py::{Check, check_extension, native_library_path}` and `build_environment.configure`; integrate the selected view through P2’s entry point. **Acceptance:** from a clean shell, explain why a Python unit command needs the linked environment, then explain an explicit override. The view performs no native preparation and distinguishes an observation from readiness admission.

2. **Intent-based discovery.** Let an agent find “check these packages,” “list native tests,” “check generated outputs” or “prepare another checkout” through existing `just --usage`, recipe groups, the JSON recipe dump, skill routes and manifests. P6/P8 own the presentation; no separately maintained command registry is needed. **Acceptance:** find the selection and pass-through syntax for a cross-package native test and the relevant generation check without reading the entire justfile. Show the underlying command and its prerequisites.

3. **Task-useful semantic navigation.** Keep semantic tools opt-in and explain revealing uses: rust-analyzer for callers, implementations and type resolution; pyrefly LSP for Python symbols and diagnostics; the library catalog for existing integration examples. P8 owns runtime activation, with existing MCP/plugin bindings absorbing tool differences. **Acceptance:** deliberately enable the capability, find implementations and callers of a known Rust trait or resolve a Python boundary type, then end the session without accumulated unused servers. Text search remains directly available.

4. **Long-command feedback and control.** Expose selected assessment scopes, immutable log paths, current step, structured results and interruption through `scripts/validation.py::{main, run_gates, execute, checkpoint}` and authentic native-operation scopes. P7 owns concise versus live output. **Acceptance:** run a small two-step bundle with one failure, follow its log while active, interrupt a child and observe retained status plus drained owned descendants. Reuse existing constrained retry/reuse facilities where applicable.

5. **Preview affected selections.** P6 should preview package/test/generator selections before execution, using Cargo metadata, git changes, nextest listing and existing generation owners. Include staged, unstaged, deleted and relevant untracked inputs. State native-feature and Python exclusions explicitly; a dependency preview does not claim exhaustive behavioral coverage. **Acceptance:** preview one crate edit, a deleted source file and a shared generator/root-config edit. Show the widened selection or unresolved scope, suggested features and excluded Python/native consumers.

6. **Lightweight concurrent-work visibility.** Make current command scopes, elapsed activity and known Cargo contention discoverable using systemd status/cgroup information and existing operation identities. P3/P9 own this capability. Any recipe/PID annotation is optional diagnostic context, not a coordination lock or authoritative job ledger. **Acceptance:** with two small commands active, identify their scopes and distinguish an active command from a stale annotation. For a bare Cargo lock owner that cannot be identified reliably, report “unknown” rather than a guessed holder.

## Foundations and gates

The judgments below concern Plan 29 as written.

| Foundation | Verdict | Reason |
|---|---|---|
| AP-01 | Satisfied | Responsibility direction is sound; existing compiler, native, fixture and assessment owners can absorb the changes. |
| AP-02 | Unresolved | Selection and runtime substitutions need the supported contracts in F04/F07. |
| AP-03 | Unresolved | Environment, native lifetime and resource placement have not yet been composed under F01/F06. |
| AP-04 | Unresolved | Run/slot, installed artifact/readiness and selected coverage distinctions need to govern behavior. |
| AP-05 | Violated | Package widening, locking scope and subagent-specific configuration are advertised beyond their defined mechanisms. |
| AP-06 | Unresolved | Cheap local environment behavior must remain independent of native/store preparation and stale readiness. |
| AP-07 | Violated | Broad serialization and unsupported concurrency claims remain; proposed aggregate isolation lacks the stated failure boundary. |

| Gate | Judgment |
|---|---|
| G1 Authority | Unresolved: common precedence and readiness must consume their existing owners. |
| G2 Semantic fidelity | Fails: P6’s package intersection conflicts with promised widening. |
| G3 Validity | Unresolved: generation checks need explicit installed-artifact prerequisites. |
| G4 Hidden behavior | Unresolved: export/preflight effects and lazy capability setup need definition. |
| G5 Consistency and recovery | Unresolved: concurrent run identity, cleanup and aggregate failure scope. |
| G6 Transformation and reuse | Unresolved: readiness caching and installed-artifact reuse premises. |
| G7 Truthful capability claims | Fails: broad locking and installed-version subagent configuration claims. |
| G8 Library leverage | Passes at Proposed design level: Cargo, nextest, just, systemd and existing harness mechanisms supply the main capabilities. |
| G9 Architectural fitness | Fails through AP-05/AP-07; remaining interactions are unresolved. |

PS-G1–PS-G3 are not applicable: no physical definitions, formulations, derivatives, solver outcomes or numerical tolerances are changed or qualified here. Preserve force-validation, scientific assertions and fixture semantics while changing harness scheduling. Production scientific adequacy receives no verdict from this review.

## Evidence and acceptance boundary

The strongest diagnosis evidence is current source, exact tool contracts and bounded coordinator observations. Existing W1/W5/W6 artifacts provide useful historical workload evidence. W1’s current-period data contain only 677 Claude shell commands, so runtime-specific rate or benefit estimates are uneven. Four drills do not establish general performance improvement. Classifier fixes support the reported study, but this review did not independently reclassify the corpus.

**Interface-checked:** decisive owners named above; Codex 0.160 role overrides; Cargo locking limitations; nextest run/slot/setup contracts; Claude Bash environment persistence; installed systemd 255 resource semantics.

**Tested, coordinator probes, baseline zero:** clean-shell ordinary recipe environment observation, explicit thread-value replacement through `native_operation.environment([], …)`, and `just assessment-list --functional-scope native`; all exited zero. The latter returned one native check and seven exclusions without executing product checks.

**Measured, historical conditions:** study timings and interrupted native-gate JUnit described by Plan 29. They are neither freshly executed performance evidence nor positive qualification of proposed remedies.

**Proposed:** all corrective behavior and expected benefit in this review. Implementation checks, fresh runtime startup behavior, store-subset timing and containment acceptance remain not run.

The acceptance examples above are bounded demonstrations, not a new mandatory matrix or permanent checking system. Run each changed capability once in the revealing case, reuse existing tests where suitable, and broaden only when a failure or material uncertainty warrants it. No broad product qualification is required to accept this harness design.

## Rule impacts

These impacts take effect only after maintainer confirmation in the owning plan.

| ID | Current rule / location | Recommended change and dependency | If retained |
|---|---|---|---|
| <a id="rc01"></a>RC01 | AGENTS.md *Start here* and justfile recipe-first wording; `.claude/rules/rust.md` bare test guidance | Make bare tools first-class while accurately documenting shared configuration versus explicit recipe/alias flags. P2/P6/P8, F04. | Recipes remain the preferred prescribed path; environment improvement is still useful, but full rubric compliance is not delivered. |
| <a id="rc02"></a>RC02 | AGENTS.md *Agent runtimes*: PreToolUse is the only hook; ADR-0161 hook wiring/verification | Permit a narrow environment-only SessionStart hook if selected. Preserve ADR-0161’s removal of end-of-turn automation. P2, F01. | Use the common executable/launcher without SessionStart; Claude bare Bash needs explicit environment composition. |
| <a id="rc03"></a>RC03 | AGENTS.md design-edit escape and `agent-hooks.py::main` | Add the chosen visible checkout override with clear activation/removal, retaining scope and session authorization. P4. | Design edits require launch-time environment or existing external environment control; the in-session escape gap remains. |
| <a id="rc04"></a>RC04 | `.config/nextest.toml` broad canonical `store` exclusion; fixture coordination owned by 28e | Replace broad serialization with effect-appropriate scheduling and per-run/per-server ownership. P5, F03. | Retain serial server consumers; the measured tail remains substantially unchanged. |
| <a id="rc05"></a>RC05 | Current runtime MCP/plugin/LSP activation in project and user configuration | Make selected optional capabilities session/project opt-in using supported mechanisms; retain deliberate enabling paths. P8, F07. | Startup process/context cost remains; capability availability remains immediate. |
| <a id="rc06"></a>RC06 | Current inherited native thread defaults and resource placement/caps | Make supported defaults overridable and define command versus aggregate placement/protection. P2/P4/P9, F01/F06. | Explicit caller values can still be replaced, and raising a child cap cannot escape stricter aggregate protection. |

Instruction relocation, stale cleanup and stronger command discovery need no additional architectural rule change when they preserve meaning. This review does not supersede accepted ADRs or amend architecture sections.

## Decision

**Revise Plan 29’s proposed contracts while retaining its main direction.** The changes needed are bounded: truthful capability scope, composable selection, existing-owner reuse, run-owned fixture resources and explicit failure/override semantics.

P1 and supported parts of P7/P8 are suitable directions now. P2/P4/P6 address immediate daily friction once their contracts are corrected. P3’s available check concurrency should be offered accurately. P5 and P9 should proceed with their distinct ownership and failure scopes established.

The next consequential decision is the common command boundary: what it configures, what it lazily prepares, which operation owns descendants and how overrides change effective limits. That decision enables the recipe and runtime adaptations without rebuilding policy independently in each consumer.

The findings belong in Plan 29 if adopted; fixture and artifact implementation obligations remain with their existing Plan 28 owners. No remediation was implemented, no plan was changed, and no product qualification or quantitative benefit is claimed.
