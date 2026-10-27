# Maintained faer arithmetic policy

The published faer 0.24.4 and faer-traits 0.24.0 baselines, archive checksums and complete
maintained deltas are recorded in [deltas/readme.md](deltas/readme.md). The checked source
receipts are derived from those archive-plus-delta reconstructions. Native build identity
includes both checked source digests and `faer-f64-nonfused-scalar-generic.v1`.

The consumed native real binary64 closure uses faer for sparse LU, dense LU, fitting rank
and covariance SVD, self-adjoint PSD EVD, norms, matrix products, matrix-vector products and
rank updates. The maintained realization keeps faer's algorithms and the requested `Par`:

- `faer-traits/src/lib.rs`: native f32/f64 and their `ComplexImpl` dispatch owners select
  `pulp::Scalar`. Real `simd_mul_add`, `simd_conj_mul_add` and `simd_abs2_add` select pulp's
  `*_e_*` operations, whose Scalar implementations use separate multiplication and addition.
  Two complex norm helpers also move from explicitly fused to the nonfused operations.
- `faer/src/linalg/matmul/mod.rs`: delete the native nano-gemm, private x86 GEMM and gemm
  arithmetic dispatch, including their now-unused threshold constants. Native fields follow
  the existing generic vertical/horizontal SIMD interface with Scalar lanes; matvec and rank
  update retain their original dispatch through the same scalar policy. Arbitrary-stride
  fallback keeps its existing sequential/Rayon implementations.
- `faer/src/linalg/matmul/triangular.rs`: delete the independent private x86 GEMM branch and
  retain the recursive blocked generic implementation and its parallel joins.
- `faer/src/linalg/matmul/internal/mod.rs`: delete the independent private x86 GEMM branch,
  report that native dispatch is unavailable, and remove its empty-scratch shortcut. Sparse
  supernodal gathered/diagonal products use the matching generic scratch and blocked kernels.

Exact-source inspection covers every Rust source under `vendor/faer/src`, the selected trait
implementations and their dispatch macro. The only remaining `gemm::` use in faer is a threading
threshold observation in `linalg/householder.rs`; it executes no GEMM arithmetic. Source controls
reject newly introduced native dispatch in the supplier tree. Production callers in
`pse-backend-native`, `pse-math` and `pse-runtime` use public faer entry points and do not inject
an explicit alternate pulp SIMD type. The quad-precision and arbitrary custom-field APIs are
outside this binary64 production closure; this policy does not claim their arithmetic.

The optional residual LM route uses the pinned LM revision's own `PivotedQR`, `utils::dot`
and `utils::enorm`. Its nalgebra 0.35 operations are elementwise component multiplication,
scaling and `axpy`/`axcpy`; the latter reaches scalar `array_axcpy` in `base/blas_uninit.rs`.
That route does not call nalgebra GEMM or its `matrixmultiply::dgemm` branch. Resolved dependency
presence alone is not evidence that matrixmultiply executes. No nalgebra supplier patch is
selected for this route.

Run `scripts/pse-env -- .venv/bin/python scripts/faer_arithmetic.py --output <directory>`
for source dispatch controls, the actual selected dependency build, an optimized concrete
binary64 probe with FMA available to the compiler, scoped LLVM operation/call-reference
inspection and adversarial plus LU/SVD/EVD/sparse-mode controls. Stable named entry points
let the inspection distinguish the deliberately fused independent counterfactual from the
consumed nonfused supplier operations. It follows local function references, including callback
addresses, rejects unresolved supplier arithmetic and refuses explicit FMA or contractible
floating-point instructions. The two primitive entry points must retain separate `fmul` and
`fadd`. Its sibling unit tests also force small/large products, transposed destinations,
matvec, rank update, EVD/SVD recursion and simplicial/supernodal LU under Seq and Rayon.

On a supplier update, re-audit all independent dispatch owners, refresh both complete deltas
and receipts, and rerun these controls plus the affected scientific consumers. Numerical
passes alone do not prove absence of FMA. These controls do not measure throughput or qualify
unexercised solver classes, platforms or scientific campaigns.
