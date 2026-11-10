# Exact algebra and solving discovery

Inspected 2026-11-09 at `d9e82627`; source observations **Interface-checked**, research directions **Proposed**. No scientific execution or performance measurement. Inherited October 27 root and Numerica findings retain their original scope.

## Existing delegation and stronger compositions

Cubic and collocation consumers already use Symbolica exact arithmetic, polynomial operations and maintained bounded root facilities. The bounded supplier additions are material: upstream version number alone does not describe the consumed contract. Request-local objective/coefficient reuse is already present in the inspected code; its acceptance status remains at the Plan66 owner.

The useful broader hypothesis is exact expression extraction → specialised coefficient domain → elimination → original-equation/physical validation → shared numerical preparation. Native `system_to_matrix` produces matrices over rational-polynomial fields, preserving selected parameter dependence. Resultants, polynomial reduction and Gröbner bases can eliminate unknowns or derive consistency conditions. Specialized GCD, coefficient-ring and modular algorithms can avoid unnecessarily expanded rational intermediates.

The next discriminator is a named small parameterised block, not a generic exact-preprocessing framework. Establish which variables remain unknown, what parameter specializations can invalidate elimination, what original domain restrictions survive, and how the original coordinates are recovered. A large transcendental block or elimination with severe coefficient growth is a revealing countercase.

## Solving has several different evidence products

Exact `solve` exposes branches, free variables and coverage guards. In maintained `solve/solution_set.rs`, generic coverage is distinguished from complete coverage. Exact roots and algebraic extensions retain embeddings/root selection rather than merely approximated values. Property predicates are three-valued; an inconclusive result cannot establish a negation. A numerical nonidentity test cannot establish pointwise nonzero behavior.

The maintained `AtomCore::nsolve_system` documentation explicitly says convergence uses Newton-step size independently of each equation's scale and does not certify a root. It could support a narrowly defined diagnostic or simple inner numerical solve, but API availability alone does not discharge selected-root, residual, scaling, failure or cancellation obligations. Operation admission and cancellation coverage remain unsettled; an iteration bound alone does not bound all supplier work. Existing native solver ownership is the comparison baseline.

## Additional exact and analytical territory

Finite-field evaluator mapping, CRT/rational reconstruction, sparse modular GCD and Hensel lifting offer exact arithmetic alternatives where characteristic-zero intermediates grow. Interpolation and Padé approximation can derive compact parameter dependence or rational approximants. PSLQ/lattice techniques can discover candidate scientific relations, with exact verification as a separate step. These are meaningful new tooling or research capabilities even without a current product caller; none should be claimed as general symbolic-function reconstruction.

Series and built-in special functions could expose constitutive asymptotics, singular limits and useful analytical preprocessing. Series retain omitted-order information that expression conversion loses. Symbolic integration requires its separate companion; branch cuts and domain conditions remain consequential. A built-in function does not establish support in every compiled numerical backend. Choose a named property-law limit or integral before deepening a backend implementation route.

## Evidence and next work

Source: `vendor/symbolica/src/atom/core.rs` (`system_to_matrix`, `solve`, `nsolve_system`), `solve/solution_set.rs`; representative consumers `crates/pse-math/src/{class_evidence.rs,implicit_cubic.rs,collocation.rs}`. Advanced algorithm leads consume the full Symbolica overview/Rust addendum; they have not all received implementation-level contract validation. Retain that distinction rather than asserting universal support or absence.

The next inquiry should identify one exact block and its complete scientific discriminator, plus one analytical tooling capability with an independent oracle. Discovery does not select adoption or activate deferred Plan66 work.

## Opportunity clusters and what they could remove

### Extract and reduce an exact block once

The useful input is a physically admitted set of equations with explicit unknowns and parameters.
For a linear block, `system_to_matrix` can retain rational-polynomial parameter dependence;
Numerica's exact matrix algorithms then supply elimination instead of application-maintained
coefficient arithmetic. For a polynomial block, retain the polynomial representation through
GCD, exact division, elimination and back-substitution rather than repeatedly returning to
ordinary atoms between operations. The result should expose recovered original coordinates,
parameter conditions and any conditions excluded by division.

This could remove repeated general-tree coefficient walks, duplicate exact conversions and
small local elimination loops where those responsibilities are actually present. It can also
enable a new diagnostic explaining parameter-dependent degeneracy. It does not follow that
eliminating the most variables is best: coefficient growth, hidden physical feasibility,
branch proliferation or lost sparse structure may make the original numerical block preferable.
The first assessment should follow one small real block into its actual residual/derivative
consumer, asking whether the reduced representation simplifies the complete operation.

### Reduce coefficient growth through native modular operations

Finite-field evaluator mapping, specialized polynomial GCD, modular reconstruction and exact
verification form a coherent alternative when characteristic-zero intermediate coefficients
become much larger than the answer. Exact arithmetic remains the required result; modular
computation changes the route to it. Bad primes and singular modular specializations are
algorithmic conditions, not proof that the original system is singular or has no solution.

The supplier already uses specialized algorithms internally. Calling low-level CRT, Hensel or
Hu–Monagan methods directly is valuable only where a higher-level operation does not already
make the appropriate choice or where a new mathematical workflow needs them. Otherwise extra
application orchestration could duplicate supplier policy. A useful discriminator is a traced
exact operation with disproportionate intermediate growth, together with a verifiable small
output and explicit failure handling for reconstruction. No such production bottleneck is
established merely by this API inventory.

### Treat analytical structure as a useful product

Series, Padé approximation, interpolation, algebraic extensions and symbolic integration could
serve scientific development, diagnostics and selected constitutive preprocessing. A series
can describe a singular limit or local expansion while retaining its truncation order; a rational
approximant can represent behavior more compactly than a large polynomial. Relation discovery
can propose a simple coefficient identity that exact arithmetic subsequently verifies.

These products must state whether they preserve an exact model or intentionally approximate it.
Rationalizing a fitted decimal or substituting a truncated series changes its mathematical
meaning. Such a product needs an explicit domain and approximation interpretation; it must not
silently enter an exact class certificate. For an antiderivative, differentiate the candidate
and inspect the original interval, poles and branches; agreement at selected numerical points
alone does not establish a definite integral across a singularity. A valuable next case is a
property law with a known analytic limit and an independent original-form check.
