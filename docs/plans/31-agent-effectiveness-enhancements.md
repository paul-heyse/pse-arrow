---
title: Agent effectiveness enhancements
status: done
date: 2026-10-09
adrs: []
review_sources:
  - ../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md
scenario_sources:
  - ../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#4-scenarios-that-distinguish-alternatives
---

# Plan 31: Agent effectiveness enhancements

## Context

This plan develops the corrections and selected enhancements from the
[agent effectiveness review](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md)
and its [evidence](../design_review/evidence/agent-effectiveness-enhancements-2026-10-09/README.md).
It owns that review's F01–F05 dispositions and the execution sequence below.
The target is a more dependable personal Codex environment: working targeted selection,
faithful arguments, predictable command effects, reachable and truthful diagnosis,
identified result retrieval, and useful access to the existing persistent database.
The maintainer authorized authoring and then production execution on 2026-10-09.
The packages below own implementation and acceptance; authoring supplied no production evidence.

The inspected checkout is `main` at `d0f2c41818a34539a910654dfea4760771603f45`, with
the new review/evidence documents preserved. Plan 30's completed persistent-service
and WebSocket application handoff is a foundation, not work to repeat. Plan 28's paused
scientific campaign remains at its existing owner. The sibling Python repository's
review supplied ideas; it supplies no proof about this Rust/Python workspace.

The governing assessment is Core 3.4, Efficient Architecture Heuristics 1.0 and the
Process Simulator profile 1.5, selected through
[the binding](../design_review/design_principles/standard.toml).
This plan carries the source review's bounded conclusions and adds a focused assessment
of the invocation, observation, reporting and database-integration foundations. It
does not reopen simulator architecture or certify scientific support.

Semantic Rust search is **excluded**, at the maintainer's instruction. Do not add an
analyzer adoption experiment, another Rust index, HIR service, usage audit or semantic
search work package. The existing rust-analyzer MCP availability and prior low-use
evidence are sufficient for that scope decision; this plan does not change its settings.

### Current foundations and evidence limits

The [agent environment guide](../dev/agent-environment.md),
[validation guide](../dev/validation-assessment.md) and
[SurrealDB substrate guide](../dev/surreal-substrate.md) remain the mechanics owners.
The existing `scripts/pse-env`, host admission, managed test runner, validation groups,
retained receipts and service supervisor already provide the necessary environment,
resource and evidence lifetimes. Corrections belong at these owners; no new scheduler,
generic command framework, job ledger or result database is needed.

The review established five specific seams. `scripts/select.py` imports its package
before preparing direct-script imports. Some recipes interpolate arbitrary arguments
into shell source. `codegen --check` leaves bootstrap and Hakari writes enabled.
`activity` discards failed systemctl transport and presents empty success, and its
prescribed launch requires the workload admission it is meant to diagnose.
The import, argument and observation controls were negatively tested; the composed
codegen effect was source/dry-run evidence, without executing its writes.

The retrospective covered a fixed recent window across six project roots: 108 sessions
and 24,377 command launches, with recoverable raw exits for 16,204. Ambiguous results,
malformed input and dynamic commands limit interpretation. These observations identify
friction and existing successful routes; they are not a causal productivity estimate.
The archived original parser scan is not a fresh scan of the subsequently packaged parser.

SurrealDB 3.3.0 was serving the owned persistent state on loopback port 18240. The
native MCP handshake, tool listing and constant query worked; the configured
`pse/canonical` schema listing returned NotFound. A working transport does not prove
that a selected database exists. The Rust application already uses the vendored,
exact-pinned WebSocket SDK. Typed PSE inspection and closeable table streams already
cover scientific identity and result questions, so raw database inspection is a
different consumer rather than a substitute scientific API.

## Decisions and accepted rule changes

The maintainer explicitly accepted both review rule changes on 2026-10-09 and requested
assessment of the remaining optional enhancements. These accepted instruction changes
precede the dependent implementation in P0; the review alone did not accept them.

