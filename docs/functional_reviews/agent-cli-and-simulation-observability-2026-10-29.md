---
title: Agent CLI and simulation observability
status: proposed
date: 2026-10-29
doc_role: review
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [architecture, design, libraries, environment, validation]
---

# Agent CLI and simulation observability

## Judgment and evidence boundary

**Proposed:** make `pse` an agent-first front door over existing native capabilities, with
simulation observation as a first-class capability alongside command execution. Its most
valuable extension is a short route from “what is running?” to the actual owner's evidence,
explanation and useful next action. Another log viewer, universal dispatcher or monitoring
daemon would not provide that by itself.

This is the authorized combined functional-extension and Plan59 improvement engagement.
The [improvement companion](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md)
owns amendments I01 onward; this document owns the functional propositions O01 onward.
These references are separate from the original CLI review's O01–O25, whose dispositions
remain with [Plan59](../plans/59-agent-cli.md). Neither new review amends that plan.

Source and plan baseline: `890aa07f76fac5739cfc9811a47e823c326be663`, initially clean,
2026-10-29. Plan59 records stopped/reverted production execution, retained A01/E05 rule work
and the D02 trial outcome. Proposed features are not delivered by those earlier activities.
The maintainer explicitly permits reconsidering major architecture choices. Direnv and
Codex configuration exclusions remain; current shell access is a sufficient consumer.

**Interface-checked:** named source paths and library contracts. **Proposed:** recommendations
and acceptance scenarios. No numerical, resource, feature-matrix or product qualification
was performed for this review, and no performance improvement was measured. Plan58's earlier
results retain their original scopes. Core3.4, efficiency heuristics1.0, test assessment1.2
and process-simulator1.5 apply through the repository binding. This engagement does not
refresh a diagnostic audit or issue formal ADR acceptance.

## The capability already present

The useful starting point is substantial native functionality, not an empty monitoring system:

| Current owner | Available meaning | Current access/integration limitation |
| --- | --- | --- |
| `pse-host::activity`, manager and foreground ownership | Actual unit/process facts, unavailable evidence, authenticated control and original-resource settlement | Agent-facing discovery and correlation require manual commands; borrowed observations are not stop capabilities. |
| `pse-diagnostics::observation` and `inspection` | Operations, phases, producers, relationships, coherent records, coverage and source-linked `Statement`/`NextObservation` | These are useful domain contracts to expose, not reproduce in CLI handlers. |
| `pse-runtime::observation` and `tooling::inspection` | Published process checkpoints, bounded process discovery and typed copied inspection | Existing Rust entrypoints have no complete standalone CLI composition. Process discovery never grants attachment/control authority. |
| Runtime telemetry/capture/review modules | Trace propagation, explicit detailed capture, frozen inspection, numerical interpretation, comparison, query specification and review packs | Many mechanisms are callable APIs; an operator still assembles their selection and execution. |
| `workflow::study_observation` | Passive study cursors, exact attempts, local live checkpoints, terminal retained progress and explicit continuity | Remote ongoing numerical telemetry can be `LiveNumericsUnavailable`; durable history is not a live transport. |
| Native canonical fixture and correctness owners | Source-build-first test command, captured metadata, isolated fixture capability and validation closure | Plan59's older fixture baseline must be reconciled to these actual owners. |

The [simulation diagnostics guide](../dev/simulation-diagnostics.md) explains the contracts.
Source inspection confirms that `tooling::inspection::Statement` consumers render existing
interpretations; the CLI should not create its own solver diagnosis from log phrases.

Current-session experience supplies concrete motivating cases: mutable `target/debug/xtask`
was overwritten by a later feature selection; optional provider construction differed from
launch; an observation test retained its initiating span; Cargo compile flags differed from
dependency cfg discovery; and operators spent effort finding logs, fixture inputs and actual
test outcomes. These are workflow evidence and hypotheses for improvement, not measured
proof that every proposed CLI feature saves time.

## Recommended target

Retain a small host front door initially. Put process control, invocation records, ordinary
tool execution and configuration composition there. Delegate product inspection, scientific
interpretation and typed data to an explicitly prepared product executable/API owner. The
host may carry the selected response bytes and schema identity without importing runtime,
model, Arrow or database dependencies to reinterpret the product response.

Use three scoped authorities:

1. **Invocation record:** what this CLI launched, selected executable/configuration, child
   outcome, failure extraction and settlement evidence.
2. **Canonical scientific state:** studies, runs, attempts, claims, scientific outcomes and
   accepted result references in SurrealDB/MX.
3. **Observations:** copied checkpoints, traces, metrics and diagnostic artifacts with their
   actual identity, coverage, freshness and retention.

