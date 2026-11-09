# faer — Comprehensive Rust Linear Algebra Library Overview

Target: faer 0.24.4 (latest verified stable release) | Research date: October 26, 2026 | Language: Rust | License: MIT

faer is a high-performance, pure-Rust numerical linear algebra library providing dense and sparse matrix representations, optimized arithmetic, matrix factorizations, direct and iterative linear solvers, eigensolvers, and sophisticated control over computational resources.

Its distinguishing characteristic is the combination of three architectural layers:

1. High-level numerical abstractions: Convenient matrix operations, reusable decompositions, and solver objects.
2. Low-level numerical kernels: Explicit control over matrix views, memory layout, scratch allocation, parallelism, and numerical algorithm parameters.
3. Structured and matrix-free computation: Sparse factorizations, symbolic/numeric separation, iterative Krylov methods, user-defined linear operators, and partial spectral computations.

This makes faer particularly relevant to scientific computing, numerical optimization, engineering simulation, large linear systems, and applications that need efficient numerical kernels without requiring an external BLAS/LAPACK installation.

The library explicitly targets medium-to-large matrix workloads rather than the small fixed-size geometry operations for which libraries such as `nalgebra` are often more suitable.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+3



## 1. Capability map

| Capability family                | Principal functionality                                                                                                      |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Dense matrix infrastructure      | Owned matrices, borrowed views, row/column vectors, strided access, transpose and conjugation, slicing, shape-aware indexing |
| Dense algebra                    | Matrix multiplication, matrix-vector products, elementwise operations, reductions, triangular operations, permutations       |
| Dense factorizations             | LLᵀ/LLᴴ, LDLᵀ/LDLᴴ, pivoted variants, LBLᵀ, LU, QR, column-pivoted QR                                                        |
| Spectral decompositions          | SVD, thin SVD, self-adjoint and general eigenvalue decompositions, generalized eigendecomposition                            |
| Sparse matrix infrastructure     | CSC/CSR matrices, structural/numeric separation, triplets, sparse arithmetic, conversion and index manipulation              |
| Sparse direct solvers            | Sparse Cholesky, LDLᵀ, LU, QR; symbolic analysis and reusable numerical factorizations                                       |
| Matrix-free numerical algorithms | Conjugate gradient, BiCGSTAB, LSMR, Krylov–Schur eigenvalue methods, matrix-free SVD                                         |
| Performance infrastructure       | SIMD, optimized GEMM, multithreading, reusable scratch workspaces, low-level kernel interfaces                               |
| Numerical extensibility          | Scalar traits, configurable algorithms, custom linear operators and preconditioners                                          |
| Interoperability                 | NumPy file I/O, `ndarray` and `nalgebra` integration pathways, complex scalars, Rust-native memory interfaces                |

The most consequential capabilities are not necessarily the individual factorizations. They are the ways those algorithms combine with reusable matrix storage, sparse structural analysis, matrix-free operators, and explicit workspace management.

### Version and project status

The latest verified crates.io release is 0.24.4, published June 24, 2026. Its immediate feature history is significant:

- 0.22: Accelerated matrix multiplication, improved pivoted QR, pivoted Cholesky variants, extended-precision scalar support, and stabilization of matrix-free solvers.
- 0.23: Generalized eigendecomposition, matrix-free self-adjoint eigensolvers and SVD, and improved multithreading.
- 0.24: Scratch-space APIs for matrix-free spectral solvers, CSC/CSR support in mixed sparse/dense multiplication, additional triangular-solve helpers, and removal of Serde support.

Since 0.18, the functionality previously distributed across crates such as `faer-core`, `faer-cholesky`, `faer-lu`, and `faer-sparse` has been consolidated into `faer`. The older component crates should not be treated as the current architecture.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



The project has also moved its primary development repository to Codeberg; GitHub remains a mirror.

Primary entry points:

- Versioned API documentation: `https://docs.rs/faer/0.24.4/faer/`
- Official documentation: `https://faer.veganb.tw/`
- Primary source repository: `https://codeberg.org/sarah-quinones/faer`
- GitHub mirror and changelog: `https://github.com/sarah-quinones/faer-rs`

## 2. Dense matrix infrastructure and data representation

### 2.1 Owned matrices, views, and storage layout

faer's foundational types are:

| Type                                   | Ownership and semantics                                   |
| -------------------------------------- | --------------------------------------------------------- |
| `Mat<T>`                               | Heap-allocated, resizable dense matrix; owns its elements |
| `MatRef<'a, T>`                        | Immutable, non-owning matrix view                         |
| `MatMut<'a, T>`                        | Mutable, non-owning matrix view                           |
| `Col<T>` / `Row<T>`                    | Owned column and row vectors                              |
| `ColRef`, `ColMut`, `RowRef`, `RowMut` | Borrowed vector views                                     |
| `Diag` / `DiagRef` / `DiagMut`         | Diagonal matrix representations                           |

An owned `Mat` uses column-major storage. Views can represent both column-major and row-major matrices, as well as arbitrary strided layouts.

This distinction is foundational: a matrix operation does not necessarily require faer to own or rearrange the input data.

The principal mechanisms include:

- `MatRef::from_column_major_slice` and `from_row_major_slice` for constructing borrowed views over contiguous memory.
- Strided constructors for interpreting matrices with nonstandard strides.
- `submatrix`, `subrows`, `subcols`, `get`, and `split_at` for creating restricted views.
- `transpose`, conjugation, and adjoint-related operations for changing how matrix data is interpreted.
- `Mat::with_capacity`, resizing and truncation methods for managing reusable owned storage.

For mutable views, Rust's exclusive-borrowing rules prevent unrestricted duplication of writable handles. The `reborrow` functionality allows temporary shorter-lived mutable or immutable views without relinquishing the original view permanently. Safe splitting also permits independent mutable access to disjoint regions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://faer.veganb.tw\&sz=32)

faer+1



Architectural significance: A numerical kernel can accept `MatRef` and `MatMut` interfaces rather than allocating owned matrices for every operation. This enables compatibility with independently managed buffers, non-contiguous slices, and nested numerical operations.

However, accepting arbitrary strides does not imply identical performance for every layout. Column-contiguous data generally offers better locality for faer's column-oriented algorithms. Some operations may require packing, temporary buffers, or slower access paths.

Source: `https://faer.veganb.tw/docs/dense-linalg/creating-matrix/`

### 2.2 Type-level dimensions and safe indexing

A more specialized capability is faer's support for branded runtime dimensions.

Its `Shape`, `Idx`, and related abstractions can associate indices with dimension identities, helping express the invariant that an index belongs to a particular dimension.

The underlying technique uses generative lifetimes to distinguish dimensions even when their sizes are determined at runtime.

This can move index validation outside inner numerical loops, allowing subsequent operations to rely on previously established bounds.

It is especially relevant to low-level kernel development because it combines:

- Runtime-sized matrices.
- Compile-time relationships between dimensions and indices.
- Reduced repeated bounds checking.
- Rust's memory-safety model.

It does not make arbitrary dynamically shaped inputs compatible automatically; shape correspondence must still be established.

The official SIMD engineering documentation explains this mechanism in detail, including its relationship to `generativity`.

Source: `https://faer.veganb.tw/docs/contributing/simd-in-faer/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://faer.veganb.tw\&sz=32)

faer



### 2.3 Arithmetic and allocation control

faer provides ordinary operators for convenient numerical expressions:

```
use faer::{Mat, Scale};

let a = Mat::<f64>::identity(100, 100);
let b = Mat::<f64>::ones(100, 100);

let sum = &a + &b;
let scaled = Scale(2.0) * &a;
let product = &a * &b;
```

The convenience syntax can allocate intermediate matrices. faer also provides lower-level APIs to control allocation and avoid unnecessary materialization.

`zip!` / `unzip!`

These support traversal of corresponding matrix entries, elementwise transformations, and writes into existing storage. They are valuable when a compound expression would otherwise generate several temporary matrices.

`linalg::matmul`

Provides an explicit in-place multiplication interface with controls for:

- The destination matrix.
- Whether existing destination values are replaced or accumulated (`Accum`).
- Scaling.
- Parallelism (`Par`).

Specialized kernels

The low-level API additionally exposes triangular solves, triangular inversion, Householder transformations, Jacobi rotations, Kronecker products, and related primitives.

These allow higher-level numerical algorithms to use existing optimized kernels rather than reconstructing operations from generic matrix arithmetic.

Sources:

`https://faer.veganb.tw/docs/dense-linalg/mat-ops/`

`https://docs.rs/faer/0.24.4/faer/linalg/`

`https://docs.rs/faer/0.24.4/faer/linalg/matmul/`

