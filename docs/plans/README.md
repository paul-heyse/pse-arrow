# Current work and plans

Plans own the execution status of work that is actually active; packets may own their own
progress. The current architecture and its supported scope live in the
[architecture sections](../authoritative_design/README.md), not in plans.

[Plan 32: Workspace content lifecycle](32-workspace-content-lifecycle.md) owns shared metadata
interpretation, explicit publication assets, producer-owned artifact lifecycle and the bounded
retirement pass. Its [checkpoint](32-workspace-content-lifecycle.md#open-items-and-current-checkpoint)
owns acceptance and handoff. The [documentation guide](../dev/documentation.md#metadata-and-aggregate-scope)
provides metadata and aggregate-scope commands across concurrent plans while preserving their
native status owners and existing substantive authoring guidance.

[Plan 28: SurrealDB unified simulation substrate](28-surrealdb-unified-substrate.md) owns the
confirmed hard-pivot target, accepted rule changes and US01–US05/EF01–EF08/F01–F05/PE01–PE06/PA01–PA04 dispositions.
Its [completion-audit reconciliation](28-surrealdb-unified-substrate.md#completion-audit-reconciliation-and-capability-decisions)
links the current corrective route and SurrealDB capability research.
The [current checkpoint](28-surrealdb-unified-substrate.md#current-checkpoint) also routes the
2026-10-07 production efficiency review and the preserved interrupted qualification handoff;
The [repository-wide efficiency extension](28-surrealdb-unified-substrate.md#repository-wide-efficiency-extension)
now schedules PE01–PE06 and comparable defects across current functionality.
The [remaining-design enhancement integration](28-surrealdb-unified-substrate.md#remaining-design-enhancement-integration)
adds that review's separately qualified F01–F05, accepted deployment-local replay decision,
stored-creation cancellation and bounded fitting/access-path investigations. Its owners remain the
existing companions; document adoption supplies no implementation or qualification evidence.
The [parallel-execution integration](28-surrealdb-unified-substrate.md#parallel-execution-integration)
routes the 2026-10-08 review and four confirmed rule decisions through the same A–H owners.
The coordinator owns its separately qualified Parallel F01–F05 dispositions; the companions
own canonical contention, temporary admission/native lifetimes, backend/deployment decisions
and concurrent ordinary studies. [28e's restart/acceptance route](28e-rebuild-retirement-and-qualification.md#parallel-extension-acceptance-and-campaign-restart)
keeps the campaign pause and links the required correction evidence. Consult the coordinator's
checkpoint for the next executable package rather than treating this index as a status ledger.
The [preparation assurance and reuse review](../design_review/reviews/design_review_preparation-assurance-and-reuse_2026-10-08.md)
investigates the prolonged Python preparation path and related consumers. Its
[coordinator route](28-surrealdb-unified-substrate.md#preparation-assurance-and-reuse-review)
owns PA01–PA04 and the unresolved runtime/scientific premises. The maintainer accepted
RC01–RC04 on 2026-10-08; [28i](28i-runtime-validity-and-interruption.md) develops receiving
validity/interruption and [28j](28j-pure-preparation-and-publication.md) develops pure
preparation/explicit publication. Their selected implementation and required consumer
integration are the current authorized scope; the coordinator checkpoint owns progress
and the boundary before remaining Plan 28 work.
The [preparation acceptance route](28e-rebuild-retirement-and-qualification.md#preparation-acceptance-and-investigation-handoff)
keeps future correction evidence separate from the narrowed failure-only continuation.
Its A–E companions develop canonical revisions, selected compilation/reuse, durable execution,
connected native queries/Arrow, and rebuild/retirement/qualification. Added
[28f](28f-shared-numerical-preparation.md), [28g](28g-bulk-data-operations.md) and
[28h](28h-native-setup-and-artifact-identity.md) develop shared numerical preparation,
bulk data operations and native setup/artifact identity. The coordinator's
[checkpoint](28-surrealdb-unified-substrate.md#current-checkpoint) owns the next execution route;
[28e](28e-rebuild-retirement-and-qualification.md#qualification-handoff-from-existing-plans)
owns assembled target qualification and the scientific campaign obligations explicitly
recorded there. Historical evidence remains available through immutable references;
retired plans create no additional work or scope-transfer obligation.

The [graph and hashing integration](28-surrealdb-unified-substrate.md#graph-and-hashing-review-integration)
routes the 2026-10-09 review through J3 indexed description settlement, B8/N13 retained
supplier topology and fresh registrations, N12 flow-policy lookup, and the selected RC04 V2
framing extension. [28k's followup outcome](28k-graph-kernels-and-hashing-investigations.md#followup-verification-and-outcome-2026-10-09)
owns the implementation, bounded measured inquiries and scoped evidence; the coordinator owns finding
dispositions. [28e](28e-rebuild-retirement-and-qualification.md#graph-and-hashing-extension-acceptance)
owns affected acceptance. RC02/RC03 production cutovers and the broader paused campaign remain
outside this selected followup scope.

[Plan 29: Agent workspace effectiveness](29-agent-workspace-effectiveness.md) is done (2026-10-07): every command runs through `scripts/pse-env` (environment, capped scopes in `pse.slice`), with effect-based canonical test scheduling, composable selection (`just affected`), run logs and preflight, and compact agent instructions with mechanics in [the agent environment guide](../dev/agent-environment.md). Its Outcome records the historical evidence; the prospective AE-25/AE-26 follow-ups now route to Plan 30.

[Plan 30: WebSocket RPC and the persistent agent environment](30-websocket-and-persistent-agent-environment.md)
integrates the [review](../design_review/reviews/design_review_websocket-and-persistent-agent-environment_2026-10-08.md)
and all six operator-confirmed rule changes. Its [dispositions](30-websocket-and-persistent-agent-environment.md#finding-dispositions)
own WP01–WP08 and the prospective AE-25/AE-26 follow-ups. Companions
[30a](30a-native-websocket-rpc-and-operation-lifetimes.md),
[30b](30b-persistent-services-and-receiver-generations.md),
[30c](30c-test-ownership-and-evidence-retention.md) and
[30d](30d-host-admission-and-timing-qualification.md) develop RPC lifetimes, service/receiver
generations, test isolation/retention and host admission/timing. Implementation and the
selected assembled qualification are complete (2026-10-09); its
[Outcome](30-websocket-and-persistent-agent-environment.md#outcome-recorded-after-implementation)
records repaired composite evidence and exclusions, including the passing original
thousand-point managed identity. The completed handoff remains for its Plan 28/29 readers;
the broader Plan 28 E4/E5 campaign remains paused.

Plans before 28 and their companions are retired to Git history. They are not a
backlog, and their open items create no work or scope-transfer obligation. Current
execution follows only the explicitly recorded scope of active plans from 28 onward.
Plan 32 owns workspace content lifecycle work; Plan 28 retains its own scope.

## Latest completed plan

[Plan 33: Efficiency principles remediation](33-efficiency-principles-remediation.md)
records the standalone implementation of all eleven efficiency findings and the six
bounded investigation decisions. Its
[Outcome](33-efficiency-principles-remediation.md#outcome-recorded-after-implementation)
owns affected Rust/native/Python acceptance, external-reference comparisons, actual
maintenance recovery and the explicit qualification limits. Enduring contracts belong to
the architecture and linked development guides. The highest-numbered plan is retained
under ADR-0096; it is not active work and does not resume Plan 28's paused campaign.

[Plan 31: Agent effectiveness enhancements](31-agent-effectiveness-enhancements.md)
records the completed tooling work, resolved F01–F05 and
[qualification outcome](31-agent-effectiveness-enhancements.md#outcome). Enduring operation
belongs to [the agent environment guide](../dev/agent-environment.md) and its linked guides.
Its retained handoff is not active work. Semantic Rust search was excluded, and native
SurrealDB MCP is opt-in per fresh Codex session.

## Workflow entrypoints

Use the [local process skills](https://github.com/paul-heyse/pse-arrow/blob/main/.codex/skills/README.md) to prepare or conduct reviews,
create implementation plans, and plan or execute authorized work. This index routes to the
owning plan or active packet checkpoint for current state, decisions and next steps; it does
not duplicate packet status. Handoff updates that owner when the actual tree changes.
The role and workflow rollout is owned by [§24.4](../authoritative_design/sections/design-change-workflow.md)
and ADR-0149 (independent review waived by the maintainer; it carries forward ADR-0147's [governance review](../design_review/reviews/design_review_library-research-writes_2026-10-01.md) and ADR-0139's [2026-09-30 review](../design_review/reviews/design_review_agent-coordination_2026-09-30.md)); it changes no production plan's lifecycle or qualification status.

## Norms

- **Naming.** `docs/plans/NN-kebab-case.md`, numbered in the order they were started.
  Numbers are never reused: `just plan` allocates the highest retained number plus one,
  so the highest-numbered plan stays until a newer plan exists.
  A coordinated series retains one numeric owner (for example `28-surrealdb-unified-substrate.md`)
  and may use alphabetic companion plans (`28a-...md`, `28b-...md`). The numeric owner
  reserves the number; `just plan` does not allocate the alphabetic companions.
- **Front matter.** `title`, `status` (`draft` | `in-progress` | `done` | `abandoned`),
  `date` and `adrs` (the decision records the plan implements). Optional
  `review_sources` and `scenario_sources` link existing definitions.
- **Living until done.** A plan is edited while it is being executed. That is the
  difference from an ADR, which is immutable once accepted.
- **Execution rhythm.** Packets are accepted by targeted tests and by deleting what they
  replace, as soon as the replacement is proven and its callers have moved. Formatting,
  lint and integrated tests run once, after all functional scope is implemented. Checkpoints record state,
  decisions and next steps, not per-command receipts. See AGENTS.md *Execution rhythm*
  and `.claude/rules/decisions.md`.
- **Outcome.** When the work lands, append `## Outcome (recorded after implementation)`
  with three sub-headings and fill all three:
  - *What was built* — what actually exists now, with the core principles §D evidence
    label for each claim.
  - *A mistake made and corrected* — at least one actual correction.
  - *Deviations from the plan, deliberate* — what was done differently and why.
    A deviation that changed a decision needs an ADR, not a paragraph here.
- **Closure.** Move enduring meaning to its contract or rationale owner, remove the plan
  from current work, and retire the completed plan and its resolved reviews once no
  reader, tool or test depends on them. Git history keeps them; no archive tree is kept.
- **Authority.** *When the code and a plan disagree, the code is what runs.* A plan is
  never cited as the reason something behaves the way it does; the architecture
  sections and the ADRs are.
- **Plans live here**, in the repository, not in an agent's scratch directory.
  `just plan` starts one.
