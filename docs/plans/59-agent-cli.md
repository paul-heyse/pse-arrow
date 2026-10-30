---
title: "Plan59: Agent-first pse CLI"
status: active
date: 2026-10-29
adrs: [ADR-0233]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md]
doc_role: plan
doc_owner: docs/plans/59-agent-cli.md
doc_topics: [current-work, environment, design, libraries]
---

# Agent-first `pse` CLI

## Purpose, scope and basis

Today agents drive the Rust system with hand-assembled commands:
- raw cargo/nextest/`scripts/pse-env` invocations;
- inline `PSE_*` fixture variables and receipt paths kept in `/tmp`;
- background runs watched by `tail` and `ps`;
- repeated reads of the `justfile` and orientation files.

The Python CLI, MCP and `pse-py` boundary that used to front this work retired on 2026-10-29.

This series gives agents one Rust front door, `pse`, inside the existing `pse-host` crate. Built
on clap, it routes each verb to the owner that already does the work:
- `pse-host` for environment, placement, admission and processes;
- the correctness adapter for Cargo and nextest;
- xtask for heavy development composition;
- product binaries for workers and services.

It adds these capabilities:
- a versioned, summary-first output contract;
- owned runs with a ledger;
- intent-level test and check verbs;
- fixture resolution;
- knowledge and workflow verbs;
- product-owned simulation inspection, follow and explicit diagnostic investigation;
- the host-tool and worktree environment changes that make agents effective.

**Audience and adoption.** The users are capable agents, chiefly Claude and Codex, who judge for
themselves which tool to use. `pse` is the strongly recommended agent interface and is adopted by
being the easiest path and by being clearly communicated. No hook enforces it. When agents bypass
`pse` for some task, that marks a capability to improve.

