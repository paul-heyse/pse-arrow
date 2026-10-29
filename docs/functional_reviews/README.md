# Functional opportunity reviews

The [SurrealDB state, execution and coordination review](surrealdb-state-execution-boundaries-2026-10-29.md)
compares the two supplied placement proposals with Plan58 and the effective patched database.
It recommends complete bounded canonical progression, explicit native preparation/physical
ownership, retained persistent MX authority, and targeted graph, reactive, connectivity and
historical-analysis extensions. Ten opportunities identify enabling library mechanisms,
candidate code retirement, alternative triggers and proposed Plan58 amendments. It changes
no plan, production implementation, deployment or qualification status.

The [Rust-first system review](rust-first-system-functional-opportunities-2026-10-28.md)
assesses complete Rust ownership of product, operational and developer workflows, optional
Python bindings, and retirement of the Python MCP stack. It recommends independently usable
native APIs/product CLIs and a separate developer CLI, with no-MCP as a valid target and
reduced RMCP as a conditional adapter. End-to-end Rust-library overview research, focused
primary-source checks and independent assessment inform the proposed opportunities. It changes
no implementation or qualification status. Subsequent operator directions are owned by
[Plan58](../plans/58-rust-owned-system.md): CLIs are optional, PyO3 implementation is deferred,
and Plan57 is entirely superseded without automatic package transfer.

The [MCP development-effectiveness review](mcp-development-effectiveness-2026-10-25.md)
assesses the completed interface as a foundation for further Codex-driven development. Its
sixteen opportunities cover faithful Cyclopts-derived recovery access, connection/lifetime
diagnosis, affected validation, native model/result inspection, structured knowledge relationships
and focused execution/telemetry investigations. Fresh-client and disposable generator probes
retain their actual scope and failures; proposals change neither accepted plans nor runtime
configuration.
The [Plan55 coordinator](../plans/55-mcp-development-effectiveness.md) owns completed delivery
and inquiry dispositions for all sixteen opportunities. Its qualification Outcome records actual
acceptance and limits; this consumed review remains supporting evidence, not a new backlog.

The [OpenTelemetry and agent-observability review](opentelemetry-and-agent-observability-2026-10-24.md)
assesses shared Python/Rust/database telemetry and structured access through the single FastMCP
endpoint. It develops twenty opportunities, a proposed Collector/OpenObserve architecture,
native propagation seams, alternatives and rule impacts. Isolated SDK and disposable backend
inquiries retain their actual scope and semantic limitations; no production telemetry or
existing plan change is authorized by the review.

The [Rust build-reuse tooling review](rust-build-reuse-tooling-2026-10-24.md) assesses
sccache, cargo-shear and cargo-hakari for one local machine.

- **Measured findings:** fresh target directories miss about a third of a cold build,
  because build-script `OUT_DIR` paths enter the sccache key (an upstream fix is in review).
  Agent sessions split the cache keyspace through `CARGO_TERM_COLOR`. sccache 0.18.0 keeps
  the Rust cache.
- **Tested prototype:** shear reaches zero errors without changing any build selection.
- **Recommendations:** sccache 0.18.0 as a verified binary, Cargo content fingerprints,
  shear as the sole dependency detector in hygiene, and no hakari (its target saves 0.8% of a
  cold build).

Its opportunities authorize no configuration or plan change.

The [FastMCP usability and tooling-evolution review](fastmcp-usability-and-tooling-evolution-2026-10-24.md)
assesses the implemented single endpoint and the Python/Rust operations beneath it. It proposes
constructible typed requests, useful selective results, progressive instructions, actionable
recovery, scoped restart requirements and additional editing/execution capabilities. Code Mode,
protocol tasks and operator views retain conditional dispositions. Actual read/presentation probes
and concurrent-source refusals have explicit evidence limits; recommendations authorize no
implementation or changes to existing Plan52/53 scope.

The [unified FastMCP capability-interface review](unified-fastmcp-capability-interface-2026-10-24.md)
develops the operator-selected single entry point, deterministic reveal and progressive
instructions, with explicit design-principle assessment. It covers native schema alignment,
tool adaptation, lazy dependencies, generated CLI fidelity and the gRPC alternative.
It supersedes the earlier review's separate developer-surface and CLI-selection framing;
its target and rule proposals do not change existing plan status or implement the interface.