Join them by explicit references. An invocation can encompass many scientific attempts;
scientific work can continue across several observing invocations. A single generic `run`
identifier must not silently mean all three. The file invocation ledger is a reasonable
initial host implementation; moving it into the canonical database would couple basic host
diagnosis and cancellation to database readiness without removing scientific-state authority.
Revisit only for a concrete shared-query or remote-management requirement.

## Functional opportunities

<a id="O01"></a>
### O01 — Discover active work with its actual scientific context

An agent should obtain a compact cross-checkout view of its invocations, selected native
processes and available scientific references, then select an exact object for detail.
This extends Plan59's live-run/cgroup view with the existing diagnostic process catalogue.
Show the source of each fact and a timestamp/coverage state; “not observed” is useful output.

Candidate interaction: `pse status` → selected invocation → linked operations/attempts.
Command names beyond existing Plan59 verbs are illustrative until plan revision. Use distinct
invocation, observation-operation, process-instance, canonical-run and attempt fields/types.
Do not build a universal execution ontology or discover arbitrary credentials/databases.

**Value and realization:** reuse bounded `list_processes`, exact process locators and declared
associations, adding invocation references at cooperating roots. Retain systemd/proc facts as
host observations. A canonical lookup is explicit and uses the existing read capability.
No database or exporter is required to observe a simple local command.

**Closure:** two concurrent checkouts and a study with multiple attempts remain attributable;
missing associations are explicit. A reused PID cannot inherit the earlier process identity.

<a id="O02"></a>
### O02 — Explain current work and waiting from owner evidence

Expose operation phases, dependencies/shared producers, last coherent checkpoints and existing
inspection statements. Let an agent follow a source-linked next observation rather than
guessing from elapsed wall time. Show preparation, admission, numerical execution, publication
and physical draining separately where the owner actually publishes them.

Use `Statement.kind`, evidence references and `NextObservation`. Add missing facts at their
performing owner only when they discriminate a real question. Classify explanations as
established, hypothesis or limitation. A stale checkpoint, high memory reading or silent
foreign solver does not establish a deadlock, non-convergence or leaked resource.

**Alternatives:** infer state from logs; add a separate monitoring scheduler; expose the
current typed reader. The last is the preferred first delivery because it preserves existing
semantics and removes presentation work without duplicating scientific interpretation.

**Closure:** shared-flight wait and producer computation remain distinct; inclusive phase
durations are not summed into exclusive CPU time. A silent native interval produces the
available evidence and a discriminating next action, not an invented diagnosis.

<a id="O03"></a>
### O03 — Follow meaningful updates with truthful continuity

Offer a snapshot and an optional bounded follow mode over the same selected objects.
Human rendering can use indicatif; JSON/NDJSON must retain clean framing and explicit final
or nonterminal state. Keep observation cursor, source sequence, durable append position and
publication time distinct. A disconnected consumer can resume only the continuity actually
supported by the source; coalesced snapshots are not an event replay guarantee.

Current `StudyObserver` attaches local `Arc<Progress>` without a cancellation lease and reads
retained terminal progress. That attachment needs the original in-process `RunHandle`; a
separate CLI executable cannot acquire it merely because it runs on the same machine.
Existing cross-process checkpoints and retained history are directly exposable. Deliver that
honest experience first. If cross-process live study monitoring is selected, add owner-produced
bounded current snapshots through the existing canonical/transport owner; decide separately
whether retained history/replay earns its cost. OTel sampling cannot supply exact numerical
replay. Cross-source order is not a global causal sequence.

Futures streams and existing Tokio ownership should compose waiting, concurrent inputs and
coalescing. Scientific producers must not wait for a terminal renderer or optional exporter.
Bound presentation queues and disclose loss. Mandatory test-result reconciliation has a
different completeness requirement from optional diagnostic samples.

**Closure:** slow/broken consumers, reconnect gaps, reset generations, absent remote live
data and terminal history remain distinct. Closing a follow view does not cancel science.

<a id="O04"></a>
### O04 — Escalate an observation into a selected diagnostic investigation

Provide explicit routes from inspection to detailed capture, frozen analysis, maintained
Perfetto query specifications and portable selected evidence export. Reuse `start_capture`,
`review_analysis`, `review_query`, `review_pack` and existing frozen readers. Preserve their
exact process/operation/source identity, original units, bounded output and loss metadata.

Keep passive reads, explicit collection/attachment and mathematical diagnostics separate
actions. A `NextObservation` can describe an eligible action without executing it. A capture
owner settles collectors; a scientific owner settles its operation. An unavailable optional
tool or collector failure must not rewrite the scientific outcome.

