---
title: "25d: Mathematical realization and response"
status: draft
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s04]
---

# 25d: Mathematical realization and response

## Context and target

FU06/FU07 expose different mathematical meanings under apparently interchangeable implicit
realizations and an unnecessarily universal second-derivative requirement. F22 requires square
root sensitivity without changing a model into an optimization problem. F25 concerns exact
rational representation. This plan supplies mathematical contracts to E1's representation
admission (F29) and E2's declared analyses. R5 corrects the response formula and validity scope.

An admitted residual relation and a selected implicit function are different contracts. A
relation admits every point satisfying its residual and domain constraints. A function additionally
identifies how its output is selected. Evaluation, acceleration, differentiation and factorable
export must consume that same meaning. Local nonsingularity proves neither global uniqueness nor
selection equivalence.

## Decisions and interfaces

- Admit relational use in constraint contexts and selected-function use in expression contexts.
  A relation cannot silently acquire one output because a numerical solver returns one root.
- A selected function carries either declared branch predicates plus its admitted equivalence
  justification, or an explicit deterministic operational selector with semantic anchor/settings.
  Ordinary initial guesses remain numerical aids. Where existing seeded behavior is intentional,
  migrate it to an explicitly declared operational selector; do not silently reinterpret it as
  a mathematical uniqueness claim.
- Each realization declares whether it can honor the selector. Polynomial acceleration may refuse
  a supported native operational selector rather than change meaning. Preserve the selected
  operation in preparation and lineage identities; do not automatically include every incidental
  solver tuning option in scientific identity.
- Deterministic selection alone does not justify implicit differentiation. Require local selector
  stability on the admitted neighborhood independently of residual nonsingularity. Without that
  evidence an operationally selected function is value-only and derivative requests refuse;
  a reproducible jump between regular roots is not a differentiable function.
- Factorable export is Exact only when its residual/branch constraints describe the selected
  graph. Residual-only export of an operationally selected function is Relaxed unless equivalence
  is established. A consumer requiring exactness receives refusal. Keep original candidate
  reevaluation and bound provenance separate; a valid relaxation bound does not make a relaxed
  point an original result.
- Separate residual derivative availability, output smoothness, inner-solver minimum and outer
  consumer demand. Compile their justified required order. First-order root iteration does not
  imply every outer computation needs a Hessian. Preserve stronger affine-rate and regime proofs.
- Keep Symbolica's exact Rational in the non-evaluating factorable export. Binary64 conversion
  is an explicit native boundary, not silent overflow fallback. ADR-0121's separately defined
  binary64 matrix certification convention remains distinct.
- Extract the regular-square response as **F_x · X_p = −F_p**, with original physical feasibility,
  structural scope, scaling maps, numerical rank, backward-error control and physical-coordinate
  output. Guard/bound/branch neighborhood validity accompanies the response. Optimization KKT
  assumptions remain separate; no dummy objective or weakened KKT test is introduced.

Symbolica owns symbolic manipulation, differentiation and polynomial isolation; faer/native owners
retain factorization and numerical solving. A new general uniqueness prover, expression evaluator
or root solver is not part of this change. Conservative relaxed export is the selected answer
when stronger selection evidence is unavailable.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="d1"></a>D1 Admitted implicit meaning | A3; H2; I1 | Own relation/function selection, domain and capability declarations across realizations | planned |
| <a id="d2"></a>D2 Faithful export and exact constants | D1; E1 | Carry selection restrictions/fidelity; replace bounded custom rational arithmetic | planned |
| <a id="d3"></a>D3 Demand-aware physical derivatives | D1/A3 | Compile inner and outer requirements consistently across compiler, provider and native oracle | planned |
| <a id="d4"></a>D4 Qualified square response | D3; E1 | Extract one reusable root-response operation and migrate fitting; expose it to E2 | planned |

E1's structural contract is independent of D4; E2 later consumes D4. This ordering breaks the
apparent mathematical-analysis/workflow cycle.

### D1 — Admission and substitution

**Implementation vision.** The admitted implicit operation contains residual body identities, ordered physical inputs and
unknowns, bounds/guards, relation-versus-function meaning, selection descriptor and derivative
capability. Native and accelerated realizations receive this product. The operational selector
carries its semantic anchor/profile identity separately from an ordinary numerical start. A
relation can feed constraint formulation; expression evaluation requires an admitted function
selection. Changing the selector therefore changes the semantic operation rather than merely
warming a different solver. Admission records what establishes branch restrictions and local
selection stability; unavailable evidence produces a capability limit, not an inferred proof.

Issue one compiler-owned implicit contract carrying residuals, physical coordinates, selection,
domain, available derivatives and smoothness. Selected branch restrictions are checked at
evaluation; unknown general equivalence is never promoted to proof. The initial supported exact
cases use restrictions/evidence the admitted representation can establish. Other selections use
the explicit operational path and conservative fidelity.

