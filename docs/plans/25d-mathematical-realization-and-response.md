---
title: "25d: Mathematical realization and response"
status: done
date: 2026-09-30
adrs: [ADR-0144]
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
| <a id="d1"></a>D1 Admitted implicit meaning | A3; H2; I1 | Own relation/function selection, domain and capability declarations across realizations | done |
| <a id="d2"></a>D2 Faithful export and exact constants | D1; E1 | Carry selection restrictions/fidelity; replace bounded custom rational arithmetic | done |
| <a id="d3"></a>D3 Demand-aware physical derivatives | D1/A3 | Compile inner and outer requirements consistently across compiler, provider and native oracle | done |
| <a id="d4"></a>D4 Qualified square response | D3; E1 | Extract one reusable root-response operation and migrate fitting; expose it to E2 | done |

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
Extend ADR-0118's scope through the proposed successor/supplement ADR-0144 and amend
blueprint §15.5.1/§25 for root sensitivity; accepted arguments remain immutable. Correct D6's wording so it
permits the existing non-evaluating factorable export while retaining library-owned mathematics.
A hashing change follows I1; physical kinds follow A; policy and public result use follow E.

E1 owns structural admission by representation, E2 declared execution, F the exhaustive policy
projections and J the generated public boundary. This plan hands them mathematical meanings and
qualified responses, not scheduler or serialization policy.

## Consumed 25c prerequisite slice

