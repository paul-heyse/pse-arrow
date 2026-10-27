# rug and malachite

Realizations considered:
- installed `rug` 1.30.0, a transitive dependency of Symbolica and not declared by the workspace;
- `malachite-q` 0.4.22, source present in the local registry from another project. It is not in
  the workspace lockfile. The latest release is 0.14.0 (2026-10-13).

Research date 2026-10-27. Sources: installed source trees under `~/.cargo/registry/src/*/` for
`rug-1.30.0` and `malachite-q-0.4.22`, searched case-insensitively for `matrix|ldl|cholesky`.

## Types

No reusable library type is warranted. Both crates are scalar arbitrary-precision arithmetic
libraries (integers, rationals, floats, and complex numbers in rug). They supply the field
arithmetic an exact factorisation needs, but no linear algebra:
- rug: the only `matrix` match is in `ops.rs`, and it is not an API;
- malachite-q: no match.

The negative is scoped to the installed versions and this search. Upstream malachite 0.14 was not
inspected.

## Current applicability

### [CBO-025](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-025) — Exact rational LDLᵀ
- **Types:** none.
- **Realization:** rug 1.30.0 is installed transitively. Malachite would be a new dependency.
- **Conclusion:** ruled out. Neither crate has matrix factorisation. Symbolica's own `Rational`
  (backed by rug/GMP for large values) already supplies the arithmetic the local kernel uses.
- **What it would discharge / what remains local:** nothing beyond the arithmetic the kernel
  already uses.
- **Prerequisites, configuration or extension:** none.
- **Semantic gaps against the CBO's preservation constraints:** not applicable.
- **Unsettled premises and what would settle them:** none.
- **Evidence:** Interface-checked (source search scope as stated).