[image](https://www.google.com/s2/favicons?domain=https://faer.veganb.tw\&sz=32)

faer+1

## 3. Dense decompositions and direct linear solvers

faer's decomposition objects are reusable numerical representations, not simply convenience functions returning transformed matrices.

They make it possible to separate an expensive factorization from subsequent operations involving different right-hand sides.

### 3.1 Factorization and solver selection

| Factorization     | Matrix requirements                                                     | Main numerical role                                             |
| ----------------- | ----------------------------------------------------------------------- | --------------------------------------------------------------- |
| LLᴴ (Cholesky)    | Hermitian positive definite                                             | Efficient, stable direct solutions                              |
| LDLᴴ              | Hermitian matrices; stability depends on pivoting and matrix properties | Symmetric/Hermitian factorization with explicit diagonal factor |
| Pivoted LLᴴ/LDLᴴ  | Structure-dependent                                                     | Permutation-aware Cholesky variants                             |
| LBLᴴ              | Hermitian, potentially indefinite                                       | Stable indefinite systems using 1×1 and 2×2 pivot blocks        |
| Partial-pivot LU  | General square matrix                                                   | Default general direct solving                                  |
| Full-pivot LU     | General dense rectangular matrix                                        | Stronger pivoting and rank-sensitive work                       |
| QR                | General rectangular matrix                                              | Least-squares problems and orthogonal transformations           |
| Column-pivoted QR | General rectangular matrix                                              | More robust rank-sensitive least squares                        |
| SVD               | General rectangular matrix                                              | Rank analysis, pseudoinverse, ill-conditioned problems          |

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 3.2 Cholesky-family factorizations

The distinction between the Cholesky variants is consequential.

LLᴴ: For a Hermitian positive-definite matrix,

\\[ A=LL^H \\]

where \\(L\\) is lower triangular.

It benefits from Hermitian symmetry, avoids general LU pivoting, and supports efficient triangular substitutions after factorization.

The high-level method `Mat::llt(Side::Lower)` or `Side::Upper` uses the specified triangular part of the input. Failure to satisfy positive definiteness can produce an error.

LDLᴴ: Represents the matrix with a separate diagonal factor,

\\[ A=LDL^H. \\]

The low-level library supports LDL-family algorithms and pivoted variants. This form can be useful where diagonal pivots, inertia-related information, or an LDL representation are desirable. A diagonal factor alone does not guarantee numerical stability for arbitrary indefinite matrices.

LBLᴴ: The `lblt` family, corresponding to Bunch–Kaufman-style indefinite factorization, supports matrices that cannot satisfy the positive-definiteness assumptions of LLᴴ.

It represents

\\[ PAP^T=LBL^H, \\]

where \\(B\\) has 1×1 and 2×2 diagonal blocks.

Those blocks and the permutation permit stable handling of many indefinite problems for which unpivoted Cholesky would be inappropriate.

faer exposes both high-level factorization objects and low-level algorithms with configurable pivoting and workspace management.

Relevant APIs: `Llt`, `Ldlt`, `Lblt`, `linalg::cholesky`, and the corresponding factorization and triangular-solve modules.

Source: `https://docs.rs/faer/0.24.4/faer/linalg/cholesky/`

### 3.3 LU decompositions

For general square systems, the typical direct factorization is partial-pivot LU:

\\[ PA=LU. \\]

`partial_piv_lu()` computes the decomposition and exposes a solver object that can be reused.

Full-pivot LU introduces both row and column permutations:

\\[ PAQ^T=LU. \\]

This provides more extensive pivot selection and can improve robustness when the matrix is nearly singular, at additional computational expense.

For particularly ill-conditioned or rank-deficient problems, the choice extends beyond simply increasing the amount of LU pivoting; QR and SVD may provide more appropriate numerical information.

The lower-level LU modules expose factorization routines separately from the high-level objects.

Source: `https://docs.rs/faer/0.24.4/faer/linalg/lu/`

### 3.4 QR and least-squares computation

QR decomposes

\\[ A=QR, \\]

where \\(Q\\) is unitary/orthogonal and \\(R\\) is upper trapezoidal.

faer uses blocked Householder representations for orthogonal transformations, rather than requiring a full explicit \\(Q\\) matrix at every stage.

This matters because a Householder sequence can be applied to a right-hand side or another matrix without first constructing the dense orthogonal factor.

The capabilities include:

- Unpivoted QR.
- Column-pivoted QR.
- Compact Householder storage.
- Computing or extracting thin factors.
- Applying Householder sequences from the left or right.
- Least-squares solving through the decomposition.
- Low-level, scratch-controlled factorization.

For tall, full-column-rank systems, QR least squares avoids explicitly forming \\(A^HA\\), which would square the condition number in the normal equations.

Column pivoting is preferable when reliable rank determination or near-linear dependence is important. SVD provides another option when rank deficiency or very small singular values require more detailed treatment.

Source: `https://docs.rs/faer/0.24.4/faer/linalg/qr/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.5 High-level reusable solver interfaces

The `faer::linalg::solvers` module exposes several traits that distinguish factorization functionality from solve functionality:

| Trait                           | Purpose                                                   |
| ------------------------------- | --------------------------------------------------------- |
| `SolveCore` / `Solve`           | Solve ordinary, transpose, conjugate, and adjoint systems |
| `SolveLstsqCore` / `SolveLstsq` | Least-squares solve functionality                         |
| `DenseSolveCore` / `DenseSolve` | Dense-specific solver operations                          |
| `ShapeCore`                     | Expose the dimensions of the represented operator         |

A factorization can therefore be reused across multiple right-hand sides and, where implemented, different operator orientations.

For example:

```
use faer::prelude::*;

let a = mat![
    [8.0, 2.0],
    [1.0, 5.0],
];

let b = col![4.0, 7.0];
let c = col![2.0, 3.0];

let lu = a.partial_piv_lu();

let x = lu.solve(&b);
let y = lu.solve(&c);
```

Illustrative API example; not executed in this research.

This is conceptually different from repeatedly invoking a one-shot solver: the factorization is retained and its cost can be amortized across subsequent solves.

For complex matrices, transpose, conjugate, and adjoint solves represent different mathematical operators and should not be conflated.

Sources:

`https://docs.rs/faer/0.24.4/faer/linalg/solvers/`

`https://faer.veganb.tw/docs/dense-linalg/linsolve/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

## 4. Spectral decompositions: eigenvalues, singular values, and generalized eigenproblems

faer's spectral functionality extends beyond basic full-matrix eigenvalue computation. It supports dense decomposition, partial eigenvalue extraction, generalized eigenproblems, and matrix-free spectral algorithms.

### 4.1 Singular value decomposition

For a general rectangular matrix,

\\[ A=U\Sigma V^H. \\]

faer provides three important computational choices:

| Operation            | API                      | Capability                                           |
| -------------------- | ------------------------ | ---------------------------------------------------- |
| Full SVD             | `Mat::svd()`             | Full left and right singular-vector bases            |
| Thin SVD             | `Mat::thin_svd()`        | Reduced singular-vector bases                        |
| Singular values only | `Mat::singular_values()` | Singular spectrum without materializing vector bases |

The singular values are nonnegative and returned in nonincreasing order.

For an \\(m\times n\\) matrix, thin SVD retains only \\(\min(m,n)\\) columns of the singular-vector matrices. This can substantially reduce storage for highly rectangular matrices.

Beyond solving least-squares systems, SVD supports numerical rank analysis, null-space investigation, pseudoinverse computation, and diagnostics of ill-conditioning.

The low-level `linalg::svd` module provides `ComputeSvdVectors`, `SvdParams`, explicit scratch requirements, and pseudoinverse construction with tolerance control. That tolerance determines which small singular values are treated as numerically zero—an important distinction between formal algebraic rank and effective numerical rank.

A significant design choice is whether the downstream operation needs the vectors, just the singular values, or only a limited set of dominant singular triplets. faer supplies different mechanisms for all three cases.

Source: `https://docs.rs/faer/0.24.4/faer/linalg/svd/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.2 Self-adjoint and general eigendecomposition

For a Hermitian matrix,

\\[ A=U\Lambda U^H, \\]

with real eigenvalues and orthonormal eigenvectors.

The corresponding high-level APIs are `self_adjoint_eigen()` and `self_adjoint_eigenvalues()`. The self-adjoint eigenvalues are returned in ascending order.

For a general square matrix, `eigen()` and `eigenvalues()` support complex eigenvalues, even when the input matrix contains only real numbers.

Important distinctions:

- General non-Hermitian eigenvalues have no guaranteed ordering in the documented API.
- The Hermitian solver can exploit stronger mathematical structure and produce orthogonal eigenvectors.
- General eigenvectors need not form a well-conditioned basis; defective and nearly defective matrices require particular care.
- The low-level `linalg::evd` API exposes Schur and Hessenberg-related mechanisms, giving more control than a single high-level `eigen()` call.

For real symmetric and complex Hermitian problems, the specialized self-adjoint path should be distinguished from the general eigensolver rather than treating both as interchangeable.

Source: `https://docs.rs/faer/0.24.4/faer/linalg/evd/`

### 4.3 Generalized eigendecomposition and QZ algorithms

faer 0.23 introduced generalized eigendecomposition, retained in 0.24.4.

This addresses matrix pencils:

\\[ Av=\lambda Bv. \\]

Such problems occur in stability analysis, constrained dynamical systems, modal analysis, and generalized spectral formulations.

The API offers both high-level `GeneralizedEigen` objects and low-level `linalg::gevd` functions. The latter include generalized Hessenberg and real/complex QZ-related implementations.

A particularly important numerical detail is that generalized eigenvalues are represented as

\\[ \lambda=\frac{\alpha}{\beta}. \\]

Retaining the numerator and denominator separately avoids prematurely forming ratios that may overflow or lose numerical information. It also matters for singular matrix pencils, where infinite generalized eigenvalues can arise.

`GeneralizedEigen::new` and `new_from_real` construct decompositions for complex and real matrix pairs, respectively; `U()`, `S_a()`, and `S_b()` expose the generalized eigenvector and numerator/denominator factors.

The low-level API can separately request left and right generalized eigenvectors.

The generalized eigensolver should not be interpreted as a guarantee that every matrix pencil has a complete, well-conditioned eigenvector basis. Its behavior and errors remain subject to the mathematical properties of the pencil and numerical convergence.

Sources:

`https://docs.rs/faer/0.24.4/faer/linalg/gevd/`

`https://docs.rs/faer/0.24.4/faer/linalg/solvers/struct.GeneralizedEigen.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

## 5. Sparse matrix infrastructure and direct solvers

Sparse functionality is one of faer's most substantial capability families.

The important conceptual distinction is between a sparse matrix's structural pattern and its numerical values. These can be processed, stored, and reused independently.

### 5.1 CSC, CSR, and structural representations

faer provides compressed sparse column (CSC) and compressed sparse row (CSR) representations, including owning and borrowed types.

For CSC, a matrix is represented through column pointers, row indices, and corresponding values. For CSR, the analogous representation uses row pointers and column indices.

The library distinguishes symbolic representations from numerical matrices:

| Representation                                  | Purpose                                                       |
| ----------------------------------------------- | ------------------------------------------------------------- |
| `SymbolicSparseColMat` / `SymbolicSparseRowMat` | Dimensions and index structure without numerical coefficients |
| `SparseColMat` / `SparseRowMat`                 | Sparse matrix with numerical values                           |
| Corresponding `Ref`/`Mut` forms                 | Borrowed access to the structure and/or coefficients          |
| `Triplet` / `Pair`                              | Coordinate-oriented construction inputs                       |

Most sparse factorization algorithms operate on CSC representations, although CSR is supported elsewhere, including mixed sparse/dense multiplication.

The library also distinguishes compressed and uncompressed representations. Uncompressed storage permits spare capacity within columns or rows, but it is not equivalent to arbitrary structural mutation through every borrowed view.

Source: `https://docs.rs/faer/0.24.4/faer/sparse/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.2 Sparse construction, duplicates, and structure preservation

`SparseColMat::try_new_from_triplets` and its CSR counterpart accept coordinate data, sort the indices, and consolidate entries with duplicate coordinates by addition.

For example, two triplets targeting the same matrix position with values 3 and 5 produce a coefficient of 8.

That behavior is useful for assembling global matrices from contributions associated with different elements or subsystems. It is not interchangeable with last-write-wins semantics or preservation of individual source entries.

Other construction and structural mechanisms include:

- `try_new_from_nonnegative_triplets`, which skips negative-index entries.
- Symbolic constructors that separate index processing from value assignment.
- Checked and unchecked construction interfaces for externally prepared structures.
- Options for avoiding sorting when input ordering and algorithm requirements permit it.
- Fallible allocation and construction, rather than assuming arbitrarily large sparse structures can always be allocated.

Structural zero and numerical zero are different concepts. A coefficient can become numerically zero while remaining present in the stored sparsity pattern. Conversely, adding a new structural position may invalidate symbolic analysis that was based on the previous pattern.

This matters for repeated numerical assembly and solver reuse.

Source: `https://faer.veganb.tw/docs/sparse-linalg/creating-matrix/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://faer.veganb.tw\&sz=32)

faer



### 5.3 Sparse arithmetic and symbolic multiplication

faer supports sparse arithmetic, sparse/dense products, triangular operations, and sparse/sparse multiplication.

For sparse matrix multiplication, it exposes the distinction between:

1. Symbolic multiplication: Determine the possible nonzero structure of the product.
2. Numeric multiplication: Calculate the coefficients into the resulting structure.

Relevant functions include:

- `sparse_sparse_matmul_symbolic`
- `sparse_sparse_matmul_numeric`
- `sparse_sparse_matmul_numeric_scratch`
- `sparse_dense_matmul`
- `dense_sparse_matmul`

`SparseMatMulInfo` provides information used for workload partitioning.

This separation permits the product structure to be reused when the sparsity patterns remain fixed, even if the input values change.

The sparse operations module additionally includes operations into preallocated destinations, such as `binary_op_assign_into` and `ternary_op_assign_into`.

A caveat is that the algebraic product's structural pattern represents possible nonzeros. Numerical cancellation can produce zero-valued entries without changing that previously determined pattern.

Sources:

`https://docs.rs/faer/0.24.4/faer/sparse/linalg/matmul/`

`https://faer.veganb.tw/docs/sparse-linalg/mat-ops/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 5.4 Sparse Cholesky, LU, and QR

The high-level sparse solver family includes:

| Factorization | High-level entry point                      | Problem family                             |
| ------------- | ------------------------------------------- | ------------------------------------------ |
| Sparse LLᴴ    | `sp_cholesky(Side::Lower)` or `Side::Upper` | Hermitian positive-definite square systems |
| Sparse LU     | `sp_lu()`                                   | General square systems                     |
| Sparse QR     | `sp_qr()`                                   | Rectangular least-squares systems          |

The associated reusable high-level objects reside in `faer::sparse::linalg::solvers`.

faer implements two major approaches to sparse factorization:

Simplicial factorization processes sparse elimination using relatively fine-grained operations. It can work well when the sparsity pattern does not contain sufficiently large dense blocks to justify blocked arithmetic.

Supernodal factorization groups related columns or elimination operations into denser blocks, enabling more efficient use of dense linear algebra kernels and cache locality.

The implementation can select between these approaches according to sparsity characteristics, while lower-level parameter types expose selection and tuning mechanisms.

Sparse triangular solves are also available directly, including variants for lower or upper triangular matrices and unit diagonals.

Sparse factorizations do not necessarily preserve input sparsity: elimination can introduce fill-in. Consequently, sparse storage size alone is insufficient to predict factorization memory use or speed.

Sources:

`https://faer.veganb.tw/docs/sparse-linalg/linsolve/`

`https://docs.rs/faer/0.24.4/faer/sparse/linalg/`

[image](https://www.google.com/s2/favicons?domain=https://faer.veganb.tw\&sz=32)

faer+1

### 5.5 Symbolic analysis versus numerical factorization

This is one of faer's most valuable advanced features.

Sparse direct solving contains two conceptually distinct operations:

- Symbolic analysis: Inspect the graph/sparsity pattern, determine elimination structure and permutations, estimate and organize factor storage.
- Numerical factorization: Use the actual matrix coefficients to construct numerical factors.

faer exposes reusable symbolic solver types:

- `SymbolicLlt`
- `SymbolicLu`
- `SymbolicQr`

Numerical objects include `Llt`, `Lu`, and `Qr`. APIs such as `Lu::try_new_with_symbolic` accept previously computed symbolic information and a matrix with the corresponding sparsity pattern.

A repeated linear system can therefore follow this conceptual execution model:

Fixed sparse structure

Dimensions, nonzero coordinates, ordering

Symbolic analysis — once per compatible pattern

Ordering, elimination structure, factor allocation planning

For each new coefficient set

Update values

Numerical factorization

Solve RHS

This arrangement can avoid repeating structural analysis when coefficients change but the pattern does not.

It can be particularly valuable for sequences of linearized systems, repeated optimization subproblems, and parameterized simulations.

The constraint is exact: reuse of a symbolic factorization requires a compatible sparsity pattern. A changed pattern is not made compatible merely because the numerical matrices have the same dimensions.

The numerical factors must also be recomputed when their underlying coefficients change. Reusing symbolic analysis is not the same as reusing an already computed numeric factorization.

The symbolic types are reference-counted, making their underlying structural information inexpensive to share relative to recomputing it.

Source:

`https://docs.rs/faer/0.24.4/faer/sparse/linalg/solvers/`

`https://docs.rs/faer/0.24.4/faer/sparse/linalg/solvers/struct.SymbolicLlt.html`

`https://docs.rs/faer/0.24.4/faer/sparse/linalg/solvers/struct.Lu.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2

### 5.6 Sparse ordering and fill reduction

The sparse linear algebra modules include:

- `amd`: Approximate minimum degree ordering.
- `colamd`: Column approximate minimum degree ordering.
- Sparse Cholesky symbolic and numerical implementations.
- Sparse LU and QR symbolic and numerical implementations.

These ordering algorithms attempt to reduce fill or improve factorization structure.

Their effects depend on the sparsity graph. A fill-reducing ordering for a symmetric positive-definite Cholesky problem is not necessarily the right ordering for a nonsymmetric LU problem, and ordering cost itself becomes relevant for small or frequently changing patterns.

For sparse LU, numerical pivoting can also affect the eventual factorization structure. Symbolic analysis does not eliminate all numerical uncertainty associated with pivoting, stability, or rank deficiency.

Source: `https://docs.rs/faer/0.24.4/faer/sparse/linalg/`

## 6. Matrix-free operators and iterative linear solvers

A major architectural distinction is that faer can operate on linear operators without requiring explicit matrix storage.

A matrix-free operator supplies methods that compute \\(Ax\\), and, where necessary, \\(A^Tx\\) or \\(A^Hx\\).

This allows the same numerical algorithms to work with a dense matrix, a sparse matrix, or an application-specific operator.

### 6.1 Operator and preconditioner traits

| Trait       | Responsibility                                                              |
| ----------- | --------------------------------------------------------------------------- |
| `LinOp`     | Matrix dimensions and application of an operator to vectors or dense blocks |
| `BiLinOp`   | Additional transpose/adjoint application                                    |
| `Precond`   | Preconditioner application, including in-place use                          |
| `BiPrecond` | Preconditioner with transpose/adjoint-related functionality                 |

`LinOp` includes interfaces such as `apply`, `conj_apply`, and `apply_scratch`.

`BiLinOp` adds the transpose/adjoint operations required by algorithms that work with both directions of a linear map.

These interfaces incorporate the same parallelism and memory-workspace mechanisms used elsewhere in faer.

Notably, this is not limited to custom operators: native dense and sparse matrix types also implement relevant operator traits.

Consequently, a solver expressed against `LinOp` or `BiLinOp` need not depend on the representation used to supply its matrix-vector products.

Sources:

`https://docs.rs/faer/0.24.4/faer/matrix_free/`

`https://docs.rs/faer/0.24.4/faer/matrix_free/trait.LinOp.html`

`https://docs.rs/faer/0.24.4/faer/matrix_free/trait.BiLinOp.html`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 6.2 Native iterative algorithms

faer 0.24.4 includes three principal matrix-free linear solver families:

| Algorithm               | Mathematical target                                   | Significant considerations                                                           |
| ----------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------ |
| Conjugate gradient (CG) | Hermitian positive-definite systems                   | Efficient for appropriate positive-definite operators; sensitive to conditioning     |
| BiCGSTAB                | General nonsymmetric square systems                   | Handles nonsymmetric operators; convergence and breakdown can be more complicated    |
| LSMR                    | Least-squares systems, including rectangular problems | Requires operator and adjoint products; useful without constructing normal equations |

These algorithms support numerical parameters such as tolerance, maximum iteration count, initial-guess behavior, and status/error reporting.

For example, `CgParams` exposes absolute and relative tolerances, a maximum iteration count, and initial-guess configuration.

The operator interfaces also allow externally supplied preconditioners. faer provides `IdentityPrecond` as a no-op implementation; specialized preconditioning algorithms can be supplied through the corresponding traits.

Matrix-free does not necessarily mean faster. It avoids forming or factoring a matrix, but each iteration performs operator products and may require many iterations. Its suitability depends on operator cost, conditioning, preconditioning quality, target accuracy, and the availability of exploitable matrix structure.

Sources:

`https://docs.rs/faer/0.24.4/faer/matrix_free/conjugate_gradient/`

`https://docs.rs/faer/0.24.4/faer/matrix_free/bicgstab/`

`https://docs.rs/faer/0.24.4/faer/matrix_free/lsmr/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

### 6.3 Matrix-free Krylov–Schur eigenvalue algorithms

faer's `matrix_free::eigen` module implements Krylov–Schur-family methods for partial spectral computation.

The released APIs include:

- `partial_eigen`
- `partial_self_adjoint_eigen`
- `partial_svd`

Each has a corresponding `_scratch` function for estimating workspace requirements.

These routines seek eigenvalues or singular values of largest magnitude, along with corresponding vectors. Unlike a full dense eigendecomposition, the algorithm can target a limited number of dominant spectral components.

`PartialEigenParams` provides controls associated with subspace dimensions and restart limits.

This makes partial spectral computation an architectural alternative to materializing and decomposing an entire matrix when only a small portion of the spectrum is required.

However, these are iterative estimates. A call can terminate upon filling the requested outputs or exhausting its restart allowance; convergence and accuracy must be assessed through the returned information and the selected tolerances.

In particular, the documented routines target largest-magnitude spectral components. That should not be confused with a native shift-invert capability for arbitrary interior eigenvalues.

Source: `https://docs.rs/faer/0.24.4/faer/matrix_free/eigen/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 7. Computational performance architecture

faer's performance model is more involved than simply providing optimized implementations of familiar linear algebra operations.

It combines optimized machine-level kernels, numerical algorithm selection, parallel execution, matrix layout awareness, and explicit allocation control.

### 7.1 SIMD and optimized matrix multiplication

The library is implemented in Rust and does not require linking against an external BLAS or LAPACK implementation for its normal computation paths.

Its numerical infrastructure uses several specialized Rust dependencies:

| Component           | Role                                                               |
| ------------------- | ------------------------------------------------------------------ |
| `gemm`              | General-purpose optimized matrix multiplication                    |
| `nano-gemm`         | Specialized smaller matrix multiplication workloads                |
| `private-gemm-x86`  | Optional accelerated x86 backend enabled through the `std` feature |
| `pulp`              | Portable SIMD abstractions and architecture-specific vectorization |
| `rayon` / `spindle` | Multithreaded computation and work scheduling                      |
| `dyn-stack`         | Explicit temporary workspace requirements and allocation           |
| `faer-traits`       | Numerical scalar representations and operations                    |

faer uses optimized SIMD approaches for x86-64 and AArch64, with architecture-specific behavior determined by the available backend and build configuration.

The `std` feature permits runtime CPU feature detection. The `nightly` feature enables experimental SIMD facilities, including x86-v4-related support.

Complex scalar processing is more sophisticated than simply passing arbitrary Rust structures into conventional scalar arithmetic. The library's `ComplexField` infrastructure provides representations and operations suitable for SIMD implementations, including generic handling of composite scalar values.

The consequence is that matrix size, scalar type, platform, instruction set, and physical memory layout can all influence which execution path is most effective.

The existence of these optimized kernels does not establish that faer is always faster than MKL, OpenBLAS, BLIS, or another implementation. Those comparisons depend on the particular operation and benchmark conditions.

Sources:

`https://docs.rs/crate/faer/0.24.4`

`https://faer.veganb.tw/docs/contributing/simd-in-faer/`

`https://github.com/sarah-quinones/faer-rs/blob/main/paper.md`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2

### 7.2 Parallelism as an explicit execution parameter

faer exposes the `Par` enum:

- `Par::Seq`: Sequential execution.
- `Par::Rayon(NonZeroUsize)`: Execution using the Rayon parallel backend.
- `Par::rayon(nthreads)`: Convenient construction of a Rayon configuration.

The higher-level APIs also support global parallelism configuration.

This enables numerical callers to coordinate the library's internal parallelism with parallel work occurring elsewhere in a program.

For example, invoking a multithreaded factorization from every task in an already heavily parallel outer loop can cause oversubscription and resource contention.

Conversely, a large isolated factorization may benefit from substantial internal parallelism.

The optimal configuration depends on the operation, matrix dimensions, workload concurrency, memory bandwidth, and CPU architecture.

Source: `https://docs.rs/faer/0.24.4/faer/enum.Par.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.3 Explicit scratch-space management

This is one of the most consequential low-level facilities.

Many faer algorithms need temporary arrays for factorization, transformations, workspace matrices, or intermediate vectors.

Rather than always allocate internally, low-level functions commonly accept a `MemStack` and provide a companion `_scratch` function that computes the required memory layout.

The typical mechanism is:

1. Determine workspace requirements with a function such as `svd_scratch`, `partial_eigen_scratch`, or a factorization-specific `_scratch` function.
2. Allocate a compatible buffer using `dyn_stack::MemBuffer`.
3. Wrap that buffer in `MemStack`.
4. Pass the workspace to the computation.
5. Reuse it for subsequent compatible operations.

The important utilities include:

| Utility                               | Role                                                                        |
| ------------------------------------- | --------------------------------------------------------------------------- |
| `StackReq`                            | Describes temporary storage requirements, including alignment               |
| `StackReq::all_of`                    | Combines requirements for memory that must coexist                          |
| `StackReq::any_of`                    | Combines alternative requirements whose storage can be reused               |
| `MemBuffer`                           | Owns a correctly allocated scratch buffer                                   |
| `MemStack`                            | Provides structured access to reusable scratch memory                       |
| `temp_mat_scratch`                    | Calculates temporary matrix storage requirements                            |
| `temp_mat_zeroed` / `temp_mat_uninit` | Constructs temporary matrix storage with different initialization semantics |

The distinction between `all_of` and `any_of` is particularly useful. The former expresses simultaneous storage requirements, while the latter can reuse the same allocation for operations whose temporary lifetimes do not overlap.

This provides a way to plan scratch capacity for an entire numerical sequence instead of repeatedly allocating temporary memory for each step.

However, scratch reuse does not automatically imply allocation-free execution. Outputs, persistent factors, representation conversions, and internally allocated high-level objects may still require memory.

Unsafe or uninitialized temporary storage also imposes stricter obligations than zero-initialized arrays. In particular, a caller must ensure that every value is initialized before it is read.

Source: `https://docs.rs/faer/0.24.4/faer/linalg/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.4 Avoiding work is often more important than accelerating kernels

faer's architecture supports several independent ways to reduce computational work:

| Mechanism                                    | Work potentially avoided                                     |
| -------------------------------------------- | ------------------------------------------------------------ |
| Reusing a factorization                      | Repeated matrix decomposition for new right-hand sides       |
| Reusing sparse symbolic analysis             | Repeated ordering and elimination-structure analysis         |
| Retaining matrix views                       | Copying or rearranging input buffers                         |
| In-place arithmetic                          | Allocating and writing intermediate matrices                 |
| Thin SVD                                     | Computing unnecessary singular-vector columns                |
| Eigenvalues-only calculation                 | Computing unneeded eigenvectors                              |
| Partial spectral methods                     | Performing a full decomposition when only a subset is needed |
| Matrix-free iteration                        | Explicitly assembling or factoring a large matrix            |
| Reusing scratch storage                      | Repeated temporary allocation                                |
| Exploiting Hermitian or triangular structure | Performing work required only for unstructured matrices      |

These mechanisms are not universally composable. For example, a matrix-free operator may avoid assembly but eliminate the opportunity to reuse a sparse numeric factorization; a sparse direct solver may outperform an iterative method when many right-hand sides reuse the same factors.

The significant architectural choice is therefore not merely which kernel has the highest throughput. It is which representation and algorithm combination performs the least necessary work while meeting numerical accuracy and reliability requirements.

## 8. Scalar types, specialized operations, and supporting functionality

### 8.1 Real and complex scalar abstractions

The principal scalar families include:

- `f32` and `f64`.
- Complex equivalents such as `c32` and `c64`.
- Generic numerical operations through `RealField`, `ComplexField`, and related traits.
- Extended-precision scalar types, including `fx128` and its complex counterpart, introduced in the 0.22 release family.

The `faer-traits` companion crate provides much of the scalar abstraction infrastructure used internally.

The numerical traits make algorithms more general than a fixed set of floating-point operations. However, implementing a scalar trait does not establish that every decomposition, SIMD specialization, or accelerated backend supports every possible custom scalar with equivalent efficiency.

This is particularly relevant for composite, extended-precision, or application-defined numerical types.

Source:

`https://docs.rs/faer-traits/0.24.0/faer_traits/`

`https://github.com/sarah-quinones/faer-rs/blob/main/CHANGELOG.md`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

### 8.2 Structured matrix and permutation operations

Beyond dense and sparse storage, faer includes functionality for common structural operations:

Diagonal matrices: `Diag`, `DiagRef`, and `DiagMut` represent diagonal data without requiring a complete dense matrix. Diagonal arithmetic and related operations can exploit that representation.

Permutation matrices: The `perm` module provides owned and borrowed permutation representations, together with row and column permutation functions, including in-place variants and scratch requirements.

Kronecker products: `linalg::kron::kron` computes the Kronecker product of two matrices.

Triangular operations: Specialized solve and inverse routines handle triangular structure directly.

Householder and Jacobi transformations: The corresponding modules expose primitives used in QR, eigensolvers, and other numerical routines.

These are significant because a large numerical algorithm can often preserve matrix structure through several transformations instead of promoting every operation to a generic dense matrix computation.

A Kronecker product operation, however, should not be confused with a matrix-free Kronecker operator. Explicitly constructing the Kronecker product still incurs the corresponding output size.

Sources:

`https://docs.rs/faer/0.24.4/faer/perm/`

`https://docs.rs/faer/0.24.4/faer/linalg/kron/`

`https://docs.rs/faer/0.24.4/faer/linalg/triangular_solve/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

### 8.3 Statistics and random matrix functionality

The `stats` module provides more than basic random sampling.

It includes:

- Row and column means.
- Row and column variance calculations.
- Configurable `NanHandling` for mean and variance computations.
- Random matrix and vector distributions.
- Complex random distributions.
- `UnitaryMat`, for generating random unitary matrices.

These capabilities can support numerical test generation, statistical transformations, and some diagnostic tasks without introducing an entirely separate matrix utility layer.

`NanHandling` is especially consequential because the treatment of missing or non-finite observations changes the meaning of the resulting statistics.

This module is useful, but it is not a full statistical modeling or probabilistic programming framework.

Source: `https://docs.rs/faer/0.24.4/faer/stats/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 9. Build configuration and deployment characteristics

faer supports both ordinary Rust environments and more constrained configurations.

Its minimum supported Rust version for 0.24.4 is Rust 1.84.0.

The main feature flags are:

| Feature         | Availability and effect                                                                            |
| --------------- | -------------------------------------------------------------------------------------------------- |
| `std`           | Default. Standard library integration, runtime CPU detection, and optional accelerated x86 backend |
| `rayon`         | Default. Parallel execution backend                                                                |
| `linalg`        | Default through `sparse-linalg`. Core numerical decomposition functionality                        |
| `sparse`        | Default through `sparse-linalg`. Sparse matrix infrastructure                                      |
| `sparse-linalg` | Default. Enables both `linalg` and `sparse`                                                        |
| `rand`          | Default. Random distributions and related support                                                  |
| `npy`           | Default. NumPy `.npy` format integration; also enables `std`                                       |
| `perf-warn`     | Optional. Logging warnings about potentially inefficient layouts                                   |
| `nightly`       | Optional. Experimental SIMD functionality requiring nightly Rust                                   |
| `unstable`      | Optional feature gate; not a general stability guarantee                                           |

The project has documented `no_std` and non-Rayon configurations, but obtaining them requires attention to the complete feature graph rather than disabling a single flag in isolation.

For example, `npy` enables `std`, so retaining `npy` while disabling default features can still reintroduce a standard-library dependency.

A significant version-specific change is that Serde support was removed in 0.24. Serde appearing in older documentation, dependency trees, or development dependencies is not evidence that the corresponding public serialization feature still exists in 0.24.4.

Source:

`https://docs.rs/crate/faer/0.24.4/features`

`https://docs.rs/crate/faer/0.24.4`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2

## 10. Interoperability with other Rust and Python numerical libraries

faer's integration model distinguishes its core numerical implementation from companion packages that translate external matrix representations.

### 10.1 `faer-ext`

The principal official interoperability companion is `faer-ext` 0.8.0, released June 20, 2026, compatible with faer 0.24.

Its supported integration families include:

| External ecosystem | Integration mechanism                                         |
| ------------------ | ------------------------------------------------------------- |
| `nalgebra`         | Matrix-view conversions through `IntoFaer` and `IntoNalgebra` |
| `ndarray`          | Matrix-view conversions through `IntoFaer` and `IntoNdarray`  |
| NumPy / PyO3       | Conversion support for Python-exposed NumPy array views       |
| Polars             | Separate Polars-oriented conversion functionality             |

These are optional integrations in `faer-ext`, not automatic dependencies of the main `faer` crate.

The important restriction is that the documented zero-copy interoperability operates on borrowed matrix views, not arbitrary transfers of owned matrices.

faer's owning allocation scheme differs from those used by `nalgebra` and `ndarray`. Consequently, moving an owned matrix directly into another library's owning matrix type is not generally equivalent to borrowing the same memory through a view adapter.

The view-based approach can avoid data copies, but the external buffer's lifetime, stride interpretation, mutability, and numerical element representation still matter.

Sources:

`https://faer.veganb.tw/docs/dense-linalg/faer-ext/`

`https://docs.rs/faer-ext/0.8.0/faer_ext/`

`https://docs.rs/crate/faer-ext/0.8.0`

[image](https://www.google.com/s2/favicons?domain=https://faer.veganb.tw\&sz=32)

faer+2

### 10.2 Roles alongside `nalgebra` and `ndarray`

The distinction among the libraries is largely one of emphasis:

| Library    | Distinctive role                                                                                                             |
| ---------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `faer`     | High-performance matrix computation, advanced dense and sparse factorization, matrix-free solvers, explicit resource control |
| `nalgebra` | General linear algebra, statically and dynamically sized mathematical objects, geometry-oriented abstractions                |
| `ndarray`  | General multidimensional numerical arrays and array-oriented operations                                                      |
| `faer-ext` | Borrowed-view interoperability between faer and supported external representations                                           |

For systems that primarily operate on matrices, faer can provide an optimized computational layer.

For systems that already represent higher-dimensional datasets using `ndarray`, faer can provide specialized matrix operations without requiring the entire array model to be replaced.

For systems with many small fixed-dimension objects, `nalgebra` may remain better aligned with their mathematical representation.

None of these libraries completely subsumes the others.

### 10.3 Python integration and NumPy files

Python interoperability exists at two different boundaries.

In-process integration: `faer-ext` can work with NumPy/PyO3-compatible array views. The Python binding layer, type conversion, and Python-facing API still require appropriate Rust/Python integration code.

File-format integration: faer's `io::npy` module supports NumPy `.npy` matrix data. This provides an interchange mechanism that does not require an active Python interpreter.

The latter should not be mistaken for complete support for Python objects, NumPy's entire API, or every NumPy-associated storage format.

Source:

`https://docs.rs/faer/0.24.4/faer/io/`

`https://docs.rs/faer-ext/0.8.0/faer_ext/`

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1

## 11. Capability combinations and architectural options

The strongest architectural value emerges when faer's abstractions are combined according to the mathematical structure and lifecycle of the computation.

| Computational situation                    | Native composition                                                          | Architectural benefit                                          |
| ------------------------------------------ | --------------------------------------------------------------------------- | -------------------------------------------------------------- |
| Repeated dense systems with fixed \\(A\\)  | `Mat` view → LU/Cholesky factorization → repeated `Solve`                   | Avoid repeated factorization                                   |
| Repeated sparse systems with fixed pattern | Symbolic sparse matrix → `SymbolicLu`/`SymbolicLlt` → new numerical factors | Amortize structural analysis                                   |
| Large sparse SPD systems                   | CSC + sparse Cholesky, or `LinOp` + CG/preconditioner                       | Choose direct factor reuse or matrix-free iteration            |
| Rectangular least squares                  | QR/column-pivoted QR, sparse QR, LSMR, or SVD                               | Different memory, conditioning, and rank-information tradeoffs |
| Dominant spectral information              | `BiLinOp` + `partial_svd` or `partial_eigen`                                | Avoid full dense spectral decomposition                        |
| Repeated low-level kernels                 | `MatRef`/`MatMut` + `_scratch` + `MemStack`                                 | Reduce copying and temporary allocation                        |
| Mixed numerical libraries                  | External matrix views + `faer-ext`                                          | Preserve existing data representations where compatible        |

### 11.1 Direct versus iterative solvers

A sparse direct factorization can be expensive initially but inexpensive to reuse for many right-hand sides.

A matrix-free iterative solver can avoid the factorization entirely but incurs repeated operator applications and potentially substantial preconditioning work.

The distinction is particularly important for changing matrices:

- When only \\(b\\) changes, retaining numeric factors can be extremely valuable.
- When matrix values change but sparsity remains fixed, symbolic reuse can still save work.
- When matrix structure changes frequently, the benefit of symbolic caching decreases.
- When matrix-vector products are much cheaper than assembly and factorization, matrix-free methods become more attractive.
- When an effective preconditioner is unavailable, iterative convergence may be poor despite cheap operator products.

These are conditional computational tradeoffs, not universally ordered performance rankings.

### 11.2 QR, SVD, and iterative least squares

For a rectangular problem, different algorithms retain different information.

QR is generally an effective direct least-squares representation, especially for tall, full-column-rank matrices.

Column-pivoted QR adds rank-sensitive pivoting and can better handle nearly dependent columns.

SVD provides the richest information about small singular values, numerical rank, and pseudoinverse behavior, typically at greater computational expense.

LSMR avoids dense factorization and can operate on a matrix-free representation, but its convergence depends on conditioning and tolerance requirements.

In other words, least squares is not a single algorithmic capability: faer supports multiple formulations with fundamentally different resource and numerical properties.

### 11.3 Structured representations versus explicitly assembled matrices

faer's diagonal, triangular, sparse, permutation, and operator abstractions make it possible to retain structural meaning instead of converting everything to a generic matrix.

A user-defined `LinOp` can represent a composition of linear operations without explicitly constructing the resulting matrix.

This is particularly valuable when the composed operator is cheap to apply or when explicit construction would create much larger intermediate matrices.

However, the reviewed APIs do not constitute a general-purpose lazy tensor expression optimizer. Custom operator composition, execution planning, and automatic simplification remain responsibilities of the surrounding numerical abstraction layer unless provided by additional libraries.

## 12. Numerical correctness, limitations, and reliability considerations

### 12.1 Mathematical and numerical assumptions

A successful Rust API call does not by itself establish that the numerical result is meaningful.

The consequential distinctions include:

Definiteness: LLᴴ requires positive definiteness. A symmetric but indefinite matrix may require LBLᴴ or another appropriate algorithm.

Rank deficiency: LU, QR, sparse factorization, and SVD do not respond identically to deficient rank. Pivoting and explicit singular-value thresholds can materially change the available results.

Ill-conditioning: A small residual

\\[ \\|Ax-b\\| \\]

does not necessarily imply a small forward error in \\(x\\) when \\(A\\) is ill-conditioned.

Approximate eigenvalues: Partial Krylov–Schur output is an iterative estimate, not a guarantee that every requested spectral component converges.

Sparse numerical stability: Fill-reducing permutations, structural analysis, and numerical pivot selection play different roles. A favorable elimination ordering does not itself guarantee stability.

NaNs and infinities: Matrix operation semantics, error handling, and convergence behavior depend on the algorithm. The library should not be treated as providing universal non-finite-input validation.

### 12.2 Error and memory behavior

Dense constructors, some dimension-sensitive operations, and low-level APIs can panic or require unsafe preconditions.

Numerical operations also expose typed errors, including decomposition errors, sparse allocation and construction failures, and iterative solver failures.

Sparse operations are deliberately designed to return recoverable errors for some resource-related failures.

This distinction matters for numerical infrastructure: an operation may fail because the matrix is singular, because the requested decomposition is mathematically inappropriate, because numerical convergence was not achieved, or because storage could not be allocated. These are different failure classes and should not be collapsed into one generic failure condition.

### 12.3 Reported open correctness issues in 0.24.4

A review of upstream issue reports identified several material cases affecting the targeted release.

These are reporter-submitted, issue-specific reproductions, not independently reproduced results from this research. They were listed as open in the GitHub mirror when reviewed; the canonical Codeberg tracker should be checked for subsequent resolutions.

| Report                                       | Scope and potential impact                                                                                                                                                                             |
| -------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Parallel matrix multiplication, issue #273   | Reports incorrect numerical outputs with multithreaded x86 GEMM for particular matrix sizes and data patterns. Follow-up reports identify defects in the underlying `private-gemm-x86` execution path. |
| Full rectangular SVD, issue #268             | Reports that low-level full-vector SVD can leave null-space columns of the larger singular-vector matrix unwritten.                                                                                    |
| Sparse simplicial QR, issue #275             | Reproduces an out-of-bounds panic in numeric factorization for a rank-deficient matrix with structurally identical columns.                                                                            |
| Partial self-adjoint eigensolver, issue #262 | Reports a pathological case in shift-invert use where the solver hangs rather than terminating.                                                                                                        |

The first issue is particularly important because it involves silent numerical corruption rather than an explicit failure. The reporter observed correct results on the sequential path for their reproduced inputs; this is not a general guarantee that sequential execution eliminates unrelated numerical defects.

The full-SVD report is similarly specific: it concerns the low-level full-vector output path for rectangular matrices. It does not establish that every SVD operation is incorrect.

Direct reports:

- `https://github.com/sarah-quinones/faer-rs/issues/273`
- `https://github.com/sarah-quinones/faer-rs/issues/268`
- `https://github.com/sarah-quinones/faer-rs/issues/275`
- `https://github.com/sarah-quinones/faer-rs/issues/262`

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+3

These are meaningful release-specific caveats for correctness-sensitive applications, especially when using the affected numerical paths. They do not negate faer's broader capabilities or establish that unrelated algorithms are defective.

### 12.4 Scope boundaries

The reviewed faer 0.24.4 APIs are principally designed for CPU-based numerical linear algebra.

The library is not presented as a native replacement for:

- GPU tensor execution frameworks.
- Distributed matrix-processing systems.
- Automatic-differentiation frameworks.
- Full nonlinear optimization systems.
- Domain-specific preconditioning and multiphysics solvers.

Those capabilities can be built around faer or supplied by other libraries, but their presence should not be inferred from faer's matrix and factorization functionality.

Similarly, while faer provides reusable linear operator interfaces, it does not automatically choose an optimal mathematical formulation for an arbitrary problem.

The surrounding system is still responsible for expressing the problem, selecting an appropriate solver family, and interpreting the results.

## 13. Primary-source reference map

The following sources provide the most useful entry points for further investigation. URLs are plain text for direct reuse.

| Area                                    | Source                                                             |
| --------------------------------------- | ------------------------------------------------------------------ |
| Versioned complete API                  | `https://docs.rs/faer/0.24.4/faer/`                                |
| Crate version, dependencies, MSRV       | `https://docs.rs/crate/faer/0.24.4`                                |
| Cargo feature configuration             | `https://docs.rs/crate/faer/0.24.4/features`                       |
| Official capability documentation       | `https://faer.veganb.tw/`                                          |
| Dense numerical algorithms              | `https://docs.rs/faer/0.24.4/faer/linalg/`                         |
| High-level decomposition objects        | `https://docs.rs/faer/0.24.4/faer/linalg/solvers/`                 |
| Sparse matrix structures                | `https://docs.rs/faer/0.24.4/faer/sparse/`                         |
| Sparse algorithms                       | `https://docs.rs/faer/0.24.4/faer/sparse/linalg/`                  |
| Reusable sparse solver objects          | `https://docs.rs/faer/0.24.4/faer/sparse/linalg/solvers/`          |
| Matrix-free solvers and operator traits | `https://docs.rs/faer/0.24.4/faer/matrix_free/`                    |
| Krylov–Schur partial spectral methods   | `https://docs.rs/faer/0.24.4/faer/matrix_free/eigen/`              |
| Generalized eigenvalue decomposition    | `https://docs.rs/faer/0.24.4/faer/linalg/gevd/`                    |
| Interoperability companion              | `https://docs.rs/faer-ext/0.8.0/faer_ext/`                         |
| Interoperability examples               | `https://faer.veganb.tw/docs/dense-linalg/faer-ext/`               |
| SIMD implementation concepts            | `https://faer.veganb.tw/docs/contributing/simd-in-faer/`           |
| Release history and feature evolution   | `https://github.com/sarah-quinones/faer-rs/blob/main/CHANGELOG.md` |
| Canonical source repository             | `https://codeberg.org/sarah-quinones/faer`                         |
| GitHub mirror and issue reports         | `https://github.com/sarah-quinones/faer-rs`                        |

## 14. Overall assessment

faer's most distinctive contribution to the Rust numerical ecosystem is that the same library spans high-level matrix algebra, advanced dense and sparse decompositions, reusable symbolic factorization, matrix-free algorithms, and low-level execution control.

The resulting capability spectrum is unusually broad:

- At the highest level, it offers convenient operations through matrices and reusable solver objects.
- At the structural level, it distinguishes symbolic sparsity, numerical coefficients, permutations, and factorization artifacts.
- At the algorithmic level, it supports direct, iterative, dense, sparse, full-spectrum, and partial-spectrum methods.
- At the execution level, it exposes SIMD-oriented representations, parallelism controls, borrowed views, and reusable scratch workspaces.
- At the integration boundary, `faer-ext` permits existing numerical representations to participate through borrowed views rather than requiring wholesale ownership conversions.

The architectural opportunity is to express numerical computation through these native abstractions and preserve useful structure across operations. This can replace substantial bespoke machinery for matrix storage, numerical kernels, factorization lifecycle management, iterative solving, and memory management.

The main qualifications are equally important: mathematical assumptions remain decisive, low-level APIs require careful resource and initialization discipline, and the identified correctness reports warrant attention for affected 0.24.4 execution paths.

In scope and depth, faer is best understood not merely as a Rust matrix library, but as a substantial numerical linear algebra foundation whose abstractions span data representation, computational algorithms, and execution mechanics.

# faer — Advanced Capabilities Addendum

Rust | Target version: 0.24.4 | Research date: November 9, 2026

This addendum extends the existing faer — Comprehensive Rust Linear Algebra Library Overview, addressing functionality that was omitted or insufficiently developed. It focuses on advanced numerical mechanisms, their public APIs, architectural implications, and operational limitations without repeating the original reference.

All capabilities discussed are available in the published faer 0.24.4 API unless explicitly qualified. Version 0.24.4, released June 24, 2026, remains the latest stable release at the research date.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



## 1. Incremental maintenance of dense matrix factorizations

One of faer's more consequential low-level capabilities is the ability to modify an existing numerical factorization when the underlying matrix changes in certain structured ways.

This is fundamentally different from retaining a decomposition to solve multiple right-hand sides. Rather than treating every change to the matrix as requiring a fresh factorization, supported update algorithms operate directly on the stored factors.

### 1.1 Rank-\\(r\\) Cholesky and LDLᴴ updates

Suppose a Hermitian matrix \\(A\\) has already been factorized and receives an update of the form

\\[ A\_{\mathrm{new}}=A+W\operatorname{diag}(\alpha)W^H, \\]

where \\(W\in\mathbb{C}^{n\times r}\\), and \\(\alpha\\) contains the real weights associated with the \\(r\\) update vectors.

When \\(r\\) is small compared with \\(n\\), explicitly updating the factorization can avoid much of the work of performing a new dense decomposition.

Faer exposes two principal implementations:

| API                                                     | Factor representation | Behavior                                                             |
| ------------------------------------------------------- | --------------------- | -------------------------------------------------------------------- |
| `linalg::cholesky::llt::update::rank_r_update_clobber`  | \\(A=LL^H\\)          | Updates the Cholesky factor in place; returns `Result<(), LltError>` |
| `linalg::cholesky::ldlt::update::rank_r_update_clobber` | \\(A=LDL^H\\)         | Updates the LDL factor in place; returns `()`                        |

Both functions accept mutable views of the existing factorization, \\(W\\), and the diagonal update weights. Their implementations mutate the update operands as part of the calculation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



The distinction between positive and negative update weights is significant.

For positive-semidefinite additions, an initially positive-definite matrix remains positive definite. Negative-weight contributions are downdates and can destroy positive definiteness.

The LLᴴ implementation checks the resulting pivots and can return `LltError::NonPositivePivot`. It does not promise transactional rollback: earlier portions of the factorization and update inputs may already have been modified when the error occurs.

The LDLᴴ update routine has different failure semantics. Its lack of a `Result` does not establish numerical robustness for arbitrary indefinite or singular updates. In particular, the absence of a pivoting mechanism comparable to dense LBLᴴ should not be interpreted as a guarantee of stable symmetric-indefinite factorization.

The principal applicability conditions are:

- The change must be expressible as the supported low-rank Hermitian update.
- The existing factors must correspond to the expected numerical representation.
- Array dimensions and the diagonal update representation must be compatible.
- For LLᴴ, positive definiteness must remain satisfied.
- The caller must account for in-place modification and the possibility of numerical deterioration after repeated updates.

These mechanisms can be valuable in sequential least-squares computation, covariance estimation, active-set optimization, incremental inference, and other calculations involving structured matrix changes.

Their advantage depends on the update rank and frequency. A sequence of many updates can accumulate floating-point error; periodic reconstruction or direct refactorization may be warranted when numerical reliability is important.

Sources:

`https://docs.rs/faer/0.24.4/faer/linalg/cholesky/llt/update/fn.rank_r_update_clobber.html`

`https://docs.rs/faer/0.24.4/faer/linalg/cholesky/ldlt/update/fn.rank_r_update_clobber.html`

### 1.2 Deletion of rows and columns from LDLᴴ factors

Faer also supports removing selected rows and columns from an existing dense LDLᴴ factorization.

The relevant functions are:

- `linalg::cholesky::ldlt::update::delete_rows_and_cols_clobber`
- `linalg::cholesky::ldlt::update::delete_rows_and_cols_clobber_scratch`

The deletion algorithm maintains the factorization of the principal submatrix obtained by removing the selected row and column indices, instead of requiring the caller to reconstruct that submatrix and factorize it from scratch.

At the API level, the operation accepts mutable factor storage, a mutable slice of deletion indices, parallelism configuration, and scratch storage. The implementation sorts the supplied indices in place and asserts that the resulting indices are unique and within bounds.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



The numerical mechanism includes compaction of the stored triangular factor and a low-rank update to account for the elimination of deleted variables.

This distinction matters because deleting a variable from an existing factorization is not equivalent to deleting the corresponding entries from the factor itself. The remaining factors must still represent the correct principal submatrix.

The associated scratch function makes it possible to calculate and provide temporary memory explicitly, consistent with faer's broader workspace-management approach.

Potential applications include changing active sets, sliding-window matrix estimation, and models that repeatedly remove variables from a fixed-size numerical state.

API boundary: Version 0.24.4 exposes `insert_rows_and_cols_clobber_scratch`, but the reviewed public API does not expose a corresponding complete `insert_rows_and_cols_clobber` operation. The presence of a workspace-sizing function should not be interpreted as released support for arbitrary factor insertion.

These dense operations also do not establish support for incremental sparse-factor updates. Sparse structural changes can affect elimination trees, permutations, and fill patterns in ways not addressed by the dense update APIs.

Sources:

`https://docs.rs/faer/0.24.4/faer/linalg/cholesky/ldlt/update/`

`https://docs.rs/faer/0.24.4/faer/linalg/cholesky/ldlt/update/fn.delete_rows_and_cols_clobber.html`

## 2. Numerical regularization and configurable pivot selection

Faer's lower-level factorization interfaces expose numerical policy controls that go beyond selecting an algorithm such as Cholesky or LU.

These controls are important because numerical factorization is not simply a matter of whether a matrix has the right mathematical structure. Small pivots, expected inertia, and ill-conditioning can affect whether a decomposition is numerically useful.

### 2.1 Dynamic regularization of Cholesky-family factors

The dense LDLᴴ implementation exposes `LdltRegularization`, with the fields:

| Field                            | Function                                                      |
| -------------------------------- | ------------------------------------------------------------- |
| `dynamic_regularization_epsilon` | Threshold used to identify excessively small diagonal pivots  |
| `dynamic_regularization_delta`   | Magnitude used when regularizing a pivot                      |
| `dynamic_regularization_signs`   | Optional expected pivot signs at successive elimination steps |

The documented behavior is to replace pivots whose absolute values fall below the specified threshold, or whose signs violate the expected sign constraints, with appropriately signed regularized values.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This is particularly relevant for factorizations arising from constrained optimization, where the signs of diagonal pivots can carry mathematical meaning.

For example, an indefinite system associated with a constrained quadratic optimization problem may have an expected positive/negative inertia structure. A numerical algorithm can sometimes use this structural expectation to regularize unsuitable pivots.

Faer exposes an analogous `LltRegularization` type for LLᴴ, although positive-definite Cholesky and indefinite LDLᴴ do not have identical pivot requirements.

The factorization interfaces also expose information objects such as `LltInfo` and `LdltInfo`, including counts of dynamically regularized pivots. The sparse Cholesky implementation propagates these regularization counts through its numerical factorization routines.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



The central numerical distinction is that regularization modifies the factorization being computed.

A resulting factorization may more closely represent a perturbed matrix

\\[ A+\Delta A \\]

than the original \\(A\\).

The perturbation may improve numerical behavior while changing the solution of the original system. Therefore, regularization is not equivalent to proving that an indefinite or singular matrix was successfully factorized without modification.

It is especially important to distinguish a controlled perturbation from numerical error caused by floating-point arithmetic.

Sources:

`https://docs.rs/faer/0.24.4/faer/linalg/cholesky/ldlt/factor/struct.LdltRegularization.html`

`https://docs.rs/faer/0.24.4/faer/linalg/cholesky/llt/factor/struct.LltRegularization.html`

### 2.2 Pivoting strategies for dense indefinite factorization

The dense LBLᴴ implementation provides a configurable `PivotingStrategy` enum.

Its public variants are:

| Strategy      | Pivot-search behavior                                       |
| ------------- | ----------------------------------------------------------- |
| `Partial`     | Searches the current pivot column                           |
| `PartialDiag` | Also examines the remaining diagonal                        |
| `Rook`        | Searches for locally optimal pivots                         |
| `RookDiag`    | Extends rook pivot search to include the remaining diagonal |
| `Full`        | Searches globally for pivots                                |
| `Diagonal`    | Deprecated alias corresponding to partial pivoting          |

The enum is marked `#[non_exhaustive]`, allowing additional variants to be introduced without changing the type's compatibility contract.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



These strategies expose a tradeoff between the cost of searching for pivots and the numerical properties of the resulting factorization.

Partial pivoting generally requires less pivot-search work. More extensive pivot searches may improve the robustness of difficult indefinite factorizations but can increase computation and data movement.

`LbltParams` provides additional control over factorization behavior, including block-size and parallel-execution parameters.

These choices are particularly consequential for matrices that are symmetric or Hermitian but not positive definite. They also distinguish faer's indefinite decomposition implementation from an unpivoted LDLᴴ factorization, which does not provide equivalent stability properties for arbitrary indefinite matrices.

It should not be assumed that full pivoting is always preferable. Its additional work may offer little advantage for well-behaved systems, while a less expensive strategy can be adequate.

Source:

`https://docs.rs/faer/0.24.4/faer/linalg/cholesky/lblt/factor/enum.PivotingStrategy.html`

## 3. Sparse intranodal symmetric-indefinite factorization

The original guide describes sparse LLᴴ, LDLᴴ, LU, and QR, but faer also exposes a specialized sparse intranodal LBLᵀ factorization within its low-level Cholesky infrastructure.

This capability is notable because positive-definite sparse Cholesky and general sparse symmetric-indefinite factorization have different numerical requirements. An indefinite matrix can require 2×2 pivot blocks and permutations to avoid unsuitable scalar pivots.

### 3.1 `IntranodeLbltRef` and numerical factorization

The relevant public interfaces are:

| API                                                          | Responsibility                                                                               |
| ------------------------------------------------------------ | -------------------------------------------------------------------------------------------- |
| `SymbolicCholesky::factorize_numeric_intranode_lblt`         | Compute a sparse intranodal LBLᵀ factorization using existing symbolic analysis              |
| `SymbolicCholesky::factorize_numeric_intranode_lblt_scratch` | Determine numerical factorization workspace                                                  |
| `IntranodeLbltRef`                                           | Borrowed representation of the resulting numerical factors and associated symbolic structure |
| `IntranodeLbltRef::solve_in_place_with_conj`                 | Solve using the intranodal factors, with optional conjugation                                |

The factorization function accepts existing symbolic Cholesky analysis, the sparse numerical input, preallocated factor-value storage, subdiagonal storage, forward and inverse pivot permutations, parallelism configuration, workspace, and `LbltParams`. It returns an `IntranodeLbltRef`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This makes the functionality substantially more explicit than the ordinary high-level sparse solver interfaces. The caller is responsible for arranging storage and maintaining the lifetime relationship between symbolic analysis, numerical factor buffers, and the borrowed factorization object.

The `subdiag` storage and pivot permutations are particularly important. They permit a factorization representation more general than a triangular factor combined with a purely diagonal matrix.

### 3.2 Intranodal versus unrestricted pivoting

The essential qualification is intranodal.

Sparse supernodal factorization organizes related columns into blocks in order to exploit dense operations within those blocks. Restricting numerical pivoting to those local structures can help preserve the elimination organization established during symbolic analysis.

However, restricting pivot selection can also constrain the algorithm's ability to find numerically suitable pivots.

Consequently, this implementation should not be equated with an unrestricted sparse Bunch–Kaufman factorization or a general-purpose robust sparse symmetric-indefinite solver that can pivot freely throughout the matrix.

Its use is conditional on the matrix's numerical properties and the effectiveness of the available pivoting.

The API also differs from the high-level sparse factorization constructors in error semantics: `factorize_numeric_intranode_lblt` returns a factorization reference directly rather than a `Result`. That signature should not be interpreted as proof that every indefinite input is numerically admissible or that singularity and instability are automatically diagnosed.

Numerical validation may need to include residual calculations, finite-value checks, and examination of the behavior of solved systems.

### 3.3 Architectural significance

The intranodal LBLᵀ path allows existing sparse symbolic-analysis infrastructure to be used with a more general numerical factor representation.

This offers an additional native option for symmetric-indefinite sparse systems without necessarily discarding sparsity structure or resorting immediately to general nonsymmetric LU.

It also exposes a distinct tradeoff: preserving a planned sparse elimination structure can reduce structural work and facilitate blocked computation, but may impose limitations on pivoting flexibility and numerical robustness.

Sources:

`https://docs.rs/faer/latest/faer/sparse/linalg/cholesky/struct.IntranodeLbltRef.html`

`https://docs.rs/faer/latest/faer/sparse/linalg/cholesky/`

## 4. Advanced sparse symbolic-analysis and ordering controls

Faer's sparse symbolic APIs are not limited to automatically choosing an elimination ordering and constructing a factorization plan. They also expose controls that allow the surrounding numerical system to influence ordering, algorithm selection, and structural analysis.

### 4.1 Custom and identity orderings

For sparse Cholesky, `SymmetricOrdering` exposes three alternatives:

| Variant                  | Behavior                                         |
| ------------------------ | ------------------------------------------------ |
| `Amd`                    | Approximate minimum degree ordering; the default |
| `Identity`               | No reordering                                    |
| `Custom(PermRef<'_, I>)` | Use a caller-supplied permutation                |

This permits structural information obtained outside faer to influence the factorization.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



An externally calculated permutation can be useful when a matrix has a known elimination structure, when ordering decisions must be shared across multiple algorithms, or when reproducibility of structural choices matters.

`Identity` is particularly relevant when the matrix is already supplied in a favorable order. It avoids the ordering computation but can produce substantially more fill-in for other patterns.

The fact that faer accepts a custom ordering does not imply that it optimizes or validates that ordering for numerical efficiency. The supplied permutation's structural consequences remain the caller's responsibility.

### 4.2 Tuning symbolic Cholesky and LU

Faer's symbolic factorization parameter types expose the mechanisms underlying its automatic choices.

`CholeskySymbolicParams` contains:

- `amd_params`: parameters for the fill-reducing ordering calculation.
- `supernodal_flop_ratio_threshold`: threshold used to select a supernodal approach.
- `supernodal_params`: additional supernodal symbolic-analysis parameters.

`LuSymbolicParams` exposes analogous controls using `colamd_params` for its column-oriented fill-reducing ordering.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



These controls are consequential because sparsity alone does not determine whether simplicial or supernodal factorization is preferable.

A matrix with many nonzeros may nevertheless have an elimination structure that offers little opportunity for efficient blocked computation. Conversely, a relatively sparse matrix may contain useful dense substructures within its factors.

The expected benefits of supernodal processing depend on the amount of arithmetic that can be performed through dense kernels, the memory required for the factor structure, and the cost of organizing that structure.

The exposed thresholds provide a way to influence that tradeoff, rather than requiring applications to accept an entirely opaque algorithm-selection policy.

### 4.3 Elimination trees, column counts, and QR structural analysis

Faer's sparse QR module exposes several of the intermediate algorithms used to build a symbolic factorization:

| API family                         | Function                                                               |
| ---------------------------------- | ---------------------------------------------------------------------- |
| `col_etree`                        | Construct a column elimination tree                                    |
| `column_counts_ata`                | Compute structural column-count information associated with \\(A^HA\\) |
| `postorder`                        | Obtain a postordering of the elimination tree                          |
| `factorize_symbolic_qr`            | Construct the reusable symbolic QR representation                      |
| Corresponding `_scratch` functions | Calculate temporary storage requirements                               |

The column elimination tree of \\(A\\) corresponds to the elimination tree associated with the structure of \\(A^HA\\). This relationship is useful in sparse QR because it connects the column-intersection graph to the anticipated factorization structure.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This is symbolic analysis, not necessarily numerical formation of the normal equations. An algorithm may analyze the structure of \\(A^HA\\) while avoiding explicit computation of its coefficients and the attendant numerical conditioning disadvantages.

Exposing these intermediate primitives is useful for numerical infrastructure that needs to inspect or control elimination structures independently of complete factorization execution.

Source:

`https://docs.rs/faer/latest/faer/sparse/linalg/qr/`

### 4.4 Symbolic analysis as an independently controlled computational stage

These APIs support three distinct modes of use:

1. Let faer's high-level sparse factorization choose its ordering and symbolic strategy.
2. Use the lower-level symbolic interfaces with application-selected tuning parameters.
3. Supply compatible external structural information, such as an ordering permutation, and allow faer to carry out the remaining numerical factorization.

The latter approaches can reduce duplicate structural analysis or allow consistent ordering choices across an application's numerical operations.

They also increase responsibility for maintaining compatible permutations, dimensions, and factorization assumptions. A custom ordering is not a substitute for numerical pivoting, and a symbolic factorization cannot guarantee that the numerical matrix will have an acceptable rank or condition number.

Sources:

`https://docs.rs/faer/latest/faer/sparse/linalg/cholesky/enum.SymmetricOrdering.html`

`https://docs.rs/faer/latest/faer/sparse/linalg/cholesky/struct.CholeskySymbolicParams.html`

`https://docs.rs/faer/latest/faer/sparse/linalg/lu/struct.LuSymbolicParams.html`

## 5. Reusable sparse assembly mappings with `Argsort`

The original overview distinguishes symbolic sparse matrices from their numerical values. A more specialized feature is faer's ability to preserve the mapping between an original collection of coordinate entries and the resulting compressed sparse representation.

### 5.1 Separating coordinate processing from coefficient assembly

A typical sparse assembly process receives entries in coordinate form:

\\[ (i_k,j_k,v_k),\qquad k=1,\ldots,m. \\]

The input may be unsorted, and several entries may contribute to the same matrix coordinate.

Building a compressed sparse matrix requires determining a canonical storage organization, sorting coordinates as necessary, and reconciling duplicates.

Faer exposes this structural preparation through:

`SymbolicSparseColMat::try_new_from_indices`

The constructor returns:

```
Result<(SymbolicSparseColMat<I, Rows, Cols>, Argsort<I>), CreationError>
```

The resulting `Argsort<I>` records the ordering relationship needed to place numerical values into the symbolic representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



The corresponding numerical constructor is:

`SparseColMat::new_from_argsort`

It accepts the symbolic structure, the previously calculated `Argsort`, and a slice of numerical coefficients.

An analogous constructor is available for the CSR representation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.2 Why this is more than symbolic-factorization reuse

Consider a numerical calculation repeatedly producing new coefficients at the same input coordinates.

There are at least three potentially expensive preparation stages:

| Stage                      | Reusable artifact                       | Work avoided                                                                    |
| -------------------------- | --------------------------------------- | ------------------------------------------------------------------------------- |
| Coordinate organization    | `Argsort` and symbolic sparse structure | Repeated sorting and mapping from input contributions to compressed storage     |
| Solver structural analysis | Symbolic Cholesky, LU, or QR            | Repeated ordering and elimination-structure analysis                            |
| Numerical decomposition    | Existing factorization                  | Repeated factorization when the matrix coefficients themselves remain unchanged |

The first stage occurs before the factorization machinery. Reusing symbolic solver analysis does not automatically eliminate the cost of rebuilding the input sparse matrix from triplets.

`Argsort` addresses this earlier assembly cost.

For calculations that repeatedly regenerate coefficients at fixed source coordinates, the assembly mapping can be retained independently of solver symbolic analysis.

This can make the entire coefficient-update sequence more efficient, not merely the factorization step.

### 5.3 Duplicate coordinates and correspondence invariants

The important restriction is that the numerical values passed to `new_from_argsort` must correspond to the same original index sequence from which the `Argsort` was produced.

This requirement is stricter than matching matrix dimensions, or even supplying an equivalent set of nonzero coordinates in a different order.

If the source contributions are reordered, the stored mapping may no longer associate each coefficient with the intended matrix position.

Duplicate input coordinates introduce another important consideration: several source coefficients may need to be combined into one stored matrix entry.

The preparation mapping must preserve the relationship between those original contributions and the compressed storage.

This means the relevant invariant is the source-coordinate arrangement and its mapping to canonical matrix entries—not merely the final number of nonzeros.

Sources:

`https://docs.rs/faer/latest/faer/sparse/struct.Argsort.html`

`https://docs.rs/faer/latest/faer/sparse/type.SymbolicSparseColMat.html`

`https://docs.rs/faer/latest/faer/sparse/type.SparseColMat.html`

## 6. Sparse pattern algebra and structural transformations

Faer's sparse arithmetic infrastructure also provides capabilities for manipulating the structure of matrices separately from their numerical coefficients.

This is particularly important for operations where the output sparsity pattern can be established before the actual values are known.

### 6.1 Symbolic union of sparse patterns

The function `sparse::ops::union_symbolic` computes the union of two input sparsity patterns.

For patterns \\(S_A\\) and \\(S_B\\), the output pattern is:

\\[ S_C=S_A\cup S_B. \\]

The union contains positions that are structurally present in either input.

This operation can be useful when constructing the destination structure for a sequence of sparse elementwise operations. The result can accommodate contributions from both source patterns without initially requiring their numerical values.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



It is important not to confuse a structurally present entry with a numerically nonzero entry. An output position may remain allocated even if the contributions cancel exactly.

### 6.2 Custom binary and ternary sparse operations

The `sparse::ops` module includes:

| Function                    | Behavior                                                                       |
| --------------------------- | ------------------------------------------------------------------------------ |
| `binary_op`                 | Apply a caller-supplied operation to entries from two sparse matrices          |
| `binary_op_assign_into`     | Apply a binary operation while preserving the destination's symbolic structure |
| `ternary_op_assign_into`    | Apply a three-input operation into an existing destination structure           |
| `add_into` / `sub_into`     | Store arithmetic results in an existing destination                            |
| `add_assign` / `sub_assign` | Update the destination without changing its pattern                            |

These functions allow sparse coefficient transformations to be expressed without repeatedly converting matrices to dense storage or building generic coordinate-based arithmetic machinery.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



The structural semantics are essential.

For `binary_op`, positions absent from both inputs are skipped. Consequently, an operation that would map two implicit zeros to a nonzero value cannot generally be represented as a sparse result without additional handling.

For destination-oriented operations, the destination's symbolic structure is preserved.

This can avoid structural allocation and maintain compatibility with downstream algorithms expecting a fixed pattern. It also means that the destination pattern must be suitable for the intended output; the operation does not automatically expand it to represent arbitrary newly created entries.

Combining `union_symbolic` with destination-oriented coefficient operations provides a useful pattern: establish a sufficiently inclusive structure once, then update numerical coefficients without continually reallocating that structure.

### 6.3 Sparse permutations, sorting, and deduplication

The `sparse::utils` module exposes additional structural algorithms, including:

- `permute_self_adjoint`
- `permute_self_adjoint_to_unsorted`
- `permute_dedup_self_adjoint`
- `transpose` and `transpose_dedup`
- `sort_indices` and `sort_dedup_indices`

Several of these have companion scratch-requirement functions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



The self-adjoint permutation operations are especially relevant to factorization preparation:

\\[ A\_{\mathrm{new}}=PAP^T. \\]

For complex Hermitian matrices, the numerical meaning of a corresponding permutation is \\(PAP^H\\); because \\(P\\) is a real permutation matrix, the transpose and conjugate transpose coincide.

The deduplicating variants can help when structural transformations produce repeated coordinates that must be consolidated.

The unsorted variants are useful when an intermediate algorithm can tolerate unsorted indices and the sorting cost can be deferred or avoided.

The distinction between a simple transpose view and an explicitly computed, sorted or deduplicated sparse transformation is important. They do not necessarily have the same allocation, ownership, or structural invariants.

This functionality provides building blocks for numerical systems that need to maintain sparse structural consistency without repeatedly implementing coordinate sorting, deduplication, and permutation logic.

Sources:

`https://docs.rs/faer/latest/faer/sparse/ops/`

`https://docs.rs/faer/latest/faer/sparse/utils/`

## 7. Specialized self-adjoint pseudoinverse and tridiagonal spectral algorithms

The general SVD and EVD functionality is already covered in the main guide. Two lower-level spectral capabilities deserve separate treatment because they permit reuse of mathematical structure that is not explicit in the higher-level interfaces.

### 7.1 Pseudoinverse from a self-adjoint eigendecomposition

Faer exposes:

- `linalg::evd::pseudoinverse_from_self_adjoint_evd`
- `linalg::evd::pseudoinverse_from_self_adjoint_evd_with_tolerance`
- `linalg::evd::pseudoinverse_from_self_adjoint_evd_scratch`

These functions construct a pseudoinverse from previously computed self-adjoint eigenvalue and eigenvector factors.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



For a Hermitian matrix

\\[ A=U\Lambda U^H, \\]

the Moore–Penrose pseudoinverse can be expressed as

\\[ A^+=U\Lambda^+U^H, \\]

where

\\[ \lambda_i^+= \begin{cases} 1/\lambda_i,&\lambda_i\text{ is retained},\\\ 0,&\lambda_i\text{ is treated as numerically zero}. \end{cases} \\]

The tolerance-controlled implementation is important because small nonzero eigenvalues can produce very large reciprocals.

When the eigendecomposition is already available, the dedicated pseudoinverse function provides a native way to reuse its factors rather than computing an additional general SVD.

This is not automatically preferable to SVD for every rank-deficient problem. Its applicability depends on self-adjoint structure, the accuracy of the eigenvalues and eigenvectors, and the tolerance used to determine effective rank.

The distinction is particularly valuable for singular symmetric or Hermitian systems in which the eigendecomposition has already been calculated for other purposes.

Source:

`https://docs.rs/faer/latest/faer/linalg/evd/`

### 7.2 Self-adjoint tridiagonalization

Faer exposes `linalg::evd::tridiag::tridiag_in_place`, together with `tridiag_in_place_scratch`.

The operation reduces a self-adjoint matrix to tridiagonal form:

\\[ A=QTQ^H, \\]

where \\(T\\) is self-adjoint and tridiagonal, and \\(Q\\) is represented through Householder transformations.

The implementation stores the tridiagonal coefficients within the matrix and retains Householder information needed to represent the transformation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This provides access to a meaningful intermediate stage of self-adjoint eigendecomposition.

Algorithms that already operate on tridiagonal matrices need not first reconstruct and process an arbitrary dense matrix.

Faer additionally exposes `tridiagonal_self_adjoint_evd`, which computes the eigendecomposition of a matrix already in the appropriate tridiagonal form.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



These primitives can be useful when constructing custom spectral procedures, applying transformations to additional data, or integrating faer with numerical methods that naturally produce tridiagonal representations.

They should not be confused with matrix-free Krylov–Schur methods: the tridiagonal routines are low-level numerical building blocks, not a general interface for arbitrary implicit operators.

Sources:

`https://docs.rs/faer/latest/faer/linalg/evd/tridiag/`

`https://docs.rs/faer/latest/faer/linalg/evd/`

## 8. NumPy `.npy` interoperability: borrowing versus copying

The main overview identifies NumPy file-format interoperability, but the distinction between interpreting an existing buffer and creating an owned matrix deserves greater precision.

Faer's `io::npy` functionality exposes the types `Npy`, `NpyDType`, and `FromNpy` under the `npy` feature.

The module is documented as file-format deserialization, rather than as a general NumPy serialization framework.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 8.1 Memory-buffer interpretation

`Npy` represents a parsed `.npy` memory buffer.

Its API includes operations for inspecting the stored dtype, determining whether a compatible borrowed view can be created, and materializing the data into an owned faer matrix.

The important distinction is:

| Operation             | Storage behavior                                                                                           |
| --------------------- | ---------------------------------------------------------------------------------------------------------- |
| `Npy::as_aligned_ref` | Borrow matrix data directly from the parsed buffer when its alignment and dtype requirements are satisfied |
| `Npy::to_mat`         | Construct an owned `Mat<T>` from the encoded matrix data                                                   |

The borrowed-view operation can eliminate a data copy when the memory representation is compatible.

However, it does not transfer ownership. The matrix view remains dependent on the underlying memory buffer, which must remain valid for the duration of the borrow.

The owned conversion has different resource characteristics because it materializes a separate matrix.

The source implementation also distinguishes NumPy's C-order and Fortran-order storage conventions. These correspond to different stride interpretations rather than different mathematical matrices.

In particular, a compatible row-major buffer can be read through an appropriate view without implying that the underlying data has been rearranged into faer's preferred owned column-major layout.

### 8.2 Type and format boundaries

The `FromNpy` abstraction associates supported Rust scalar types with NumPy dtype identifiers.

The supported native numeric families include `f32`, `f64`, `c32`, and `c64`; an unsupported dtype is not automatically converted into an arbitrary faer scalar type.

A `.npy` array also does not imply support for the full range of NumPy metadata or arbitrary-dimensional array semantics.

The library's matrix representation is fundamentally two-dimensional, and the `.npy` functionality should be treated as a particular interchange path rather than as a replacement for `ndarray`, Python's NumPy implementation, or a general tensor-format system.

Performance qualification: The ability to borrow a matrix without copying it does not guarantee that all subsequent numerical algorithms will execute most efficiently on that layout. A later factorization may still benefit from or require conversion to a more suitable representation.

Sources:

`https://docs.rs/faer/latest/faer/io/`

`https://docs.rs/faer/latest/faer/io/npy/`

## 9. Native numerical comparison utilities and extended-precision representation

### 9.1 Approximate comparisons for numerical testing

Faer includes a numerical comparison utility module, `utils::approx`, with the types:

| Type                                              | Role                                                                    |
| ------------------------------------------------- | ----------------------------------------------------------------------- |
| `ApproxEq<T>`                                     | Scalar approximate-comparison configuration                             |
| `CwiseMat<Cmp>`                                   | Componentwise matrix comparison adapter                                 |
| `ApproxEqError`                                   | Scalar comparison error representation                                  |
| `CwiseMatError`, `CwiseColError`, `CwiseRowError` | Shape-sensitive error representations for matrix and vector comparisons |

`ApproxEq` exposes both absolute and relative tolerances. Its implementation uses the absolute difference between two values and compares that difference against the configured thresholds, including a relative threshold scaled by the larger operand magnitude.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Conceptually, a comparison is accepted when the values satisfy an absolute or relative error criterion:

\\[ |a-b|\leq\varepsilon\_{\mathrm{abs}} \quad\text{or}\quad |a-b|\leq\varepsilon\_{\mathrm{rel}}\max(|a|,|b|). \\]

The corresponding matrix adapter applies the configured scalar comparator componentwise, with explicit handling of dimension mismatches.

These mechanisms can reduce the amount of custom numerical comparison code needed for regression tests, decomposition checks, and validation of numerical transformations.

However, entrywise approximate equality is not equivalent to a rigorous numerical correctness assessment.

A componentwise comparison does not directly measure the backward error of a factorization, the residual of an eigenpair, or the conditioning of a linear system.

For example, validating a solution to \\(Ax=b\\) may require evaluating a normwise residual relative to the magnitudes of \\(A\\), \\(x\\), and \\(b\\), rather than testing whether individual coefficients are close to an expected result.

The native comparison utilities are therefore useful test primitives, not substitutes for mathematically appropriate numerical diagnostics.

Source:

`https://docs.rs/crate/faer/latest/source/src/utils/approx.rs`

### 9.2 `fx128` and `cx128` are composite scalar representations

The original overview identifies the extended-precision scalar types but does not describe how they are represented.

In faer 0.24.4, `fx128` is represented by a structure containing two `f64` components:

```
#[repr(C)]
pub struct fx128(pub f64, pub f64);
```

The complex extended-precision type `cx128` contains real and imaginary `Quad` components.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



These representations should not be confused with a native IEEE binary128 floating-point value simply because their type names contain `128`.

A composite representation using two `f64` values has different arithmetic, normalization, interoperability, and performance characteristics from a processor-native or compiler-supported binary128 type.

In particular, the storage width alone does not establish the number of effective significand bits or the exponent range of the arithmetic.

The practical consequences are:

- Extended precision is a property of the scalar arithmetic implementation, not merely the number of bytes stored.
- SIMD acceleration and optimized matrix kernels may have different performance characteristics for composite scalars.
- Binary serialization and foreign-function interfaces must respect the actual component representation.
- Numerical tolerances should reflect the scalar type's effective arithmetic properties rather than assume IEEE binary128 behavior.

These distinctions become particularly important when numerical applications mix standard floating-point types with extended-precision intermediates or attempt to interchange scalar data with external numerical libraries.

Sources:

`https://docs.rs/faer/latest/faer/type.fx128.html`

`https://docs.rs/faer/latest/faer/type.cx128.html`

## 10. Additional architectural implications

The capabilities documented here reveal several further ways that faer can serve as a numerical infrastructure library, beyond providing high-performance implementations of individual linear algebra algorithms.

### 10.1 Reuse can occur at multiple independent layers

For a sequence of related numerical problems, reusable work may exist at several distinct levels:

| Reuse level               | Native mechanism                                      | Condition for reuse                                      |
| ------------------------- | ----------------------------------------------------- | -------------------------------------------------------- |
| Input assembly            | `Argsort`, symbolic sparse storage                    | Same source-coordinate arrangement                       |
| Matrix algebra            | Precomputed sparse output structure, `union_symbolic` | Compatible operand and output patterns                   |
| Symbolic factorization    | `SymbolicCholesky`, symbolic LU/QR                    | Compatible factorization structure                       |
| Numerical factorization   | Retained factor object                                | Same numerical matrix                                    |
| Incremental factorization | Dense Cholesky/LDL update or deletion routines        | Change fits the supported factor-maintenance operation   |
| Execution workspace       | Scratch requirements and reusable buffers             | Compatible dimensions, parameters, and workspace demands |

These layers are neither interchangeable nor automatically managed by one common cache.

For example, new coefficients at unchanged source coordinates may permit both assembly-mapping reuse and symbolic-factorization reuse. They generally do not permit reuse of the old numerical factors without additional computation.

A low-rank change to a dense positive-definite matrix may instead permit direct numerical-factor modification, bypassing a complete refactorization.

The appropriate strategy depends on which properties of the problem remain invariant.

### 10.2 Greater control also increases numerical responsibility

Several of faer's advanced mechanisms exchange automatic behavior for explicit control.

Custom sparse permutations expose ordering decisions. Intranodal indefinite factorization preserves a constrained sparse elimination structure. Dynamic regularization alters numerical pivots. In-place factorization updates modify existing state rather than creating an entirely new decomposition.

These mechanisms can reduce computational overhead or expand the range of supported numerical formulations.

They also introduce responsibilities for maintaining structural invariants, interpreting perturbations, validating numerical outputs, and distinguishing mathematical failure from implementation or resource failure.

The important architectural opportunity is therefore not simply to expose more tuning parameters. It is to use faer's native structural and numerical mechanisms where the mathematical assumptions and invariants of the computation can be established and preserved.

Together, these additions show that faer's lower-level capabilities extend substantially beyond ordinary dense and sparse factorization: they provide native mechanisms for factor maintenance, constrained indefinite solving, structural preparation, symbolic control, sparse pattern algebra, and specialized numerical validation.