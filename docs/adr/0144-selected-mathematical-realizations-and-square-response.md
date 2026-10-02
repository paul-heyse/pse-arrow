---
id: ADR-0144
title: Preserve selected mathematical meaning and qualify square responses
status: proposed
date: 2026-10-01
deciders: [paul-heyse]
level: decision
principles: [AP-02, AP-03, AP-04, DP-01, DP-04, DP-13, PS-07, PS-10, PS-12]
blueprint: [§D6, §7.2, §7.5, §9.4, §9.5, §14.3, §15.5.1, §19.4, §25]
review: docs/design_review/reviews/design_review_selected-mathematical-realizations_2026-10-01.md
evidence: Tested
supersedes: []
superseded-by: null
revisit: A supported selector cannot preserve its value or its established derivative neighborhood across realizations.
verification: Plan 25d selected-root, export-fidelity, exact-rational, derivative-demand and regular-square-response controls under Linux force-validation, 2026-10-01; bounded D6 and identity contract review remains Proposed; broader qualification remains 25k.
standard: core-3.3/process-simulator-1.3
scenarios: [docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs06, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md#fs07]
---

# ADR-0144: Preserve selected mathematical meaning and qualify square responses

## Context

Plan 25d addresses residual relations being exported as selected functions, unconditional
second-order demand, bounded rational overflow and fitting-private square response. Existing
ADR-0100/0105 and ADR-0118 limits need an explicit successor contract; their retained arguments
are preserved while this decision remains proposed.

## Scope

Amend blueprint §D6/§7.2/§7.5/§9.4/§9.5/§14.3/§15.5.1/§19.4/§25 within the
maintainer-authorized D1–D4 implementation. The non-evaluating factorable projection is a
backend transport derived from Symbolica, never an alternative expression evaluator.
NLP KKT analysis retains its existing optimization meaning. This supplements the affected
contracts pending the decision PR, which reconciles predecessor status; it does not edit or
accept an existing ADR locally.

## Drivers

One admitted scientific selection must govern evaluation, acceleration, export and derivatives.
Capabilities must remain replaceable without imposing unrequested Hessians. A regular square
flowsheet must provide physically qualified response without an objective. Exact coefficients
and semantic identities must not depend on bounded arithmetic or a library's in-memory hash.

## Options

Residual-only Exact export loses function selection; universally refusing exports discards valid
relaxations. Retaining a private fit response duplicates local analysis. The selected design uses
compiler-owned meaning, conservative fidelity, shared requirement algebra, existing Symbolica
rational/root/differentiation operations and faer response factorization. No custom uniqueness
prover, evaluator, differentiator or root solver is introduced.

## Outcome

A relation admits original residuals and domains. A function additionally declares branch
selection or an operational anchor/settings. Ordinary starts are numerical aids. Preserve
existing authored score selectors; intentionally seeded models declare their operational choice.
Unproved selection stability permits values only. Realizations unable to honor a selector refuse.

Factorable export carries restrictions and fidelity evidence. Exact selected-function export
requires equivalent graph restrictions; unsupported selection yields a valid Relaxed superset or
an exact-only refusal. Initial exact controls cover nondegenerate affine and explicitly restricted
square-root branches. Candidate original-space assessment and bound provenance remain separate.

**Proposed qualification clarification, 2026-10-02:** coupled cubic-density flowsheets
may explicitly author a residual-relational density model using the existing physical
residual, phase interval, validity and original-space checks. The outer solve then owns
the density coordinate. This is a chosen relational problem, not evidence that the
operationally selected density function is equivalent or differentiable. Preserve the
operational model and its value-only/refusal controls separately; starts and bounds do
not justify its nonlinear selector neighborhood. Positive physical oracles remain attached
to the explicitly chosen formulation. No new uniqueness or neighborhood prover is implied.

The factorable DAG retains Symbolica Rational, including large coefficients and exact folding.
Only an explicit backend boundary requests binary64 and rejects invalid conversions. Library
Hash is confined to in-memory interning. Changed content frames encode canonical normalized
numerator/positive denominator, selection and capabilities under new owner-issued versions;
historical frame spellings and ADR-0121 binary64 matrix certification are retained.

Pure derivative requirement algebra belongs to pse-kernels. The native owner derives the
selected adapter minimum, runtime resolves numerical profiles once, and compiler/provider/native
contracts consume the same requirements. Availability, smoothness, selection stability, inner
minimum and outer demand remain distinct.

The native local-analysis owner solves F_x X_p = -F_p for complete original square equality
scope. Ordered coordinates, physical feasibility, scaling, numerical rank, backward error and
guard/bound/branch neighborhood evidence accompany physical responses or typed withholding.
Fitting and Root parameter sensitivity share this operation. Existing sensitivity requests and
result tables are reused; root requests refuse reduced-Hessian/covariance propagation, and KKT-only
fields are absent. A qualified root can remain usable when its response is withheld.

### Consequences

Migrate affected authoring, compiler, providers, native adapters, fitting, public request/result
and generated consumers together. Remove bounded rational arithmetic, unconditional Second
contracts, duplicate demand rules and the fitting-private scaled response. Required E/H/I/J
slices remain bounded; the enclosing plans retain their unimplemented obligations.

### Compensating controls

Domain and selector evidence survive lowering; absent evidence cannot become equivalence or
smoothness. Targeted force-validation controls exercise multiple roots, unstable selection,
exact/relaxed export, large rationals, C1/C2 demand and regular/withheld square response.
Resource-bounded library solves and independent perturbations retain numerical acceptance.

### Confirmation

The bounded review establishes proposed architectural fitness only. Plan 25d owns executed
controls and actual limits. Series integration, scientific journeys and static qualification
remain in 25k. This ADR stays proposed until its separate decision-PR route is completed.

## Pros and cons

The design retains library algorithms and one production authority. Migration costs include
non-Copy rational ownership, changed identity versions and explicit capability refusals for
selection evidence that was previously implicit.

## More information

[Plan 25d](../plans/25d-mathematical-realization-and-response.md),
[coordinator](../plans/25-design-remediation.md), ADR-0082, ADR-0100, ADR-0105,
ADR-0118 and ADR-0121. Accepted predecessor arguments remain immutable.

## Status history

- 2026-10-01 — proposed before the maintainer-authorized implementation; decision-PR acceptance remains separate.
