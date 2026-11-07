# SurrealDB 4 canonical cutover design review

**Proposed, scoped to ADR-0256.** The review considers the fresh-generation
cutover against main `9d80d9836`, core 3.4, efficient architecture 1.0, process
simulator 1.5, and the repository binding. The change scenario is a replacement
of the canonical server with protected readers and owner-issued retirement.

Fresh reconstruction preserves domain and custody ownership (AP-03, AP-04)
with one runtime composition. An in-place conversion saves regeneration but
makes recovery depend on an irreversible supplier format change and carries
old state into a design-phase replacement. A long-lived dual route duplicates
both the trusted boundary and its qualification.

The selected remote MX profile retains native effects and explicit settlement.
A v4 client or SDK swap that weakens connection lifetime, typed errors, request
bounds or SSI would materially change the design. The private candidate must
prove those properties on actual effectful calls before cutover (AP-07, DP-09).
New query features earn adoption through a named consumer and measured useful
work. Scientific qualification remains with its original independent owners
(PS-09, PS-13).

**Disposition:** fresh reconstruction is the proposed implementation direction.
Durable activation remains conditional on the receiving rehearsal. Formal ADR
acceptance and measured performance claims are separate from this assessment.
[ADR-0256](../../adr/0256-surrealdb-v4-canonical-cutover.md) ·
[Functional review](../../functional_reviews/surrealdb-v4-cutover-2026-11-07.md)
