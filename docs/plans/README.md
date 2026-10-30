# Current work and plans

Native package tables own current status and remaining decisions. Index inclusion does not
resume stopped work or establish implementation, qualification or measured benefit.

| Plan | Work and status owner |
| --- | --- |
| [59: Agent CLI](59-agent-cli.md#current-state-and-next-step) | Current execution: native CLI, owned invocations, test/fixture selection, product observation and Rust code intelligence. |
| [58: Rust-owned system](58-rust-owned-system.md#remaining-bg-outcome--2026-10-29) | Selected A–G implementation complete; [58h](58h-qualification-and-publication.md) assembled scientific/reference/performance and deployment qualification remains separate. |
| [54: Rust build reuse](54-rust-build-reuse.md) | Compiler-cache/reuse work and its separately owned qualification handoff. |
| [53: Test architecture](53-test-architecture-target.md) | Test architecture and existing remaining qualification/decision scope. |
| [40: Simulation diagnostics](40-simulation-performance-diagnostics.md) | Native simulation diagnostics and its independent campaign status. |
| [32: Workspace content lifecycle](32-workspace-content-lifecycle.md) | Document metadata, retention and native scope interpretation. |
| [28: Unified simulation substrate](28-surrealdb-unified-substrate.md#current-checkpoint) | Canonical substrate and its independently authorized remaining work. |

Plan59's [dependency route](59-agent-cli.md#dependencies-and-route) and companion package
Consumes sections identify readiness; letters are responsibility groups, not serial lanes.
Conditional options and paused campaigns retain their recorded triggers and authority.

For supported implementation and historical qualification, use
[blueprint §24.2](../authoritative_design/sections/operations-and-validation.md#section-24-2).
Completed plans and superseded Plan57 are not a backlog.

Create or revise plans through [create-plan](../../.codex/skills/create-plan/SKILL.md);
prepare execution through [plan-execution](../../.codex/skills/plan-execution/SKILL.md).
Plans live here and use never-reused numeric identities with optional lettered companions.
The [documentation guide](../dev/documentation.md#metadata-and-aggregate-scope) owns lifecycle
metadata, publication and retirement. Architecture owns contracts, ADRs rationale, and the
owning plan scheduled disposition and execution state.
