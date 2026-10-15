---
title: Simulation performance diagnostics supporting evidence
doc_role: index
doc_owner: docs/design_review/reviews/design_review_simulation-performance-diagnostics_2026-10-14.md
doc_retention: while-dependent
---

# Simulation performance diagnostics: supporting evidence

Evidence for the [principal design review](../../reviews/design_review_simulation-performance-diagnostics_2026-10-14.md), inspected on 2026-10-14 EDT against `a08f5fa091db1840fc95b46c4d4f849ce4c5de62` and the existing working tree. This folder supplies bounded evidence and analyses, not implementation authorization, current finding dispositions or a second overall judgment.

| Document | Role and boundary |
| --- | --- |
| [Instrumentation and live inspection](instrumentation-and-live-inspection.md) | Source-led investigation of observation lifetime, preparation, task/process correlation, recent history, resource scope and interruptions. No product execution. |
| [Library capabilities](library-capabilities.md) | Exact-version/native-fork catalog distinguishing current collectors, unexposed interfaces, instrumentation opportunities and unavailable or uncertain observations. No builds or runtime probes. |
| [Sparse backends and native timers](linear-backend-diagnostics.md) | Exact sparse-factor fields and costs, MUMPS in-factor progress, Ipopt structured timing and Clarabel extraction limits. Native capability is distinct from current bridge exposure or safe concurrent access. |
| [Contexts and analysis](contexts-and-analysis.md) | All simulation-context axes, causal explanation recipes and 26 selectable verification scenarios. These are not a mandatory Cartesian test campaign or timing gate. |
| [Tooling and probes](tooling-and-probes.md) | Collection/export alternatives and two tiny owned-process perf capability probes. Sampling access was demonstrated; useful symbolized stacks were not. No simulation benchmark or host configuration change. |

The principal reviewer was fresh to the scoped implementation, independently inspected decisive source, and reconciled the inquiries with the coordinator's evidence. Supporting recommendations remain **Proposed**. Existing source paths are **Implemented** or **Interface-checked** at the stated scope, not newly qualified numerical behavior.

The stopped cold benchmark supplies a motivating observation only. Its Criterion log estimated roughly 622 seconds per end-to-end cold operation after warmup; it did not publish completed phase data and does not locate that time within the solver. The current 192-variable mathematical closure differs from the older 96-variable result. Neither that historical result nor this review supplies a comparative speed claim.

The review's follow-up belongs to a future separately authorized plan-creation step. Active Plan 28/32/39 status, existing product corrections, scientific gates and resource ownership remain at their current owners. Environmental/provenance proof campaigns are permanently eliminated; diagnostic identities, timestamps and coverage do not reinstate them.

## Publication verification

**Tested, documentation only; zero-failure target:** `PSE_RESOURCE_CLASS=light just turn-end` completed its ADR-index and formatting steps with zero failures; `just docs` published 450 chapters with scoped search, and local file-target inspection found zero missing targets across the eight review/navigation/evidence files. The first `just turn-end` attempt used the default workload class and was refused before execution by an exclusive heavy-work owner; the explicit lightweight route completed without overriding that owner.

The broader `scripts/pse-env --docs --resource-class light -- python3 -m scripts.document_lifecycle validate` reported **one error against zero expected**: the pre-existing `docs/plans/39b-deployment-assurance-and-preparation.md::execution-packets` selector no longer identifies a native table after the earlier proof-target withdrawal. This review does not update that active plan or its scope binding. The new review/evidence documents were inventoried without a metadata error. No product test or simulation campaign is implied by these publication checks.