**Value:** one source-linked investigation instead of another general debugger framework.
Support source/preparation and solver evidence through existing native owners; do not recreate
every retired Python campaign or debugger proxy. Optional backend trace IDs may navigate to
an investigation, but empty backend output remains ambiguous about execution.

**Closure:** export supplied selected evidence, reopen it without original paths, and retain
partial/expired/unsupported classifications. Comparison explains conditions and observations;
it makes no causal performance claim without independent evidence.

<a id="O05"></a>
### O05 — Make configuration and failure explanations directly actionable

An agent should see the effective command, native capabilities, fixture inputs, configuration
origins and error causes before making an edit. Compose clap's explicit selections with typed
configuration, then use Figment provenance where it replaces manual composition. Preserve the
existing explicit `Environment` input; do not source `.envrc.local` to fabricate provenance.

The same resolved operation value should drive `explain` and execution. Explanations redact
secret values while retaining source/category information. Present typed diagnostics with
causes, locations, related failures and useful next actions through thiserror/miette. Keep
infrastructure failure, compiler failure, setup failure, test failure, incomplete observation
and numerical outcome separate. Remedies are guidance, never automatic retries or permissions.

**Closure:** explicit false/empty/absent selections, precedence conflicts and invalid nested
settings are explained at their actual source. The executed selection matches the explanation
when the inputs are unchanged; a changed external premise is explicitly re-resolved.

<a id="O06"></a>
### O06 — Make the selected test/fixture journey one coherent operation

Consolidate capability derivation, correctness admission, source build, immutable executable
selection, private canonical fixture and result reconciliation. Use actual native owners,
not a new global fixture-permission registry or a second nextest scheduler. Preserve selected
features/targets/ignored-test policy and list the actual selected test population.

The current `NativeTestCommand` captures metadata before service creation and lends a typed
fixture capability; `ChildExitPremise` can retain state after unknown scientific joins.
A ledger terminal, leader exit or empty process cgroup does not replace that disposal decision.
Prepared artifact paths must identify the selected immutable artifact rather than mutable
`target/debug` output that another feature build can overwrite.

**Closure:** source-build/setup failure, zero-match selection, missing terminal records,
signal termination and cleanup uncertainty yield useful distinct results. A successful real
native fixture journey runs without manually assembling temporary receipt paths.

<a id="O07"></a>
### O07 — Preserve engineering context through inspection, edits and handoff

Connect changed-source/owner routes, exact locked API lookup, disposable probes, selected
failure records and compact handoff. Reuse Plan59's useful D verbs, but distinguish syntactic
`syn` extraction from semantic symbol resolution. An API lookup must identify version, source,
target, features and rustdoc format rather than claiming universal API availability.

Give a brief enough context for the next decision: failure, supporting evidence, exact source
selection and next action. A handoff is a view of current owners plus explicitly authored
intent, not a second mutable plan/status ledger. Documentation checks should report the
relevant change; unrelated historical link failures must not trigger automatic cleanup.

**Closure:** a wrong API assumption can be tested in a private probe without modifying the
workspace lock. After compaction, the next agent finds the exact operation and unresolved
failure without rerunning a completed workload. Shared-index commit tests preserve concurrent
staged intent and dirty work, not merely the final commit's path list.

<a id="O08"></a>
### O08 — Keep optional interfaces as adapters over working capabilities

Human progress, shell completion and a small RMCP adapter can improve access, but none should
own scientific lifetime or become necessary for CLI operation. Use parser-derived discovery
for host commands; let delegated owners validate their own grammar. RMCP is deferred until an
actual client benefits from typed tools/resources/subscriptions beyond shell execution.

If selected, compare a small `ServerHandler`/router over existing handlers with no MCP.
Protocol tasks, cancellation and progress do not replace invocation supervision, native joins
or canonical attempt policy. Capability advertisement and schema dialect must match the actual
client. A disconnected tool client must have deliberately defined domain behavior.

**Closure:** ordinary shell use works independently; an eventual adapter is qualified through
an actual host/client journey. No FastMCP parity or duplicate operation catalogue is required.

## Delivery priorities and open decisions

First establish identity/result/control contracts and a small integrated invocation journey;
then add passive process/operation inspection and the actual fixture consumer. Build follow,
diagnostic escalation and targeted engineering aids on those owners. Optional remote-live
numerics and MCP come after their source/consumer requirements are settled.

The review recommends existing checkpoints for initial local observation and the existing
canonical store for scientific state. It leaves remote-live transport, optional backend query
integration and any extraction of passive readers into a new crate conditional on a concrete
consumer and dependency benefit. Before implementation, resolve those choices in the affected
plan package; this is not permission to silently choose a new engine or always-on service.

The improvement companion maps these capabilities and necessary corrections to Plan59's
remaining packages, including migration, retirement, failure cases and decision-route impacts.