**Implemented/Tested, 2026-10-01; scoped focused verification recorded in [25c Verification](25c-process-composition-and-conservation.md#verification):** Conditional unit problems select explicit-map or owned square root realization before execution. The required native derivative order is derived from the selected adapter capability and consumed by the conditional compiler/runner. The full D1–D4 work now supersedes the limits of that slice; its original evidence remains owned by 25c. The maintainer authorized only this required slice and its complete affected consumer migration. The original slice was partial; [25c](25c-process-composition-and-conservation.md) owns the slice evidence.

## Completion and handoff

D1–D4 are complete, 2026-10-01, against `2c253c82` plus retained 25c closure
changes. Required E/H/I/J slices and affected consumers are integrated; their enclosing
packets remain open. ADR-0144, the bounded Proposed design review and blueprint revision
90 carry the enduring contract amendments. ADR acceptance remains a separate decision-PR
step; no accepted predecessor record was edited.

The [series coordinator](25-design-remediation.md) owns finding dispositions. Continue with
the remaining lettered packets in their dependency order. Full integration, static checks,
scientific journeys and performance qualification remain in
[25k](25k-integrated-qualification-and-closure.md). This completed plan is retained while
those packets and ADR-0144 consume its scoped evidence; it is not a new backlog.

## Verification

**Tested, 2026-10-01:** local Linux, pinned nightly and locked dependencies. All Nextest
commands below explicitly enable `pse-relations/force-validate` through the recipes.
Licensed numerical tests source `.envrc.local`, run through `scripts/native_exec.sh`
(memory-capped execution), and use one test thread. Compiler controls use the same licensed
environment through `direnv exec .` and the memory-cap wrapper. Baseline is zero failures.
Excluded tests are outside these filters, not evidence of product qualification.

| Owner and scope | Recipe and selection | Final result against zero failures |
|---|---|---|
| Pure requirements and provider binding | `direnv exec . just unit-package pse-kernels 'test(requirement_tests)'` | 2 passed, 0 failed; 41 excluded |
| Compiler selection, C1/C2, nested demand and regime evidence | `direnv exec . bash scripts/memory-cap.sh just unit-package pse-compiler 'test(implicit_selection_requires) \| test(implicit_c1_provider) \| test(kernel_nested_implicit_provider) \| test(kernel_nested_hints) \| test(implicit_nested_value) \| test(regime) \| test(implicit_minimum_score_unproved)' --test-threads 1` | 7 passed, 0 failed; 228 excluded |
| Exact constants, canonical identity, export fidelity, parametric order and implicit derivatives | `bash scripts/native_exec.sh just unit-package pse-math 'test(factorable) or test(parametric_plan_keeps_objective_and_differentiates_parameters) or test(implicit_)' --test-threads 1` | 20 passed, 0 failed; 122 excluded |
| Affected implicit derivatives after the final bound-neighborhood correction | `bash scripts/native_exec.sh just unit-package pse-math 'test(implicit_)' --test-threads 1` | 5 passed, 0 failed; 137 excluded; overlaps the preceding selection |
| Native square response, strict graph transport and implicit execution | `bash scripts/native_exec.sh just unit-package pse-backend-native 'test(square_response) or test(selected_graph_tests) or test(implicit::tests)' --features pse-backend-native/kinsol --test-threads 1` | 12 passed, 0 failed; 143 excluded |
| Shared fitting/public Root response, withholding, nested provider demand and request admission | Runtime command below | 22 passed, 0 failed; 273 excluded |

The runtime command was:

```bash
source .envrc.local
bash scripts/native_exec.sh just test-package pse-runtime --lib \
  --features pse-runtime/native-solvers \
  -E 'test(root_response_tests) or test(root_unavailable_tests) or test(kernel_nested_) or test(kernel_regime_selection_executes_branch_hints_and_refuses_ties) or test(steady_response_solves_the_compiled_implicit_closure) or test(bounded_rank_diagnostic_does_not_disable_sparse_candidate_evaluation) or test(library_parameter_rank_has_independent_controls) or test(sensitivity_request_in_solve_settings)' \
  --test-threads 1
```

**Tested controls:** exact restricted positive/negative square-root graphs versus unproved
Relaxed graphs; exact-only refusal; arbitrary Rational coefficients and fixed canonical
preimages; stable C1 First versus unavailable Second; native residual minimum without
promoting hints or unused providers; operational/multi-root selectors without proved
neighborhoods; selected-function bound and singularity refusal; complete original matching
including isolates; independent perturbed roots at nontrivial physical scaling; rank,
backward-error, active-bound, guard, selector and resource withholding. Public Root tests
check physical primal units, absent KKT/objective/dual fields and retained feasible base
solutions when optional response preparation is unavailable. The resource-publication
control injects the typed memory-unavailable result; it is not an allocator-pressure campaign.
KINSOL, Ipopt, POUNCE and SCIP public controls each exercise regular response and active-bound
withholding through the selected base adapter. Fitting exercises the same operation on a
compiled implicit closure.

This is **composite focused evidence**. An earlier runtime selection had 11 passes and
2 failures against zero: fresh matching's thread stack was incorrectly counted against the
numeric worker limit, and a new test incorrectly assumed a value-demand hint had no provider
seed. Both were corrected; the affected 2 tests passed, then the combined 16 passed.
After the explicit-adapter correction below, the final combined selection passed 22 tests
with zero failures.
An earlier native recipe selection failed before running tests because its selected packages
did not expose the explicit force-validation feature; the corrected `unit-package` recipe
above selects `pse-relations` as well. The first new explicit-adapter test build also
failed before execution on one assertion comparing an optional backend to a bare backend;
the assertion was corrected. Its first numerical run had 4 passes and 4 preparation
failures: the fixture requested First base compilation for Ipopt/POUNCE profiles whose
exact-Hessian base solver requests Second. The fixture now respects the selected adapter
order, while optional Root response still requests First. No failing test or compile error
is accepted as a baseline.

**Interface-checked:** final `source .envrc.local; just check`,
`source .envrc.local; just check-solver-contracts` and
`source .envrc.local; just check-native-python` all exited successfully after functional
changes, covering workspace targets, native contracts and the linked Python boundary.
Cargo reports the upstream future-incompatibility warning in `proc-macro-error2 v2.0.1`;
this does not establish a zero-warning static gate. **Implemented:** changed registry and
identity declarations were regenerated with `source .envrc.local; just codegen`; ADR index
was regenerated with `just adr-index`. Generated files were not hand edited.

**Not run:** `just hygiene`, broad integration/component/solver/Python journeys, full parity,
distribution builds, powerset/manual static gates, scientific campaigns and performance
measurements. These remain series 25k qualification. No Measured or whole-series Tested
claim is made. The independent design review remains Accept at **Proposed** evidence;
it is not implementation acceptance or broader product qualification.

## Outcome (recorded after implementation)

### What was built

**Implemented/Tested** under the focused commands and conditions above:

- D1 admits residual relations separately from selected functions. Explicit branch or
  operational anchor/settings meaning survives preparation, realization and identity.
  Missing graph equivalence stays Relaxed; missing local selection stability stays value-only.
  Seeded Peng–Robinson density explicitly declares its operational selection.
- D2 keeps Symbolica Rational through the non-evaluating DAG, removes bounded custom
  arithmetic and overflow-to-float fallback, and performs finite binary64 conversion only
  at the native boundary. Library-checked nondegenerate affine and restricted-square-root
  cases establish graph fidelity; unproved claims cannot become Exact. Strict predicates
  retain their meaning, and closed native endpoint projections carry Relaxed fidelity.
- D3 uses pure shared requirements for residual availability, output smoothness, selector
  stability, adapter minimum and actual consumer demand. Reverse nested demand propagation
  binds honest provider/native descriptors. Unconditional Second compilation and duplicate
  consumed demand matches are removed; C1 First and C2 Second remain distinct capabilities.
- D4 provides one regular-square local response `F_x X_p = -F_p`, retaining ordered physical
  coordinates, original feasibility/matching, scaling, numerical rank, backward error,
  neighborhood validity and allocation ownership. Fitting's private scaled solve is deleted.
  Existing Root sensitivity requests and generated validity tables publish the same physical
  response, with typed withholding that preserves a qualified base root.

**Implemented:** required E1/E2 original square scope and Root request dispatch, H2 implicit
syntax/checked occurrence consumers, I1 versioned canonical mathematical/selection frames,
and J2 generated request/result consumers. Wider route/default/qualification, authoring,
identity/reuse/resource and Python-boundary consolidation remain their owning packets.
Blueprint §D6/§7.2/§7.5/§9.4/§9.5/§14.3/§15.5.1/§19.4/§25 now describe these contracts
through ADR-0144. KKT meaning and covariance's local statistical approximation stay distinct.

### A mistake made and corrected

A separation margin between scored regimes was initially treated as proof that the native
root inside each regime was stable. A regime can still contain multiple regular roots.
Promotion now requires the checked within-alternative selection evidence; unsupported native
nonlinear regimes remain value-only. A new two-root-within-one-regime test exercises that
refusal. Active declared unknown bounds likewise withhold selected-function derivatives
without blocking values or native residual iteration.

The first native lowering of strict sign predicates used a small positive margin. That
excluded valid roots arbitrarily close to zero. It now exports the closed endpoint as a
Relaxed projection and refuses exact-only use, retaining original strict candidate assessment.
Fresh matching's stack reservation was also moved outside the separate numerical worker
allowance; its shared-pool reservation remains explicit.

Independent source review found that selecting response by native representation instead of
Root intent sent explicit NLP/global Root sensitivities into optimization-only KKT analysis.
Dispatch now preserves the selected base adapter, then performs one shared original-space
Root response independently. The same optional-response contract applies across supported
KINSOL, Ipopt, POUNCE and SCIP routes; public route regressions accompany the correction.

### Deviations from the plan, deliberate

The reusable response lives in the existing native local-analysis owner, using faer, while
Symbolica retains expression derivatives and exact arithmetic. No new solver/prover/framework
was introduced. Initial checked graph cases are deliberately narrow; lack of stronger proof
receives the planned Relaxed/value-only/refusal behavior.

Public Root parametric response requests First and reuses the existing sensitivity/result
boundary; base compilation retains the selected adapter's actual derivative requirements.
Reduced-Hessian and covariance propagation remain unavailable for Root; no dummy objective,
optimization dual or KKT sufficiency claim is added. Broader prerequisite packets and 25k
qualification remain open, as authorized. ADR-0144 stays proposed pending its decision PR.
