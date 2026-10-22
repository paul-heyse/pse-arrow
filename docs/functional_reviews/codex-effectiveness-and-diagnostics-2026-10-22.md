---
title: "Codex effectiveness and diagnostic tooling"
date: 2026-10-22
doc_role: review
doc_owner: docs/functional_reviews/README.md
doc_retention: while-dependent
doc_topics: [environment, validation, design, libraries]
---

# Codex effectiveness and diagnostic tooling

## Judgment and scope

**Proposed:** make Codex's ordinary interaction with this repository more seamless by
composing the capabilities already owned here, and add several substantial diagnostic
capabilities that the current interfaces do not supply. The strongest immediate work is
one-time environment activation, dependable selected-store and language-service attachment,
explanations of waiting and retained resources, a product-test result reader, and direct access
to the database diagnostics already compiled into the selected server. Mixed-language crash
inspection, actual scheduling evidence and suspended-task inspection would extend what Codex
can diagnose when existing operation observations are insufficient.

This is a functional opportunity review for Paul as sole operator and Codex as sole programming
agent, including its Codex subagents. It covers tooling, commands, configuration, CLIs, source
navigation, change scoping, verification, execution, performance insight and recovery. Existing
pain supplies evidence of useful work; opportunities can also enable activities that have no
current caller or recorded failure. Earlier engagement-specific exclusions, including semantic
search and off-CPU inspection, do not constrain this request.

The workstation's selected ordinary build defaults are sixteen compiler jobs and unbounded
ordinary builds. Scientific/reference allocations have their own meanings. This review proposes
no new machine-wide memory budget or concurrency governor. Generated scientific/test data can
be regenerated; historical comparison is not automatically a preservation obligation. Actual
process ownership, scientific truth and references to artifacts remain meaningful even in a
personal project.

The supplied local input `docs/libraries/surrealdb_tooling_idea.md` provided leads. Its bundle,
"essential" labels, deployment assumptions and schema-authority proposal are not accepted
requirements. The review evaluates those suggestions against the actual custom build and existing
owners. It also examines capabilities outside that document.

### Source baseline and evidence limits

**Interface-checked:** `main` at `79a8dbfc98046cb4097633dbfbe26b26df9e6c27`, with the supplied
untracked idea document as additional input. The inspected baseline includes selected Plan49a–e
completion and preserves independent remaining 49f–h scope. Core 3.4, Efficient Architecture
Heuristics 1.0, Process Simulator 1.5 and the pse-arrow binding shape this assessment. The profile
matters where inspection touches scientific validity, solver outcomes or reproduction; a richer
diagnostic view never establishes scientific usability by itself.

The inquiry indexed user messages, tool calls and results in **23 local Codex session files** for
the period **2026-10-21 18:45:30.753 UTC through 2026-10-22 18:45:30.753 UTC**, ending at the original
review request. These comprise the main Plan49 thread and 22 child threads. Entries were selected
by timestamp and repository, including commands targeting its worktrees. Call IDs deduplicated
inherited tool records; repeated polls, quoted logs and search matches were treated as leads,
not independent incidents or measured time wasted. Representative incidents were read and
followed into current source and landed repairs. This is not a manual assessment of every log
record. Raw transcripts and credentials are not republished.

Three bounded contributions covered command/environment owners, SurrealDB tooling and native
diagnostic opportunities. Their evidence was reconciled with the actual consumers. Sources include
the current architecture and operating guides, active-plan checkpoints, exact installed and pinned
upstream source, official documentation and bounded read-only observations described below.
Current-code descriptions are **Implemented/source-inspected**. New recommendations are
**Proposed**. No tool installation, dependency change, product build, product test suite, database
mutation, service lifecycle action or performance campaign was performed for this review.

## What already exists and should be used

The repository has considerably more capability than a list of recipes suggests:

| Existing foundation | What it already supplies | Boundary to preserve |
| --- | --- | --- |
| `scripts/pse-env`, direnv, build/native owners | Checkout environment, local overrides, compiler cache, native selections and execution placement | One environment authority; ordinary and scientific allocations have different purposes |
| `just activity`, `pse-env --explain`, `just affected` | Read-only process/scope observations, configuration explanation, Rust reverse-dependency test preview | Partial observations and uncovered consumers are explicit; these are not complete causal analyses |
| `just result`, assessment and test runners | Assessment checkpoints, bounded tails, selected test inventories, JUnit, native evidence and invocation terminals | An assessment reader and a product-test terminal answer different questions |
| Server, host-admission and test-resource owners | Authenticated server identity, registration, draining, parking, recovery, retention and reclamation | Inspection is separate from reconciliation or disposal; unknown effects are not success |
| Plan40/47 diagnostics | Early operation IDs, copied observations, process/native-thread locators, CPU capture, timelines, numerical analysis and offline review packs | Span lifetime is not thread residency; overlapping intervals are not exclusive elapsed time |
| Salsa, mathematical and relational observations | Query events and retention, evaluator structure/storage facts, actual DataFusion plans/rules/partition metrics | Representation counts, observed execution and attributed causes are distinct |
| Selected SurrealDB build | SurrealMX 1.0, SSI, gRPC, Surrealism, native MCP, generated schema/functions and MX statistics | Diagnostic integration must target this build and store, with existing semantic owners |
| SurrealKit and schema generation | Pinned beta.6 with patched gRPC SDK, generated declarations, immutable rollout production and guarded maintenance | The registry/generator owns schema meaning; SurrealKit supplies planning/execution mechanisms |
| Document lifecycle and reading routes | Current/reference/history scope, native plan-state aggregation, owner links, published search | Derived context is a view of owners, not another status ledger |

The [environment guide](../dev/agent-environment.md), [diagnostic guide](../dev/simulation-diagnostics.md)
and [qualification guide](../dev/validation-assessment.md) are useful entry points. Source routes
at the end identify the underlying owners. The opportunities below extend these foundations.

### Recent friction and its present disposition

The main local thread is `01a15da2-f527-7e42-9fac-5e8cb9c6ff55`. The following references retain
enough identity to find representative evidence in local Codex history without copying raw logs.
Dates/times are UTC; child IDs identify separate Codex evidence sources.

