# Plans

A plan records **how work is sequenced and verified**. A decision record
([`../adr/`](../adr/)) records **what was decided and why**. The two
have different lifecycles, which is why they live in different directories.

| Plan | Covers | ADRs | Status |
|---|---|---|---|
| [01 — Repository configuration](01-repository-configuration.md) | Repository identity, layout and migration, the Rust workspace and supply chain, the solver container, Python packaging and tests, the CI job graph, decision records, GitHub setup, the agent environment, the docs site | ADR-0001 – ADR-0038 | in-progress |
| [02 — Blueprint revision 5 contracts](02-blueprint-revision-5-contracts.md) | Design amendment for R4-01–R4-13 and L1–L9; separate Proposed implementation handoff | ADR-0039 – ADR-0048 | done (documentation; runtime handoff Proposed) |
| [03 — Wave 1 foundations](03-wave-1-foundations.md) | Historical foundation implementation and receipts; remaining scope carried into Plan 05 | ADR-0039 – ADR-0059 | abandoned execution strategy; superseded by 05 |
| [04 — Wave 2 semantic compilation](04-wave-2-semantic-compilation.md) | Complete graph/reference/inspection requirements retained by Plan 05; old execution strategy superseded | Contracts through ADR-0065 | abandoned execution strategy; superseded by 05 |
| [05 — Native logical-plan hard pivot](05-native-logical-plan-hard-pivot.md) | Historical implementation and receipts; every unfinished HP00–HP13 outcome carried into Plan 06 | Affected contracts through ADR-0067 | abandoned execution sequence; superseded by 06 |
| [06 — Provider-contract hard pivot](06-provider-contracts-hard-pivot.md) | Historical provider implementation and receipts; useful functions re-scoped to the unified simulator target | Inherited contracts through ADR-0067 | abandoned execution sequence; superseded by 07 |
| [07 — Unified DataFusion and Delta Lake hard pivot](07-unified-datafusion-delta-hard-pivot.md) | Broader simulator inventory and partial native/Delta implementation receipts; current architecture work re-scoped into Plan 08 | ADR-0065, ADR-0066, proposed ADR-0068 | partial implementation; execution sequence replaced by 08 for the newly narrowed scope |
| [08 — Schema-first native data pivot](08-schema-first-native-data-pivot.md) | Implemented schema/native compiler/provider/publication replacements and legacy deletions; remaining E00–E10 obligations and scoped receipts | ADR-0065, ADR-0066, proposed ADR-0068 and ADR-0069 | done; carried remaining scope completed and qualified by Plan 09 |
| [09 — Native caching and pivot completion](09-native-caching-and-pivot-completion.md) | Shared native caching, exact consumed-input reuse, prepared strata and every remaining Plan 08 obligation; [execution inventory](09-execution-inventory.md) | ADR-0046, ADR-0050, ADR-0065, ADR-0066; proposed ADR-0068–ADR-0070 | done; C00–C12 and carried Plan 08 scope qualified |
| [10 — Native contract compilation and consolidation](10-native-contract-consolidation.md) | Complete native consolidation reassessment: resolved contracts, shared validation, engine/testkit boundaries, relational compiler work, compact numerical preparation, native ownership, Delta and Python consolidation; full deletion and acceptance scope | ADR-0071–ADR-0073 | historical implementation and incomplete N18 receipts; unresolved acceptance carried through 11 into 13 |
| [11 — Integrated work reuse and native execution performance](11-integrated-native-performance.md) | Historical native execution design and evidence | Existing contracts through ADR-0074 | incomplete I18/I19 acceptance was carried into Plan 13; not automatically inherited by Plan 14 |
| [12 — Expand the petgraph skill into rust-graphs](12-rust-graphs-skill.md) | Agent tooling: rename the petgraph skill to `rust-graphs` and add rustworkx-core, leiden-rs, graphops, graphina, rust-igraph and raphtory at uniform full depth (library ladder, coverage and interop matrices, seams, probes) | none required (tooling) | done; 14 verify checks and every probe family passing |
| [13 — Rust computation architecture and carried acceptance](13-rust-computation-architecture.md) | Historical typed compiler, Salsa/graph and MathIR execution design and receipts | Proposed ADR-0076–ADR-0081 at its checkpoint | execution scope superseded by 14; historical W19 incomplete and W20 unrun; reuse requires new-target evidence |
| [14 — Library-owned process simulator hard pivot](14-library-owned-process-simulator.md) | Combined math reviews: library-owned arithmetic, properties, native solvers, initialization, indexed assembly, scoped dynamics/fitting, hard deletions and new target-derived acceptance | ADR-0082–0087 | complete for local Linux design-stage scope; [M22 evidence](14-m22-execution.md) |
| [15 — Rust build performance and persistent compilation reuse](15-rust-build-performance.md) | Compiler caching, artifact retention, dated nightly frontend parallelism, native prerequisites and measured build-profile improvements; no crate restructuring or increased check-first workflow | tooling; default-toolchain governance amendment if needed | draft; assessment and execution packets B00–B06 |
| [16 — Data-model architecture and consolidated review remediation](16-data-model-architecture.md) | Aggregated comprehensive/follow-up reviews: semantic admission and composition, physical/numerical contracts, native strategies and results, correct reuse, publication, consolidation and final qualification | ADR-0082–0086; new/superseding decisions scoped in P00 | draft; P00–P18 pending |

