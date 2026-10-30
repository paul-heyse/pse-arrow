---
title: "Plan59: Agent-first pse CLI"
status: active
date: 2026-10-29
adrs: [ADR-0233]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md]
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
- Status: **Active.** The operator authorized execution of the full scope on 2026-10-29.

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
  automatic host rebuild and its fail-on-error behaviour rely on this.

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
| O13 | scheduled | [59a](59a-cli-foundation.md) | Generations pruned; fingerprints recorded; effectful verbs rebuild a stale host |
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

<a id="rule-changes"></a>
## Rule changes

The operator accepted each of these on 2026-10-29. Each is scheduled in
[59a A01](59a-cli-foundation.md#a01) before the packages that depend on it.

| Change | Decision | Route |
| --- | --- | --- |
| **R1.** `pse` is the strongly recommended agent interface. Its catalog is derived from the parser, not authored, so Plan58 I09's "universal catalogue" concern does not apply. This replaces Plan58 §Purpose's "CLIs are optional conveniences". | Directed by the operator | Amend Plan58 §Purpose and its Outcome wording; `docs/plans/README.md` |
| **R2.** Supervising runs that agents start (wait, result, cancel) is allowed. Build-stage telemetry and alerts stay retired. | Accepted | Amend the Plan58 target row on build observation, 58b's and 58h's wording, and the environment guide §Detached watches |
| **R3.** ADR-0233 narrowings: one CLI grammar and a per-run supervisor are selected; the ledger is the stop authority for detached runs; "launch never builds" applies to read-only verbs, and top-level effectful verbs rebuild a stale host. A failed rebuild fails the verb with the build error and never runs an older generation (operator decision, 2026-10-29). | Accepted | Amend proposed ADR-0233 in place, plus a `blueprint.md` revision row for the affected §3.2 and §24.1 host text |
| **R4.** Protected documents inform instead of block. The edit guard keeps blocking generated paths and other non-authored outputs; for architecture sections and accepted ADRs it adds route context. `.design-edit` and `PSE_DESIGN_EDIT` retire. | Accepted | `crates/pse-host/src/agent_hooks.rs`, AGENTS.md "Off-limits", CLAUDE.md, the `.claude/settings.json` deny list, and `.claude/rules/decisions.md` immutability wording (supersession stays the route for accepted ADRs) |
| **R5.** Any agent may commit its own paths through `pse commit`, which uses a private index. Files committed whole are formatted at commit. | Accepted | AGENTS.md checkout workflow ("coordinator owns the shared index") and execution-rhythm formatting wording |

## Target

`pse` is one clap command tree in the `pse-host` crate's library. Thin `[[bin]]`s expose it as:
- `pse`;
- `pse-host`, which keeps the hook entry stable;
- `cargo-pse-correctness`, which Cargo invokes as an external subcommand.

Every verb shares four contracts:

| Contract | Owner | What consumers rely on |
| --- | --- | --- |
| clap usage contract | [59a](59a-cli-foundation.md#a02) | Shared `GlobalArgs` (`--format`, `--budget`, `-C`, `-v/-q`), `SelectionArgs`, `PlacementArgs` and `OutputArgs`; typed domain value parsers (`PackageSpec`, `TargetSpec`, `RunId`, `CapabilitySet`, `Duration`); explicit selection groups; `ValueSource` provenance; `defer` subtrees; a federated xtask subtree read at runtime from a generated schema. Never prefix inference or `multicall`. |
| Output and exit contract | [59a](59a-cli-foundation.md#a03) | A versioned envelope, summary first, with `--fields` and `--budget`; NDJSON streams ending in `summary`; diagnostic records with §23.2 codes, `help` and `next`. Exec-adapter verbs forward the child's exit. Managed verbs use 0/1/2/125, and 130 when interrupted. `launched` and a still-`running` wait exit 0 with the status in the envelope. Protocol outputs keep their raw formats: `pse-env --print`, the `--version` probes, the agent-hook protocol, and line-oriented tool listings. |
| State store | [59a](59a-cli-foundation.md#a04) | One resolver for every host state path; the checkout store `$XDG_STATE_HOME/pse-arrow/<checkout-id>/` holding runs, fixtures, API cache, probes, Codex scratch and handoffs. History and `latest` are derived by scanning `runs/`; there is no separate index. |
| Run ledger | [59b](59b-runs-and-continuity.md#b01) | Run IDs generated by the launcher, per-run directories, terminal records written by the supervisor or finalizer, and the ledger as stop authority (a reconcile by recorded description and invocation ID) |

**What stays the same:**
- `scripts/pse-env` exec forwarding (child exit, 128+signal, 125 for boundary failure).
- The agent-hook stdin protocol.
- The correctness adapter's force-validation closure and refusal rules.
- Admission and placement semantics.
- The cold-start closure of `pse-host`: no scientific, Arrow/DataFusion, database-client or
  product-schema dependencies.
- Products remain callable without `pse`.

**Intentional behaviour changes:**
- Managed verbs exit 0 only on a reconciled pass (blueprint §24.1 P00).
- Top-level effectful verbs rebuild a stale host, and a failed rebuild fails the verb.
- The edit guard informs instead of blocking for documents.
- Retired recipes and wrappers are deleted, not shimmed.

**Credible future changes the target accommodates:**
- **A new verb** is a derived subcommand plus a handler. The catalog, generated reference, `-h`
  and schema follow automatically.
- **A new heavy verb owned by xtask** appears through the federated schema without changing the
  host.
- **A new run kind** (for example a benchmark runner) adds an extractor to the ledger. `wait`,
  `result` and `cancel` are unchanged.
- **A thin RMCP adapter**, if a consumer ever needs one, maps the same envelopes and schemas.

## Dependencies and route

```text
59a: A01 rule edits ─> A02 clap tree (+ `pse run` port) ─> A03 envelope ─> A04 store ─> A05 host tools ─> A06 catalog, federation, explain, status
59b: B01 ledger (needs A03/A04) ─> B02 extraction ─> B03 detached/wait/cancel ─> B04 composite runs ─> B05 handoff (needs A06)
59c: C01 capabilities (needs A02) ─> C02 test/check (needs B01–B02; its recipe-documentation edits need D06) ─> C03 affected (needs D06's map) ─> C04 fixtures ─> C05 discovery
59d: D01–D03 need A03/A04 · D04–D07 need A06 · D06 guard needs A01 (R4) · D08 commit needs A01 (R5), D01, D06
59e: E01 worktrees need A04/A05 (B03 for --warm) · E02 needs A06/B05 · E03 follows each verb · E04/E05 independent
```

**Why this order.**
- The rule-change edits come first because later packages implement behaviour those rules
  currently forbid.
- The clap tree, envelope and store are the contracts every verb consumes.
- After [59a A04](59a-cli-foundation.md#a04), the 59b, 59c and 59d packages are largely
  independent. They share edits to `crates/pse-host/src/cli/*` and its handler registry, so
  coordinate edit scope rather than serializing them.
- `pse test` and `pse check` (59c C02) consume 59b's foreground runs and extractors (B01–B02),
  and use B03 for `--detach` once present.
- D06 (the generated-outputs map and the guard change for R4) comes before C03, which consumes
  the map. It also comes before C02's recipe retirements, whose documentation edits include an
  architecture section.
- Communication (59e E03) is continuous. Each verb's AGENTS.md row, guide route and skill mention
  ship with that verb.

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

## Current state and next step

**2026-10-29 — execution stopped by the operator; production code reverted.** The rule changes
(A01) and the edit guidance (E05) remain. The code from A02 (the clap tree), C01 (capability
derivation) and D03 (the outline library) was reverted to the pre-execution tree at the
operator's request, and those packages are planned again. The D02 trial outcome is kept. No
package is in progress. Further execution needs new operator authorization.
