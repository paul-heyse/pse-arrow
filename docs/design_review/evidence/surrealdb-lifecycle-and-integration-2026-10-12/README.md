# SurrealDB lifecycle and integration review evidence

Supporting evidence for the [principal review](../../reviews/design_review_surrealdb-lifecycle-and-integration_2026-10-12.md), conducted on 2026-10-12 against source commit `677394474a543c884a8a21d1a2626a6820ff834e`. A concurrent commit captured the initially dirty Plan 36 tree before the findings were settled. The later user-supplied code-intelligence paper is an additional review input, not part of that source baseline.

The principal owns the reconciled findings, target architecture, foundation and gate judgments, rule impacts and overall decision. Local SCI/CON labels identify bounded assessments; they are not additional implementation backlogs. Finding disposition and rule confirmation await future plan creation. Existing plans and reviews were not changed, and no product implementation is authorized by these documents.

| Evidence | Question and scope |
|---|---|
| [Scientific artifact lifetimes](scientific-artifact-lifetimes.md) | Authored/prepared state, requests/attempts, result/seed/analysis dependencies, selective retention, scientific history and numerical restart distinctions. Independent bounded assessment. |
| [Connection and transport lifetimes](connection-and-transport-lifetimes.md) | Borrower release versus physical close, ordered restoration, request admission, dispatch uncertainty and actual byte/driver drain. Independent bounded assessment. |
| [Operational lifecycle and recovery](operational-lifecycle-and-recovery.md) | Disposable fixtures, runner outcomes, exact resource metadata, service admission/drain, maintenance/backup/restore, MCP and logical versus physical disposal. Coordinator inspection. |
| [Library capabilities and tooling](library-capabilities-and-tooling.md) | Exact-version native mechanisms and tools, both papers' corrections, alternatives and conditions for adoption. Library research with primary-source links. |

The review used the [simulation-state paper](../../../white_paper_on_surreal_db_in_simulation_state_management.md) and [code-intelligence paper](../../../surrealdb_white_paper_code_intelligence.md) as leads. Context7 documentation and official version-tagged source were used for library questions. The stale utilization catalog and capability maps were not used.

The coordinator independently inspected the decisive study admission/seed acquisition/retirement paths, shared runtime store/heartbeat/close paths, bounded WebSocket restoration and malformed-response branches, exact resolved hashmap removal semantics, operational resource admission/removal/recovery, and SurrealKit cleanup/lock-refresh source. The fresh principal reviewer separately inspected decisive source and reconciled these assessments. The final review explains qualifications and disagreements rather than adding local verdicts together.

Evidence strength is source inspection (**Implemented** routes), inspected interfaces (**Interface-checked**) and architectural reasoning. Corrections and benefits are **Proposed**. No fresh runtime probes, product tests, native campaigns, service actions, storage cleanup or performance measurements were performed. Existing test bodies and selected historical results were read as counterevidence and possible distinguishing controls, without promotion to current assembled qualification. In particular, the earlier four seal controls do not establish a successful managed 1,000-point sweep after the final change.

Unexamined breadth includes complete numerical/physical qualification, every canonical query and migration, real disk-reclamation latency, power-loss behavior, remote deployment, large-workload memory/throughput and proposed alternative-tool execution. These are explicit coverage limits; absence of an executed probe is not itself a product defect. Documentation rendering checks, if performed, establish publication only.