| Evidence | Observed workflow | Current disposition and opportunity |
| --- | --- | --- |
| L01: Oct21 20:22, child `01a1601d-82a9-7c10-af5d-18e6737c703a`, call `call_66d6c2b9b7754dddb7eb952401d8fb93`; Oct22 09:03, child `01a162d6-f90f-72a3-993f-bef851bfda0b`, call `call_aba578739b76405f997d402db7df477d` | `compiler_cache.py` failed with `KeyError: SCCACHE_DIR`; subsequent commands repeatedly used `env -u BASH_ENV` | Current launch code still inherits the startup hook into nested Bash; O01 addresses the composition seam |
| L02: Oct22 00:05–00:14 user messages; root call `call_3985cd362593420b8be85107a825d33c` at 00:19 | Manual two-job/memory overrides, requests for sixteen jobs, build lock waits and requests for regular commits | Job/default and aggregate classification repairs have landed. Explain actual build state and avoid restoring obsolete overrides, O04/O05 |
| L03: Oct22 06:44–06:54 user messages | Low visible CPU, uncertainty about cleanup versus useful testing, then cancellation of the oversized native sweep | Representative functional selection is now the accepted route; O06/O20 improve explanations and small diagnostic cases without restarting that sweep |
| L04: Oct22 14:55 root update; 15:20 operator reply | Compilation finished but teardown could not qualify because user-manager inotify watches were exhausted | Operator restored the documented limits. O06 should expose host prerequisite failures near the wait they explain |
| L05: Oct22 15:19 operator reply, historical invocation `d63c4731f12e4cc7b2a0d7eef19f7fac` | Historical unfinished allocation/fixture records obstructed new reference work; operator explicitly selected disposal of regenerable state | Historical disposal is recorded in Plan49. O09 supplies an ordinary supported workflow for this class of action |
| L06: Oct22 16:42–17:00 root updates | Managed-worker read conflicts required following worker logs into readiness/readback consumers; later cancellation overflowed a Tokio worker stack | Read conflict repairs `74e61d985`/`8473c9418` and future-allocation repair `edf23b4cb` have landed. O10/O13–O16 improve diagnosis, not blanket retries or larger stacks |
| L07: Oct22 17:30, child `01a163f1-2b41-7612-ab5b-b1cfac65e74b` | Follow-through found incomplete fitting policy extraction despite existing pure helpers | Repair `8529c10ad` landed. O03/O04 help inspect real consumers and current commitments; no automated completeness claim follows |
| L08: root/worker tool outputs throughout the window | Large manifests/process dumps, truncated output, manual report-path searches and noisy obsolete-profile cleanup messages | Existing compact assessment and frozen-review readers help. O04/O07 extend selective retrieval to the remaining run/artifact surfaces |

These are investigation seeds, not eight unresolved defects. In particular, the checkout-Python
launcher repair (`465e6800c`), compile-default repairs (`3c256eb6a`, `a9d783c28`), local observation
fixture isolation (`7d31c768e`) and the final product repairs above must not be proposed again as
unwritten work. Regular discrete commits are already the selected workflow.

### Small observations made during this review

**Tested, bounded environment observation:** a local Python probe called
`pse_env.compose(root, os.environ)` and launched one nested noninteractive Bash that reported nine
selected nonsecret compiler/environment fields. With the inherited startup hook, both child and
control exited zero and retained those fields; the inherited hook emitted two direnv lines and
the control with `BASH_ENV` removed emitted none. This **did not reproduce the historical
`SCCACHE_DIR` loss**. It confirms repeated activation, alongside source evidence and the historical
failures; it does not qualify a proposed repair. An initial exploratory variant omitted local
overrides and was not used as evidence of a configuration defect.

**Tested, bounded live observation:** invoking `.venv/bin/python scripts/activity.py --json`
through a read-only subprocess returned exit 1 with 93 observed units, no checkout build processes
and an explicit "unit disappeared during observation" error. The systemd unit race is already
reported truthfully; it is not an empty-success bug to repair. Read-only kernel inspection found
524288 inotify watches, 1024 instances and an Apport `core_pattern`. This confirms restored
inotify values and the host's selected crash route at observation time; it establishes neither
remaining watch capacity nor recoverable core availability. Process counts are momentary.

**Interface-checked, local diagnostic prerequisites:** a selective PATH/version check found perf,
Samply, GDB 15.1, py-spy 0.4.2, Heaptrack 1.5.0 and rust-analyzer-mcp. The checkout's Python
reports `PY_HAVE_PERF_TRAMPOLINE=1`. Memray, rr, surreal-lsp and coredumpctl were not found on that
invocation's PATH. This is tool/build-option presence, not successful attachment, capture,
unwinding, native-library compatibility or proof that an absent tool is unavailable elsewhere.

## A. Seamless editing and feedback

<a id="O01"></a>

### O01 — Activate the environment once per top-level Codex command

**Proposed; compelling first change.** Eliminate the recurring need to prepend
`env -u BASH_ENV` while preserving checkout selection and explicit compiler/native settings.
The machine configuration injects `~/.config/codex/bash-env.bash`; that bridge unloads inherited
direnv state, sources workstation defaults and activates the selected checkout. It leaves
`BASH_ENV` inherited. Just then launches additional Bash interpreters, and `scripts/sccache` has
a Bash shebang. `pse_env.compose`/`execute` pass the mapping through. The cache owner later
requires `SCCACHE_DIR`, making reactivation after compiler selection a consequential boundary.

Prefer the small machine-bridge change: unset `BASH_ENV` after successful initial activation.
Codex supplies the configured hook afresh for each top-level command; nested workload shells
then consume the selected environment. Keep `scripts/pse-env` as the workload composition owner.
A repository child-environment defense is an alternative where non-Codex parents also inject
startup hooks; avoid two independent normalization policies. Document explicit re-entry when
changing checkouts inside one long-running shell.

