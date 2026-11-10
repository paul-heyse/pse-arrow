# Numerica: discovery briefing

The [advanced-addendum follow-up](addendum-followup.md), inspected at `924b0c3b2`, adds targeted research priorities and exact-source clarifications after the original discovery baseline.

Inspected 2026-11-09 at `d9e82627f` on `main`, including the supplied untracked
Numerica overview. Unrelated Plan72/SurrealKit working changes were present and are
outside this source interpretation. **Interface-checked:** representative consumers
and installed registry source for locked Numerica 3.0.1, checksum
`836accdb415b7667573bd12676e0b2eb4440e3f60b110038d96a9086f5651c15`.
**Proposed:** research directions below. No performance measurements or execution probes.
The [October 27 account](README.md) retains its original research dates; this briefing
refreshes only the named contracts. It does not establish a complete resolved-reference census.

## Current leverage and distinctive opportunities

Numerica already provides the exact rational and floating domains consumed through
Symbolica. Current `pse-math::library::admission::bounded_evaluator` constructs a dynamic
`HyperDual<Complex<Rational>>` and invokes Symbolica's `Dualizer`/`vectorize`; this is
existing library differentiation, not a missing AD implementation. `jets.rs` owns requested
product/layout policy rather than a second arithmetic engine. `convexity.rs` uses exact
rational symmetric-pivot LDLᵀ/Gram evidence; general matrix elimination is not a substitute
for that witness. The runtime's design generator uses egobox-doe and Xoshiro; probability
integration is a different responsibility.

The [full overview](../../../libraries/scientific_calcs/numerica.md) supplies broader
territory. The most promising questions concern composing existing domains and preparing
only necessary derivative products, rather than replacing all numerical machinery.

1. **Reuse derivative structure before introducing specialization.** Dynamic `HyperDual`
   stores values separately from an `Arc<DualShape>` containing its multiplication table.
   Constructing a fresh shape validates ancestors and builds the table; `variable()` inherits
   the shape. Determine whether compatible requests repeatedly rebuild identical shapes and
   whether the existing prepared-product owner can reuse their structure. Fixed macros
   precompute arithmetic tables, but would specialize Rust code for shapes that are currently
   selected at runtime. First discriminator: identify a stable, repeatedly consumed shape
   whose creation survives existing preparation reuse, then compare source-level work and
   resulting product demand. Retain dynamic construction when shapes genuinely vary.
2. **Use exact matrix algorithms for a named elimination product.** Dense fraction-free
   elimination, sparse CSR solving and modular fields could support exact model reduction or
   diagnostics without local Gaussian elimination. The next question is an actual block with
   rational coefficients and a useful reduction result, validated in original coordinates.
   Neither sparse solving nor rank supplies exact symmetric inertia or Gram/negative-direction
   witnesses; do not reopen the local convexity kernel on those API names alone.