**Completed execution sequence:** Plan 09 integrates and completes the caching review
and all remaining Plan 08 E00–E10 architecture work. Its final acceptance review,
source archives, exact coverage ledger and measurements distinguish implementation,
behavioral qualification and performance evidence. No compatibility path or transition
period remains. Plan 07 retains the broader unscheduled simulator-function inventory;
that additional functionality was outside Plan 09.

[Plan 10 execution inventory](10-execution-inventory.md) retains its implementation
checkpoint and incomplete acceptance evidence.

**Current implementation direction:** [Plan 14](14-library-owned-process-simulator.md)
supersedes Plan 13's execution scope. It combines both library-math reviews and the
maintainer's requirement to select reuse on evidence of relevance to the new target.
M00–M22 is complete for the local Linux scope in the [M22 packet](14-m22-execution.md). The
[M09–M10 packet](14-m09-m10-execution.md),
[foundation contract](14-math-foundation-contract.md) and
[execution inventory](14-execution-inventory.md) record current source boundaries.

**Proposed next architecture work:** [Plan 16](16-data-model-architecture.md)
consolidates the two 2026-09-24 codebase reviews into one dependency-ordered target.
It preserves Plan 14's historical qualification and Plan 15's separate build-cache
scope. Implementation has not started.

Plan 13's [repair checkpoint](13-w19-repair-checkpoint.md),
[W15–W20 packet](13-w15-w20-execution.md) and [inventory](13-execution-inventory.md)
retain historical implementation and unsuccessful/incomplete qualification evidence.
They are not resume instructions or an inherited backlog. No W19/W20, I18/I19 or
older obligation enters Plan 14 merely because it remains unfinished. Useful graph,
Salsa, publication and resource mechanisms must satisfy Plan 14's evidence-based
reuse assessment. Historical receipts and incomplete verdicts remain unchanged.

## Norms

- **Naming.** `docs/plans/NN-kebab-case.md`, numbered in the order they were
  started. Numbers are never reused.
- **Front matter.** `title`, `status` (`draft` | `in-progress` | `done` |
  `abandoned`), `date`, `adrs` (the decision records the plan implements), and
  `phase` (the delivery phase from blueprint §25).
- **Living until done.** A plan is edited while it is being executed. That is the
  difference from an ADR, which is immutable once accepted.
- **Execution rhythm.** Packets are accepted by targeted unit tests and by deleting
  what they replace, as soon as the replacement is proven and its callers have moved.
  Integration, component, solver, Python and performance journeys, together with
  format/lint/governance/codegen/doc/ADR checks and any source seal, run once in a
  final qualification stage at the end of the plan — full qualification, not a sample.
  Checkpoints record state, decisions and next steps, not per-command receipts. See
  AGENTS.md *Execution rhythm* and `.claude/rules/decisions.md`.
- **Outcome.** When the work lands, append `## Outcome (recorded after
  implementation)` with three sub-headings and fill all three:
  - *What was built* — what actually exists now, with the charter §D evidence
    label for each claim.
  - *A mistake made and corrected* — at least one. A plan whose outcome records
    no mistake was either not executed or not read honestly.
  - *Deviations from the plan, deliberate* — what was done differently and why.
    A deviation that changed a decision needs an ADR, not a paragraph here.
- **Authority.** *When the code and a plan disagree, the code is what runs.* A
  plan is never cited as the reason something behaves the way it does; the
  blueprint and the ADRs are.
- **Plans live here**, in the repository, not in an agent's scratch directory.
  `just plan` starts one.