Focused controls use y²−p with two allowed roots, a restricted positive branch and a declared
operational selector. Different numerical guesses must not silently change a mathematically
selected branch. Unsupported acceleration receives a specific capability refusal. Delete
adapter-local reconstruction and residual-only representations that implicitly stand for functions.

### D2 — Export and rational migration

**Implementation vision.** Export returns auxiliary coordinates, original residual rows, admitted branch restrictions,
fidelity and the reason supporting equivalence or relaxation. The factorable DAG consumes this
product; it cannot initialize every implicit expansion to Exact. For y²-p=0, residuals over both
signs relax a positive-selected function. An established positive-branch restriction can make
the exported graph exact. Solver bounds and candidate points carry their respective provenance;
original candidate assessment remains a separate step. Rational constants stay exact library
objects through this DAG, including coefficients larger than machine integers, until a backend
explicitly requests a numeric representation with its documented rounding semantics.

Lower representable branch restrictions with the residual relation. Carry the selected fidelity
through candidate and bound source labels, retaining original-space assessment. If a selector
cannot be represented, a valid superset remains a relaxation; do not turn that limitation into
a claim of a demonstrated false bound.

Replace the local bounded Rational with the library representation and migrate all consumers,
including ownership changes caused by losing Copy. Keep exact coefficients until the documented
backend conversion. Remove custom rational arithmetic and its overflow-to-float path, not the
independent curvature/certification operations.

Focused controls distinguish positive/negative root feasible sets, exact versus relaxed labels,
large exact coefficients and explicit conversion. Coalescing/boundary roots do not obtain local
derivative validity merely because their graph exported exactly.

### D3 — Derivative demand

**Implementation vision.** Produce one derivative-requirements record containing available residual order, justified output
smoothness, selector-neighborhood validity, inner-solver minimum, requested output order and
resulting residual compilation order. Generate both provider descriptors and native oracle
contracts from this record. A regular stable C1 function with First output demand and a Newton
First requirement compiles First; Second demand produces a named unavailable-capability refusal.
A deterministic root-switching selector can remain value-only despite C2 residuals. Physical
coordinate differentiation consumes A3's composed wrapper rather than applying caller-specific
rescaling after evaluation.

Own the canonical profile-demand operation and its numerical requirement algebra here; E2
consumes it when migrating callers and does not supply a prerequisite implementation.
The compiler-issued provider and nested native OracleContract must agree. Availability and
smoothness are separately represented, including domain/selection neighborhood restrictions.
A3's physical wrapper is differentiated through the existing library machinery.

Focused controls: a regular C1 closure supports value/First and refuses Second specifically;
a C2 closure supports Second; reduced-coordinate results obey the physical chain rule;
regime/boundary restrictions and an unstable deterministic selector withhold unsupported
derivatives even at regular roots. Delete unconditional Second
compilation/declarations and consumed duplicate demand matches.

### D4 — Root response and fitting migration

**Implementation vision.** The operation receives a qualified original point, ordered state and parameter coordinates,
E1's structural assessment, scaling maps and the required Jacobian blocks. It returns the response
matrix in physical coordinates together with rank, backward-error and domain/branch-neighborhood
evidence, or a typed withheld-response reason. The ordered coordinates travel with the matrix
so consumers cannot mistake column position for parameter identity. Fitting and public sensitivity
consume this same product; neither rebuilds the scaled solve. A successful base-point root with
insufficient local validity can still be a scientific state while its sensitivity is withheld.

Extract the fitting-private operation into the mathematical/local-analysis owner. Consume E1's
mode-qualified complete square equality assessment; do not replace it with equation-count equality.
Retain numerical rank at explicit scaling, the negative parameter-Jacobian RHS and backward-error
acceptance. Return typed validity/refusal facts and physical responses.

Focused controls compare a regular square response with independent perturbation, and cover
singular/rank-deficient, guard-boundary, active-bound and branch-transition cases. Fitting and a
public root analysis consume the same operation. Delete the private duplicate after migration.
Clarify curvature method versus local statistical approximation for covariance; do not invent a
new statistical defect from the existing Exact enum's name.

## Authority and handoff

Use a short ADR for the implicit contract reconciling ADR-0100/ADR-0105, with blueprint §7.5/§9.5.
Amend ADR-0118's scope and blueprint §15.5.1/§25 for root sensitivity. Correct D6's wording so it
permits the existing non-evaluating factorable export while retaining library-owned mathematics.
A hashing change follows I1; physical kinds follow A; policy and public result use follow E.

E1 owns structural admission by representation, E2 declared execution, F the exhaustive policy
projections and J the generated public boundary. This plan hands them mathematical meanings and
qualified responses, not scheduler or serialization policy.

## Execution and evidence

All changes and expected benefits here are **Proposed**. Packet status is planning state;
no implementation or new product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define what those tests must establish, not claims
that tests with particular names already exist. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

## Outcome (recorded after implementation)

### What was built

Not implemented; record actual behavior and evidence labels at closure.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