| Review rule | Decision and consequence | Change route / dependency |
|---|---|---|
| [RC01](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#rc01) | Accept, 2026-10-09. A fixed read-only diagnostic may run without workload admission or compiler preparation, so admission failure remains diagnosable. | P0 updates `AGENTS.md` and the environment guide; P2 implements that narrow route. This grants no arbitrary unadmitted command execution. |
| [RC02](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#rc02) | Accept, 2026-10-09. A composed recipe supports a mode across its entire operation or rejects it before effects. | P0 clarifies recipe composition guidance; P1 gives full codegen a zero-argument publication contract and directs comparison to `codegen-check`. |

On 2026-10-09 the maintainer selected **SurrealDB's native MCP**, explicitly rejecting
a custom adapter after considering WebSocket attachment. P5 therefore registers the
running server's Streamable HTTP `/mcp` endpoint. MCP is the tool protocol; this endpoint
uses HTTP as its transport. The existing scientific application's SDK remains WebSocket.
Do not create a stdio-to-WebSocket bridge, new MCP server, SDK binding or datastore.

These are tooling/instruction changes within the existing decisions, so no ADR is
scheduled. Dependency additions, if needed for a concrete correction, follow the exact-pin
policy without a wholesale re-resolution. A discovered change to a governed scientific
contract or crate structure must take its applicable decision route before implementation;
it is not implicitly included here.

## Architectural drivers and scenarios

Use [S01–S07 in the source review](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#4-scenarios-that-distinguish-alternatives)
at their original scope. Only the database portion of S06 is selected; its Rust-search
portion is excluded. The material scenarios for this plan are:

| Stimulus | Required behavior and owner |
|---|---|
| Select a Rust package/test from a normal checkout, S01 | The direct selector reaches the existing runner without caller `PYTHONPATH` repair and retains explicit force-validation. |
| Pass a spaced state path, multiword filter or shell punctuation, S02 | The native parser receives identical argv items; the shell does not reinterpret data. |
| Ask for generation comparison, S03 | `codegen-check` compares without rewriting outputs; unsupported arguments to full `codegen` fail before any stage. |
| Admission or the user manager is unavailable, S04 | The fixed observer still launches, reports partial observations and distinguishes unavailable from an observed empty result. |
| Resume an assessment after failure or interruption, S07 | A read command preserves the selected run identity, incompleteness, failed gates and provenance of reused evidence. |
| Inspect an existing database's schema/indexes, database part of S06 | Native MCP and native inspection agree for an explicit existing context; a missing database remains an error without initialization. |

A new argument-taking recipe should supply a native operation and a known class, rather
than add a command schema to a second registry. A new validation gate should continue to
use the existing gate/result declarations so the result reader needs no gate-specific
parser. A database namespace change should require an explicit selected context and
connection restart rather than silently reuse credentials/session defaults. These future
changes explain the boundaries; they are not additional implementation scope.

## Target design

### Native invocation and whole-operation effects

Keep recipes thin. Adopt positional argument forwarding with quoted `"$@"` and properly
quoted named values for every affected variadic recipe, rather than escaping interpolated
shell fragments or copying native option grammars. Inventory all such bindings in the
current justfile, including selector, native, SurrealDB, worktree and bundle routes;
representative tests do not excuse leaving equivalent unsafe consumers behind.
The native CLI continues to own validation, defaults and selection semantics.

Fix direct-script initialization in `scripts/select.py` before package imports. Both
direct and module routes consume the same selector/runner. Do not replace it with a shim
or retain two command-construction implementations. Existing explicit force-validation,
native prerequisites and managed test resource ownership survive the correction.

Make full `just codegen` zero-argument. It still publishes the complete generated target,
including required bootstrap stages and Hakari updates. `just codegen --check` and
`just codegen --only ...` then fail at Just's argument boundary before any command.
Comparison uses the existing `just codegen-check` and named focused check recipes; raw
xtask remains available for its native grammar. This removes a misleading composition
without inventing a candidate-checkout regeneration framework. Checking must never
refresh stale outputs before comparing them or temporarily mutate/restore the dirty tree.
If stale declarations prevent a full comparison from compiling, report that limitation
as a failure; do not reinterpret it as successful comparison of every output.

### Observation and known resource classes

Give `activity` a fixed direct interpreter route to the existing standard-library observer.
It requires neither the venv nor solver/compiler setup. A per-recipe interpreter can bypass
the global workload wrapper while forwarding argv unchanged; no generic bypass flag is
needed. Move the justfile's eager solver-image lookup to its actual consumer
`bootstrap-solvers`, retaining the explicit image override. Otherwise even an observer's
recipe can fail during unrelated global command substitution.

Keep observation read-only. Failed/timeout/malformed systemctl output must carry an
unavailable status, operation and useful error, with a nonzero CLI result when the requested
observation is unavailable. Preserve any independent process/limit information gathered.
An empty unit list means a successful observation of no units. A disappearing individual
unit may be reported as such rather than invalidating all observed processes. JSON and
human output must preserve the same distinction. Do not call lifecycle/reconciliation
operations such as host admission `status` merely because their names sound observational.

Workload recipes still use the existing capped environment. Bind their known existing
resource class before the **outer** `pse-env` allocation, using a per-recipe interpreter
default or equivalently narrow launch input. Binding an inner command after the global
wrapper acquired a functional allocation is too late. Preserve precedence: explicit CLI
choice, explicit caller/local configuration, known recipe default, then the existing
fallback for arbitrary commands. Verified inherited ownership remains authoritative.
Do not parse arbitrary shell source or introduce another operation/class registry. Existing
class budgets and aggregate ownership stay at their current declaration.

### Selected assessment and identified results

Change `assessment` to forward native flags directly. For example,
`just assessment --group ready --output build/assessment/example` passes those arguments to
`scripts.validation`; the recipe no longer consumes the first flag as a positional output
directory. The output directory must be fresh and inside the checkout's gitignored output
tree, preserving the existing evidence-placement contract. Remove the positional grammar
and migrate examples/callers in the same change.
The validation owner selects prerequisites through its existing selected gate definitions.
Remove unconditional native preparation from the assessment recipe when a selected group
does not require it, preserving required native setup, thread controls, receipts and actual
drain for groups that do. Do not duplicate the group-to-prerequisite map in Just.

Add a small read command, `just result RUN_PATH [--failures] [--gate NAME] [--tail N]
[--json]`, over the validation runner's existing files. The first scope is assessment
directories printed by that runner, not every product run format. Resolve any latest-link
once, read one atomic checkpoint, and identify the concrete run and its artifact paths.
Report missing, malformed and incomplete records explicitly. A successfully read failed
assessment may give reader exit zero, but must show its recorded failed result clearly;
reader success never means assessment success. Preserve reused receipt origin and do not
label it as a newly tested run. Read selected log tails rather than entire logs, and never
execute gates, reclaim fixtures, rewrite checkpoints or invent completion state.

Filtered command discovery remains native `just --usage`, `--show` and selective transient
JSON inspection. Add a few owner-linked examples where they remove actual confusion, not
a persistent command catalog. Explain raw nextest/affected fixture-retention consequences
and point to the managed runner when its association is needed. Python readiness guidance
must distinguish installed extension kind/path and lock observations from Rust-source
freshness; point to `py-sync`/`py-sync-native` after relevant source edits, without automatic
rebuilds before every query or another freshness cache.

### Native SurrealDB MCP integration

Use the existing server's `/mcp` endpoint, not `surreal mcp` opening its datastore in another
process. The pinned native network implementation reuses the running datastore with
Streamable HTTP; the stdio entry point creates its own datastore and is not a remote
attachment. Direct registration avoids an additional SDK/runtime, tool schema and session
lifecycle. The application's WebSocket client and its scientific admission/retry contracts
are unrelated to this agent-facing transport and remain owned by their current implementation.

Register a named, task-focused Codex MCP server in the existing project runtime configuration,
with the actual loopback port from the selected owned state, an inspection tool allowlist
and the existing root VIEWER selection principal. Keep it disabled by default and document
session opt-in; activate it for the pilot. Selecting native MCP does not require making
database tools part of every coding session. Tool allowlisting describes exposure;
it does not enforce database read-only authority. Do not expose owner credentials or
assume a generic query tool is harmless. Inspect the exact native tool argument/context
contract before choosing the smallest allowlist needed for schema and bounded diagnostics.

Supply credentials through Codex's supported `http_headers_helper` using a small helper at
the existing supervisor owner, not a custom MCP adapter. The helper receives an explicit
state path and expected endpoint, reads private owned credentials, validates state/listener
association without starting or reconciling services, and emits only the required header
JSON to Codex. Resolve paths from the checkout/explicit state; never depend on ambient cwd
or unproven propagation of `PSE_SURREAL_STATE`. Fail on missing state, wrong endpoint,
invalid ownership or unavailable credentials; redact secrets from diagnostics and artifacts.
No second credential store, command-line password or committed plaintext header is needed.

Bind the helper's expected endpoint to the registration. Codex caches headers, so changing
state, port or credential generation requires restarting/reconnecting that MCP registration;
the helper is not per-query deployment admission. Give the fixed header helper a narrow
launch at the environment/supervisor owners that uses the base environment, acquires and
releases ordinary light ownership, and avoids compiler/native preparation. Merely selecting
the light class is insufficient: current `pse_env.compose` still calls compiler configuration.
This helper launch accepts only its state/endpoint inputs, never an arbitrary executable;
it is distinct from the admission-free activity route. Test that compiler configuration
is not called and light ownership is released on both success and failure.
The persistent database remains in its existing service budget; there is no new daemon.

Context must be explicit in inspection calls. Namespace/database tool parameters can override
headers/defaults, and a root VIEWER can see more than one context. Report the actual selected
context and inspect only the requested one. Establish existence before schema inspection;
do not initialize `canonical`, define tables, register a receiver or reopen scientific write
admission to make a pilot pass. Retained/quiesced databases remain eligible for forensic reads
independently of scientific receiver admission. Use typed PSE APIs when interpreting authored
models, runs, attempts or publication identities; generic database rows do not replace them.

The pilot must include an explicit existing namespace/database, schema/index inspection and
a bounded diagnostic, compared with native CLI inspection of that same context. Include a
missing-context control, authentication failure and actual VIEWER write refusal/readback in
an authorized disposable fixture. A VIEWER write can return an empty non-error result, so
the control must establish unchanged data rather than rely on status alone. Transport errors,
statement errors and missing context must stay distinguishable from an empty successful read.
No HTTP-to-WebSocket bridge, official-plugin packaging or additional MCP framework is selected.

## Optional enhancement dispositions

Rows record the selected design choices; delivered scope and evidence are recorded in the
Outcome below. This table assesses the review's optional opportunities without turning every idea into a backlog.

| Opportunity | Decision / owner | Reason or concrete revisit trigger |
|---|---|---|
| Filtered recipe discovery | Adopt P4 examples of native usage/show/transient JSON. | Existing discovery owns the grammar; no durable second catalog. |
| Assessment ergonomics and selected prerequisites | Adopt P3. | One native flag grammar and the current gate owner remove positional ambiguity and unnecessary setup. |
| Identified result retrieval | Adopt P3's bounded read command. | Existing checkpoint/log files suffice; no service or result index. |
| Fixture feedback for selected/raw runs | Adopt P4 guidance; defer a new `affected --run` binding. | Runner association is intentionally conservative. Revisit a binding only for a repeated concrete workflow that needs automatic association. |
| Python extension freshness | Adopt P4 truthful prerequisites. | Current observations do not prove source freshness; stronger deployment qualification stays at its existing owner. |
| Typed model/run/manifest inspection | Adopt P4 executable examples using existing APIs and stream cleanup; defer a scientific CLI/MCP adapter. | Revisit only when an actual repeated typed consumer cannot be served adequately by those examples. |
| Semantic Rust search | Excluded by maintainer. | No search evaluation, new index or usage measurement in this plan. |
| SurrealDB native MCP | Adopt P5 and its actual-context pilot. | User selected native MCP on 2026-10-09; no custom server or WS adapter. |
| SurrealDB plugin/external bridge | Do not adopt. | Adds packaging/runtime/lifecycle without a required capability beyond native MCP, CLI and typed APIs. |
| Structured compiler feedback | Adopt P4 guidance for Cargo JSON diagnostics. | Use library/native output, not a bespoke diagnostic authority. |
| Compiler profiling | Defer a campaign. | Revisit an identified slow compilation phase with scoped native timings/self-profile; solver runtime is a different measurement. |
| Coordination and role routing | Retain existing roles; P4 adds owner-linked examples only where useful. | No additional startup tour, mandatory trace or agent ledger is justified. |
| Broad plugin/tool exposure pruning | Defer policy changes. | Tool inventory counts do not prove overhead; revisit an observed startup or tool-selection problem. Preserve explicit session opt-in. |

## Plan

Packages group coherent behaviors. One integration owner owns shared Just/configuration and
documentation surfaces. Bounded delegated investigation or independent judgment may help,
but parallel editing of the justfile, manifests or supervisor declarations requires explicit
ownership. Implementation is complete; the checkpoint and Outcome below own closure evidence.

| Packet | Responsibility / dependencies | Scenarios / acceptance | Replaced code / deletion |
|---|---|---|---|
| [P0](#p0-instruction-contracts) | Accepted RC01/RC02 instruction contracts. | Fixed diagnostic exception and whole-operation effect meaning are stated consistently. | Retire unconditional wording that makes the observer require workload preparation. |
| [P1](#p1-invocation-corrections) | Selector, variadic recipe migrations and zero-argument codegen; requires P0's RC02. | S01–S03 direct entry, exact argv and rejection before effects. | Remove pre-bootstrap import failure, unsafe interpolation and misleading full-codegen flag forwarding. |
| [P2](#p2-observation-and-launch-class) | Truthful reachable activity and outer launch class; requires P0's RC01, coordinates recipe edits with P1. | S04 transport failure, unavailable admission and composed launch classification. | Remove swallowed systemctl errors and eager unrelated image lookup; replace class inference where recipe meaning is known. |
| [P3](#p3-assessment-and-result-reads) | Native assessment grammar/prerequisites and result reader; consumes P1 forwarding. | S02/S07 selected group, exact run identity, incomplete/reused evidence. | Remove positional output grammar and unconditional unnecessary native setup. |
| [P4](#p4-consumed-tool-guidance) | Focused discovery, fixture/freshness, compiler and typed API examples; follows actual P1–P3 interfaces. | S05 useful owner-linked examples with truthful scope. | Migrate superseded examples; no duplicate capability registry. |
| [P5](#p5-native-mcp-and-pilot) | Native MCP credentials/registration and existing-context pilot; uses P1 forwarding and P2's launch owner, adding its fixed admitted-light helper entry. | Database part of S06; context/auth/error/write-refusal controls. | No custom adapter or datastore; no schema migration. |
| [P6](#p6-integration-and-handoff) | Assembled tooling qualification and enduring guidance; requires implemented P1–P5. | One composed acceptance pass, baseline zero, explicit exclusions. | Retire replaced tests/fixtures and completed plan/review according to ADR-0096 after moving enduring meaning. |

### P0: Instruction contracts

Edit `AGENTS.md`, the environment guide and applicable tooling guidance before implementing
their dependent exceptions. Describe the fixed observer route and preserve ordinary capped
workload launch. State that native options do not automatically apply to every stage of a
composed recipe. This is an instruction change, not functional delivery. No accepted ADR or
architecture section needs an incidental edit.

### P1: Invocation corrections

Own `scripts/select.py`, the affected recipe bindings and targeted script tests.
Exercise the real direct-script route from an ordinary checkout; a module-only test cannot
expose F01. Use a fake native argv witness under real Just to verify spaced paths, multiword
filters and literal shell punctuation, including an effect sentinel that must not fire.
Just dry-run alone is insufficient for execution fidelity.

For full codegen, use stage spies to prove unsupported arguments run **zero** stages.
Retain full publication order and the comparison owner's existing scratch/drift controls.
After the corrected selector passes its focused controls, run one real selected unit journey
with force-validation; do not compile the whole workspace solely to prove import routing.
Delete each replaced binding and obsolete grammar test when its actual consumers move.

### P2: Observation and launch class

Own `scripts/activity.py` and the narrow `scripts/pse_env.py`/launch interpretation changes.
Test successful empty observation against nonzero systemctl, timeout, malformed JSON and
partial unit disappearance. Compose the actual justfile under an unavailable-admission
control: the observer executes, acquisition is never called and no solver-image command is
evaluated. An isolated activity function test does not establish F05 closure.

Exercise a known compile recipe whose body wraps Cargo in a shell, caller override and nested
ownership. Verify the intended class is selected before allocation, without changing policy
budgets or admitting arbitrary commands. Preserve existing host-admission lifecycle/drain
tests; the fixed observer is outside that lifecycle and must not mutate it.

### P3: Assessment and result reads

Own `scripts/validation.py` and its existing reporting/selection declarations; keep any
reader module a small consumer of these formats. Use the actual recipe to prove flag-first
selection and a spaced `--output` path reach the parser. A nonnative selected group must
avoid native preparation; a native group must retain its required producer/receiver controls.

For result reads, use independently specified fixtures for success, failed gate, interrupted
checkpoint, reused receipt, missing/malformed record and unknown gate. Assert selected tails
and concrete artifact paths, not snapshots copied from the reader's own transformation.
Ensure no runner, cleanup or checkpoint write is called. Documentation/callers migrate with
the grammar; no compatibility positional path remains.

### P4: Consumed tool guidance

Place command/environment examples in the existing development guides, not a new standing
agent prompt. Show filtered discovery, retrieval from a printed assessment path, managed
versus raw fixture association, and correct Python rebuild choice. For typed inspection,
use existing `ModelingPackage` inspection/source tables and `Runtime` run/attempt/
manifest or diagnostic access, preserving identifiers and stream close/cancel semantics.
Keep examples tied to an actual small supported fixture; do not enumerate the entire store.
Cargo JSON examples explain how to obtain structured spans; this is not a profiling campaign.

### P5: Native MCP and pilot

Own the supervisor's credential/context read helper, `.codex/config.toml` registration and
the environment guide's connection/restart instructions. First interface-check the actual
installed Codex helper/config support and the pinned server tool schemas; the authoring
probe established parsing only, not a working registered session. Prefer an explicitly
bound state/endpoint over implicit environment propagation. Activate the native registration
only after the helper's secret-redaction and failure controls pass.

Run the pilot on the existing server without restarting, initializing or replacing it.
Select a genuinely existing context explicitly. If no suitable context exists, record that
blocker and retain the missing-context evidence rather than creating `canonical`. Use the
existing authorized disposable test-fixture owner for the write-denial control. Confirm a
fresh Codex session exposes and actually consumes the selected native tools. Keep findings
about typed scientific meaning separate from raw schema/row agreement.

### P6: Integration and handoff

After all functional scope is implemented, run the assembled checks below once, fix failures
at their owners and rerun the actual failed recipe. Reconcile consumer inventory, status and
replaced mechanisms. Update enduring guidance, then retire completed working documents and
repair inbound links under ADR-0096. Keep one checkpoint here until that handoff; no
per-command receipt log or additional status file is needed.

## Finding dispositions

| Finding reference | Scenario | Disposition | Decision / work owner | Closure evidence required |
|---|---|---|---|---|
| [F01](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#f01) | S01 | Resolved | P1 | `test_select` direct/module controls and the selected force-validation Rust unit journey passed. |
| [F02](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#f02) | S02 | Resolved | P1, consumed by P3/P5 | `test_recipe_arguments` actual Just argv/effect-sentinel controls and variadic-binding inventory passed. |
| [F03](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#f03) | S03 | Resolved | P0/P1 | `test_recipe_arguments` zero-stage rejection passed; assembled codegen comparisons passed without repair. |
| [F04](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#f04) | S04 | Resolved | P2 | `test_activity` empty, failed, timeout, malformed and partial controls passed. |
| [F05](../design_review/reviews/design_review_agent-effectiveness-enhancements_2026-10-09.md#f05) | S04 | Resolved | P0/P2 | Actual Just unavailable-admission observer and compile-class controls passed; live activity succeeded without an outer wrapper. |

## Verification

**Tested (2026-10-09):** Package controls above exercise the changed behavior. Extend the existing
`scripts/tests/test_recipe_arguments.py`, selector tests, `test_pse_env.py`,
`test_host_admission.py`, `test_validation.py` and `test_surreal_server.py` where their
owners fit. Compile checks and targeted tests accompany implementation, without mid-scope
hygiene/integration campaigns. New mechanisms receive their controls in the same change.

**Tested (2026-10-09):** P6's assembled acceptance used `just setup-test`, applicable
`just native-setup-unit` and `just surreal-test` controls, plus one actual
`just unit-package pse-structural structural_identity_preserves_independent_block_order`
journey. The recipe retains force-validation. The MCP pilot must exercise the actual native
endpoint and selected context, not substitute a constant query for schema/read evidence.
Run `just hygiene`, `just governance` and `just docs` at functional scope end, and applicable
manual native/powerset lints if touched. Follow normal `ready` triggers after dependency,
toolchain or skill-selection changes or environment-shaped failures. Root runs `turn-end`
after changed-file turns. Report each executed command, mode, scope, zero failure baseline
and actual result; a repaired composite pass is not a clean first pass.

No scientific model, execution algorithm or feature graph change is planned. Whole solver,
parity, performance and scientific campaign qualification is therefore not required to close
this tooling scope. If implementation changes those contracts, reassess qualification at
that boundary rather than claiming a targeted tooling pass covers them. Plan 28 campaign
status does not change.

**Proposed, optional measurement:** Compare a concrete discovery/result-read task and the
native MCP inspection pilot against existing native commands for answer correctness, calls,
elapsed time and context volume under stated conditions. Record startup/preparation separately
from steady-state calls. These observations can inform retention, but no numerical speedup
or broad agent-effectiveness claim is a completion condition. Do not add telemetry or another
log collection service solely for this plan.

**Interface-checked during authoring:** Current source inspection and independent focused
advice settled the fixed-observer, outer-class, codegen and reporting boundaries. Context7
documentation was checked against the exact installed/pinned source for transport choices.
A disabled temporary Codex registration parsed `http_headers_helper` and a tool allowlist;
it did not execute the helper, save configuration or connect to a database. The source review's
native HTTP MCP controls remain historical evidence, not newly executed Plan 31 acceptance.

## Current checkpoint and open items

P0–P6 are complete for this tooling scope. F01–F05 are resolved by the controls above.
The actual native MCP session used an existing explicit context; the SDK and CLI retain
WebSocket. Registration is disabled by default and enabled per fresh session. There is no
remaining implementation or qualification work in this plan. The scientific campaign in
Plan 28 remains paused and is not qualified by these results.

Enduring instructions and examples live in AGENTS.md, the tooling rule and the existing
agent environment, validation, native execution and SurrealDB development guides. Replaced
bindings, inference assumptions and stale fixtures have been removed. The highest-numbered
plan remains under ADR-0096 until a newer plan exists. Its newly authored source review and
evidence are retained for this uncommitted handoff: no immutable Git archive exists for them
yet. They are resolved inputs, not active work; retire them through Git history when that
archive exists and repair historical references then. No second status owner was created.

Transport references used to settle P5:
[SurrealDB 3.3 native network MCP](https://github.com/surrealdb/surrealdb/blob/v3.3.0/surrealdb/server/src/ntw/mcp.rs),
[Codex 0.161.0 MCP configuration](https://github.com/openai/codex/blob/rust-v0.161.0/codex-rs/config/src/mcp_types.rs)
and [header helper implementation](https://github.com/openai/codex/blob/rust-v0.161.0/codex-rs/rmcp-client/src/http_headers.rs).

## Outcome

**Implemented (2026-10-09):** P0–P6 delivered direct selector bootstrap, positional recipe
arguments, zero-argument full codegen, known outer resource classes, truthful admission-free
activity, flag-first selected assessments, and the bounded `just result` checkpoint reader.
Existing guides now explain native discovery, fixture association, Python rebuild choices,
structured Cargo feedback and typed package/run/result inspection. No semantic Rust search,
custom MCP server, dependency change or scientific implementation was added.

**Implemented:** Codex registers SurrealDB's native HTTP MCP endpoint with `info`, `list`
and `query` tools, disabled by default. The fixed header helper verifies the existing private
VIEWER credentials and actual owned service/listener under one eight-second deadline. It uses
an ordinary admitted light scope without compiler preparation; stale invocation/inode/binding
proofs fail closed, and cleanup must finish before credentials are emitted. Database SDK
connections remain WebSocket. Start a fresh session with
`codex -c mcp_servers.pse-surreal.enabled=true`.

**Tested (2026-10-09, local Linux checkout, zero failure baseline):**

| Command / mode | Scope and result |
|---|---|
| `just setup-test` | Repaired assembled run: 524 unit controls, zero failures/errors/skips; `build/setup-tests/setup-test.xml`. |
| `just native-setup-unit` | 44 native generation, admission and lifecycle controls passed; no scientific solver execution. |
| `just surreal-test` | 81 supervisor and native MCP helper/protocol controls passed. |
| `just unit-package pse-structural structural_identity_preserves_independent_block_order` | One selected Nextest unit passed with explicit force-validation; nine tests outside selection. |
| `just hygiene` | All 28 checks qualified, including Python lint/types, code generation comparisons, default/no-default Clippy and Rust docs; complete, source unchanged, zero provenance errors. Evidence: `build/assessment/20261009T160407.641805Z-hygiene-1892814-f5e43f`. |
| `just governance` | All 12 checks qualified, complete and source unchanged; `build/assessment/20261009T155956.807993Z-governance-1858314-c7d6d6`. |
| `just docs` | Published 288 chapters with scoped search. |
| `just ready` | Skills/environment checks passed after environment-shaped diagnosis. |
| `just turn-end` | ADR index and formatting passed after assembled functional repairs. |
| Direct `mcp-headers` with inherited `PSE_*` values removed | Final helper returned only the Authorization header key, exit zero, empty stderr; credentials were not printed in evidence. |
| Direct `just activity --json` | Live unit observation available, no observation errors, without an outer admission wrapper. |

**Tested:** A fresh ephemeral Codex session enabled the actual registered native MCP and
consumed `list`, `query` and a missing-context `list`. It selected existing
`pse/canonical_test_2cb06580347d416eb13556e001d35b55`, inspected
`canonical_interpretations` and read one row. The schema and row agreed exactly with the
released WebSocket CLI under VIEWER. Absent `pse/canonical` returned NotFound without
initialization. Session output is `/tmp/plan31-codex-pilot.jsonl`; the original service
instance `82507491-2bbc-4c4d-9627-c3c8ffd5210d` remained unchanged.

**Tested:** `scripts/pse-env -- .venv/bin/python -m scripts.tests.surreal_fixture_check
--mcp-only --mcp-state /home/paul/.local/state/pse-arrow/surreal-functional-v2
--evidence build/plan31-mcp-fixture` passed contextual read, bad-auth refusal,
missing-context refusal and write-denial controls. Privileged readback proved the forbidden
record absent even when the native write call returned an empty result without a protocol
error. The unique owned namespace was positively retired and clients closed. The retained
service and databases were preserved. `build/plan31-mcp-fixture/passed.json` owns the receipt.

### Mistakes corrected

The first setup run had 14 failures, 12 errors and four explicit opt-in skips against zero;
the first hygiene run failed Python lint (56 findings) and typing (47 diagnostics). These
were repaired at their owners and the actual recipes rerun. Fixtures now isolate inherited
ownership, mock the current launch producer/import seams and preserve exact routing and
state assertions. Explicit host controls moved to `plan30_*_check.py`, preserving their
assertions and opt-in switches rather than weakening setup's zero-skip requirement. The
recovery instrument had incorrectly assumed reconfiguration refused a budget-valid server
reduction; it now tests reduced-profile recovery refusal through the actual exact-profile
validator without mutation. No live recovery qualification was run.

An early helper trial treated empty kernel scope state as sufficient cleanup despite
lingering active manager metadata; another could rebind stale ownership. Cleanup now retires
only its own verified empty scope/ancestor, bindings cannot be overwritten, and bounded
inheritance includes inode identity. The task's verified empty failed transients were retired;
unrelated and resident service ownership was preserved. Nonblocking bounded metadata reads
also reject FIFO/special-file substitutions. Direct activity now supplies manager address
defaults so an ordinary Codex shell can observe the user manager.

### Deliberate deviations and limits

A second disposable server could not enter the occupied single functional store lane. The
native MCP negative controls therefore used a uniquely owned disposable namespace on the
existing service, under the existing fixture lease and cleanup owner. No retained database
writes, server restart, parking or admission-policy change was performed. An initial toy
query used the reserved `value` spelling without quoting; it was corrected, the failed
owned fixture resumed safely and its namespace positively removed before the fresh passing run.

**Proposed, not measured:** Broader agent efficiency or numerical speedup remains unclaimed.
The pilot establishes native tool availability and raw schema/row agreement, not typed
scientific meaning or scientific correctness. Solver, parity, performance and the paused
scientific campaign were not run; native/powerset lints were not applicable because no Rust
feature graph or scientific contract changed. This is a repaired composite tooling pass,
not a clean first pass or whole-product qualification. All changes remain uncommitted in
the existing checkout.