**Basis.**
- Source: the
  [agent-first Rust CLI functional review](../functional_reviews/rust-agent-cli-2026-10-29.md),
  revisions `0df43e599`, `41a9d953a`, `d223d029e` and `68e075493`. Opportunity references `O01`–`O25`
  mean anchors in that review. Its [integrated clap capabilities](../functional_reviews/rust-agent-cli-2026-10-29.md#clap-capabilities)
  section is this series' clap usage contract.
- Baseline: `68e075493`, inspected 2026-10-29.
- Standard: core 3.4, efficient-architecture heuristics 1.0, test assessment 1.2 and
  process-simulator 1.5, under the pse-arrow binding.
- Amendment basis: the [observability review](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md)
  and [Plan59 improvement review](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md),
  published at `e907d8a1f`, inspecting `890aa07f76fac5739cfc9811a47e823c326be663`.
- Status: **Active planning.** Earlier execution was stopped/reverted; the maintainer authorized
  this document amendment, not restarting production implementation. See the current checkpoint.

**Reading route.**
1. This coordinator: target, shared decisions, rule changes, dependencies, completion.
2. [59a CLI foundation](59a-cli-foundation.md): the front door, output contract, catalog, state
   store, host tools, and the rule-change edits.
3. [59b Runs and continuity](59b-runs-and-continuity.md): ledger, supervision, wait/result/cancel,
   compaction handoff.
4. [59c Tests and fixtures](59c-tests-and-fixtures.md): `test`, `check`, `tests`, fixtures, and the
   reconciled verdict.
5. [59d Knowledge and workflow verbs](59d-knowledge-and-workflow-verbs.md): `changes`, `brief`,
   `dep`, `api`, `probe`, `outline`/`show`, `owner`, `docs check`, `ref`/`new`, `commit`.
6. [59e Environment and adoption](59e-environment-and-adoption.md): worktrees, harness surfaces,
   communication and recipe migration.
7. [59f Simulation observability and diagnostics](59f-simulation-observability-and-diagnostics.md):
   product delegation/correlation, passive inspection, follow, study history and selected investigation.

## Scope decisions

**Operator directions, 2026-10-29:**
- **Direnv changes are on hold.** A separate environment effort is changing uv and environment
  handling around `.envrc.local`. Excluded: O09 (direnv output quieting) and any direnv step in
  O23. `pse worktree new` copies `.envrc.local` and keeps its current propagation behaviour.
- **Codex changes are limited to what Codex needs to call the CLI**, which is only the shell and
  the shared AGENTS.md guidance. Codex works differently from Claude and its setup stays as is.
  Excluded:
  - O10 item 1 (generating role contracts into Codex `developer_instructions`);
  - O10 item 4 (Codex `exec` habits);
  - Codex hooks in O20 and O25;
  - the rust-analyzer MCP for Codex (O08, O11);
  - the `rustdoc-mcp` trial for Codex (O15).

  Ledger session grouping uses Claude's hook-provided session ID or an explicit
  `--session`/`PSE_SESSION`, with no Codex configuration change.
- **Enablement over governance.** Where a current rule costs agents effort, the series changes
  the rule (see [Rule changes](#rule-changes)). It does not preserve ceremony.
- **Concurrent agents use separate worktrees.** Each worktree has its own `target/` and
  `target/host-tools`, so one agent's in-progress change cannot break another's `pse` verbs. The
  isolation protects other checkouts, but cannot make the owning checkout's broken source
  buildable. Existing-work control is therefore exempt from new-work rebuilding under amended R3.

<a id="opportunity-dispositions"></a>
## Opportunity dispositions

| Opportunity | Disposition | Work owner | Required evidence or trigger |
| --- | --- | --- | --- |
| O01 | scheduled | [59a](59a-cli-foundation.md) | Agents run the fixture episode and common work through `pse` verbs; one library `Cli` behind thin bins |
| O02 | scheduled | [59a](59a-cli-foundation.md) | Every verb emits the versioned envelope with the two exit classes; envelopes validate against `pse schema` |
| O03 | scheduled | [59b](59b-runs-and-continuity.md) | Detached runs waited on once; failures read through `pse result --failures`; lost observation is never reported as success |
| O04 | scheduled | [59c](59c-tests-and-fixtures.md) | `pse test`/`check` derive features, capabilities and filters; reconciled verdict; `--changed` from the host-resident `affected` |
| O05 | scheduled | [59c](59c-tests-and-fixtures.md) | Integration targets run with no hand-set fixture variables or `/tmp` receipt paths |
| O06 | scheduled | [59a](59a-cli-foundation.md) (catalog, explain, provenance); [59d](59d-knowledge-and-workflow-verbs.md) (generated reference) | `pse commands --json` with `valid_values`; `pse explain` matches what runs |
| O07 | scheduled | [59d](59d-knowledge-and-workflow-verbs.md) | `pse changes` and `pse brief` used for orientation and delegation |
| O08 | scheduled | [59d](59d-knowledge-and-workflow-verbs.md) | `pse dep`, `outline` and `show` used in place of registry greps and `sed` ranges. Rust-analyzer MCP for Codex excluded. |
| O09 | deferred | Operator's environment work | Direnv quieting on hold by operator direction |
| O10 | scheduled (items 2–3); excluded (items 1, 4) | [59e](59e-environment-and-adoption.md) | Plan index slimmed; Claude SessionStart brief. Codex items excluded by operator direction. |
| O11 | scheduled | [59c](59c-tests-and-fixtures.md) (fixtures), [59a](59a-cli-foundation.md) (host ceiling), [59e](59e-environment-and-adoption.md) (start routes) | Items land with their owning packages; the clap overview is already corrected. Rust-analyzer MCP for Codex excluded. |
| O12 | scheduled | [59e](59e-environment-and-adoption.md) | Edit guidance in AGENTS.md and the tooling rule |
| O13 | scheduled | [59a](59a-cli-foundation.md) | Safe generation-use protection and coherent fingerprints; new work rebuilds, existing-work control uses its compatible prepared generation. |
| O14 | scheduled | [59d](59d-knowledge-and-workflow-verbs.md) | `pse ref` and `pse new` used for citations and new documents |
| O15 | scheduled | [59d](59d-knowledge-and-workflow-verbs.md) | `pse api` and `pse probe` answer exact pinned-API questions; rustdoc-mcp trialled for Claude only |
| O16 | scheduled | [59d](59d-knowledge-and-workflow-verbs.md) | `pse owner` and guard context; documents inform instead of block ([R4](#rule-changes)) |
| O17 | scheduled | [59d](59d-knowledge-and-workflow-verbs.md) | `pse docs check --changed` before document commits |
| O18 | scheduled | [59d](59d-knowledge-and-workflow-verbs.md) | `pse commit --paths` in a shared tree ([R5](#rule-changes)) |
| O19 | scheduled | [59c](59c-tests-and-fixtures.md) | `pse tests <pattern>` returns runnable commands |
| O20 | scheduled (Claude) | [59b](59b-runs-and-continuity.md) | A session resumes after compaction from `pse handoff` context |
| O21 | deferred | This coordinator | Adopt soft claims at the first concurrent-edit collision, recorded in the checkout store |
| O22 | scheduled | [59e](59e-environment-and-adoption.md) (continuing) | Start route, guides, roles/skills and `next` hints point to `pse` as each verb ships |
| O23 | scheduled (without direnv) | [59e](59e-environment-and-adoption.md) | `pse worktree new` yields a ready checkout whose first build is warm |
| O24 | scheduled | [59a](59a-cli-foundation.md) (resolver and store), [59b](59b-runs-and-continuity.md) (runs) | One state resolver; per-run directories replace `/tmp` logs |
| O25 | scheduled (Claude) | [59e](59e-environment-and-adoption.md) | Claude session-start brief and status line |

## Simulation observability dispositions

These O01–O08 refer only to the new [observability review](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md),
not the original O01–O25 above. Scheduled means selected work, never implementation acceptance.
Conditional portions retain the stated trigger; companions own package progress.

| Opportunity | Disposition | Work owner | Required evidence or trigger |
| --- | --- | --- | --- |
| [O01](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O01) | scheduled | A03; B01; F01/F02 | Existing identity associations and cross-checkout copied process/operation discovery; unknown associations stay unknown. |
| [O02](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O02) | scheduled | F02 | Owner statements, phases and relationships explain actual evidence without invented hang/convergence diagnoses. |
| [O03](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O03) | scheduled | F03/F04; B output | Bounded follow, gaps/resets and retained history; additional live publication is gated by F04's explicit decision. |
| [O04](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O04) | scheduled | F05 | Selected actual capture/analysis/export consumer with original identity, coverage and collector settlement. |
| [O05](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O05) | scheduled | A03/A06; B02 | Original typed facts/causes, honest provenance and shared resolved operation; secrets redacted. |
| [O06](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O06) | scheduled | C07/C01–C05; A05 | Exact selection, both fixture modes and immutable selected tools; actual native journey without manual input assembly. |
| [O07](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O07) | scheduled | D01–D09; B05; E03 | API/probe identity, scoped documentation, preserved concurrent edits/staging and compact handoff. |
| [O08](../functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md#O08) | scheduled | A06; F03/F06 | Human presentation and scoped discovery selected; RMCP deferred until an actual client needs tools/resources/subscriptions beyond shell access. |

## CLI improvement dispositions

These I01–I11 refer to the [new improvement review](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md).
The work below integrates the proposals into this plan; no finding is resolved by scheduling it.

| Improvement | Disposition | Work owner | Required evidence or trigger |
| --- | --- | --- | --- |
| [I01](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I01) | scheduled | A03/A07; B01; F01 | Distinct invocation/scientific/observation identities, explicit existing-fact associations and independent outcome dimensions. |
| [I02](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I02) | scheduled | A07/A05; B03 | Control works with broken current sources; new-work rebuild identity remains coherent and fail-closed. |
| [I03](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I03) | scheduled | A03/A07; B01–B04; C02 | Acknowledgement, wait expiry, child outcome, lost evidence and actual settlement distinguished; real service-entry journey. |
| [I04](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I04) | scheduled | A03; B02; F02 | TypedDiagnostic facts, error causes/related failures, Unicode source locations and settlement leaves preserved. |
| [I05](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I05) | scheduled | A03; B01/B02; F03 | Typed summaries versus opaque raw records; framed/projection-aware output and independent required result completeness. |
| [I06](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I06) | scheduled | F01–F06 | Prepared product reader, passive local evidence, retained studies and explicit escalation; conditional bridge/backend choices recorded. |
| [I07](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I07) | scheduled | C07/C01–C05; A05 | Exact binary/feature/filter identity, preserved absence controls, both actual fixture owners and pinned tool generation. |
| [I08](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I08) | scheduled | A04/A05; B03; E01 | Generation use beyond ledger entries, safe active-work preflight, independent fixture disposal and partial creation outcomes. |
| [I09](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I09) | scheduled | A02/A04/A06; D01/D02/D07 | Owning parsers enforce grammar; byte-exact activation preserved; typed Figment subset only if it earns adoption. |
| [I10](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I10) | scheduled | D01–D09; B05; E02/E03 | Complete API/probe identity, scoped checks and guarded Git transaction with explicit post-commit partial outcome. |
| [I11](../plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md#I11) | scheduled | A02/A03/A06; D07 | Scoped candidate discovery, honest incompleteness and early parsing that cannot consume a child argument tail. |

<a id="rule-changes"></a>
## Rule changes

The operator accepted each of these on 2026-10-29. Each is scheduled in
[59a A01](59a-cli-foundation.md#a01) before the packages that depend on it. A01's original
completion is retained; [A07](59a-cli-foundation.md#a07) schedules the newly confirmed amendments
and contract publication before dependent implementation. Formal ADR status is unchanged.

| Change | Decision | Route |
| --- | --- | --- |
| **R1.** `pse` is the strongly recommended agent interface. Its catalog is derived from the parser, not authored, so Plan58 I09's "universal catalogue" concern does not apply. This replaces Plan58 §Purpose's "CLIs are optional conveniences". | Directed by the operator | Amend Plan58 §Purpose and its Outcome wording; `docs/plans/README.md` |
| **R2.** Supervising runs that agents start (wait, result, cancel) is allowed. Build-stage telemetry and alerts stay retired. | Accepted | Amend the Plan58 target row on build observation, 58b's and 58h's wording, and the environment guide §Detached watches |
| **R3.** New-work verbs rebuild a stale host and fail on rebuild error. Cancel/wait/read of an existing invocation instead use its compatible prepared control generation without rebuilding current sources; recorded identity and actual settlement remain mandatory. One host grammar, delegated owner grammars and a per-invocation supervisor remain selected. | Accepted; control exception explicitly confirmed 2026-10-29 | A07 amends proposed ADR-0233 and schedules affected architecture publication before A05/B03 consumers; no such authority files are edited by this plan amendment. |
| **R4.** Protected documents inform instead of block. The edit guard keeps blocking generated paths and other non-authored outputs; for architecture sections and accepted ADRs it adds route context. `.design-edit` and `PSE_DESIGN_EDIT` retire. | Accepted | `crates/pse-host/src/agent_hooks.rs`, AGENTS.md "Off-limits", CLAUDE.md, the `.claude/settings.json` deny list, and `.claude/rules/decisions.md` immutability wording (supersession stays the route for accepted ADRs) |
| **R5.** Any agent may commit its own paths through `pse commit`, which uses a private index. Files committed whole are formatted at commit. | Accepted | AGENTS.md checkout workflow ("coordinator owns the shared index") and execution-rhythm formatting wording |
| **R6.** Managed command success is separate from subject state. Launch acknowledgement and an expired observation wait may exit0 while explicitly reporting launched/running; a completed pass requires reconciled outcome evidence. Exec adapters retain child exits. | Explicitly confirmed 2026-10-29 | A07 schedules §24.1 wording; A03/B01–B03/C02 implement the distinction without treating acknowledgement as scientific or test success. |

## Target

`pse` is one clap command tree in the `pse-host` crate's library. Thin `[[bin]]`s expose it as:
- `pse`;
- `pse-host`, which keeps the hook entry stable;
- `cargo-pse-correctness`, which Cargo invokes as an external subcommand.

Capabilities share these contracts:

| Contract | Owner | What consumers rely on |
| --- | --- | --- |
| clap usage contract | [59a](59a-cli-foundation.md#a02) | Typed shared host arguments, CliInvocationId and explicit selections; narrow owner dispatch preserves literal tokens and the owning parser. Generated catalogues describe grammar, never reconstruct arbitrary validators. Scoped discovery reports candidate freshness/exhaustiveness. No prefix inference or multicall. |
| Output and exit contract | [59a](59a-cli-foundation.md#a03) | A versioned envelope, summary first, with `--fields` and `--budget`; NDJSON streams ending in `summary`; diagnostic records with §23.2 codes, `help` and `next`. Exec-adapter verbs forward the child's exit. Managed verbs use 0/1/2/125, and 130 when interrupted. `launched` and a still-`running` wait exit 0 with the status in the envelope. Protocol outputs keep their raw formats: `pse-env --print`, the `--version` probes, the agent-hook protocol, and line-oriented tool listings. |
| State store | [59a](59a-cli-foundation.md#a04) | Scoped resolver with resource-specific lifetimes; checkout invocations, immutable preparation references, API/probe/scratch/handoff state. History/latest derive from invocation records, not a second mutable index. Scientific data and live fixture disposal stay with their original owners. |
| Run ledger | [59b](59b-runs-and-continuity.md#b01) | Run IDs generated by the launcher, per-run directories, terminal records written by the supervisor or finalizer, and the ledger as stop authority (a reconcile by recorded description and invocation ID) |
| Product observation | [59f](59f-simulation-observability-and-diagnostics.md#f01) | Product-owned requests/schemas/interpretation; host transport bounds; explicit references join invocation, scientific, canonical, diagnostic and process identities. Passive readers own no scientific cancellation or result authority. |

The ledger's ID is a CLI invocation identity, not a scientific RunId, canonical key or diagnostic
OperationId. Child outcome, command success, observation coverage, scientific outcome and physical
settlement remain separate. The finalizer may record a crash without establishing drain/disposal.
The output invariant core survives projection and budget selection; opaque upstream records are
not schema-qualified scientific/test facts. A03/B define the authoritative shared contracts.

**What stays the same:**
- `scripts/pse-env` exec forwarding (child exit, 128+signal, 125 for boundary failure).
- The agent-hook stdin protocol.
- The correctness adapter's force-validation closure and refusal rules.
- Admission and placement semantics.
- The cold-start closure of `pse-host`: no scientific, Arrow/DataFusion, database-client or
  product-schema dependencies.
- Products remain callable without `pse`.

**Intentional behaviour changes:**
- Managed verbs distinguish successful acknowledgement/observation from a reconciled subject pass (R6).
- New-work verbs fail on rebuild error; compatible existing-work control never requires rebuilding (R3).
- The edit guard informs instead of blocking for documents.
- Retired recipes and wrappers are deleted, not shimmed.

**Credible future changes the target accommodates:**
- **A new verb** is a derived subcommand plus a handler. The catalog, generated reference, `-h`
  and schema follow automatically.
- **A new heavy verb** uses owner-generated discovery and its owning parser, without importing
  product dependencies into the host or duplicating validators in a generated parser mirror.
- **A new run kind** (for example a benchmark runner) adds an extractor to the ledger. `wait`,
  `result` and `cancel` are unchanged.
- **A thin RMCP adapter**, if a consumer ever needs one, maps the same envelopes and schemas.

## Dependencies and route

Letters group responsibility, not serial execution lanes. A design/interface can enable parallel
authoring, but a production consumer needs the actual working prerequisite. The lifecycle binding
records package edges; the table below explains the delivered capability behind those edges.

| Consumer/change | Prerequisite capability | Reason and readiness |
| --- | --- | --- |
| A03/A05/B03 changed contracts | A07 decision/architecture route; original A01 remains done | Confirmed R3/R6 and identity/finality contracts precede dependent code; planning does not claim authority publication completed. |
| B01 foreground invocation | A02 runnable host, A03 output/identity, A04 invocation paths | Establish one real invocation/result journey before requiring a complete catalogue or migration of every state root. |
| C01/C02/C04/C05 selection consumers | C07 normalized selection | Package/binary/features/filters/ignored policy are shared; compatible groups preserve feature-absence cases. |
| C02 managed tests | B01/B02 working recording/extraction plus C01 | Actual nextest and correctness ownership; C04 supplies the selected fixture integration, not every fixture kind. |
| B03 detached/control | B01 generic recording, A03 and A05 control protocol/generation protection | Generic detached work does not wait for full B02 nextest extraction. Real service-entry admission and original settlement remain required. |
| A05 pruning; E01 removal | All-entry generation-use protection; B03 active invocation references when available | Unrecorded adapters count as users. Do not delete generations/checkout paths on ledger absence. |
| C03 affected; C02 recipe-documentation retirement | D06 generated-input/output map and informing document guard | Only these slices need the map/route; native test behavior need not wait on all D convenience verbs. |
| F01 product delegation | A03 transport/identity and A06 narrow owner-dispatch slice | Product design can proceed in parallel; CLI integration requires the working dispatch, not the whole catalogue. |
| F02 inspection | F01 working reader; B01 for invocation-linked navigation | Explicit checkpoint reads can ship before invocation-history views. No database is required for local process evidence. |
| F03/F04/F05 | F02 inspection and respective existing product owners | Follow, retained study reads and selected capture can develop independently; F04 live-publication decision does not block existing checkpoint/capture consumers. |
| D01–D07 engineering aids | Applicable A03/A04/A06 slices; D06 for generated-path flags | Owner forwarding preserves parser authority. Dependency/API tooling need not gate lifecycle control or simulation observation. |
| D08 commit | D09 transaction decision, D01 hunk identities and D06 map | No custom commit adoption before concurrent staging and post-commit partial outcomes have a sound route. |
| E01 warm checkout; E02 context | A04/A05; B03 for warm detached work; B05/A06 for context | Distinguish created from ready and refuse removal while actual users remain. |
| E03 adoption; F06 integrated consumers | Each working verb; F01–F05 selected core | Update current guidance and delete replaced consumers alongside capability delivery. |

Recommended first wave: A07 and the minimum A02/A03/A04 plus A05 capture/pin slice, C07 design and B01/B02;
integrate one actual foreground test/fixture consumer. Follow with A05/B03 safe detached control
and F01/F02 passive inspection, which share output/identity but retain separate domain owners.
Then add follow, retained studies, capture/analysis/export and ready engineering aids. Generation
pruning and worktree removal follow proven use protection. Optional live-publication/backends/MCP
are decision branches, not prerequisites to useful local delivery.

<a id="foundation-state"></a>
## Foundation state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| P01 | planned | 59 | Every opportunity row above resolved or deferred with its trigger; agents use the delivered verbs in ordinary work; scope-end `just hygiene` and the AGENTS.md manual checks pass at the zero baseline |

## Completion and verification

The series completes when Codex and Claude use the delivered verbs in ordinary work in place of
the hand-built forms the review recorded. That is a functional judgment by the agents using the
tools, not a measurement campaign.

During implementation each package runs its targeted unit and parser tests, as AGENTS.md
describes:
- `Cli::command().debug_assert()`;
- `try_parse_from` cases for argument groups and pass-through;
- `insta` snapshots of help and envelopes;
- schema validation of envelopes;
- the focused behavioural controls each package names.

At scope end, run once: `just hygiene`, `just docs`, `just governance` and the integration
journeys the packages name, reported against the zero baseline. No benefit quantification is
required.

Completion also requires an actual product-reader/inspection/follow journey and selected F05
capture→analysis/export use, with coverage/loss and independent scientific ownership preserved.
Cross-source timestamps are not global causal order; solver status is not a CLI success code.
The new reviews' closure scenarios belong to their named packages: broken-source cancellation,
supervisor crash/unknown drain, unrecorded adapter retention, changed tool generations, both
fixture modes, feature-absence/duplicate test identities, output pressure, diagnostic causes and
concurrent Git operations. Local snapshots alone cannot qualify physical or scientific lifetimes.

## Current state and next step

**2026-10-29 — execution stopped by the operator; production code reverted.** The rule changes
(A01) and the edit guidance (E05) remain. The code from A02 (the clap tree), C01 (capability
derivation) and D03 (the outline library) was reverted to the pre-execution tree at the
operator's request, and those packages are planned again. The D02 trial outcome is kept. No
package is in progress. Further execution needs new operator authorization.

### Review integration checkpoint — 2026-10-29

**Proposed:** the two new reviews are integrated as scheduled work and bounded investigations
across A–F. R3's existing-work exception and R6's command/subject distinction were explicitly
confirmed during preparation. A07 schedules their actual authority publication; no ADR/architecture
or production edits are part of this authoring change. All other accepted R1–R5 decisions remain.
Existing A01/E05 completion and the D02 trial retain their original evidence. This amendment does
not resume the stopped execution or qualify any new package. Review opportunities are not defects
by definition, and scheduling does not resolve a finding. No performance benefit is measured.

**Tested (document amendment only):** `scripts/pse-env -- target/debug/xtask lifecycle validate`
validated 1,662 native scope items with zero errors. A bounded `uv run --no-sync python` check
covered these seven Plan59 documents, all19 new source dispositions, local links/package anchors
and50 declared dependency edges with zero errors/cycles against baseline0. `git diff --check`
and `scripts/pse-env -- just turn-end` passed. Independent read-only integration advice found no
material omission/contradiction in the assessed drafts. These process/document checks do not
establish runtime implementation, scientific qualification or measured benefit. No product test,
native-provider preparation or service change was performed for this authoring task.
