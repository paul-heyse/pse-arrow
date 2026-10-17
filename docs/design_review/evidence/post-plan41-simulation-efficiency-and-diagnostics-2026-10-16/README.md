---
doc_provenance: authored
doc_retention: while-dependent
title: Post-Plan41 simulation efficiency and diagnostics evidence
doc_owner: docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
---

# Evidence role and limits

These bounded inquiries support the [principal review](../../reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md).
They provide source and library evidence, not independent overall decisions or a second
disposition ledger. The original observations remain in [39d's dated audit](../../../plans/39d-qualification-and-plan38-cutover.md#post-plan41-simulation-audit--2026-10-16).

Inspected baseline: `main` at `6af92d1dee1ffa0b80dca71e03a63b489f2d940c`, including the seven
pending audit/collector/benchmark/documentation changes listed by that audit. No production
corrections, dependency changes or new scientific simulations were performed for this review.
Core 3.4, Efficient Architecture 1.0 and Process Simulator 1.5 supply the assessment standard.

The coordinator inspected decisive source and retained audit artifacts. Separate read-only
code inquiries covered preparation, publication/stack and diagnostics; a library researcher
checked the async, tracing and Salsa contracts. A fresh principal reviewer assesses the design
independently. Prior conclusions, including the audit's remedies, remain leads to challenge.

- [Preparation and reuse](preparation-and-reuse.md): actual consumers, unused checker work,
  source lookup semantics, structural hit paths and remaining expression leads.
- [Publication and caller stack](publication-and-caller-stack.md): separate IPC packing and
  metadata grouping, empty memberships, acknowledgements and real caller-thread exposure.
- [Library alternatives](library-alternatives.md): resolved capabilities and limits; current
  documentation is checked against pinned/local source rather than transferred wholesale.
- [Diagnostic lifetimes](diagnostic-lifetimes.md): retention/export loss, managed discovery
  and the already delivered collector repairs.

**Interface-checked:** current source and library contracts. **Measured / Tested, historical
audit conditions only:** the retained eleven-case observations, 64 MiB main-stack probe and
selected controls in 39d. Those executions are not repeated or promoted to current whole-product
qualification. Proposed remedies have no new performance measurements. Full Clippy/type checks,
statistical timings, broad solver campaigns and environment/provenance proofs are outside this
review. Scientific/source/result identity and truthful cancellation/drain remain relevant contracts.
