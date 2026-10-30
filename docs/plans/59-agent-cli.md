---
title: "Plan59: Agent-first pse CLI"
status: active
date: 2026-10-29
adrs: [ADR-0233, ADR-0234]
review_sources: [docs/functional_reviews/rust-agent-cli-2026-10-29.md, docs/functional_reviews/agent-cli-and-simulation-observability-2026-10-29.md, docs/plan_improvement_reviews/plan59-cli-and-observability-2026-10-29.md, docs/functional_reviews/rust-code-intelligence-cli-2026-10-29.md, docs/plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md]
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
- Status: **Execution resumed by the operator for remaining A–I**, with only the necessary
  J01/J02 shared-contract exception. The latest checkpoint owns actual progress and limits.

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
8. [59g Rust semantic intelligence](59g-rust-semantic-intelligence.md): reusable sessions,
   source semantics, candidates, provenance and semantic edit proposals.
9. [59h API and dependency intelligence](59h-api-and-dependency-intelligence.md): capability
   discovery, examples, dependency explanations and migration briefs.
10. [59i Compiler investigations](59i-compiler-investigations.md): selected original evidence,
    ownership/async, representation and code-generation investigations.
11. [59j Source queries and change proposals](59j-source-queries-and-change-proposals.md):
    server-free addressed reads, structural queries, common proposals and contributor navigation.

## Library integration target amendment — 2026-10-30

**Proposed target; implementation paused.** The [library integration review](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md)
published at `0c8c17ebf` assesses production `09e878957`. Its I01–I16/RC01–RC04 references below
are qualified to that review, not the earlier reviews' similarly named findings. The full
selected standard and library evidence inform this amendment; no new diagnostic audit or
production qualification is claimed. The operator selected all sixteen recommendations for
planning and explicitly accepted these impacts on2026-10-30:

| Impact | Decision and consequence | Publication/work route |
| --- | --- | --- |
| RC01 | accepted: effect-shaped scoped protection for passive reads; durable custody before retained paths/children/finalizers; old unknown claims remain protected | A08 → A05/B03; amend proposed ADR-0233/0234 and applicable blueprint §21.7/§24.1 through their decision route |
| RC02 | accepted: independent raw stdout/stderr files replace encoded merged retention; compact offset index only for an actual observation-order consumer | A08 → B01/B02 and A03/A04 consumers; direct format migration with original-byte retention |
| RC03 | accepted: intentional immature wire/schema changes for owned pages and shared proposals; migrate consumers directly without compatibility shims | A08 → A03/J02 and affected producers; explicit versions/schema identity and preservation of unique original evidence |
| RC04 | accepted: syntax-only queries/previews do not require semantic preparation | G07/J03/G08 and native dependency bindings; semantic operations retain G01/G03 |

A08 owns new governing publication obligations; A07's prior completion remains historical.
No ADR/architecture amendment or production change occurs merely by revising these plans.

The combined target is an owned parser/intent → resolved/admitted stage → captured execution
route. Explain and execution use the same stage premises; explain never fabricates a prepared
fixture or actual test population. B's managed foreground/detached service path shares custody,
not scientific authority. Passive readers and reusable analyzer sessions retain independent
lifetimes. Product scientific inspection, canonical state and units stay with their owners.

A03's typed producer views and directional Serde/Schemars contracts replace shape-blind detail
truncation as the means of obtaining useful bounded answers. Generic projection remains
mechanical. B finalizes authoritative terminal results from actual evidence before projecting
an envelope. A04 distinguishes protected local publication, artifacts, authored inputs and
original diagnostic evidence. A05 protection follows actual retained effects with a qualified
promotion/pruning boundary; pruning stays disabled until that boundary and settlement work.

[59j: Source queries and change proposals](59j-source-queries-and-change-proposals.md) adds
J01 addressed source/pages, J02 shared proposals, J03 syntax queries/previews, J04 declared
generated-source navigation and J05 integrated consumer acceptance. D03's implemented reader
is the basis, G07 supplies semantic proposals, and G08 separately decides filesystem application.
D08/D09 retain Git publication; none of these mechanisms implies multi-file atomicity.

<a id="library-integration-dispositions"></a>
## Library integration dispositions