## Library mechanisms and research coverage

The library researchers read all nine supplied overviews end to end in bounded sections,
then used Context7 followed by exact installed/published sources for consequential details.
Independent assessment of the integrated drafts corrected two assumptions: fixture protocols
coexist, and process proximity does not provide another process's `Arc<Progress>`.

| Library/document | Concrete mechanism, use and limit |
| --- | --- |
| [clap](../libraries/clap.md) | Derive/builder composition, typed parsers, `value_source`, fallible parsing and introspection. Full `Command::build()` expands the tree. Arbitrary parsers/private conditional rules and `unstable-ext` objects are not an executable JSON grammar. [4.6.7 Arg source](https://docs.rs/clap_builder/4.6.7/src/clap_builder/builder/arg.rs.html). |
| [Figment](../libraries/figment.md) | `Provider`, `Metadata`, tags, merges/profiles and typed extraction support nested configuration. Built-in Env reads globals and uses lossy string conversion: preserve injected `OsString` activation separately. Use captured-input providers only where typed composition removes enough plumbing. [0.10.19 API](https://docs.rs/figment/0.10.19/figment/struct.Figment.html); exact published archive inspected after web retrieval failure. |
| [Futures](../libraries/futures.md) | `FuturesUnordered`, `StreamExt` and bounded buffering compose observer inputs. FuturesUnordered is not itself a population bound; fail-fast combinators can drop unfinished peers. Original-owner settlement remains necessary. Exact installed futures-util0.3.34 source inspected. |
| [indicatif](../libraries/indicateif.md) | `MultiProgress`, stderr targets and refresh cadence present copied observations. Hidden println can discard messages; suspend holds a rendering lock. Keep durable errors/machine output independent. Percentage/ETA requires meaningful work counts, not solver convergence. [0.18.6 API](https://docs.rs/indicatif/0.18.6/indicatif/struct.MultiProgress.html). |
| [RMCP](../libraries/rmcp.md) | `ServerHandler`, typed tools, resources and subscriptions can adapt working operations. TaskManager is not durable invocation authority; lag/disconnection needs domain catch-up. Require an actual consumer. [3.7.0 tagged guide](https://github.com/modelcontextprotocol/rust-sdk/blob/rmcp-v3.7.0/README.md). |
| [Schemars](../libraries/schemars.md) | `draft2020_12().for_serialize()` gives directional schemas, not validation or domain enforcement. Exact1.2.2 RawValue schema delegates to unrestricted JSON. Projected views need an explicit contract. |
| [Serde/serde_json](../libraries/serde_serde-json.md) | Typed summaries, streaming decoding and RawValue avoid generic trees. RawValue can retain newlines, breaking NDJSON if embedded verbatim. Preserve raw bytes in artifacts or normalize stream records. [1.0.151 RawValue](https://docs.rs/serde_json/1.0.151/serde_json/value/struct.RawValue.html). |
| [thiserror/miette](../libraries/thiserror_and_miette.md) | Typed errors/presentation compose with existing TypedDiagnostic facts. Preserve causes, diagnostic sources, source identity and related failures. Exact7.6.0 [JSON handler](https://docs.rs/miette/7.6.0/src/miette/handlers/json.rs.html) confirms the URL escaping limitation; use Serde for the machine record. |
| [Rust tracing](../libraries/rust-tracing.md) | Per-poll instrumentation, filtering and W3C propagation/links extend existing ExecutionContext/Carrier. Completed-span export is not an active catalogue. Optional tracing-appender lossy mode needs queue/record bounds and disclosed loss; non-lossy output can block producers. |

These are source-checked design leads, not dependency additions. Context7 sometimes returned
main-branch material; release-specific conclusions use exact installed/published sources.
Figment, indicatif, RMCP and optional appenders are candidates, not assumed workspace dependencies.
Features, exact pins and any new crate route belong to subsequent implementation.

## Review closeout

**Tested (documentation/process only):** `scripts/pse-env -- just turn-end` passed; it built
the documentation/ADR helper, refreshed the unchanged ADR index and ran the required formatters.
`scripts/pse-env -- target/debug/xtask lifecycle validate` passed with zero errors over 1,634
native scope items. A bounded `uv run --no-sync python` inspection checked these two reviews'
local link targets and all19 explicit O/I anchors: zero errors against baseline0.
`git diff --check` passed. These results do not test the proposed CLI or scientific behavior.
An initial direct lifecycle invocation preceded helper construction and returned127; the
post-construction invocation above supplies the actual successful metadata result.

Publication adds only the two reviews and their collection-index entries. Current Plan59
package states, production code, configuration and concurrent library research stay unchanged.