Reconcile the guide's historical `local-env` profile wording and its later "without direnv"
claim with the actual promoted default. Preserve caller/local/repository precedence, bootstrap
fallback and failure on an unapproved checkout. The selected sixteen jobs and unbounded mode
already exist; propagation is the issue. [Environment source][src-env];
[official shell/configuration reference](https://developers.openai.com/codex/config-reference/).

**Delivery example:** a fresh Codex command in main and then a worktree selects each checkout;
nested Just, Cargo, cache-wrapper and native children retain the selected values without a manual
hook override. Exercise a missing venv and failed direnv activation as distinct cases. The current
bounded probe is insufficient for that acceptance.

<a id="O02"></a>

### O02 — Make useful Codex capabilities attach to the intended session

**Proposed; compelling.** Offer a straightforward way to start a Codex session with the selected
canonical store and semantic Rust navigation, and report what actually attached. Both MCP
registrations and their opt-in launch commands are already documented, but disabled by default. This session exposed neither native
SurrealDB nor rust-analyzer tools. Configuration presence is not evidence of working attachment.

The SurrealDB registration fixes port 18240 and the `surreal-functional-v2` state, while
ordinary work can select another state through `PSE_SURREAL_STATE`. Build session attachment from
an explicit selected state and its verified endpoint using the existing helper. Freeze that
selection for the connection; reconnect deliberately after state/credential changes because
cached headers must not silently inspect a different store. Preserve the exact identity checks
and VIEWER access. Do not start an embedded database to make inspection appear available.

Use the native `info/list/query` tools and focused resources such as
`surrealdb://schema/ns/{namespace}/db/{database}/table/{table}`. Exact 3.3.4 source also exposes
instructions/version resources and schema/query/graph prompts. The modern protocol is stateless:
pass namespace/database explicitly; `use` is not the session-selection mechanism. Preserve
`truncated`/`tables_truncated_at` markers. Resource/prompt support in the actual Codex client needs
a real interaction, not source-only acceptance. [Native MCP source](https://github.com/surrealdb/surrealdb/tree/v3.3.4/surrealdb/mcp).

For Rust, start with the already configured rust-analyzer MCP, its separate target directory and
selected features. Verify references, implementations, call hierarchy, macro/proc-macro and native
feature behavior in this workspace. Python navigation should cover an actual generated-stub →
facade → PyO3 implementation journey. The pinned installed Pyrefly 1.3.1 exposes `pyrefly lsp`;
its official language-service documentation includes definitions, references and call hierarchy.
Assess that existing server before another Python LSP, then select a Codex adapter. Its CLI
reports background indexing and a default workspace indexing limit; an incomplete index must not
be treated as an exhaustive reference search. Native Rust/PyO3 linkage still needs the catalogue
and Rust navigation. [Pyrefly IDE documentation](https://pyrefly.org/en/docs/IDE) supplies the
mechanism; the current Ruff/Pyrefly CLI remains the quality authority. SurrealQL assistance is
covered by O12. A session capability view should distinguish configured, enabled, connected,
scope selected and unavailable. Existing Codex configuration and `/mcp` facilities should carry
this; a new launch platform is unnecessary. [Agent configuration][src-codex];
[official MCP guidance](https://developers.openai.com/codex/mcp/).

**Delivery example:** a newly launched session finds a Rust trait implementation and inspects the
current store's exact table through MCP. A stale or stopped selection yields an actionable
unavailable result without starting, switching or initializing a store.

<a id="O03"></a>

### O03 — Combine semantic navigation with explicit change-impact leads

**Proposed; high value.** Codex should be able to follow a proposed change through Rust, PyO3,
Python, generated contracts, schema declarations and actual tests without reconstructing every
connection from text searches. `just affected` already provides an honest Rust package
reverse-dependency preview, including untracked/deleted files and explicit omissions. Preserve it.

Use rust-analyzer's references, implementations and call hierarchy for semantic questions;
retain `rg` for vocabulary, dynamic dispatch/configuration and generated declarations. The
existing `cargo.targetDir=true` avoids editor/build contention at the cost of extra artifacts.
Native-feature and proc-macro scope must be stated. [rust-analyzer configuration](https://rust-analyzer.github.io/book/configuration.html)
and [LSP extensions](https://github.com/rust-lang/rust-analyzer/blob/master/docs/book/src/contributing/lsp-extensions.md)
establish capabilities, not completeness for this workspace.

Compose explicit cross-boundary leads from the existing contract catalogue, generator ownership,
test inventory and architectural routes. Extend `affected` with machine-readable reasons and
links where that is useful; do not invent an authoritative universal impact classifier. Rust
package reach does not establish Python, fixture, SQL, numerical or behavioral coverage.

A local semantic/text search index remains eligible under this broad request. Prefer native
symbol and scoped document retrieval initially; consider an additional index when a concrete
question remains costly. Any index should expose source revision, scope and exclusions and remain
rebuildable. Canonical scientific graphs, code references and document relationships have
different meanings and should not become one universal graph authority. [Selection source][src-selection].

**Delivery example:** changing a product document identifies its declaration, generator, Rust
consumer, PyO3 conversion, Python facade and relevant test selections, with unresolved dynamic
consumers visible. A source inspection still settles whether every actual consumer migrated.

<a id="O04"></a>

### O04 — Make commands, context and outputs easy to consume selectively

**Proposed; high value when built into existing surfaces.** Improve the interaction from "what
am I trying to do?" to the right existing command, exact selection and useful output. `just
--list/--usage/--show`, JSON recipe dumps, the reading guide and document lifecycle queries already
supply parts of this. Add concise task routes and structured descriptions to their owners,
including effects, prerequisites, output handle and meaningful coverage limits. Derive grammar
and defaults from their existing declarations; author workflow meaning without duplicating the
command catalogue.

For common flows, Codex should discover "check this package," "inspect this invocation,"
"explain this wait," "read this schema" or "resume this plan" without repeatedly loading entire
guides and manifests. Render short summaries with identifiers and paths; retrieve selected fields,
pages and tails on demand. Preserve full raw output once rather than printing huge nested manifests
or process argument lists. A summary should state omitted/partial detail and retain the command's
actual exit status. Existing assessment readers and frozen review packs are good patterns.

Derive a resume/change context from current native plan items, owner links, Git changes and
selected evidence. A context view can point out historical baseline paragraphs without changing
status or declaring completion. Preserve regular discrete commits and current checkpoints; do
not create a second task ledger or a mandatory per-command receipt process. Reusable bounded
Codex log extraction can support an operator-requested retrospective without continuous transcript
ingestion. [Document/command source][src-context]; [report source][src-results].

Treat the current Codex version, loaded configuration and hook/capability support as observed
facts. The inspected CLI is 0.163.0; guide references to 0.160 require rechecking. Official hooks
support `Edit`/`Write` matcher aliases for `apply_patch`, and the local guard parses Codex patches;
their spelling is not evidence that the guard is broken. Loaded/trusted hooks still require
runtime verification. [Official hooks](https://learn.chatgpt.com/docs/hooks) and
[configuration layers](https://learn.chatgpt.com/docs/config-file/config-advanced).

**Delivery example:** a fresh session obtains the current task's relevant owners, pending scope,
exact command grammar and selected result fields without a full manifest dump. Absent evidence
remains absent. No reduction in elapsed development time is claimed before observation.

<a id="O05"></a>

### O05 — Explain compilation, cache behavior and build contention

**Proposed; compelling.** Answer "what is the build doing?" with actual stage and evidence:
compiler execution, linking, build script, cache reuse/miss/bypass, Cargo artifact-lock wait or
host admission. Sixteen configured jobs do not imply sixteen runnable compiler units. The
existing activity observer explicitly cannot identify an arbitrary Cargo lock owner.

Use Cargo JSON progress/compiler messages, selected-target metadata, existing cache diagnostics
and actual process/cgroup observations. A command-scoped progress view can associate phases and
their observed age with its invocation. Identify a lock owner only when authenticated association
supports it; otherwise report a lock wait with owner unknown. Keep cache counters scoped to the
observed interval and distinguish a shared daemon's counters from one command's results.

Surface the existing distinction between concurrent check-type work and artifact-producing
builds. Separate Clippy feature graphs currently have a known artifact-lock upgrade cycle and
should run sequentially until a different safe realization is demonstrated. Independent builds
may use genuine isolated worktrees when worth the additional cold artifacts; do not force all
heavy work to be serial or share target directories. Existing compiler timing/cache recipes are
the first mechanisms to expose. [Build/cache owners][src-build].

**Delivery example:** during a cold build, Codex can tell Paul that native code generation is
active, a linker is running or a command is waiting for another artifact producer. A focused
rebuild reports cache behavior without pretending it predicts solver performance. Preserve the
selected default repairs rather than reintroducing two-job overrides.

## B. Execution, tests and recovery

<a id="O06"></a>

### O06 — Explain live waits and their actual owners

**Proposed; compelling.** Extend read-only observation to answer "what blocks progress, and
what fact would let it continue?" Combine `activity` with the existing
`host_admission.readonly_snapshot`, selected service state, process identity and runtime
operation/checkpoint links. The host's current `status()` calls `reconcile()` and is not a
substitute for its read-only snapshot.

Expose allocation ownership, parked/resume intent, competing workload class and known resource
relationships. Admission currently keeps its retry reason locally and reports it only on deadline
exhaustion; it does not persist a per-waiter queue/reason. Add owner-side live wait evidence where
needed, preserving unknown reasons and owners rather than presenting them as existing fields. Follow only the relevant neighborhood, with explicit
unavailable observations and continuation. Do not create a second scheduler or lock ledger.
Diagnose host prerequisites such as inotify exhaustion when evidence indicates that they explain
teardown. Read configured limits and relevant user-manager errors together; configured watch
counts alone do not measure remaining capacity. [Activity/admission owners][src-activity].

Keep the existing truthfulness of unit-disappearance observations. A vanished `/proc` process is
a normal point-in-time omission; report enumeration limitations where they matter rather than
turning ordinary races into hangs. Distinguish process CPU, thread CPU, wall time, live phase and
last checkpoint. A stale checkpoint or quiet native interval alone does not establish a deadlock.

**Delivery example:** a reference allocation explains that an owned normal store is parked, which
allocation holds exclusion, and what supported resume action applies after release. An empty
scope awaiting user-manager teardown shows the actual manager error. Inspection performs no
reconciliation, cancellation or service startup.

<a id="O07"></a>

### O07 — Read product-test evidence through one stable run handle

**Proposed; compelling.** Extend result inspection to Rust/Python/native product test runs.
`just result` currently reads assessment `checks.json` v5. Product runners separately produce
selected catalogs, JUnit, native/provenance documents, invocation terminals and fixture records.
Finding the relevant combination currently requires knowing several paths and formats.

Add a kind-aware reader over those existing outputs, using an invocation or printed run handle.
Show selected tests, reported pass/failure rows, selected-but-unreported tests, invocation-level
interruption, actual binary/extension association, terminal/physical-drain status and links to
retained resources with their retention reasons. Individual in-flight interrupted test identity
may be unknown. `test_run.finish` already joins selected catalog, results, report errors, workload
exit, owned drain, artifacts and frozen terminal; the extension is unified reading/discovery.
Keep assessment and product-test semantics explicit. JUnit alone is not proof of a reconciled
terminal; a readable report is not a passing run. Reuse existing readers and parsers, tolerating
incomplete output as incomplete rather than synthesizing success. [Run/result owners][src-results].

Provide short failure summaries and selective raw log/JUnit access. An explicit invocation
association should resolve ambiguity instead of choosing a stale `latest` path or unrelated
`target/nextest/default/junit.xml`. Listing recent handles is useful, but exact identity remains
the basis of a conclusion.

**Delivery example:** Codex reads a managed Rust run and a Python run, sees their actual selections,
failure diagnostics and drain/retention state, and follows the correct worker log. A SIGINT run
preserves invocation interruption and unreported selections without inventing per-test outcomes
or a new scientific failure count.

<a id="O08"></a>

### O08 — Compose reference fixtures under one execution owner

**Proposed; compelling.** Provide a reusable task route for the manual sequence currently
documented as setup, reference allocation, service start, canonical initialization, selected
study and quiesce/drained stop. Build/setup can intentionally precede the runtime owner.
All phases requiring the reference lifetime should run under one owner. Starting a service under one short-lived owner and testing under another can correctly
leave the second command with an unavailable service.

Compose the existing supervisor, test-resource and host-admission owners; use one explicit
state/profile/worker selection and preserve the literal study command. Return the run handle
and independent setup, test, cancellation and teardown outcomes. Restore a previously parked
normal service according to its retained resume intent, and explain a refusal instead of hiding
it. No new fixture registry, port allocator or execution framework is needed. [Fixture owners][src-lifecycle].

Make the route useful for a small functional control as well as a selected reference study.
Its test population should match the contract being assessed; production lane count or timeout
settings do not determine a functional test's necessary repetition count. Preserve explicit
force-validation and native-provider selection.

**Delivery example:** an ordinary command can prepare a private reference fixture, execute six
selected controls and settle it; interruption exercises the same owned teardown path. Failure
retains the correct evidence, while ordinary work subsequently resumes the intended normal store.
This is a workflow proposal, not an instruction to rerun the canceled sweep.

<a id="O09"></a>

### O09 — Support explicit abandonment and regeneration of disposable state

**Proposed; compelling with a narrow authority change.** Provide an intelligible supported workflow
for future obsolete generated invocations that cannot acquire a normal terminal, then regenerate
their fixtures. The exact historical invocation in L05 has already been dispositioned: its owned
dead scopes, unfinished guard and allocation were retired, and the poisoned store was stopped
after all 103 worker inventories were absent/drained. Unknown historical creation effects remain
incomplete; no successful terminal or acknowledgement was invented. The proposed delta is a
repeatable owner-level operation for this class of case, not more recovery for that invocation.

First explain why an invocation/resource is retained: live owner, uncertain creation effect,
pin, borrower, evidence reference or incomplete terminal. Existing `pse-test-resources`
`list/status/pin/release/release-reference/reclaim/resume` and supervisor recovery commands are
the semantic owners. Exact-resource reclamation already applies ownership/cleanup gates;
`resume` projects frozen truth and does not rewrite a result. Add an
explicit abandon/retire route for an exact invocation and its owned generated resources. It must
authenticate physical owners/drain and reconcile references before release; an abandoned run
remains abandoned, with unknown effects recorded where they cannot be resolved. It must not
fabricate a passed terminal, worker acknowledgement or successful scientific outcome.

Use producer roles to distinguish regenerable scratch/state from retained evidence and
external/unowned files. A plain `cargo clean` or manual deletion is not a replacement for this
semantic operation. Compose the existing `build-storage` observation and Plan32 lifecycle routes
so Codex can see what consumes disk, why it is held and which owner can dispose it. Avoid scans
on every command and avoid a generic garbage collector. [Lifecycle/storage owners][src-lifecycle].

**Delivery example:** a fresh fixture reproducing the historical blocked-invocation condition can
be explicitly abandoned after owned processes are authenticated absent, its fixture retired and
fresh generated state prepared. An unrelated live owner or protected shared reference prevents that release with a
specific explanation. Preservation policy changes need the existing decision route; this review
does not silently weaken evidence retention throughout the system.

## C. Database and query tooling

<a id="O10"></a>

### O10 — Diagnose actual queries and rehearse alternatives on disposable fixtures

**Proposed; compelling.** Codex should obtain the actual owned query, bound parameter identities,
schema/index definitions and execution plan for a readiness, publication or result-selection
operation. Generated schema and query owners already exist; targeted readiness plan controls
already inspect parts of this. Do not maintain another handwritten collection of equivalent
diagnostic queries. Expose or derive the actual query through its owner where necessary.

Use `INFO FOR DB STRUCTURE`, focused table structure, `INFO FOR INDEX ... ON ...`, and the native
`EXPLAIN FORMAT JSON` / `EXPLAIN ANALYZE FORMAT JSON` statements where compatible with the exact
3.3.4 query shape. Index status and index definition are distinct observations. `ANALYZE` executes
and drains the inner plan, so classify it as an experiment with actual work/effects. Preserve
raw plan output because its upstream shape can evolve. The exact
[ExplainPlan/AnalyzePlan source](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/core/src/exec/operators/explain.rs)
preserves the inner plan's access mode and records elapsed time, rows and batches.
[Database/query owners][src-database];
[EXPLAIN](https://surrealdb.com/docs/reference/query-language/statements/explain),
[INFO](https://surrealdb.com/docs/reference/query-language/statements/info).

A valuable further workflow is a disposable query/index experiment: populate representative
fan-out, execute the current operation, try an alternative, compare correctness and examined
work/plan observations, and produce a proposed source change. Reuse O08's lifecycle rather than
opening another datastore or introducing a general tuning service. Static plan structure can
identify unnecessary work; throughput/latency claims require the actual experiment.

**Delivery example:** a readiness scan explains its index and examined work, including continuation
and contention, and an alternative is rehearsed without changing the canonical store. A timeout
or truncated plan remains explicit. Never infer per-operation work merely from shared-server
counters collected concurrently.

<a id="O11"></a>

### O11 — Expose the MX and server telemetry already available

**Proposed; compelling and unusually direct.** The local MX patch already registers eleven gauges:
`live_keys`, `live_versions`, `approx_data_bytes`, `active_readers`, `oldest_pinned_version`,
`gc_candidate_keys`, `commit_ring_occupancy`, `async_queue_depth`, `async_queue_bytes`,
`async_dropped_batches` and `poisoned`. The server exports these as
`surrealdb_storage_surrealmx_*` through authenticated `/metrics`. Existing source tests exercise
held-reader pinning, release and anonymous filtering; they were inspected, not rerun here.

Add a supervisor-owned metric snapshot and optional before/after comparison using its existing
private authentication. Return selected metric values with store/process identity, observation
time and coverage. Full metrics require a root-level authenticated session; the existing
`ON ROOT ROLES VIEWER` selection account satisfies the source-level gate. Reuse that private
selection account through the supervisor without exposing credentials or unnecessarily using
OWNER credentials. Namespace/database credentials do not satisfy the same gate. This supplies useful insight
without first deploying Prometheus. [Local MX patch][src-mx];
[MX 1.0 statistics](https://github.com/surrealdb/surrealmx/blob/v1.0.0/src/stats.rs),
[server storage bridge](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/server/src/observe/storage_metrics.rs),
[metrics authentication](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/server/src/observe/session.rs).

Combine these with query/RPC/transaction metrics and process/disk observations. Explain the
measures: estimated resident key/value bytes are not RSS or disk usage; zero asynchronous queue
depth is not synchronous-AOL durability proof; a retained version is not automatically a leak.
`oldest_pinned_version` is normally absent when no reader pins a snapshot; distinguish that from
an unavailable exporter. Shared counter deltas have shared scope. Additional MX fields such as
`merge_ring_occupancy` or
`poison_reason` are eligible where they answer a concrete diagnostic question.

**Delivery example:** Codex observes reader/version retention before and after cancellation,
distinguishes a pinned reader from growing live data, and identifies a poisoned engine. This
does not claim the reader was released by a particular operation without its own association.

<a id="O12"></a>

### O12 — Give SurrealQL and schema changes an early, source-aware feedback loop

**Proposed; promising after a bounded compatibility check.** Use offline language assistance and
the existing SurrealKit rehearsal path to catch query/schema problems before an expensive worker
or scientific run. SurrealKit beta.6 is already pinned and built against the patched gRPC SDK;
the registry/generator and immutable rollout producer remain the schema authorities.

The inspected language-server revision `5aa6a12a29b7def0e7df662c09ac9203e66f5033` is version 0.6.3
and supports `surreal-lsp check ... --json`, stdin with a source filename, `--workspace` schema
context, declared `--param` names and `stdio`. Its headless checker never connects to a database.
A generated schema workspace and query extraction with source mapping fit this repository.
The exact server's `validate --stdin` is an alternative syntax oracle. The LSP's builtins identify
engine revision `9d9a5b069`, while the selected server uses
`1ac2b1dba7cd177621a4cf5d507caebeb2414395`. Its grammar documents valid unsupported syntax, so it cannot become
the production parser authority or justify rewriting valid queries to placate it.
[Headless checker](https://github.com/surrealdb/surrealql-language-server/blob/5aa6a12a29b7def0e7df662c09ac9203e66f5033/src/native/check.rs),
[grammar limits](https://github.com/surrealdb/surrealql-language-server/blob/5aa6a12a29b7def0e7df662c09ac9203e66f5033/docs/grammar-gaps.md).
Preserve its diagnostic/scan distinctions: exit 0 means no above-threshold diagnostic, 1 means
diagnostics, and 2 covers usage/read/target-scan loss. Context-directory skips can still lose
schema knowledge; exit 0 alone does not establish complete workspace coverage.

Compose generated schema delta, diagnostics, existing SurrealKit plan/lint, application to an
owned disposable fixture and its declarative tests. Reuse `scripts/surrealkit.py`, rollout
production and guarded maintenance. Derived live type/schema snapshots can help inspect drift;
they do not replace declarations. [Selected SurrealKit README](https://github.com/surrealdb/surrealkit/blob/92ebc3cc89b99f00d21c7d2cbf5136d728367061/README.md).

The published `@surrealdb/surql-fmt@0.1.0-beta.3` package offers check/stdin/write modes, but its
declared source repository returned 404 during research. Keep it optional until provenance and
representative formatting/round-trip behavior are understood. Generated SQL formatting belongs
in the generator; frozen rollout artifacts must not be rewritten independently.

**Delivery example:** a query with an undeclared parameter or misspelled field yields an early
source-linked diagnostic; a valid engine construct beyond the LSP grammar remains accepted by
the engine. A proposed schema/index change is rehearsed through the existing rollout mechanism.

<a id="O13"></a>

### O13 — Make structured logs and optional cross-process traces actionable

**Proposed; scoped logging first, richer trace collection when useful.** The supervisor fixes
`--log=warn` and already captures bounded private rotating logs. Native MCP's info-level
`surrealdb::mcp::audit` target is consequently suppressed. Its source exposes useful tool,
subject, scope, outcome, error kind and duration fields without query text/row payload.

Add explicit diagnostic filter/format selection through the existing supervisor, with process,
store, invocation and operation references that can be joined to current locators. Add selective
log retrieval to O07/O10 instead of printing every service's output. Preserve existing capture
and classify missing spans/uncertain associations honestly. A query conflict should identify the
operation/read boundary, retry population and terminal outcome where owned facts provide them;
do not enable arbitrary mutation retries as a diagnostic convenience.

An on-demand OTLP collector and trace viewer is a useful candidate for difficult multi-process
workflows. SurrealDB Community supplies metrics/OTLP/ordinary structured logging; durable
Enterprise audit/slow-query NDJSON pipelines are distinct surfaces. Turning on OTLP alone does
not establish context propagation through Rust, gRPC, Surrealism and database execution. Examine
actual span coverage before choosing an exporter or making attribution claims.
[Observability configuration](https://surrealdb.com/docs/manage/observability/configuration),
[exact MCP audit contract](https://github.com/surrealdb/surrealdb/blob/v3.3.4/surrealdb/mcp/README.md).

A permanent Prometheus/Grafana/Loki/Tempo deployment becomes compelling when repeated longitudinal
queries justify its collector, storage, retention and query interfaces. It is an available
realization, not the prerequisite for O11. An operations MCP should expose the same owners only
when it improves repeated interactions beyond shell/native MCP access.

**Delivery example:** Codex follows one failed managed operation to its worker and database facts,
with explicit unmatched events. A diagnostic log selection can be enabled and reverted through
the selected service owner. Trace collection overhead and long-term usefulness remain unmeasured.

## D. Deeper diagnosis when ordinary observations are insufficient

<a id="O14"></a>

### O14 — Inspect mixed Python/Rust/native stacks and native crashes

**Proposed; high value.** Extend the current CPU capture and checkpoint prefix with actionable
mixed-language stacks and postmortem information. A Python journey may be in orchestration,
PyO3, Rust preparation, a released-GIL worker or a native solver; one sampler need not cover all
of them. Recent stack-overflow diagnosis required manual symbol/frame investigation because a
usable core was not readily available.

First test CPython 3.14's `-X perf` / `PYTHONPERFSUPPORT` and `-X perf_jit` with the existing Linux
perf/Samply route. JIT mode needs `perf inject --jit` and retained symbol/unwind artifacts.
The local interpreter reports `PY_HAVE_PERF_TRAMPOLINE=1`; actual capture, unwinding and
released-GIL/native-thread coverage still need checking. **py-spy** is complementary for
`dump --pid ... --json` and `record --native`; installed 0.4.2 adds
Python 3.14 support. Native collection pauses the target and cannot combine with nonblocking
mode. Python-thread sampling is not arbitrary Rust-worker coverage.
[Python perf guide](https://docs.python.org/3.14/howto/perf_profiling.html),
[py-spy](https://github.com/benfred/py-spy).

For crashes, compose selected Python `faulthandler`, GDB batch thread backtraces and the host's
actual core route with the last checkpoint and matching executable/shared-object identities.
Python 3.14 C-stack output is conditional on platform/build support. This host uses Apport;
do not assume `coredumpctl` can retrieve its crashes. A live GDB attach stops the target. Missing,
truncated or inaccessible cores and optimized-out frames remain explicit.
[faulthandler](https://docs.python.org/3.14/library/faulthandler.html),
[GDB backtraces](https://sourceware.org/gdb/current/onlinedocs/gdb.html/Backtrace.html).

Use direct established tools first and integrate recurring capture/summary needs with the existing
capture owner. Matching symbols are a concrete postmortem requirement, not a reason to restore a
general environment-attestation campaign. No new Rust-wide signal handler or debugger MCP is
required by the proposal.

**Delivery example:** an owned crashing fixture yields the signal, useful native thread stacks,
matching symbol references and last operation checkpoint. A small Python/native journey identifies
its Python frames and separately covered worker populations. Local tool feasibility is unprobed.

<a id="O15"></a>

### O15 — Attribute low-CPU wall time with actual scheduling evidence

**Proposed; high value.** The current `cpu-clock:u` capture measures sampled user CPU. Wall minus
CPU does not explain waiting. Add an explicit scheduling capture using `perf sched record` and
`perf sched timehist`, or an established equivalent, to distinguish running, sleeping and
runnable-but-unscheduled intervals.

Join actual PID/TID/start identity and monotonic alignment to existing native-thread scopes.
Retain unmatched events and alignment uncertainty. A futex wait does not identify a lock owner
or prove deadlock. Targeted syscall/lock inspection can answer a narrower subsequent question.
Scheduling classes distinguish sleeping, runnable delay and execution. Identifying an I/O wait,
supervisor join or native phase requires joined owner/thread facts or selected syscall evidence;
the scheduler trace alone does not establish that cause. This combination helps explain idle
capacity or oversubscription on the specified workstation. Tracepoint availability, permissions, capture
overhead and actual parsing require a bounded local feasibility check.
[Linux perf scheduling](https://github.com/torvalds/linux/blob/master/tools/perf/Documentation/perf-sched.txt);
[existing capture/analysis][src-diagnostics].

`perf stat` is a useful companion after locating a hot path: task-clock, cycles/instructions,
faults, branch/cache events and context switches can test a concrete CPU/memory hypothesis.
Preserve event availability and multiplexing/running fractions; no generic IPC threshold or
language-based performance conclusion is justified.

**Delivery example:** one controlled worker, one sleep and one contention interval produce
distinct observed classes. A real pipeline explains idle capacity without treating overlapping
phases as disjoint elapsed time. This is active capture, not an expanded passive polling loop.

<a id="O16"></a>

### O16 — Inspect suspended async work and task scheduling

**Proposed; high value with a deliberately selected diagnostic build.** OS stacks often show
Tokio parked while the interesting chain lives in suspended futures. Existing span lifetimes
do not expose pending await locations or task polling residency.

The pinned Tokio 1.53.2 offers experimental `Handle::dump()` with `taskdump` and `tokio_unstable`
on supported Linux architectures. An on-demand dump can reveal pending await locations. It
pauses/synchronizes workers and re-polls tasks, may not complete when a worker is blocked, and
has local-executor/population coverage limits. Preserve an independently available observer and
an explicit diagnostic deadline; a timeout polled on the same wedged runtime may itself never
run. Report an unavailable/incomplete dump when an independent timeout cannot be observed.
It must not be presented as passive observation or guaranteed deadlock
recovery. [Exact API](https://docs.rs/tokio/1.53.2/tokio/runtime/struct.Handle.html#method.dump).

Tokio Console is the richer candidate when poll/scheduled/idle distributions, wakes and
instrumented resources are the question. Its tracing/subscriber/build requirements add obligations.
Use task dumps for pending locations when sufficient; qualify Console for scheduling dynamics.
Do not assume Console's raw recording is a finalized offline report: its current recorder still
has a termination/flush-on-drop TODO. [Console source](https://github.com/tokio-rs/console).

Integrate the selected capability with existing operation/capture identities. Do not enable
unstable instrumentation across ordinary builds simply because a diagnostic is possible.

**Delivery example:** a readiness task awaiting a channel produces a useful trace, while a blocked
worker produces either evidence or an explicit timeout. A diagnostic build remains distinct from
the ordinary scientific/qualification configuration. No behavior or overhead was tested here.

<a id="O17"></a>

### O17 — Diagnose allocation churn and retained memory by allocating stack

**Proposed; useful selected investigation.** RSS, admission reservations and Salsa's known-byte
census answer different questions from "which code allocated this temporary population?" Large
available RAM supports investigation but does not remove the value of locating avoidable work.

Use **Heaptrack** for an actual Rust/native executable and headless `heaptrack_print` summaries
of allocation calls, peak consumers and temporary allocations. Use **Memray `--native`** for
Python extension journeys where Python/native allocating stacks matter. Python allocator detail
is a separate, more expensive selection. Retain matching binaries/libraries/debug information
for native interpretation. Pools bypassing intercepted allocation functions need coverage
qualification or explicit omission. [Heaptrack](https://github.com/KDE/heaptrack),
[Memray native mode](https://bloomberg.github.io/memray/native_mode.html).

Profile the produced executable rather than Cargo. Keep tools optional until a repeated need
justifies a capture adapter, and do not change the production allocator solely to make it visible.
A whole-process profile is not an exact operation-private peak. Reuse the existing frame/worker,
reservation and retention facts to distinguish logical population from allocation-site behavior.

**Delivery example:** repeated preparation separates temporary instruction-export churn from
retained compiled products and Python wrappers, with custom-pool omissions explicit. No memory
reduction, exact peak or local profiler compatibility is claimed by this review.

## E. Explain and reproduce the work being changed

<a id="O18"></a>

### O18 — Explain edits, Salsa execution and retained preparation together

**Proposed; high value after the Plan49c changes.** Give Codex an edit-to-preparation comparison:
what authored entity/field changed, which selected roots were requested, and which query/products
executed, validated, were discarded or remained retained. Existing Salsa events and retention
observations provide a foundation; their opaque ingredient/key locators need safe semantic
labels at their owning query boundary.

Compare unchanged, source-only, value-only and semantic edits. Separate required root-wide checks
from fragment reuse and preserve workspace generation/revision. Salsa 0.28.5 events do not prove
why a query executed, so label direct facts and inferred explanations separately. Do not invent
a cache-miss cause or exact shared-wait duration from an execution event.
[Exact Salsa events](https://docs.rs/salsa/0.28.5/src/salsa/event.rs.html);
[compiler observation owner][src-preparation].

Retention inspection remains an exclusive maintenance operation: the current use of
`memory_usage(&mut self)` can cancel/drain clones. Do not run it inside live tracked queries or
passively poll it. A user-facing explanation should show the freshness and scope of its last
census. No second dependency graph or invalidation authority is needed.

**Delivery example:** a source-label edit preserves semantic products; a local physical edit
shows affected fragment execution and separately retained root assembly. Clean recomputation
checks result equivalence when validating the delivered feature. Counts support a structural
explanation; a speed claim still needs elapsed measurements.

<a id="O19"></a>

### O19 — Explain generated mathematical work and actual relational execution

**Proposed; high value, mostly consuming existing facts.** Codex should answer why a selected
First/Second demand produced particular evaluator regions, derivative components, crossings,
provider/control stages, operation/instruction populations and frame/worker storage. Existing
`CompiledOrderCensus`, `EvaluatorFacts` and assembly observations already expose much of this.
Build readable source-linked comparisons and optionally export a selected instruction
representation; do not reintroduce another arithmetic census.

Symbolica's installed 3.0.1 `export_instructions`/`count_operations` mechanisms underlie these
facts. Representation counts are not execution time or branch frequency, and provider costs
remain separate. [Symbolica evaluation](https://symbolica.io/docs/numerical_evaluation);
[math facts][src-math].

For relational execution, consume the actual `PlanObservation` graph, fired rules, partition
metrics, terminal status and collection scope. Existing review analysis and frozen tables already
publish source-linked `engine.actual_policy`, `engine.lifetime` and `engine.operator_metric`
statements. The new delta is structure-aware before/after comparison and interpretation, not
another initial engine-metric reader. Show changed operators, expansion/filtering,
partition skew and available spill/memory facts. Per-partition and aggregate metrics have
different populations. Re-running `EXPLAIN ANALYZE` is new execution, not inspection of the
original run. [DataFusion EXPLAIN](https://datafusion.apache.org/user-guide/sql/explain.html);
[engine observation][src-engine].

Keep mathematical execution, relational execution and canonical lifecycle graphs distinct while
making their relationships navigable. Plan49g TOOL08 already owns bounded canonical
blocker/source/consumer explanations; consume that product response when available rather than
adding a parallel graph service or all-path traversal.

**Delivery example:** an added second-order demand explains the added components and crossings;
a relational regression points to its changed actual operator and skewed partition. A canonical
blocker response names revision, edge role and incomplete continuation. None of these proves a
performance winner or scientific dependence from graph connectivity alone.

<a id="O20"></a>

### O20 — Extract small reproducers and test diagnostic behavior directly

**Proposed; valuable new workflow.** Turn a complete selected frozen failure context into a small
local driver/request using existing public diagnosis or scientific execution. Preserve model,
case, settings, coordinates, failure attribution and required native/provider capabilities.
Extraction itself must not reconstruct missing source, contact a store or rerun science.

Begin with extraction and explicit rerun for the original failure family and predicate. Existing
`Runtime.diagnose_documents`, Python/CLI diagnosis and original-source point/matrix/term analysis
provide useful numerical targets; they do not reproduce readiness, concurrency or cancellation
failures. Those require the relevant operation/lifecycle fixture and causal conditions. Distinguish
original failure reproduced, related diagnostic reproduced and not reproduced. A matching broad
error variant alone does not establish reproduction. The existing request's origin is attribution,
not replay; preserve the original outcome independently. A future
automatic reducer needs a precise preserved failure predicate and scientific meaning; deleting
equations until something fails differently is not a valid reproducer. [Diagnostic/review owners][src-diagnostics].

Pair this with small direct fixtures for stale checkpoints, worker exit, unavailable core/trace,
incomplete source context, refused capture, partial engine execution, retained resources and
busy native joins. Test a reader, explanation or parser without initializing an unrelated full
scientific campaign. Existing source-isolated policy controls illustrate the desired boundary.
Coordinate existing diagnostic fixtures with Plan40e's assembled diagnostic acceptance and the
already selected representative functional assessment in Plan49h and actual runner inventories;
production capacity settings do not choose test repetition counts.

**Delivery example:** Codex extracts a store-independent point diagnostic, verifies the selected
numerical failure predicate and source/phase conditions, changes one owner and reruns the case. Missing provider/source context yields a
specific unavailable result before new scientific execution. This does not automatically
minimize models or qualify unexercised scientific families.

## Recommended composition and order

All opportunities above are recommendations, not accepted implementation packages. The review
recommends the following coherent direction:

These are parallel workstreams with local dependencies, not a strict sequence. O18/O19 can
consume existing facts alongside the new capture work.

1. **Make the ordinary loop dependable.** O01–O05 remove repeated setup and improve
   useful feedback. Expose real capability/configuration state and use the existing owners.
2. **Make running and completed work explainable.** O06–O09 provide read-only wait explanations,
   product-run evidence, owned reference execution and explicit generated-state abandonment.
   These share identifiers and lifecycle facts, not a new scheduler or ledger.
3. **Use the database's native diagnostic value.** O10/O11 are particularly direct because the
   selected engine already provides the important mechanisms. O12 adds earlier feedback after
   compatibility checks; O13 adds useful structured capture and an optional path to richer traces.
4. **Add capabilities that materially expand diagnosis.** O14–O16 answer mixed-stack/crash,
   low-CPU wall-time and pending-async questions that current passive observations cannot settle.
   O17 is a useful specialized investigation rather than always-on instrumentation.
5. **Help Codex assess and reproduce changes.** O03 and O18–O20 connect source consumers,
   incremental preparation, generated execution and failure reproduction. Some foundations
   already belong to Plan49g and existing diagnostic plans; preserve those owners.

Useful representative delivered workflows are:

| Scenario | Capability composition | What success would demonstrate |
| --- | --- | --- |
| Fresh session changes a Rust/Python contract | O01–O05, O07 | Correct checkout/tool attachment, actual consumers, selected correctness commands and correct installed extension/run association |
| A native test appears idle | O05–O08, O13–O16 | Distinguish build/admission wait, active native work, OS sleep and pending async work with stated coverage |
| Readiness/query performance regresses | O02, O10–O13, O19 | Actual selected query/schema/plan, relevant MX facts and a representative disposable comparison |
| Cancellation leaves a retained resource | O06–O09, O11, O14/O16 as needed | Explain actual ownership/drain/reference state and perform supported explicit recovery without inventing a terminal |
| A local edit rebuilds too much preparation | O03, O05, O17–O19 | Identify actual query/product changes, structural/allocating work and limits of attribution |
| A failure arrives as a frozen review pack | O04, O07, O14, O20 | Inspect recorded evidence/stacks, or extract a supported reproducer and explicitly rerun for new diagnosis or capture |

These scenarios guide acceptance; they do not prescribe another comprehensive campaign before
each change. Choose targeted probes for consequential uncertainty. Quantitative claims require
named measurements under their actual workload; none is made here.

### Alternatives and capabilities to keep available

- **A thin operations MCP** can improve repeated structured interactions after useful owner-level
  operations exist. Prefer exposing those operations directly; do not recreate the supervisor,
  schema authority, result semantics or process registry in an MCP service.
- **A full local telemetry stack** is eligible when longitudinal metrics/log/trace questions
  justify it. Direct MX snapshots and scoped capture provide value sooner; a dashboard alone
  does not make diagnostic evidence accessible to Codex.
- **rr** is a worthwhile deep-debugging candidate for compact FFI lifetime/corruption failures:
  record/replay and reverse execution can reveal the writer of a bad value. Its CPU/kernel/syscall,
  process-tree/shared-memory constraints and effectively single-core recording make it unsuitable
  as a parallel-performance measurement route. Local feasibility remains unprobed.
  [rr](https://rr-project.org/).
- **Selective agent knowledge and semantic search** remain eligible. Prefer focused source-backed
  knowledge and demonstrated retrieval questions; installing every official skill or indexing
  every historical document would add stale context and maintenance without an established benefit.
- **Cloud CLI/MCP, Kubernetes, Surreal Sync, PostgreSQL-wire access and TS type generation** have
  real ecosystem roles but no compelling consumer established by this local Rust/Python inquiry.
  Revisit when the deployment, ingestion or client workflow actually needs them.

The idea document's newer `surreal db ...` launcher examples do not describe this selected custom
binary, which directly implements commands such as `start`, `validate` and `mcp`. Installing the
launcher must not silently replace the MX/Surrealism build. SurrealKit is already integrated;
making authored `.surql` a second schema authority would conflict with the registry/generator.
The formatter and LSP need version/provenance checks. Community telemetry, Enterprise pipelines,
hosted Cloud MCP and the local `/mcp` endpoint are distinct capabilities.

## Coordination and rule implications

The [current-work index](../plans/README.md) and owning plans keep status. This review changes no
package state and reopens no completed Plan49a–e acceptance. Plan49g TOOL01/BND05 and TOOL05
provide the existing product/tooling contract routes; TOOL08 already owns canonical graph
explanations. Plan49h owns representative qualification, and 49f owns its independent boundary
and scientific obligations. Plans32 and40 retain artifact lifecycle and diagnostics acceptance.
Earlier Plan29–31 tooling foundations are implemented history, not new backlog. O09 extends the
Plan49 TOOL04/RUN01 recovery-disposition handoff and 49g TOOL04 interrupted-reclamation contract,
alongside Plan32 producer lifecycle. These are precise extension points, not blanket ownership
for every proposed agent-tooling feature.

Several proposed changes need deliberate integration decisions:

- Generated-state abandonment changes retention/recovery behavior; scope it to exact owned
  invocations and preserve truthful outcomes. Disposable data does not erase live ownership.
- GDB, py-spy native mode, scheduler capture, task dumps and allocator profiling are active
  diagnostics with different effects and coverage. Do not present them as passive readers.
- Instrumented Tokio diagnostic builds, Python runtime capture options and retained debug symbols
  have explicit consumers. Python options need supported interpreter behavior, not inherently a
  new build. These do not restore withdrawn all-role provenance campaigns or silently alter ordinary builds.
- New query/metric/graph/product documents should enter the existing catalogue and generated
  boundary route. Diagnostic interpretation remains separate from scientific qualification.
- Codex-only tailoring can simplify guidance and activation without adding artificial
  approval flows, enterprise deployment layers or routine quality gates. Existing guarantees may
  be redesigned through the applicable decision route when a simpler realization preserves them.

The next planning step can select a coherent subset or the complete recommended scope, route
overlap into existing owners and specify interfaces and acceptance. This review itself approves
neither installation nor production changes.

## Source and investigation routes

Repository links below fix the inspected source baseline. Symbol names identify the relevant
responsibility; these are not an exhaustive symbol inventory. Current operating/plan links above
remain their native owners. Primary upstream links preserve concrete mechanisms and release
conditions for subsequent planning. Sources were inspected on 2026-10-22.

- **Environment:** `pse_env::{base_environment,compose,execute}`, launcher, compiler selection;
  workstation `~/.codex/config.toml`, `~/.config/codex/bash-env.bash` and its README were read
  selectively for nonsecret configuration. The machine files are local observations, not
  repository-declared defaults. [Repository environment owner][src-env].
- **Build:** `build_environment.configure`, `compiler_cache::{ensure,supervised_compile}`,
  `.cargo/config.toml`, managed sccache and the environment guide. [Build owner][src-build].
- **Selection/context:** `affected::{changed_files,workspace_packages,closure,main}`, recipe
  metadata, document lifecycle `aggregate/snapshot`, published reading/search routes.
  [Selection][src-selection], [context][src-context].
- **Activity/lifecycle:** `activity::{units,build_processes,main}`,
  `host_admission::{readonly_snapshot,acquire,reconcile,status}`, supervisor and test-resource
  lifecycle/recovery. [Activity][src-activity], [lifecycle][src-lifecycle].
- **Reports:** `validation_result::{read_receipt,selected_result}`, `test_run.execute_selected`,
  native test/pytest inventories, JUnit and invocation terminals. [Results][src-results].
- **Database/schema:** `canonical_studies`, generated schema/functions, rollout producer,
  SurrealKit wrapper/maintenance, `surreal_server.mcp_authorization` and exact installed custom
  source. [Database][src-database], [MX patch][src-mx].
- **Runtime/tooling:** observation, inspection, CPU/span capture, review analysis and source
  context/export/reproduction consumers. [Diagnostics][src-diagnostics].
- **Preparation/math/engine:** `workspace::{observe_query_event,observe_retention}`,
  `CompiledOrderCensus`, `EvaluatorFacts`, assembly observations and `PlanObservation`.
  [Preparation][src-preparation], [math][src-math], [engine][src-engine].

Targeted absence searches covered the named owners and `crates/`, `python/`, `scripts/`,
configuration/manifests and developer documentation for py-spy, faulthandler/core adapters,
off-CPU and Console integrations. They support the stated gaps in those surfaces, not host-tool
absence or universal impossibility. The exact language-server source and published formatter
metadata were inspected without installation. No additional recommendation-audit stage or
periodic review obligation is introduced.

[src-env]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/pse_env.py
[src-codex]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/.codex/config.toml
[src-build]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/build_environment.py
[src-selection]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/affected.py
[src-context]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/document_lifecycle.py
[src-activity]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/activity.py
[src-lifecycle]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/test_resources.py
[src-results]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/test_run.py
[src-database]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/crates/pse-operations/src/canonical_studies.rs
[src-mx]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/scripts/surrealdb-source/mx-1.0-controls.patch
[src-diagnostics]: https://github.com/paul-heyse/pse-arrow/tree/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/crates/pse-runtime/src/tooling
[src-preparation]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/crates/pse-compiler/src/workspace.rs
[src-math]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/crates/pse-math/src/execution.rs
[src-engine]: https://github.com/paul-heyse/pse-arrow/blob/79a8dbfc98046cb4097633dbfbe26b26df9e6c27/crates/pse-engine/src/session/observation.rs