3. **Compose numerical representations around the claim.** Rational derivatives, compensated
   precision, multiprecision, error estimates and balls offer distinct products. Recheck the
   exact consumed function set before using ball evaluation for certification; the locked
   source explicitly certifies basic arithmetic but not inherited transcendentals. A useful
   next case is a named cancellation-sensitive property or diagnostic with an independent
   original-coordinate check. The [Plan66 precision owner](../../../plans/66d-mathematical-representation-and-numerical-execution.md#symbolica-precision)
   retains its existing disposition; this broader discovery does not change its status.
4. **Add adaptive expectation workflows where the probability law is explicit.** Continuous,
   discrete and nested grids, mergeable training/statistics and RNG state export could combine
   with batched Symbolica kernels for expensive expectations. Current DOE recipes and covariance
   propagation do not already promise Monte Carlo inference. First establish a selected J03
   measure/dependence and failure-retention contract, then ask whether the library grid can
   represent its sampling weights and adaptation. Narrow correlated peaks and adaptive-estimator
   derivative semantics are countercases.
5. **Investigate a smaller dependency boundary only where it removes a real closure.** A
   numerics-only crate could import Numerica directly instead of a Symbolica facade. This can
   reduce coupling; it does not remove Symbolica from `pse-math` while that same crate still
   uses atoms and evaluators. Cargo feature unification means one direct dependency cannot
   undo GMP/MPFR selected elsewhere. Trace actual crate consumers and build units before
   predicting a clean or incremental Rust build improvement. Backend changes also change
   numerical behavior and deployment, so do not treat pure Rust as an automatic speedup.

## Capability coverage

| Overview territory | Discovery disposition and next question |
| --- | --- |
| §§1–2: ring/domain separation, ordering, sampling and polynomial kernels | Current exact domains are leveraged. Broader custom coefficient domains remain useful research for exact preprocessing; prefer existing high-level Symbolica polynomial operations before invoking low-level multiplication/division kernels. Which actual repeated bulk operation is not already delegated? |
| §3: adaptive integers, GCD, factoring, CRT, PSLQ, lattices | Exact integers are current foundations. CRT/modular arithmetic and relation discovery are selective research directions for coefficient-growth or scientific tooling problems. Factorization and primality have no traced independent production consumer in this pass. PSLQ produces candidates, not identities. |
| §§4–5: fractions, reconstruction, modular domains and primes | Potentially valuable together for exact linear/algebraic computation. Modulus bounds, bad primes, denominator invertibility and exact verification remain necessary. No claim of a ready-made complete modular solver. |
| §6: F64, DoubleFloat, Float, complex, error propagation and intervals | Distinct representations merit claim-specific investigation. `F64` equality collapses signed zeros and NaNs; it must not silently replace scientific identity encoding. Broad precision escalation and certified transcendental interval replacement are unsupported by this discovery. |
| §7: dynamic/fixed hyperduals and selected components | Already used dynamically through Symbolica; highest-value inquiry is shape and demanded-product reuse. Ancestor closure is required and invalid dynamic construction panics. Fixed shapes trade runtime preparation for Rust specialization. |
| §8: dense/sparse linear algebra and exact reconstruction | Real exact elimination capability; no automatic replacement for faer factors, native solvers or symmetric certificates. `solve_any` picks a compatible solution rather than a minimum-norm or full parameterized family. |
| §9: adaptive Monte Carlo, discrete/nested grids, statistics, RNG | Useful new expectation/integration hypothesis after measure semantics. Sampling/training/update is externally schedulable; reproducibility requires stream/state, update and accumulation policy, not just a seed. |
| §10: combinations, permutations, multiset partitions | Potential removal of small custom enumerators where multiplicities and output shape match. `unique_permutations` returns a materialized vector; this does not automatically reduce peak memory for large populations. Require a concrete caller before deeper comparison. |
| §§11–12: features/backends, serialization, interoperability and adjacent libraries | Build-boundary research is conditional. Serialization is type/feature specific; mathematical domain context and rounding must survive conversion. Existing Symbolica use means direct Numerica imports alone do not remove a heavy dependency. |
| §13: modular exact work, exact AD, error-aware AD, discovery/verification, sensitivity | Keep these composed workflows eligible. Exact derivative coefficients are supported; nesting arbitrary scalar/domain types still requires trait and function coverage. Differentiating adaptive random estimators has additional statistical meaning. |
| §§14–17: limitations, version changes, source map and synthesis | Constraints above qualify the shortlist. Published overview gaps about interval signatures and CSR are resolved by the locked-source checks below; latest-release assertions and all newer candidate behavior remain separately dated research, not inferred from this snapshot. |

## Evidence and handoff

Local source was inspected under `~/.cargo/registry/src/*/numerica-3.0.1/`:
`src/domains/{dual.rs,rational.rs,float/interval.rs}`,
`src/tensors/{matrix.rs,sparse.rs}`, `src/numerical_integration.rs`,
`src/combinatorics.rs`, and `Cargo.toml.orig`. Reproduce by resolving the locked package
and checksum, rather than treating the local registry directory name as portable identity.
Versioned API/source routes appear in the [domain and certification account](domains-and-certification.md).

Local interpretation consumed `pse-math/src/{jets.rs,library/admission.rs}` and
`pse-runtime/src/math/sampling.rs`, plus the existing exact-convexity and Plan66 accounts.
Source reads are representative; no absent spelling is a repository-wide absence claim.

Technical topics: [domains and certification](domains-and-certification.md), [differentiation and integration](differentiation-and-integration.md).

Fresh defining-package API exports and their limits are recorded in the [integrated evidence account](../symbolica/coverage.md#generated-evidence-and-limitations).
