# Numerical domains and certification

Reusable type: `numerica/domains-and-certification`.
Research date 2026-11-09. **Interface-checked** against installed Numerica 3.0.1, checksum
`836accdb415b7667573bd12676e0b2eb4440e3f60b110038d96a9086f5651c15`.
No runtime probe or benchmark. Source locations refer to that exact package.

## Resolved certification and exact-solving contracts


[`RealBall`](https://docs.rs/numerica/3.0.1/numerica/domains/float/struct.RealBall.html)
is midpoint/radius storage with directed-rounding certified `+`, `-`, `*`, `/` and inversion.
`ComplexBall` documents certified rational construction, negation, conjugation, basic arithmetic,
nonnegative integer powers and disjointness. Both source contracts explicitly exclude inherited
transcendentals from certification. The existence of a numeric trait implementation therefore
does not certify every operation invoked through it. `from_bounds` panics for reversed bounds;
`new` normalizes a negative radius by absolute value.

`Rational::to_f64` in `domains/rational.rs` casts fixed-width numerator and denominator separately
then divides. The large-integer path uses a mantissa-precision multiprecision intermediate.
Neither path promises direct correctly rounded rational-to-binary64 conversion across every
subnormal boundary. Use exact rational arithmetic for identities; qualify directed conversion or
rounding-cell checks separately when a certified floating result is required. Ordinary high
precision and propagated error estimates are different guarantees from interval containment.

## Exact matrix behavior

[`Matrix::solve`](https://docs.rs/numerica/3.0.1/numerica/tensors/matrix/struct.Matrix.html)
requires a vector right-hand side, augments the matrix, reduces, checks inconsistent rows, then
back-substitutes. An underdetermined result retains rank and the reduced augmented matrix in
its error. `solve_any` consumes that case, initializes zero free coordinates and fills pivot
coordinates. It returns one compatible solution; it is not a minimum-norm or canonical symbolic
solution-family interface. `solve_fraction_free` offers a domain-specific exact route.

[`SparseMatrix`](https://docs.rs/numerica/3.0.1/numerica/tensors/sparse/struct.SparseMatrix.html)
is CSR. `try_from_csr` validates array lengths/endpoints, monotone row pointers and in-range,
strictly increasing columns; `from_csr` panics on invalid input. Sparse `solve`, `solve_parallel`
and `det` are real capabilities. General elimination does not supply symmetric inertia,
positive-semidefinite decisions or Gram witnesses; no such public routine was found in the
inspected dense/sparse source. This is an exact-version/source-scope limitation.

## Build and persistence boundary


Numerica 3.0.1 `Cargo.toml.orig` declares Rust 1.89 and separate integer-gmp/integer-malachite,
float-mpfr/float-astro choices. Defaults select GMP/MPFR. Features are additive in a resolved
Cargo graph. An additional direct dependency does not remove another consumer's selections.
Serde/bincode derive support is explicit per type and feature (including RealBall); this does
not imply serialization of every numerical object or its external domain interpretation.

## Interpretation

These findings resolve the broad overview's uncertainty about public interval types, certification scope, CSR storage and underdetermined dense solving at the locked version. Generic field algorithms are useful for exact elimination, but they do not produce a symmetric factor witness. Error estimates do not replace enclosures. Direct Numerica use narrows a Rust dependency closure only when the consuming crate no longer needs Symbolica; an import spelling change alone removes neither native arithmetic nor symbolic compilation. These are source-established distinctions, not measured performance claims.