All findings below are **scheduled**, not resolved by authoring. Companion tables own package
progress. The review's observed defects, missing obligations and enhancements retain their
original meaning; scheduling does not turn every opportunity into a defect.

| Finding reference | Scenario reference | Disposition | Decision/work owner | Evidence or revisit trigger |
| --- | --- | --- | --- | --- |
| [I01](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I01) | explain→selected execution with unchanged premises | scheduled | A06/C07; B/D02/F/G–I consumers | Same-stage value; unavailable preparation causes no effects |
| [I02](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I02) | foreground/detached nested payload settlement | scheduled | B01/B03, A05/C04/G02 | Actual original drain, custody transfer and release |
| [I03](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I03) | scoped status/follow under reset/lag | scheduled | B03, F02/F03 | Native reconciliation, incarnation fence and gap handling |
| [I04](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I04) | killed passive read and retained child/path | scheduled | A05/A08, B03/E01/G02 | Qualified guard/promotion/pruning race; no old-claim deletion |
| [I05](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I05) | stalled stream sink and independent extractor | scheduled | B01/B02/B03 | Exact raw files, independent failures and flush/drain |
| [I06](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I06) | exact healthy ID plus unrelated corrupt history | scheduled | A04/A06, B01/B03 | Direct lookup; prefix ambiguity and incomplete latest/history |
| [I07](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I07) | nested outline budget and stale continuation | scheduled | A03/A06; J01 and D/F–I producers | Producer pages, indivisible records and source premises |
| [I08](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I08) | structural and semantic preview without conflation | scheduled | J02/J03, G07/G08 | Shared proposal, explicit authority, no session barrier for syntax |
| [I09](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I09) | staged/working/blob source navigation | scheduled | J01, D01/D08/D09/E01 | Exact source kinds, paths, supported Git policy |
| [I10](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I10) | original failure→useful next action | scheduled | D01; B02/F/G09/H06/I producers | Incremental packet without implicit compilation/preparation |
| [I11](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I11) | two observers, failed science, missing live numerics | scheduled | F01–F06 | Original statements/units/clocks, passive detachment |
| [I12](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I12) | tail/range/export versus protected local writes | scheduled | A03/A04; B/D02/F05/H/I | Backend limits, retention classes, original evidence protection |
| [I13](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I13) | valid wire view versus invalid domain finality | scheduled | A03; B01/B03 and F/G/H/I adapters | Checked owner construction, directional schemas and raw fidelity |
| [I14](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I14) | actual producer→consumer journey plus failure | scheduled | coordinator and all companions | Scope-specific targeted evidence and deletion after migration |
| [I15](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I15) | compatible/incompatible multi-root tests | scheduled | C07/C01/C02/C05, H03 | No repeated compatible graph admission; preserve negative features |
| [I16](../plan_improvement_reviews/plan59-library-integration-and-target-design-2026-10-30.md#I16) | generated hit→declared emitter/regeneration | scheduled | J04, D06/G05/D01 | Declared contributor lineage; unknown source stays unknown |

## Scope decisions

### Code-intelligence amendment basis

The [Rust code-intelligence review](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md)
at `a569b52e4` adds the G–I companions below. Its O01–O16 identities are scoped to that review,
not the original O01–O25 or observability review. Source assessment and library research remain
Interface-checked at their recorded limits; every new target is **Proposed**. Production execution
remains stopped. A01/E05 and the concluded D02 trial keep their original evidence.

- [59g semantic intelligence](59g-rust-semantic-intelligence.md): reusable analysis sessions,
  semantic reading/navigation, candidate tests/completion, macros and semantic edit proposals.
- [59h API/dependency intelligence](59h-api-and-dependency-intelligence.md): capability search,
  examples, dependency explanations, API evolution and rustdoc diagnostics over D02 acquisition.
- [59i compiler investigations](59i-compiler-investigations.md): selected ownership/async,
  representation and code-generation investigations, with conditional pin-specific drivers.

### Code-intelligence rule decisions

The operator explicitly confirmed all five review impacts during preparation on 2026-10-29.
These decisions select planning scope; governing publication is scheduled in A07 before any
dependent implementation. Existing R1–R6 are unchanged.

| Review impact | Confirmed decision | Route |
| --- | --- | --- |
| RC01 | Extend the CLI/output scope using existing envelope, store and discovery owners. | A03/A04/A06; D01–D03; G–I consumers. |
| RC02 | Target reusable owned analyzer sessions beyond individual queries, with explicit preparation/status/teardown. | A05/B03 operating slices and G01–G03; A07 routes any enduring contract/service change. |
| RC03 | Preserve C07 selection, C05 actual tests and F scientific evidence authority. | C05/C07, B02, F01/F02 and corresponding G/H/I adapters. |
| RC04 | Retain MCP/editor/Codex configuration exclusions; CLI may use stdio LSP. | Clarify D/E guidance; do not revive rejected servers. |
| RC05 | Edits/probes/generation/compiler preparation are explicit effects under current ownership and qualification rules. | G07/G08, D02, H/I; ordinary dependency policy and scoped decision routes. |

The operator additionally selected **preview plus conditional apply**: G07 delivers previews;
G08 must settle concurrency, protected paths and partial-failure behavior before application.
A proportionate failure to establish that route retains useful previews, not an invented
multi-file atomicity promise. No installation or runtime configuration change follows here.

<a id="code-intelligence-dispositions"></a>
## Code intelligence dispositions

All references in this table address the [code-intelligence review](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md).
Scheduling is not resolution. Companion package tables own execution state.

| Opportunity | Disposition | Work owner | Required evidence or trigger |
| --- | --- | --- | --- |
| [O01](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O01) | scheduled | G01/G04; D03 | Resolve actual use sites, preserve ambiguity and retain cheap syntax reads. |
| [O02](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O02) | scheduled | G04 | Selected annotations/types/captures with backend-supported fields and explicit unknowns. |
| [O03](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O03) | scheduled | G05; D01/C03 | Scoped references plus separately conservative impact selection. |
| [O04](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O04) | scheduled | G05 | Bounded calls/impl neighborhoods without exhaustive runtime claims. |
| [O05](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O05) | scheduled | H01; D02 | Usable public paths, constraints and capability search in exact documentation context. |
| [O06](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O06) | scheduled | H02; D02 | Authored example/setup/mode; explicit probe execution, extraction not execution. |
| [O07](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O07) | scheduled | G06; C05/C07 | Related-test candidates handed to actual selection/listing owner. |
| [O08](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O08) | scheduled | G06 | Useful method/import candidates; synthesis conditional on an actual expected-type need. |
| [O09](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O09) | scheduled | G07/G08 | Revision-bound previews first; conditional apply only after its publication decision. |
| [O10](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O10) | scheduled | B02; G04/G09; H05 | Original compiler facts, separate IDE diagnostics, scoped rustdoc diagnostics. |
| [O11](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O11) | scheduled | H03; D02/C07 | Declarations, metadata, unit plans and observed artifacts stay distinct. |
| [O12](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O12) | scheduled | H04 | Context-matched API evolution using native compatibility tools and selected consumers. |
| [O13](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O13) | scheduled | G05; D06 | Expansion/origin with honest mapping limits and actual generator ownership. |
| [O14](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O14) | scheduled | I01/I02/I05 | Selected ownership/async explanations; driver only when I05 establishes its need/fit. |
| [O15](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O15) | scheduled | I03–I05 | Selected constants/layout/instances; context-bound evidence, no portable ABI/performance proof. |
| [O16](../functional_reviews/rust-code-intelligence-cli-2026-10-29.md#O16) | scheduled | D01; G09/H06; F02 | Progressive investigation packets composed from available original sources; no automatic enrichment. |

Review capability families not selected as ordinary delivery retain their companion-owned
triggers: static indexes/term search in G, archives/Guppy/scraping in H, driver/Polonius in I.
Tiny editor actions, a replacement checker, custom compiler passes, eager corpora and routine
build monitoring are not adopted. A conditional inquiry may close with a reasoned deferral;
it cannot claim the deferred capability was delivered.

## Retained scope decisions

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

### Code-intelligence dependency integration

G/H/I are responsibility groups, not a new serial barrier in front of A–F. Read their
package-level Consumes sections and native lifecycle edges for exact prerequisites.

| Consumer | Minimum supplied capability | Independence and completion boundary |
| --- | --- | --- |
| G01 | Review and pinned backend evidence | Design/coverage investigation can begin before the CLI works. Native embedding requires a demonstrated protocol gap. |
| J01/J02 | D03 working source basis, A03 owned-view slice, then addressed-source contract | No semantic preparation; J02 proposal can serve semantic and syntax producers independently. |
| J03/J04 | J01/J02 for syntax previews; J01/D06 for generated navigation | Independent producers; no G03 session gate. |
| G07 | G03 selected semantics plus J02 proposal | Shares proposal without depending on the J03 pattern backend. |
| J05 | J01–J04 and A06/D07 selected discovery/reference slices | Actual server-free consumer; D01 packet composition is incremental integration. |
| G02/G03 | A03/A04/A05, B03 authenticated ownership slice, then C07 context slice | Reusable sessions require working placement and source synchronization; no whole B02/test-fixture prerequisite. |
| G04–G07 | Working G03 context and selected native operations | Reading, navigation, candidates and previews can progress independently; C05 only gates actual test handoff. |
| H01–H04 | D02 acquisition/probes, shared output and exact context | Main library journey needs no live analyzer. Local-use enrichment is optional, never a whole-G barrier. |
| H05 scoped rustdoc checks | D04 dispatch, A03 diagnostics and C07 selected check context | D02 API lookup is optional enrichment; ordinary rustdoc checks do not wait for API/probe delivery. |
| I01–I04 | Original B02 diagnostics, D02 probes, C07 selected compiler actions | No live analyzer prerequisite. Existing artifacts/probes precede a custom driver. |
| G08 | J02 proposal, actual J03 or G07 producer, D06 owner/guard contract | Syntax application needs no semantic producer. Investigation may select apply or preview-only; D09 is not a filesystem transaction. |
| G09/H06 and D01 packet integration | Their selected working producers and D01 base brief | Compose incrementally; base D01 never waits on consumers that themselves use it. |
| Optional F02 source navigation | Matching original source/build identity and a working source reader | Base F inspection/follow does not wait for G, H or I. |

Existing recommended first waves below remain useful. Establishing source intelligence is
not a prerequisite for ordinary command control, fixture delivery or scientific observation.

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
| A05 pruning; E01 removal | Effect-shaped protection for every retaining consumer; B03 original settlement | Unrecorded adapters and exported paths count as users. No deletion on ledger/PID absence; qualify scoped guard and durable promotion first. |
| C03 affected; C02 recipe-documentation retirement | D06 generated-input/output map and informing document guard | Only these slices need the map/route; native test behavior need not wait on all D convenience verbs. |
| F01 product delegation | A03 transport/identity and A06 narrow owner-dispatch slice | Product design can proceed in parallel; CLI integration requires the working dispatch, not the whole catalogue. |
| F02 inspection | F01 working reader; B01 for invocation-linked navigation | Explicit checkpoint reads can ship before invocation-history views. No database is required for local process evidence. |
| F03/F04/F05 | F02 inspection and respective existing product owners | Follow, retained study reads and selected capture can develop independently; F04 live-publication decision does not block existing checkpoint/capture consumers. |
| D01–D07 engineering aids | Applicable A03/A04/A06 slices; D06 for generated-path flags | Owner forwarding preserves parser authority. Dependency/API tooling need not gate lifecycle control or simulation observation. |
| D08 commit | D09 transaction decision, D01 hunk identities and D06 map | No custom commit adoption before concurrent staging and post-commit partial outcomes have a sound route. |
| E01 warm checkout; E02 context | A04/A05; B03 for warm detached work; B05/A06 for context | Distinguish created from ready and refuse removal while actual users remain. |
| E03 adoption; F06 integrated consumers | Each working verb; F01–F05 selected core | Update current guidance and delete replaced consumers alongside capability delivery. |

Next authorized wave: A08's changed-contract publication and the necessary A03/A04/A05
slices alongside B01/B03's actual foreground custody/capture/settlement journey. Establish
one test/fixture consumer with B02 and C07 compatible-root admission, then safe detached
control. J01/J02 source/views and F01/F02 passive inspection can progress on their independent
prerequisites; neither waits for semantic/compiler-private preparation. Shared design is not
an implemented prerequisite, and an early slice does not complete its enclosing package.
Then add follow, retained studies, capture/analysis/export and ready engineering aids. Generation
pruning and worktree removal follow proven use protection. Optional live-publication/backends/MCP
are decision branches, not prerequisites to useful local delivery.

<a id="foundation-state"></a>
## Foundation state

| Package | Status | Owner | Completion |
| --- | --- | --- | --- |
| P01 | planned | 59 | Every opportunity row above resolved or deferred with its trigger; agents use the delivered verbs in ordinary work; scope-end `just hygiene` and the AGENTS.md manual checks pass at the zero baseline |

## Completion and verification

Code-intelligence completion additionally requires the G/H/I consumer scenarios and explicit
conditional decisions. Native inference, API projection, compiler verdicts and runtime evidence
retain separate provenance and coverage. Structured-field availability is checked against the
selected provider; no hover/debug-string parser may invent authoritative structure. Source
generations, Unicode positions, dirty edits, pending preparation and stale proposals receive
revealing negative cases. The full review disposition table and all conditional branches must
have an owner and outcome; merely recording an API inventory does not establish usable delivery.

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

Execution is **active by the operator's subsequent instruction** for remaining A–I. The
operator selected the minimal J01/J02 prerequisite exception; J03–J05 remain outside this
execution scope. Historical paused checkpoints and their evidence remain below.
The series remains incomplete. Continue with the joined invocation-owner journey; do not treat
the implemented library foundations or targeted checks as an assembled managed-run result.

### Native consumer integration checkpoint — 2026-10-30

Compatible multi-root Cargo selection now preserves target and feature policies, splits only
typed feature-absence conflicts and reuses metadata within the invocation. Native nextest
reconciliation binds the captured population and exact binary identities; retries, ignored
policy, setup exit105, missing records and ambiguous protocol identities retain separate outcomes.
No result here certifies external fixture readiness or physical settlement.

`pse history`, `pse result` and bounded `pse log` read the original protected ledger without
new-work admission. History retains per-source failures and avoids terminal payload reads;
missing terminal evidence remains unknown completion. Stream windows retain original offsets
and non-UTF8 bytes. Native xtask parser discovery works outside a checkout, before owner effects;
generated federation/forwarding is still open. `pse dep` provides exact locked source-qualified
facts and lazy candidates, preserving metadata-context limits and name/version ambiguity.
Protected immutable launch/custody/settlement records, private consumed request transport and
an opaque held parent-launch exclusion are integrated prerequisites, not process authority.

Focused checks against zero failure baseline passed: compatible-root/metadata controls18;
correctness adapter27; extraction12 including actual pinned nextest; native notify/descendant
controls7; invocation/view/CLI controls30; dependency/catalog/CLI controls27; xtask discovery1.
The prepared CLI history/result/schema routes and built xtask discovery outside the checkout
were also exercised. The original native cleanup failure exposed a departed-main drain bug and
premature test-directory disposal; both were corrected, and that exact failed unit/cgroup is
confirmed absent. Its failed attempt remains failed evidence.

Actual service-owned placement/admission, shared supervisor/finalizer, original terminal
construction, nested custody and managed test execution remain in progress. Source-audited
fixture declarations and explicit-checkout/environment staged test consumers are being joined;
no assembled run, test, product or semantic journey is claimed. Scope-end static and integrated
qualification remain pending while functional scope is open. Execution remains active.

### Foundation execution checkpoint — 2026-10-30

A08 has published the accepted Proposed amendments in ADR-0233/0234 and blueprint317.
Independent focused governing-target advice found no required correction; it is not a formal
Accept verdict and does not extend A07's earlier review footprint. Formal ADR status is unchanged.

**Implemented:** exact invocation lookup bypasses unrelated history, prefix ambiguity precedes
chosen-record decoding, and collection reads retain typed row failures. Global latest refuses
incomplete history with original diagnostic children. Brief status skips retained results;
full reads avoid allocating opaque payloads. Candidate providers preserve partial coverage.
Response schema preparation is reused once while remaining before artifact retention; native
Schemars reference decoding and raw lexical fidelity are preserved.

Independent raw capture replaces the merged codec, with confirmed per-stream prefixes and
separate write/flush/sync uncertainty. The minimal replacement-only source/proposal contract
retains immutable working text, checked Unicode ranges, candidate/selected distinctions and
stale/overlap refusal. It performs no application and does not complete full J01/J02 or G07.

**Tested:** `just unit-package pse-host 'test(cli::tests) | test(invocation::tests) |
test(status::tests) | test(catalog::tests) | test(source_proposal) |
test(managed::streams::tests)'` passed61 selected tests with zero failures against baseline zero
(312 unselected) on the integrated tree. ADR lint passed162 records. Scope-end static checks
and assembled managed/product/semantic journeys have not run; their packages remain open.

Next connect actual notify-service/finalizer custody to B01/B03 and migrate C07/B02's
implemented selection/reconciliation into their CLI consumers. Current producer
tests are not evidence of a joined run/test journey or physical resource settlement.

### Library integration authoring checkpoint — 2026-10-30

**Proposed:** all sixteen review recommendations are scheduled in the targets, packages and
native bindings. A08 owns the four operator-accepted rule amendments; J01–J05 add source
queries/proposals without semantic preparation. Existing D03/A07 completion retains its
original scope; new extensions are open work. Independent integration reading corrected the
stale live execution-status sentence and added missing I09 Git history/rename and supported
hook/filter qualification obligations. No runtime or formal decision status changed.

**Tested (documents only):** `target/debug/xtask lifecycle validate` validated1,720 scope items
with zero errors against baseline zero. A focused read-only link/dependency check found no
missing local Plan59 references and no cycle across121 Plan59 dependency edges. `just turn-end`
and `git diff --check` passed. No product test, provider build or performance campaign ran;
the historical runtime results below keep their original evidence and conditions.

### Authorized entire-series execution — 2026-10-29

The operator authorized the prepared full Plan59 execution approach. A02's shared clap tree
and three binaries are integrated and their targeted parser/protocol and bootstrap tests pass.
A04's captured-environment path resolver and existing consumer migration are integrated;
invocation-history acceptance remains with B01. A07's target amendments are published in
proposed ADR-0233 and blueprint315, with independent scoped acceptance at Proposed strength.
D09 selected the Git publication strategy after eleven disposable-repository probes; D08 is
still implementation work. Existing A01/E05 completion and the D02 trial are preserved.

E04 is complete: the index is a compact current-work route, with status retained at its owning
plans. **Tested (documents only):** `scripts/pse-env -- target/debug/xtask lifecycle validate`
reported 1,698 scope items and zero errors after the index change. No scientific campaign,
provider rebuild, new datastore or measured benefit is established.

### Paused integration checkpoint — 2026-10-30

**Implemented:** all handed-off worker changes are integrated in the main checkout. A03 has
typed envelopes, native diagnostic causes, directional schemas and bounded projections that
retain full artifacts. A06 now exposes parser-derived command/help/catalog/schema and passive
status composition through the public CLI. Native `gix` observation supplies byte-preserving
Git facts without rewriting the index. Configuration provenance, shared resolved explain and
owner federation remain unfinished. An unexported, uncompiled provenance draft is preserved
locally at `.local/plan59-handoff/provenance-parser-draft.rs`; it is not delivered functionality.

A05 has immutable three-binary publication, source/build fingerprints, actual entry pins,
eligible stale top-level rebuild/re-exec and authenticated matching-generation cold copy.
The fixed source helper copies without destination Cargo; a source mismatch requires the
destination's own seed rather than letting an old helper build changed target policy. The
genuinely absent-target parent failure is fixed. Main host preparation also repaired the
legacy tool-directory permissions through the validating bootstrap owner. Positive
descendant/finalizer-based retirement of unresolved generation claims and pruning remain
pending; generation deletion is not enabled.

B01's immutable checkout-bound selection/result ledger and independently drained stdout/stderr
tee are integrated. B02's strict native Cargo-message extraction retains original diagnostic
spans, byte origins and completeness independently of child status. C01's actual unit-graph
capability join, C07's resolved selection foundations and C05's source-built immutable binary/test
inventory are integrated. Listing is not execution or setup; nextest population/setup/terminal
reconciliation and the joined run/test CLI consumers remain unfinished.

B03 has actual root/member separation, delegated controller-free payload subgroup capabilities,
authenticated service-main readiness notifications and cooperative task supervision. Native
fixtures exercise gated child attachment, nested/escaped descendants, original inode cleanup
and finite allocation inheritance. Serialized subgroup recovery remains test-only until a
protected production ledger owns adoption. The existing raw foreground root has not yet opted
into the delegated creation path. Proposed ADR-0234 and blueprint316 describe the shared
foreground/detached service supervisor and finalizer target; the actual supervisor, origin
lease, launch cutoff, original resource/permit settlement and wait/cancel controls remain pending.
Cooperative task completion and leader exit do not establish external descendant settlement.

D03 is complete within its selected scope: native `outline`/`show` and the shared `lib-outline`
route use lossless error-tolerant Rust syntax and CommonMark section structure, preserve exact
source ranges and ambiguity, and expose typed schemas. Batch reads use shared Rayon through
`tokio-rayon`, retain order and task ownership through cancellation, and preserve redacted
panic causes. `artifact read` uses the existing `object_store` local backend for bounded
continuations and native conditions, preserves opaque bytes, and does not claim an atomic
local snapshot. G07/G08 semantic editing remains planned; the reviewed AST-grep, tree-sitter
and typed fragment libraries are mechanism candidates, not implemented refactoring services.

**Tested:** `just unit-package pse-host 'all()'` ran 351 host library tests with 351 passing,
zero failures and zero skips in the default native correctness route, including the repaired
real inventory listing and native subgroup cleanup fixtures. `just test-package pse-host
--test catalog --test cli_output --test source_commands --test artifact_commands` ran 19
public CLI tests with 19 passing and zero failures/skips. `scripts/bootstrap-host` prepared
and probed all three main-checkout binaries. An actual disposable checkout then passed
`scripts/bootstrap-host --from <prepared-source>` from an absent target directory: no
destination `target/debug`, all copied binaries executable, byte-identical manifest and clean
Git state. The disposable checkout was removed. `just clippy` passed both workspace/all-target
`--keep-going --locked` modes (default and `--no-default-features`, each `-D warnings`) with
zero warnings/errors. `just family-check`, `just turn-end` and
`just adr-frontmatter-check adr-index-check` passed. All failure baselines are zero. These
are host/CLI and static checks, not full product/scientific qualification.

All workers have stopped; no owned test/build sessions, native services or finalizers remain.
The three integrated worker checkouts (`pse-plan59-clap`, `pse-plan59-selection`,
`pse-plan59-state` under the local cache) retain protected generation-use records. They are
preserved rather than removed: absent original PIDs and clean Git trees do not prove actual
descendant settlement. Their branches and the local draft remain available for handoff.
No generation pruning or push was performed during this closeout.

Next authorized implementation should join B01/B03 into one actual foreground invocation
under the shared service owner, then add detached/control journeys and B02's reconciled test
consumer. Complete A05 claim retirement only with that real lifetime evidence. Other A–I
packages and full-series integration/scientific/performance qualification remain outstanding;
no measured performance benefit is claimed by this checkpoint.

### Code-intelligence planning checkpoint — 2026-10-29

**Proposed:** G–I and the affected A–F slices schedule the code-intelligence review. RC01–RC05,
reusable sessions and preview plus conditional apply were explicitly confirmed. Document
authoring changes no runtime, tool preparation, formal ADR status or prior qualification.
Existing execution remains stopped; no production package has resumed. Begin with the existing
foundation route or G01's read-only capability investigation when execution is later selected.

**Tested (document amendment only):** `scripts/pse-env -- target/debug/xtask lifecycle validate`
validated 1,698 scope items with zero errors against baseline zero. A scoped document check
covered ten Plan59 documents, 299 local file/anchor references, all sixteen new opportunity
dispositions and acyclic dependency relationships; the final binding has 101 Plan59 dependency
edges after removing an unnecessary optional-lookup prerequisite. `just turn-end` completed.
Independent integration reading corrected an unqualified availability claim and kept H05
independent of optional API acquisition. These checks establish document integration only;
no new runtime capability, product test, preparation or performance benefit is established.

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