The [agent tool discovery and access review](agent-tool-discovery-and-access-2026-10-24.md)
assesses repository-wide agent tooling, including capabilities beyond Plans47/50/51/52.
It recommends source-backed conceptual discovery and selected optional FastMCP access,
compares CLI alternatives, and records Pydantic 2.14 and generated-CLI probe results.
Its opportunities and rule clarifications are proposals; existing plans retain scheduled scope.

The [agent-effectiveness review](agent-effectiveness-opportunities-2026-10-23.md) explores
reasoning and source context, semantic editing, behavioral/scientific verification, execution,
local research and operator interfaces on the completed Plans50–51 foundation. Its recommendations
and rule questions are proposals, with free-to-run local tooling as the selected constraint;
they authorize no installation or implementation.
The [Plan52 coordinator](../plans/52-agent-effectiveness-and-engineering-workflows.md) owns
the completed selected scope, operator-confirmed directions, conditional dispositions and
qualification limits; the review remains source evidence rather than implementation status.

The [SurrealKV and SurrealMX review](surrealkv-surrealmx-functional-extensions-2026-10-22.md)
explores project, scientific-data, evidence-analysis, storage-topology and delivery capabilities.
Its opportunities are proposals; the [Plan51 coordinator](../plans/51-engineering-workspaces-and-evidence.md)
owns the completed selected scope, inquiry decisions and qualification limits. This review remains
source evidence rather than current implementation status.

Reviews here explore valuable additions or improvements to product, developer, operator and
research capabilities. The operator selects an outcome, workflow, subsystem or tooling/library
ecosystem; reviewers discover useful possibilities and assess their realization within that
scope. A prior diagnostic audit, existing caller or design-principle violation is not required.

Use `plan-functional-extension-review` (source: `.codex/skills/plan-functional-extension-review/SKILL.md`)
for conversational preparation and `functional-extension-review` (source: `.codex/skills/functional-extension-review/SKILL.md`)
for the review. Design principles shape proposed realizations alongside the functional-value
argument. The diagnostic [design-review workflow](../../.codex/skills/design-review/SKILL.md)
retains its separate audit prerequisite and review conventions.

## Outputs and traceability

Use `<subject>-<YYYY-MM-DD>.md` for the principal review, with its scope, inspected source
baseline and evidence limits near the beginning. Explain current capabilities, useful
opportunities, resulting behavior, concrete enabling mechanisms and precise sources, alternatives,
interactions and recommended priorities. Organize flexible Markdown around the argument; no
fixed field schema, feature count or complete ecosystem inventory is required.

Give opportunities stable references such as `O01`, cited as `review-path#O01`; explicit anchors
make these usable across documents. Preserve references when a recommendation changes, and
explain correspondence when opportunities split or combine. Link related diagnostic findings
when useful without requiring a finding or relabeling an opportunity as a defect.

A review proposes value and direction. It does not approve changes to the functional target,
rules or an existing plan, and it does not authorize installation or implementation. When selected
opportunities move into a plan, retain their source references and concrete mechanisms in the
relevant target and packages. The owning plan holds current scheduled scope and disposition;
unscheduled recommendations remain here with meaningful next decisions or revisit conditions.
There is no separate mutable opportunity register or required periodic refresh.

## Recommendation audits

Use `design-review-audit` (source: `.codex/skills/design-review-audit/SKILL.md`) for a requested or
explicitly planned reassessment. It inherits this review's functional scope and assesses value,
scope, realization, interactions and missed opportunities. It does not initiate a broad diagnostic
audit or require the design-review report's full slot structure.

The [companion](audits/README.md) owns dispositions and net changes; the principal review is
revised in place into the complete resulting argument with dated attribution and recoverable
input. Planning consumes that clean integrated review. Editorial dispositions do not change
accepted implementation scope or plan progress.

Retain reviews and companions while they have current consumers. Follow the repository's
document lifecycle: carry surviving obligations and enduring knowledge to their owners before
retirement; Git preserves earlier accounts. Creating this scaffold performs no review or audit.
