# ndarray — Comprehensive Rust Library Capability Reference

Target: ndarray 0.17.2 (stable, released January 10, 2026) | Research date: October 28, 2026 | Language: Rust | Minimum Rust version: 1.64

Primary sources

- Official API: `https://docs.rs/ndarray/0.17.2/ndarray/`
- Upstream repository: `https://github.com/rust-ndarray/ndarray`
- Release history: `https://github.com/rust-ndarray/ndarray/blob/master/RELEASES.md`

## 1. Architectural orientation

`ndarray` is Rust's general-purpose N-dimensional array abstraction, combining multidimensional storage, typed shapes, non-owning strided views, broadcasting, numerical operations, and efficient multidimensional iteration.

Its most important architectural property is the separation of element storage, array geometry, and computation. The same algorithms can operate over owned arrays, borrowed data, reference-counted arrays, sliced views, and nonstandard memory layouts without requiring every operation to materialize a new array.

The library is more than a matrix container. Its reusable abstractions support multidimensional data transformations, stencil calculations, numerical kernels, heterogeneous element types, memory-efficient processing, and higher-level scientific computing libraries.

### Core capability map

| Capability family          | Principal mechanisms                                                      | Architectural significance                                               |
| -------------------------- | ------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Storage and ownership      | `ArrayBase`, `Array`, `ArcArray`, `CowArray`, `ArrayView`, `ArrayViewMut` | Separate algorithms from allocation, borrowing, and sharing              |
| Generic array interfaces   | `ArrayRef`, `LayoutRef`, `RawRef`, storage traits                         | Express element access, metadata access, and unsafe access separately    |
| Dimensions and layouts     | `Ix0`–`Ix6`, `IxDyn`, `Dimension`, `ShapeBuilder`, strides                | Support static and runtime ranks, arbitrary layouts, and foreign buffers |
| Structural transformations | Slicing, indexing, axis permutations, reshaping, broadcasting             | Construct virtual transformations without moving data when compatible    |
| Multidimensional execution | `Zip`, `NdProducer`, `azip!`, axis iterators, chunks, windows             | Compose efficient traversal over different shapes of data and work       |
| Numerical operations       | Elementwise arithmetic, reductions, cumulative operations, `dot`, GEMM    | Provide general-purpose numerical kernels and transformations            |
| Parallel execution         | Rayon integration, parallel producers, parallel `Zip`                     | Divide array-oriented workloads without manual partitioning              |
| Interoperability           | Raw buffers, slices, Serde, BLAS, ecosystem crates                        | Connect external memory and specialized numerical engines                |

### Defining boundaries

`ndarray` is primarily an eager, CPU-oriented, strided-array library.

- It provides arrays and operations, not a lazy expression optimizer or execution planner.
- Broadcasting and views can avoid allocations, but ordinary arithmetic expressions are not automatically fused into a single kernel.
- It supports numerical types as well as arbitrary Rust element types; numerical methods impose their own trait requirements.
- Matrix multiplication is native, while comprehensive linear algebra factorizations, advanced statistics, random distributions, and file-format interoperability belong principally to companion libraries.
- Its Rust ownership model provides explicit control over borrowing and mutation, but views do not eliminate the need to reason about data lifetime, layout, and aliasing.

Why this architecture matters: `ndarray` can serve as a common multidimensional computation layer rather than requiring independent abstractions for arrays, slices, transposed matrices, broadcasted inputs, numerical reductions, and parallel traversal. Its most substantial advantages emerge when these capabilities are combined.

The sections below examine those combinations, including the important additions in 0.17.x.

## 2. Storage, ownership, and generic array interfaces

### 2.1 `ArrayBase`: the common representation

The underlying type is `ArrayBase<S, D>`, where:

- `S` describes the data representation and access capabilities, with the element type exposed through `RawData::Elem`.
- `D` describes dimensionality and shape representation.
- The array's shape, signed strides, and data pointer determine how logical indices map into storage.

This separation lets the library reuse multidimensional algorithms across different ownership modes, without treating each mode as an unrelated container.

| Public type                        | Ownership semantics               | Important characteristics                                          |
| ---------------------------------- | --------------------------------- | ------------------------------------------------------------------ |
| `Array<A, D>`                      | Unique, owned storage             | General-purpose mutable array; commonly backed by a `Vec`          |
| `ArcArray<A, D>`                   | Shared, reference-counted storage | Cheap clones and copy-on-write mutation when shared                |
| `CowArray<'a, A, D>`               | Borrowed or owned                 | Retains a borrowed view until ownership is needed                  |
| `ArrayView<'a, A, D>`              | Borrowed, immutable               | Lightweight, non-owning shape/stride descriptor                    |
| `ArrayViewMut<'a, A, D>`           | Borrowed, mutable                 | Mutable access constrained by Rust borrowing and non-overlap rules |
| `RawArrayView` / `RawArrayViewMut` | Raw pointer-based                 | Low-level interoperability; element dereferencing remains unsafe   |

Ownership distinctions that matter:

- `Array` versus `ArcArray`: an `ArcArray` clone shares its allocation; mutation of non-unique shared storage can trigger a full copy. Reference-counting avoids copying at sharing time, not necessarily at writing time.
- `CowArray` versus `ArcArray`: `CowArray` expresses borrowed-versus-owned data; `ArcArray` expresses shared ownership. They address different lifetime and sharing problems.
- `ArrayView` versus `CowArray`: a view cannot become independently owned without an explicit conversion; a `CowArray` can retain the borrowed representation or transition to owned data.
- `view()` versus `to_owned()`: constructing a view does not duplicate elements; `to_owned()` produces an independently owned array.
- `into_owned()`, `into_shared()`, `is_unique()`, and `try_into_owned_nocopy()`: support ownership transitions and inspection where copying costs matter.

An owned array is not necessarily contiguous in standard row-major order. Ownership and memory layout are independent properties.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 2.2 The major 0.17 development: `ArrayRef`, `LayoutRef`, and `RawRef`

Version 0.17 introduced reference abstractions that substantially improve generic library API design.

| Interface                              | Access contract                         | Appropriate use                                                   |
| -------------------------------------- | --------------------------------------- | ----------------------------------------------------------------- |
| `&ArrayRef<A, D>`                      | Read array elements and geometry        | Storage-independent numerical or transformation inputs            |
| `&mut ArrayRef<A, D>`                  | Mutate elements and applicable geometry | Generic in-place operations                                       |
| `AsRef<LayoutRef<A, D>>`               | Inspect shape and strides               | Shape validation, layout inspection, metadata-oriented algorithms |
| `AsMut<LayoutRef<A, D>>`               | Modify geometry                         | Structural operations not requiring element access                |
| `RawRef<A, D>` through `AsRef`/`AsMut` | Raw pointer access                      | Specialized low-level or unsafe implementations                   |

`ArrayRef` is the `Deref` target of owned, shared, and viewed array types. This is analogous to the relationship between `Vec<T>` and `[T]`: algorithms can accept a reference-like interface rather than introduce a generic storage parameter for every function.

Before 0.17, a commonly used generic interface was `&ArrayBase<S, D>` with `S: Data<Elem = A>`. That remains supported. The new interfaces offer a more focused way to express access requirements.

Several subtle behaviors deserve attention:

1. `ArrayRef` does not independently own shape and stride metadata. An `ArrayView` does. If an operation changes the geometry of a view, that does not inherently change the parent array's geometry.
2. Mutable references and shared ownership interact. Obtaining mutable element access from shared array representations must preserve unique mutation semantics; it can require copy-on-write preparation.
3. The interfaces introduced in 0.17.1 are unsized. Generic `AsRef`/`AsMut` bounds intended to accept references to these interfaces may require `+ ?Sized`.
4. `RawRef` is a separate capability boundary. Accessing raw pointers is not a substitute for the safety guarantees of `ArrayRef`.

These interfaces are particularly valuable for extension traits, reusable kernels, shape-checking utilities, and generic numerical libraries that should not depend on a caller's storage representation.

The original 0.17.0 release was yanked after a soundness issue involving these reference types. Version 0.17.1 corrected the representation, and 0.17.2 fixed further implementation issues. Thus, examples written for the initial 0.17 announcement require attention to the subsequent corrections.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`; `https://github.com/rust-ndarray/ndarray/blob/master/RELEASES.md`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 3. Dimensionality, memory geometry, and structural transformations

### 3.1 Static versus dynamic dimensionality

`ndarray` has two complementary dimensionality systems.

Statically ranked arrays: `Ix0`, `Ix1`, `Ix2`, `Ix3`, `Ix4`, `Ix5`, `Ix6`, exposed through aliases such as `Array1`, `Array2`, `Array3`, and corresponding view and shared-array aliases.

Dynamically ranked arrays: `IxDyn`, commonly `ArrayD<A>`, where the number of dimensions is determined at runtime.

Static ranking makes axis count part of the Rust type and enables rank-specific methods and trait implementations. Dynamic ranking supports heterogeneous ranks through one type, useful when dimensionality is determined from data or runtime configuration.

Significant facilities include:

- `Dimension`, `IntoDimension`, and `RemoveAxis` for generic dimensional operations.
- `raw_dim()` to retain an array's dimensionality type, versus `shape()` to obtain runtime extents.
- `into_dyn()` and `into_dimensionality::<IxN>()` for representation conversions; the latter checks rank at runtime.
- `insert_axis`, `remove_axis`, and `index_axis` to alter or reduce rank.
- `Axis(n)`, `len_of(axis)`, `axes()`, and `ndim()` for explicit axis-oriented programming.

A zero-dimensional array is a scalar array containing one element, not an empty array. An array with a zero-length axis, by contrast, has zero elements. This distinction affects constructors, reductions, and broadcasting.

Architectural implication: rank-polymorphic algorithms need not use `IxDyn` everywhere. Static-rank components can coexist with dynamic-rank boundaries, preserving stronger type information where it is available.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/trait.Dimension.html`; `https://docs.rs/ndarray/0.17.2/ndarray/type.IxDyn.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.2 Signed strides as the fundamental geometry model

For an array with base pointer \\(p\\), index vector \\(i\\), and element strides \\(s\\), the conceptual address calculation is:

\\[ p+\sum\_{k=0}^{n-1}i_k s_k \\]

Strides are measured in elements, not bytes, and are signed.

This allows one storage allocation to support multiple logical geometries:

| Layout              | Mechanism                                              | Consequence                                                  |
| ------------------- | ------------------------------------------------------ | ------------------------------------------------------------ |
| C / row-major       | Last axis has the fastest-changing contiguous elements | Common Rust and C-compatible representation                  |
| F / column-major    | First axis changes fastest                             | Useful for column-major numerical interfaces                 |
| Transposed          | Axis permutation changes shape/strides                 | Usually no data movement                                     |
| Reversed            | Negative stride                                        | Logical reverse traversal without reversing storage          |
| Strided subset      | Step greater than one, or arbitrary compatible strides | Sparse sampling of an existing buffer                        |
| Broadcasted         | Zero stride along expanded axes                        | One stored element can be read at multiple logical positions |
| Noncontiguous slice | Gaps between visited elements                          | Requires careful treatment by linear-buffer consumers        |

Relevant operations include `t()`, `swap_axes`, `permuted_axes`, `permute_axes`, `invert_axis`, `reverse_axes`, `ShapeBuilder::f()`, and explicitly specified strides.

The distinction between logical element order and physical memory order is critical:

- `is_standard_layout()` tests the standard C-layout property.
- `as_slice()` and `as_slice_mut()` require standard contiguous layout.
- `as_slice_memory_order()` accepts compatible contiguous layouts in their actual memory order.
- `as_standard_layout()` supplies a copy-on-write representation, borrowing the original when already standard and otherwise copying into standard layout.
- `into_raw_vec_and_offset()` exposes underlying owned-buffer details, including an offset that may matter after slicing or layout transformations.

Contiguity is therefore not equivalent to row-major layout; nor does owning a `Vec` mean the logical array covers that vector as one zero-offset, row-major slice.

Performance implication: many transformations are constant-time metadata changes, but subsequent elementwise or matrix operations may experience poorer locality or require a materialized contiguous representation. Avoiding an immediate copy does not guarantee that the whole computation is faster.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/struct.LayoutRef.html`; `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 3.3 Slicing, indexing, and independent subviews

The `s![]` macro describes multidimensional slices with ranges, fixed indices, negative indexing relative to axis ends, and stepped traversal.

The important conceptual operations are:

- `slice()` / `slice_mut()`: borrow a subarray using a slice descriptor.
- `slice_axis()` / `slice_each_axis()`: construct slices programmatically, including dynamic axis-dependent ranges.
- `index_axis()` / `index_axis_mut()`: select one index along an axis and reduce rank.
- `slice_collapse()` / `index_axis_inplace()`: change the geometry of an existing array or view.
- `split_at()`, `multi_slice_mut()`, and `multi_slice_move()`: obtain independently usable disjoint regions.

The ability to create multiple disjoint mutable subviews is especially consequential. It permits independent writes to regions of one allocation while maintaining Rust's aliasing guarantees. `multi_slice_mut` checks that requested mutable regions do not overlap.

For indexed access, `get()` and `get_mut()` provide checked optional access, indexing syntax can panic for invalid indices, and `uget()` / `uget_mut()` are unsafe unchecked alternatives.

A slice is generally a view, not a materialized selection. By contrast, arbitrary index gathering through `select(Axis, indices)` produces a new array. These are different cost and ownership models.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`

### 3.4 Reshape semantics and copy policy

The reshape interfaces make a deliberate distinction between changing a view's interpretation and permitting a copy.

| Operation                        | Behavioral distinction                                                                                                       |
| -------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `into_shape_with_order`          | Consume and reshape when the source is compatible with the requested contiguous traversal order; reject incompatible layouts |
| `to_shape`                       | Reshape into a `CowArray`, borrowing where possible and allocating when necessary                                            |
| `into_shape_clone`               | Consume an array and permit cloning/reordering where required                                                                |
| `flatten` / `flatten_with_order` | Produce a one-dimensional copy-on-write representation with a defined traversal order                                        |
| `into_flat`                      | Flatten an owned representation according to its supported layout rules                                                      |
| `insert_axis` / `remove_axis`    | Change dimensional interpretation without inherently changing element count                                                  |

The two ordering conventions are `Order::RowMajor` and `Order::ColumnMajor`. They affect the mapping of the ordered sequence of elements to the new shape; they are not merely display preferences.

The older `into_shape` and `reshape` APIs were deprecated in the 0.16 series in favor of more explicit reshape behavior.

Key distinction: a reshape operation that retains element count need not be implementable as a view. Stride compatibility determines whether the new indexing geometry can describe the same underlying storage.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`; `https://github.com/rust-ndarray/ndarray/blob/master/RELEASES.md`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 3.5 Broadcasting and virtual expansion

Broadcasting treats size-one axes as compatible with larger axes and extends them logically using zero strides.

`broadcast()` returns an optional read-only view when the requested shape is compatible. The operation can expand an array's logical shape without duplicating its elements.

`Zip::and_broadcast()` integrates compatible broadcastable producers into a multidimensional traversal; `Zip::and()` otherwise requires matching producer shapes.

Elementwise arithmetic also supports documented broadcasting combinations, but it is important not to assume every operation, assignment, or producer automatically applies arbitrary NumPy-style broadcasting.

Why it matters: broadcasting can replace explicit repeated arrays for coordinate grids, per-row or per-column coefficients, multidimensional masks, and elementwise parameter application. Its zero-stride representation also explains why broadcasted views are read-only: permitting ordinary mutable aliasing would make multiple logical indices refer to the same writable element.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/struct.Zip.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 4. Multidimensional traversal: iterators, producers, and `Zip`

This is one of `ndarray`'s most important capability families. It provides higher-order traversal abstractions that understand dimensional geometry and can avoid manually calculating indices.

### 4.1 Axis-oriented iteration

Different iteration interfaces encode substantially different operations:

| Interface                  | Produced items                                       | Typical computation                                      |
| -------------------------- | ---------------------------------------------------- | -------------------------------------------------------- |
| `iter()` / `iter_mut()`    | Individual element references                        | Scalar traversal                                         |
| `indexed_iter()`           | Index and element                                    | Position-dependent algorithms                            |
| `outer_iter()`             | Subviews along axis 0                                | Iterate over rows or outermost slices                    |
| `axis_iter(Axis)`          | Subviews with one axis removed                       | Process independent slices along an arbitrary axis       |
| `rows()` / `columns()`     | Row or column views                                  | Matrix-oriented processing                               |
| `lanes(Axis)`              | One-dimensional lanes along the chosen axis          | Axis reductions, filters, transforms                     |
| `axis_chunks_iter`         | Nonoverlapping views along an axis                   | Blockwise processing with a possible smaller final chunk |
| `exact_chunks`             | Fixed-shaped multidimensional blocks                 | Tiling; incomplete trailing blocks are excluded          |
| `windows`                  | Overlapping multidimensional subviews                | Stencils and neighborhood calculations                   |
| `windows_with_stride`      | Multidimensional windows at specified step intervals | Sliding or subsampled neighborhood processing            |
| `axis_windows`             | Overlapping windows along a chosen axis              | Rolling operations and one-axis stencils                 |
| `axis_windows_with_stride` | Axis windows with configurable step                  | Windowed downsampling and rolling feature extraction     |

The distinction between chunks and windows is fundamental. Chunks partition the represented data into nonoverlapping regions. Windows can overlap and therefore share underlying elements. This affects ownership, mutable access, parallelism, and write safety.

Mutable axis iterators and chunk iterators can yield separate writable regions when nonoverlap is guaranteed. Overlapping windows instead expose read-only views.

The 0.17 series added `axis_windows_with_stride`, along with lifetime-preserving `into_*_iter` variants useful when transferring ownership of views into iterators.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.2 `NdProducer`: multidimensional producers are more powerful than ordinary iterators

An `NdProducer` represents a source of items with an N-dimensional shape. Its items need not be scalar elements: they may be references, mutable references, array views, or other producer-specific objects.

This lets the same traversal infrastructure operate over:

- Array elements;
- Coordinates and indexed elements;
- Rows, columns, and lanes;
- Blocks and sliding windows;
- Mutable output regions;
- Producers with distinct element types but matching iteration geometry.

An ordinary Rust `Iterator` primarily defines a sequence. `NdProducer` additionally exposes dimensionality and supports splitting and geometry-aware traversal, enabling optimized lockstep operations.

Architectural significance: producers provide a reusable abstraction for multidimensional work distribution. A reduction over windows, an in-place transform over disjoint chunks, and a fused operation over multiple arrays need not each implement a bespoke indexing engine.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/trait.NdProducer.html`

### 4.3 `Zip`: fused, heterogeneous, layout-aware computation

`Zip` combines several multidimensional producers and applies a closure to corresponding items.

Its important operations include:

| Operation            | Capability                                               |
| -------------------- | -------------------------------------------------------- |
| `Zip::from` / `.and` | Compose producers with matching shape                    |
| `.and_broadcast`     | Include a producer broadcastable to the existing shape   |
| `Zip::indexed`       | Include multidimensional coordinates                     |
| `.for_each`          | Apply an operation to each producer-item tuple           |
| `.fold`              | Accumulate a result over the joined producers            |
| `.fold_while`        | Support early termination                                |
| `.all` / `.any`      | Predicate evaluation across joined inputs                |
| `.map_collect`       | Produce a new array from joined inputs                   |
| `.map_assign_into`   | Write generated results into an existing producer        |
| `.split`             | Divide the work while preserving locality where possible |
| `azip!`              | Compact macro form of lockstep producer traversal        |

A particularly useful distinction is that `Zip` can combine producers whose items have different types. For example, an output producer might yield mutable scalar references while another yields immutable references and a third yields one-dimensional views.

This provides several opportunities:

- Loop fusion: calculate an expression from several inputs and write its result in one traversal.
- Output reuse: write directly into an existing array rather than allocate a temporary result.
- Layout-aware execution: when input layouts agree, contiguous traversal improves the opportunity for compiler vectorization.
- Custom kernels: express domain-specific elementwise and neighborhood calculations without replacing the underlying multidimensional traversal machinery.

`Zip` does not guarantee a particular visitation order. Algorithms depending on a sequential traversal order should use an interface that documents that order, rather than relying on incidental layout behavior.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/struct.Zip.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 4.4 Execution fusion versus mathematical broadcasting

These are complementary, not interchangeable.

Broadcasting changes the logical address mapping so that operands of compatible shapes can be read together without expanding storage.

`Zip` fusion combines several computations into one traversal without requiring each intermediate array to be materialized.

The combination can eliminate both repeated input storage and intermediate output allocations. However, the closure still executes eagerly, and constructing a `Zip` does not create a persistent computation graph or deferred optimization stage.

This is a key distinction from lazy tensor systems and query execution frameworks.

## 5. Parallel execution and computational locality

The optional `rayon` feature enables `ndarray`'s parallel execution facilities.

### 5.1 Parallel capabilities

The library integrates with Rayon at several levels:

- Element-level traversal: `par_iter`, `par_iter_mut`, and parallel iteration over array views.
- Subview-level traversal: parallel `AxisIter`, `AxisIterMut`, `AxisChunksIter`, and `AxisChunksIterMut`.
- Parallel lockstep computation: `Zip::par_for_each`, `par_map_collect`, `par_map_assign_into`, and `par_fold`.
- Parallel macros: `par_azip!` for multidimensional parallel operations.
- Parallel mutation: `par_map_inplace` and `par_mapv_inplace` on suitable array types.

The parallel producer interfaces preserve the underlying borrowing constraints. Distinct writable regions can be processed in separate tasks without requiring interior mutability or user-managed synchronization for every element.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/parallel/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.2 Parallel granularity is a substantive design choice

`Zip::par_fold` divides work among tasks, executes local folds, then combines their results. The documentation specifically notes that coarser partitioning—such as summing each row sequentially within a parallel row-level traversal—can be more efficient than scheduling fine-grained elementwise work.

The reasons are architectural rather than specific to one benchmark:

1. Large contiguous chunks reduce coordination overhead and improve locality.
2. Sequential inner loops can be easier to vectorize.
3. Partitioning independent subviews can simplify writes and avoid unnecessary shared state.

Parallel reduction order is not deterministic in general. Floating-point addition is not mathematically associative at finite precision, so parallel reductions can produce different final rounding from sequential or differently partitioned execution.

Rayon parallelism is also distinct from threading performed inside a matrix-multiplication backend or BLAS library. Combining them carelessly can cause oversubscription.

Important limitation: enabling `rayon` does not mean every `ndarray` arithmetic operation automatically becomes parallel. It exposes parallel APIs for explicitly parallelizable work.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/struct.Zip.html`; `https://docs.rs/ndarray/0.17.2/ndarray/parallel/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 6. Elementwise and axis-oriented numerical computation

### 6.1 Arithmetic, transformations, and output reuse

The ordinary arithmetic operators implement elementwise array operations where the operands and element types support them.

Relevant capabilities include:

- Elementwise `+`, `-`, `*`, `/`, and corresponding assignment operators.
- Arithmetic with scalar operands.
- `map`, `mapv`, `map_inplace`, `mapv_inplace`, and `mapv_into` for transformations with different ownership and consumption semantics.
- `zip_mut_with` and `Zip` for transformations using multiple arrays.
- `scaled_add` for an AXPY-like update, combining an existing array with a scaled second array.
- `assign` and `assign_to` for copying values into another compatible array.
- `mapv_into_any` for consuming and transforming arrays, with reuse opportunities when source and destination types permit.

Allocation distinction: `&a + &b` normally produces an output allocation, whereas in-place mutation can reuse storage. Consuming an owned input may also allow allocation reuse for supported operations. Those opportunities are implementation-dependent and should not be generalized into a guarantee that arbitrary chained expressions are fused.

The use of a numeric-looking operator does not imply matrix algebra: `a * b` is elementwise multiplication, not the matrix product.

### 6.2 Reductions, grouping, and axis-local functions

`ndarray` includes both general reduction primitives and specialized numerical operations.

| Family                  | Important APIs                          | Distinction                                                           |
| ----------------------- | --------------------------------------- | --------------------------------------------------------------------- |
| Whole-array aggregation | `sum`, `product`, `fold`                | Reduce every element                                                  |
| Axis aggregation        | `sum_axis`, `product_axis`, `fold_axis` | Remove one axis                                                       |
| Axis mapping            | `map_axis`, `map_axis_mut`              | Map each one-dimensional lane to a result                             |
| Mean                    | `mean`, `mean_axis`                     | Account for element count; empty inputs require special handling      |
| Dispersion              | `var`, `var_axis`, `std`, `std_axis`    | Welford one-pass variance; configurable degrees-of-freedom correction |
| Ordered accumulation    | `accumulate_axis_inplace`               | Successive cumulative computation along one axis                      |
| Cumulative product      | `cumprod`                               | Produce products accumulated along an axis                            |
| Finite differences      | `diff`                                  | Compute repeated forward differences along an axis                    |
| Partial ordering        | `partition`                             | Position a selected order statistic without fully sorting every lane  |

Several deserve closer attention.

`accumulate_axis_inplace` provides a reusable primitive for prefix computations. It guarantees forward progression along the selected axis, even though other traversal ordering is unspecified.

`diff(n, axis)` computes the nth-order forward finite difference along a selected axis. This makes certain discrete derivative and difference calculations native operations rather than requiring manual shifted-array subtraction.

`partition(kth, axis)` produces a partially ordered copy: the selected element is in its sorted position, with appropriate values on each side, but without sorting each partition. The native API requires `Ord` and other trait bounds; ordinary floating-point values do not implement `Ord`, so floating-point order statistics require additional consideration or companion functionality.

Variance and standard deviation use a Welford one-pass approach. The `ddof` argument changes the denominator and has explicit validity constraints; these methods should not be treated as interchangeable with unparameterized variance implementations.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 6.3 Floating-point, complex, and geometric operations

The native floating-point methods cover common transcendental and mathematical functions, including `exp`, `ln`, `sqrt`, trigonometric and hyperbolic functions, rounding, and power operations.

The 0.17 series extended these with functions including `exp_m1`, `ln_1p`, inverse trigonometric functions, hyperbolic functions, inverse hyperbolic functions, and `hypot`.

`exp_m1` and `ln_1p` are particularly useful because they provide numerically appropriate formulations for expressions involving small arguments, where directly evaluating `exp(x) - 1` or `ln(1 + x)` can lose precision.

Other facilities include `is_nan`, `is_any_nan`, `is_all_nan`, `is_infinite`, `triu`, and `tril`, with availability depending on element type and trait bounds.

For complex arrays, `split_complex()` can produce separate real and imaginary views, including mutable views, over existing complex-valued storage. This is a valuable interoperation mechanism when real-valued components must be processed independently without duplicating the entire complex array.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`; `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`; `https://github.com/rust-ndarray/ndarray/blob/master/RELEASES.md`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 7. Native linear algebra and backend selection

### 7.1 Matrix and vector operations

The core crate provides a deliberately bounded set of linear algebra operations.

| Operation                    | Native capability                                      | Important distinction                                                           |
| ---------------------------- | ------------------------------------------------------ | ------------------------------------------------------------------------------- |
| Vector dot product           | `dot`                                                  | Multiplies corresponding elements and sums; does not conjugate complex operands |
| Matrix–vector multiplication | `dot`, `general_mat_vec_mul`                           | Supports matrix/vector algebra with compatible shapes                           |
| Matrix–matrix multiplication | `dot`, `general_mat_mul`                               | Native matrix multiplication, distinct from elementwise `*`                     |
| Scaled accumulation          | `scaled_add`                                           | Update an existing array using a scaled operand                                 |
| Kronecker product            | `linalg::kron`                                         | Construct a blockwise matrix product                                            |
| Matrix structure             | `eye`, `from_diag`, `diag`, `diag_mut`, `triu`, `tril` | Construct or access special matrix structures                                   |

`general_mat_mul` implements:

\\[ C \leftarrow \alpha AB+\beta C \\]

This is especially useful where repeated matrix products must accumulate into an existing output buffer. It can avoid the extra array allocation and subsequent addition associated with evaluating the expression in separate steps.

`general_mat_vec_mul` provides the corresponding scaled matrix–vector computation.

An important 0.17 distinction: `dot` now supports `IxDyn` arrays, but this is not general N-dimensional batched matrix multiplication. Dynamic-dimensional operands must represent one- or two-dimensional vectors or matrices; higher-rank operands cause a panic.

For complex vector products, `dot` does not conjugate an operand. Therefore, a complex dot product must not automatically be interpreted as a Hermitian inner product.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/linalg/index.html`; `https://docs.rs/ndarray/0.17.2/ndarray/linalg/trait.Dot.html`; `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 7.2 Native multiplication versus BLAS

By default, `ndarray` uses the Rust `matrixmultiply` implementation for supported matrix multiplication.

The optional `blas` feature enables calls into a separately configured BLAS implementation where the element type and memory layout permit. Supported acceleration paths include real and complex matrix multiplication.

| Execution mechanism        | Role                                                | Consequences                                        |
| -------------------------- | --------------------------------------------------- | --------------------------------------------------- |
| `matrixmultiply`           | Native matrix-multiplication backend                | No independently configured external BLAS required  |
| `matrixmultiply-threading` | Enables the backend's threading feature             | Threading behavior is separate from Rayon           |
| `blas`                     | Permits compatible operations to call BLAS routines | Requires external backend configuration and linking |
| `rayon`                    | Parallelizes higher-order array processing          | Does not itself select a BLAS implementation        |

BLAS is therefore an optional execution path, not a requirement for ordinary matrix multiplication.

Physical layout matters. Native or BLAS-backed multiplication may benefit from contiguous matrices, while unsupported layouts can require an alternative path or data rearrangement.

A library exposing a BLAS-backed API does not guarantee that every possible input layout reaches the same backend.

### 7.3 Where native linear algebra ends

`ndarray` does not attempt to implement every major matrix decomposition or numerical solver. QR, LU, Cholesky, eigendecomposition, SVD, and solving general linear systems are primarily supplied by companion numerical libraries.

This distinction allows `ndarray` to remain a broadly reusable array substrate while specialized packages provide more substantial numerical algorithms and their backend dependencies.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/linalg/index.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 8. Construction, data movement, and low-level extension mechanisms

### 8.1 Array construction and structural assembly

The construction API supports far more than zero-initialized arrays.

| Mechanism                                    | Functionality                                                       |
| -------------------------------------------- | ------------------------------------------------------------------- |
| `array!`, `arr0`, `arr1`, `arr2`             | Construct arrays from Rust values                                   |
| `from_shape_vec`                             | Adopt a vector as array storage subject to shape/layout validation  |
| `from_shape_fn`                              | Populate values using multidimensional coordinates                  |
| `from_shape_simple_fn`                       | Populate values without requiring coordinate-dependent generation   |
| `from_elem`, `zeros`, `ones`, `default`      | Initialize arrays with a repeated or default value                  |
| `linspace`, `range`, `logspace`, `geomspace` | Generate numerical sequences                                        |
| `eye`, `from_diag`, `from_diag_elem`         | Construct identity and diagonal matrices                            |
| `meshgrid`                                   | Generate multidimensional coordinate arrays from coordinate vectors |
| `concatenate`                                | Join arrays along an existing axis                                  |
| `stack`                                      | Add a new axis and stack arrays along it                            |
| `append`, `push`, `reserve`                  | Modify the extent or capacity of owned arrays                       |

`meshgrid`, added in 0.17, supports `MeshIndex::IJ` and `MeshIndex::XY` conventions. The difference affects the ordering of coordinate axes, particularly in two-dimensional Cartesian grids.

It produces coordinate arrays, which is convenient but can carry a memory cost for large grids. Where only virtual coordinate expansion is required, appropriately shaped broadcasted views may achieve the same input semantics without storing entire replicated coordinate matrices.

`stack` and `concatenate` also have distinct rank semantics: stacking introduces a dimension, while concatenation expands an existing one. Both construct owned results.

Although arrays can grow, the underlying library documentation emphasizes that growing and shrinking N-dimensional arrays is potentially expensive. Array-oriented storage is generally optimized more for fixed geometry and traversal than for frequent structural mutation.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/fn.meshgrid.html`; `https://docs.rs/ndarray/0.17.2/ndarray/fn.stack.html`; `https://docs.rs/ndarray/0.17.2/ndarray/fn.concatenate.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



### 8.2 Uninitialized arrays and ownership-preserving movement

`ndarray` supports advanced construction paths for elements that are expensive to initialize, clone, or allocate repeatedly.

`Array::uninit` and `build_uninit`:

- Allocate array storage containing `MaybeUninit<A>`.
- Expose a mutable view of the uninitialized elements.
- Permit initialization without first creating throwaway default values.
- Allow conversion to initialized elements using `assume_init()` once all required storage has been initialized.

`assume_init()` is unsafe because the caller must establish that every relevant element has actually been initialized. An incomplete initialization path, including one interrupted by an error or panic, must not be incorrectly treated as complete.

`move_into` and `move_into_uninit`:

- Move elements from one owned array into a destination of matching shape.
- Allow the destination to have a different physical layout.
- Avoid per-element cloning when ownership can be transferred.
- Support writing into already allocated, possibly uninitialized output storage.

These mechanisms are valuable for non-`Copy` elements, large intermediate arrays, layout conversions, and specialized buffer management.

They also explain why array storage abstractions extend beyond numerical scalars: moving ownership of strings, records, or other Rust objects can be materially different from copying numeric data.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 8.3 Foreign buffers, raw views, and unsafe boundaries

A borrowed Rust slice can be exposed as an array view using shape-aware constructors. This makes `ndarray` suitable for interpreting existing contiguous buffers multidimensionally without necessarily copying their contents.

When a buffer is provided through a raw pointer, `from_shape_ptr`, `RawArrayView`, `RawArrayViewMut`, and `RawRef` expose lower-level functionality.

The relevant safety requirements include:

- Correct element alignment and pointer validity.
- Valid address arithmetic for the specified shape and strides.
- Appropriate allocation bounds.
- Proper lifetime relationships between storage and views.
- No invalid mutable aliasing.
- Correct initialization of elements before safe references are constructed.

Some raw-pointer constructors impose more restrictive stride requirements than ordinary array views created safely through slicing. In particular, their documentation specifies nonnegative strides in the relevant constructors; that restriction should not be confused with the general ability of `ndarray` views to represent negative-stride slices.

`RawArrayView::cast` can reinterpret pointer element types subject to element-size constraints, but accepting a cast does not prove that accessing the resulting elements is valid. Alignment, representation, and aliasing obligations remain.

Architectural implication: the library can act as a safe multidimensional interface over memory owned by other systems, provided the unsafe boundary correctly establishes the array's validity. Foreign-buffer interoperability is not automatically safe merely because a pointer has been wrapped in an array type.

### 8.4 `MathCell` and shared-view interior mutability

`cell_view()` converts a mutable borrow into a shared view whose elements behave as `MathCell<A>`.

`MathCell` wraps `Cell<T>` while supporting arithmetic-related operations. This enables certain algorithms to perform mutations through shared element references, without making every traversal interface require an exclusive mutable element reference.

It is a specialized mechanism for algorithms needing cell-style mutation semantics. It is not a general replacement for unique mutable views, nor is it a cross-thread synchronization mechanism: `MathCell` is not `Sync`.

This is an easily overlooked distinction between safe interior mutability and safe parallel mutable access.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/struct.MathCell.html`; `https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 9. Build features and execution dependencies

Verified against the 0.17.2 crate feature documentation and Cargo metadata.

| Feature                            | Availability and effect                                                                  |
| ---------------------------------- | ---------------------------------------------------------------------------------------- |
| `std`                              | Enabled by default. Provides standard-library-dependent numerical routines and utilities |
| `serde`                            | Adds Serde serialization and deserialization support                                     |
| `rayon`                            | Adds parallel producers, iterators, methods, and `par_azip!`; implies `std`              |
| `approx`                           | Implements approximate-equality traits from the `approx` crate                           |
| `blas`                             | Enables optional BLAS-backed multiplication with external backend configuration          |
| `matrixmultiply-threading`         | Enables threading support in the `matrixmultiply` backend                                |
| `portable-atomic-critical-section` | Passes the `critical-section` feature to `portable-atomic`                               |

### `no_std` support

Disabling the default `std` feature allows use without the standard library, subject to dependencies and the target's allocation requirements.

A meaningful limitation is that methods including `linspace`, `range`, `logspace`, `geomspace`, `var`, and `std` require `std` in 0.17.2.

Thus, `no_std` support preserves the core array model and useful operations, but not the complete numerical API.

### Optional backend management

`rayon`, BLAS, and `matrixmultiply` threading do not represent one unified thread-management mechanism. They can each introduce independent parallel execution behavior.

The 0.17.2 crate itself requires Rust 1.64 or newer. Companion packages may have higher Rust requirements; for example, the current PyO3 NumPy binding has a substantially higher minimum Rust version.

Also note that the 0.17 series removed the old `serde-1` feature name: the current feature is `serde`.

Sources: `https://docs.rs/ndarray/0.17.2/ndarray/doc/crate_feature_flags/`; `https://docs.rs/crate/ndarray/0.17.2/features`; `https://docs.rs/crate/ndarray/0.17.2/source/Cargo.toml.orig`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



## 10. Companion libraries and ecosystem integration

The surrounding ecosystem matters because `ndarray` deliberately leaves several sophisticated capability families to specialized packages.

The versions below were verified on October 28, 2026. They are separate packages, not modules automatically included by ndarray.

### 10.1 Numerical and statistical companions

| Package          | Verified version | Capabilities and boundaries                                                                                                                |
| ---------------- | ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `ndarray-linalg` | 0.18.1           | LAPACK-backed QR, LU, Cholesky, eigendecomposition, SVD, inverse, linear solves, and related numerical operations                          |
| `ndarray-stats`  | 0.7.0            | Order statistics, medians, quantiles, covariance, correlation, moments, skewness, kurtosis, entropy, divergence, distances, and histograms |
| `ndarray-rand`   | 0.16.0           | Random array constructors, probability distributions, reproducible RNG integration, and sampling along axes                                |

`ndarray-linalg` is the principal extension when basic matrix multiplication is insufficient. It builds on LAPACK through lower-level bindings and requires a compatible configured backend, such as OpenBLAS, Netlib, or Intel MKL.

This adds deployment and linking considerations beyond base `ndarray`. The selected backend can affect performance, platform compatibility, and runtime threading. Multiple conflicting LAPACK backend selections are not interchangeable.

Source: `https://docs.rs/ndarray-linalg/0.18.1/ndarray_linalg/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`ndarray-stats` extends the library with advanced statistical operations, including order statistics that go beyond the native `partition` interface. The 0.7 series uses the newer `ArrayRef` abstraction, reducing the need to constrain statistical algorithms to concrete owned array types.

Source: `https://docs.rs/ndarray-stats/0.7.0/ndarray_stats/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`ndarray-rand` provides array construction from probability distributions and sampling over existing arrays. Version 0.16 uses `rand` 0.9 and `rand_distr` 0.5 and adds `RandomRefExt` for the 0.17-era `ArrayRef` interface.

Its `SamplingStrategy` distinguishes sampling with and without replacement. The crate reexports its compatible `rand` and `rand_distr` versions, which is useful because otherwise mismatched major versions can create incompatible trait implementations.

Source: `https://docs.rs/ndarray-rand/0.16.0/ndarray_rand/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 10.2 NumPy file formats and memory mapping

`ndarray-npy` 0.10.0 provides `.npy` and `.npz` interoperability:

- `read_npy`, `write_npy`, and associated extension traits.
- `NpzReader` and `NpzWriter`.
- `ViewNpyExt` and `ViewMutNpyExt` for creating array views over compatible `.npy` buffers, including memory-mapped files.
- `write_zeroed_npy` for producing a zero-filled file that may be sparse at the filesystem level.

The combination of file-backed buffers and array views can be particularly valuable for datasets larger than the available RAM: the operating system handles paging while the application uses `ndarray`'s multidimensional geometry.

However, file mapping, element validity, alignment, and mapping lifetime remain separate responsibilities. Creating a zero-copy view also does not guarantee zero-copy access by every downstream numerical operation.

The crate's documented element coverage is primarily fixed-size primitive numeric types, complex floating-point types when enabled, and `bool`; arbitrary structured NumPy dtypes are not generally covered.

Source: `https://docs.rs/ndarray-npy/0.10.0/ndarray_npy/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



An adjacent package, `ndarray-npz` 0.4.2, provides more advanced viewing of uncompressed `.npy` members within `.npz` archives, including memory-mapped access.

Version caveat: the published `ndarray-npz` 0.4.2 package depends on `ndarray` 0.16.1 and `ndarray-npy` 0.9.1. Its capabilities should not be assumed to integrate directly with 0.17.2 types without resolving that compatibility boundary.

Sources: `https://docs.rs/ndarray-npz/0.4.2/ndarray_npz/`; `https://docs.rs/crate/ndarray-npz/0.4.2`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 10.3 Python NumPy interoperability through PyO3

The separate Rust crate `numpy` 0.30.0, maintained by the PyO3 ecosystem, integrates Rust `ndarray` types with the Python NumPy C API.

Its role is not to implement another multidimensional computation backend, but to bridge the representation and lifecycle of Rust and Python arrays.

Important capabilities and constraints include:

- Typed access to Python-owned NumPy arrays.
- Sharing or converting compatible data between Rust and NumPy.
- Constructing NumPy arrays from Rust-managed data.
- Borrowing and mutability constraints associated with Python-owned memory.
- Layout and dtype compatibility requirements.
- Python interpreter and PyO3 integration requirements.

The package supports multiple semver-incompatible `ndarray` releases through a broad dependency range (`>=0.15, <=0.17`). Its documentation explicitly warns that Cargo may resolve more than one `ndarray` version, requiring dependency unification where types must interoperate.

Version 0.30 requires Rust 1.85 or newer and Python 3.9 or newer, despite the lower MSRV of core `ndarray`.

Sources: `https://docs.rs/numpy/0.30.0/numpy/`; `https://github.com/PyO3/rust-numpy`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



### 10.4 Other array representations

The `ndarray` documentation also identifies `nshare` for conversions involving `ndarray`, `nalgebra`, and `image`.

These libraries have different primary abstractions:

- `ndarray`: variable-rank, multidimensional, strided data and general array transformations.
- `nalgebra`: algebra-oriented matrix/vector types and geometric constructs.
- `image`: pixel buffers, color formats, and image-oriented operations.
- `nshare`: conversion and borrowing interfaces among selected representations.

Compatibility and copy behavior depend on element representation, dimensions, memory order, and each companion's dependency version. Similar mathematical shapes do not imply identical physical layouts or element semantics.

Source: `https://docs.rs/nshare/0.10.0/nshare/`

## 11. Cross-cutting distinctions and architectural opportunities

The most consequential choices in `ndarray` are not generally about which constructor or numerical method to call. They concern how several native mechanisms compose.

| Design distinction                           | What ndarray enables                                                  | Important condition                                                     |
| -------------------------------------------- | --------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| Views versus materialization                 | Slicing, transposition, and broadcast without copying                 | Downstream work must tolerate resulting strides                         |
| Owned versus shared storage                  | Cheap read-only sharing and controlled mutation                       | Mutating shared data may clone the allocation                           |
| Static versus dynamic rank                   | Type-checked dimensional algorithms alongside runtime-generic arrays  | Rank-specific operations may require a checked conversion               |
| Producer versus iterator                     | Geometry-aware traversal, fusion, and work partitioning               | Shape compatibility and visitation-order guarantees vary                |
| Chunks versus windows                        | Independent block processing or overlapping neighborhood analysis     | Mutable overlap must be prohibited or handled separately                |
| Elementwise math versus matrix algebra       | Native arithmetic and linear algebra within one array model           | Ordinary `*` is not a matrix product                                    |
| Eager arithmetic versus fused traversal      | Direct array expressions or explicitly fused custom kernels           | Intermediate allocations are not automatically eliminated               |
| Scalar traversal versus bulk algebra         | Generic per-element kernels alongside optimized matrix multiplication | Larger operations can justify specialized execution backends            |
| Array semantics versus external data formats | Reuse external buffers and file-backed storage                        | Representation, alignment, lifetimes, and conversion costs still matter |

### Three particularly useful capability combinations

1\. Strided views + broadcasting + `Zip`

Together, these can express a multidimensional transformation over a subset of an existing buffer, with logically expanded parameters and a single traversal into the destination.

This can remove the need for manually constructed repeated operands, explicit multidimensional index calculation, and temporary intermediate arrays.

2\. Axis producers + disjoint mutable views + Rayon

These provide a structured model for parallel blockwise operations. The library handles multidimensional partitioning and safe borrowing while the computation focuses on each block.

That reduces the need for manually partitioned pointer arithmetic and per-element locks, provided the work can be divided into independent regions.

3\. `ArrayRef` + companion extension traits + backend-specific numerical routines

Generic interfaces allow numerical packages to work with a broader set of owned and borrowed array representations. Higher-level algorithms can be separated from storage choice, while specialized numerical routines provide capabilities beyond the core library.

The resulting separation between data representation, traversal, and mathematical algorithms is one of the library's strongest architectural properties.

### Important limitations

- No general lazy evaluation: transformations execute eagerly; optimization across expressions requires explicit composition or another execution framework.
- No automatic GPU execution: the core array model does not itself implement CUDA or GPU tensor execution.
- No native sparse-array abstraction: the primary data model is dense, even though views can represent strided selections.
- No general N-dimensional batched `dot`: the new dynamic-rank implementation remains limited to one- and two-dimensional operands.
- No automatic global scheduling: parallel traversal, matrixmultiply threading, and BLAS backend threading must be considered separately.
- No implicit null bitmap: elements may be `Option<T>` or another representation, but the array abstraction itself is not a nullable Arrow-style column with a separate validity bitmap.
- No universal zero-copy guarantee: views are cheap, but reshapes, layout normalization, concatenation, ownership changes, or foreign-library requirements may introduce copies.
- No universal numerical error-return convention: invalid shapes can yield `ShapeError` in some APIs, while other operations panic on incompatible inputs. Numerical domain errors, NaNs, and floating-point special values remain relevant independently of shape validation.

The project also explicitly describes its API as evolving; hidden implementation details should not be treated as stable public interfaces.

Source: `https://docs.rs/ndarray/0.17.2/ndarray/`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 12. Version-specific conclusions

The most important changes in the 0.17 series are architectural as well as functional.

| Release | Significant change                                                                                                                                                                               |
| ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| 0.16.0  | More explicit reshape APIs; older `reshape` and `into_shape` methods deprecated                                                                                                                  |
| 0.17.0  | Introduced `ArrayRef`, `LayoutRef`, `RawRef`, `diff`, `partition`, `meshgrid`, `cumprod`, dynamic-dimensional `dot`, additional numerical methods, and strided axis windows; subsequently yanked |
| 0.17.1  | Corrected an unsound reference-type implementation; reference types became unsized                                                                                                               |
| 0.17.2  | Patched additional new-reference implementation issues and expanded higher-dimensional `ArcArray` aliases                                                                                        |

For an advanced reader, the defining development is the move toward generic array-reference interfaces, which sits alongside the library's already mature strided-view and multidimensional-producer architecture.

The combined capability model is:

Storage-independent references → typed multidimensional geometry → virtual views and broadcasting → fused or parallel traversal → native or companion numerical execution.

This makes `ndarray` especially valuable as a reusable multidimensional data and computation substrate. Its boundaries are equally important: it is not a tensor compiler, general sparse engine, GPU runtime, or complete linear algebra suite. Its role is to provide an efficient, composable array model on which those more specialized capabilities can operate.

Primary release evidence: `https://github.com/rust-ndarray/ndarray/blob/master/RELEASES.md`&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

# ndarray (Rust) — Capability Reference Addendum

Target version: `ndarray` 0.17.2\
Research date: November 9, 2026\
Scope: Additional native functionality, advanced API semantics, representation and interoperability guarantees, and relevant companion libraries not fully covered in the original reference.

This addendum supplements the existing `ndarray` capability reference without repeating its treatment of ownership models, ordinary slicing, multidimensional producers, numerical reductions, parallel traversal, or core linear algebra.

`ndarray` 0.17.2, released January 10, 2026, remains the latest published stable version at the research date. Unless explicitly identified as companion functionality, the mechanisms below belong to the released `ndarray` 0.17.2 API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



## 1. Additional structural transformations and layout mechanisms

### 1.1 Dynamic-rank dimension elimination with `squeeze()`

`ArrayBase<_, IxDyn>` provides `squeeze(self) -> Self`, which removes singleton axes while preserving the represented elements and their multidimensional indexing semantics.

Unlike `into_dimensionality()`, which changes the dimensionality type after validating an expected rank, `squeeze()` derives the output rank from the runtime shape.

```
use ndarray::Array3;

let a = Array3::<f64>::zeros((4, 1, 8)).into_dyn();

let b = a.squeeze();

assert_eq!(b.shape(), &[4, 8]);
```

The operation is a structural transformation: it removes axes from the shape and corresponding strides, without requiring elementwise materialization.

A significant distinction from NumPy is that `squeeze()` does not necessarily produce a scalar array when all dimensions are singleton. In the documented behavior, an input with more than one dimension retains at least one dimension. Consequently, shape `[1, 1]` becomes `[1]` rather than `[]`.

This is relevant to runtime-rank numerical interfaces, batched computations, and tensor transformations where singleton dimensions are introduced for broadcasting and subsequently need to be removed.

It also means that `squeeze()` is not an exact substitute for an operation requiring all singleton axes to collapse to rank zero.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`

### 1.2 Axis coalescing with `merge_axes()`

`merge_axes(take, into) -> bool` provides a specialized alternative to general reshaping.

It merges two axes when their existing strides permit their traversal to be represented by one axis, without changing the sequence of accessible elements.

The defining condition is that movement along the original axes, with `into` varying fastest, must be equivalent to movement along the resulting merged axis.

On successful merging:

- The `into` axis acquires the product of the original axis lengths.
- The `take` axis becomes length one, or length zero when the product is zero.
- The operation retains the total dimensionality, rather than deleting the merged-away axis.
- The original data allocation is not reorganized.

```
use ndarray::{Array3, Axis};

let mut a = Array3::<f64>::zeros((2, 3, 4));

assert!(a.merge_axes(Axis(1), Axis(2)));
assert_eq!(a.shape(), &[2, 1, 12]);
```

If merging is impossible, the method returns `false` and preserves the original dimensions and strides. Reversing the order of `take` and `into` can change whether merging succeeds.

This differs from `to_shape()`, which can allocate when necessary, and `into_shape_with_order()`, which validates whether a requested reinterpretation is supported.

`merge_axes()` therefore provides a directly inspectable stride-compatibility operation rather than a general-purpose reshape policy.

It can be useful for reducing the number of logically independent traversal axes, constructing flattened views of compatible regions, or simplifying higher-dimensional iteration.

A related method, `max_stride_axis()`, returns the axis with the largest absolute stride, preferring axes with length greater than one. It supplies layout information useful when constructing custom traversal or axis-coalescing algorithms. Selecting a maximum-stride axis is not, by itself, a universal optimization: optimal traversal also depends on which axes should vary in the inner loop and on the layouts of other operands.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.LayoutRef.html`

### 1.3 In-place geometry manipulation without rank changes

Several layout-oriented methods support changes that can be expressed through shape, stride, and pointer metadata rather than copying array elements.

Their effects differ in ways that matter for rank-polymorphic algorithms.

| Method                             | Structural effect                                                               |
| ---------------------------------- | ------------------------------------------------------------------------------- |
| `collapse_axis(axis, index)`       | Selects one position and changes the axis length to one, preserving rank        |
| `slice_collapse(...)`              | Applies slicing in place, collapsing indexed axes instead of removing them      |
| `slice_axis_inplace(axis, slice)`  | Modifies the represented region along one existing axis                         |
| `slice_each_axis_inplace(closure)` | Determines a slice for each axis programmatically                               |
| `insert_axis_inplace(axis)`        | Inserts a singleton axis into a dynamically ranked representation               |
| `index_axis_inplace(axis, index)`  | Selects an index and removes that axis from a dynamically ranked representation |

The distinction between collapsing and removing an axis is particularly important.

For example, selecting one row of a matrix can produce either a rank-one vector or a rank-two representation with one row. Both may expose the same elements, but their broadcasting behavior and suitability as inputs to rank-constrained algorithms differ.

`slice_collapse()` has narrower shape semantics than general slicing. In particular, it cannot handle `NewAxis` components, since inserting an axis would violate its contract of preserving the number of dimensions.

These operations make it possible to implement generic shape-transformation logic without repeatedly allocating arrays or converting all inputs into explicitly owned dynamic-rank storage.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.LayoutRef.html`

### 1.4 Structural index removal and element ownership

`remove_index(axis, index)` removes an indexed position along an axis, shifts subsequent positions down, and decreases that axis's length by one.

Its most consequential characteristic is its treatment of the elements that disappear from the logical array.

The removed elements are not immediately destroyed or deinitialized. The implementation moves them outside the array's currently represented region, where they remain associated with the underlying storage.

This is materially different from a `Vec::remove()` operation.

For arrays of primitive numeric values, the distinction may have little observable consequence. For arrays containing owned strings, heap-allocated objects, reference-counted resources, or other values with nontrivial destruction behavior, it affects resource lifetime.

It follows that `remove_index()` should not be interpreted as guaranteeing immediate release of the removed elements' resources.

The operation also does not make multidimensional arrays into efficient general-purpose insertion/deletion sequences: moving elements and maintaining compatible geometry can remain costly.

Version 0.17 extended `remove_index()` to views in addition to owned arrays. That makes the method particularly relevant for generic algorithms operating on array-reference interfaces, but it also means that the logical mutation can be local to the view rather than redefining the shape of an independently held parent array.

The important ownership distinction is between removing elements from the visible indexed region and ending their storage lifetime. The method performs the former without guaranteeing the latter.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`

`https://github.com/rust-ndarray/ndarray/blob/master/RELEASES.md`

## 2. Broadcasting, slicing, and indexing semantics

### 2.1 Binary arithmetic does not provide NumPy-style co-broadcasting

A significant difference between `ndarray` and NumPy concerns how the output shape of an elementwise binary operation is determined.

NumPy commonly determines a mutually broadcastable result shape from both operands. `ndarray`'s ordinary binary operators instead support broadcasting the right-hand operand to the left-hand operand's shape.

This is an asymmetric contract.

Consider arrays with shapes:

- `a`: `[1, 3]`
- `b`: `[2, 1]`

NumPy can co-broadcast these arrays to `[2, 3]`. In `ndarray`, a direct expression using `&a + &b` does not independently derive that common output shape. The right-hand operand cannot be broadcast to the left-hand operand's `[1, 3]` shape.

The desired result can instead be expressed through explicit views:

```
use ndarray::Array2;

let a = Array2::<f64>::ones((1, 3));
let b = Array2::<f64>::ones((2, 1));

let av = a.broadcast((2, 3)).unwrap();
let bv = b.broadcast((2, 3)).unwrap();

let result = &av + &bv;

assert_eq!(result.dim(), (2, 3));
```

`Zip::and_broadcast()` follows a related principle. The initial producer determines the traversal shape; an additional producer may be broadcast to that shape, but `Zip` does not automatically enlarge its existing traversal dimensions to accommodate a new operand.

This distinction affects the implementation of generalized elementwise kernels, tensor-algebra expressions, and APIs receiving operands with heterogeneous shapes.

A useful architectural approach is to separate broadcast-shape resolution from broadcasted execution. Once a common target geometry has been established and validated, explicit views and `Zip` can execute the operation without materializing repeated input elements.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/ndarray/0.17.2/ndarray/doc/ndarray_for_numpy_users/`

`https://docs.rs/ndarray/0.17.2/ndarray/struct.Zip.html`

### 2.2 Negative-step slicing selects a range before reversing it

Negative-step slicing in `ndarray` is not identical to Python or NumPy slice notation.

Its semantics can be understood as two operations:

1. Select the specified half-open range using the ordinary range boundaries.
2. Traverse the selected region from the back when the step is negative.

For example:

```
use ndarray::{array, s};

let a = array![0, 1, 2, 3];

assert_eq!(
    a.slice(s![1..3;-1]),
    array![2, 1]
);

assert_eq!(
    a.slice(s![1..;-2]),
    array![3, 1]
);
```

The first slice selects positions 1 and 2, then reverses them. The second selects positions 1 through 3 and walks backward in steps of two.

This differs from NumPy's interpretation of `start:stop:negative_step`, where the starting coordinate itself participates in the reverse traversal.

The distinction matters when translating Python indexing expressions, constructing stencils, or implementing coordinate transformations that depend on exact endpoint behavior.

Negative indexing and negative stepping are separate mechanisms: a negative range bound denotes an offset from the end of an axis, while a negative step determines traversal direction within the selected range.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/macro.s.html`

### 2.3 General advanced indexing and boolean-mask selection

`ndarray` supports several indexing models, but they do not amount to the full advanced-indexing system offered by NumPy.

Ordinary slicing produces views of regularly describable regions. `select(Axis, indices)` gathers arbitrary positions along one chosen axis and constructs a new array.

The latter is useful for reordered rows, sampled columns, or repeated index selections, but it is not a universal coordinate-gather operator.

In particular, the native NumPy-compatibility documentation identifies index-array indexing and boolean-mask indexing as unsupported capabilities.

For coordinate-dependent work, the available alternatives include indexed iteration, coordinate producers, elementwise predicates, and custom gathering or scattering logic.

A related semantic difference concerns comparison operators.

An expression such as `a == b` compares arrays for equality; it does not construct an array of elementwise comparison results in the same manner as NumPy.

For an elementwise mask, `mapv()` or `Zip::map_collect()` can be used:

```
use ndarray::{array, Zip};

let a = array![1, 2, 3];
let b = array![1, 4, 3];

let mask = Zip::from(&a)
    .and(&b)
    .map_collect(|x, y| x == y);

assert_eq!(mask, array![true, false, true]);
```

This produces a Boolean array with the same traversal shape as its input producers.

It does not, by itself, provide NumPy-style `a[mask]` selection. Converting predicate results into a compact selection requires an additional gathering or filtering operation.

An important performance distinction follows: a Boolean mask can be an appropriate persistent representation when subsequent operations reuse it, whereas a fused predicate inside `Zip` can avoid allocating a mask when the only goal is to accumulate, transform, or conditionally update values.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/doc/ndarray_for_numpy_users/`

## 3. Additional generic interfaces, lifetimes, and array access

### 3.1 `AsArray`: generic conversion into borrowed array views

`AsArray<'a, A, D>` provides a conversion-oriented interface for functions that accept arguments capable of becoming `ArrayView<'a, A, D>`.

Its definition is:

```
pub trait AsArray<'a, A: 'a, D = Ix1>:
    Into<ArrayView<'a, A, D>>
where
    D: Dimension,
{}
```

The trait is implemented for compatible types that support conversion into the corresponding array view.

For example:

```
use ndarray::AsArray;

fn sum<'a, V: AsArray<'a, f64>>(data: V) -> f64 {
    let view = data.into();
    view.sum()
}

assert_eq!(sum(&[1., 2., 3.]), 6.);
```

This is distinct from the `ArrayRef` mechanism discussed in the original guide.

`&ArrayRef<A, D>` supplies a storage-independent interface to an existing array representation. `AsArray` instead defines an argument-conversion boundary, allowing ordinary Rust containers or other compatible inputs to be normalized into a borrowed array view.

The distinction affects API design:

| Interface                | Principal abstraction                                                |
| ------------------------ | -------------------------------------------------------------------- |
| `&ArrayRef<A, D>`        | Reference to an existing multidimensional array                      |
| `AsArray<'a, A, D>`      | Input convertible into a borrowed array view                         |
| `AsRef<LayoutRef<A, D>>` | Access to multidimensional geometry without requiring element access |
| `IntoNdProducer`         | Input convertible into a multidimensional work producer              |

`AsArray` is valuable when APIs should accept existing slices and array views without allocating temporary arrays. It is not a replacement for `ArrayRef` in every generic numerical algorithm: it specifically requires a compatible conversion into `ArrayView` and does not itself express mutable element access.

The trait is not object-safe, so its intended use is through generic bounds rather than as a dynamically dispatched trait object.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/trait.AsArray.html`

### 3.2 `IndexLonger`: element references that outlive temporary view descriptors

`IndexLonger<I>` exposes three methods:

- `index(self, index)` for reference-returning indexed access.
- `get(self, index)` for optional reference-returning access.
- Unsafe `uget(self, index)` for unchecked access.

These resemble ordinary array indexing, but they make a different lifetime guarantee.

When indexing through an ordinary borrowed view, Rust generally ties the returned element reference to the borrow used for the indexing expression.

`IndexLonger` instead permits a returned reference to retain the lifetime associated with the view's underlying data.

```
use ndarray::{ArrayView, IndexLonger};

let data = [0.0; 256];

let element = {
    let view = ArrayView::from(&data[..])
        .into_shape_with_order((16, 16))
        .unwrap();

    IndexLonger::index(&view, [0, 1])
};

// The view descriptor no longer exists.
assert_eq!(element, &0.0);
```

This is useful for algorithms that construct temporary multidimensional views but need to return references into a longer-lived backing allocation.

The distinction is especially relevant to generic indexing utilities and borrowed data structures.

#### Mutable-view semantics

`ArrayViewMut` implements the trait differently from immutable `ArrayView`.

Obtaining a longer-lived mutable element reference consumes the mutable view. This is necessary to preserve exclusive borrowing and prevent continued access through a view that has effectively transferred its mutable reference.

The operation therefore does not weaken Rust's aliasing guarantees; it exposes a lifetime relationship that the ordinary indexing interface cannot express.

`IndexLonger` is also distinct from `CowArray`: it neither creates ownership nor provides copy-on-write behavior.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/trait.IndexLonger.html`

### 3.3 Unsafe indexing has additional ownership and build-mode constraints

The original reference describes `uget()` and `uget_mut()` as unsafe unchecked indexing methods. Two details deserve further qualification.

First, the official documentation notes that these methods are unchecked only in non-debug builds of `ndarray`. They should not be assumed to eliminate every bounds check in all build configurations.

Second, the unsafe contract for mutable access is stricter than index validity alone.

For `uget_mut()`, the caller must ensure that the data is uniquely held by the array. This condition is naturally satisfied by appropriate `Array` and `ArrayViewMut` representations, but is not automatically guaranteed for `ArcArray` or `CowArray`.

These constraints also apply to related unsafe mutation methods such as `uswap()`.

This matters because unsafe access bypasses protections that would otherwise enforce unique mutation or prepare shared storage for copy-on-write access.

The existence of an `unsafe` mutable element method is therefore not evidence that calling it on a shared representation is valid merely because the indices are in range.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`

## 4. Additional multidimensional traversal and mapping capabilities

### 4.1 Virtual coordinate producers: `indices()` and `indices_of()`

`ndarray` includes coordinate-generating producers that expose multidimensional index positions without constructing an array containing those coordinates.

`indices(shape)` constructs an index producer for a specified shape. `indices_of(array)` provides the coordinates of an existing array.

Both use the `Indices<D>` producer infrastructure.

These can be useful when computing values from their position rather than exclusively from input elements.

For example, coordinate-dependent kernels may need to evaluate a function of row, column, or higher-dimensional indices, or combine positions with values through a `Zip`.

The distinction from a coordinate array created by `meshgrid()` is that an index producer represents a traversal over coordinates, not a set of materialized coordinate-valued arrays.

In many cases, `Zip::indexed()` provides the more direct interface because it incorporates coordinate production into the traversal itself.

The upstream documentation recommends preferring higher-order operations and ordinary elementwise traversal over indexed algorithms where possible. Generating and processing index tuples introduces work that may be unnecessary for computations driven solely by element values.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/fn.indices_of.html`

`https://docs.rs/ndarray/0.17.2/ndarray/iter/struct.Indices.html`

### 4.2 Producer dimensionality is not necessarily item dimensionality

A significant feature of `NdProducer` is that its dimensionality describes the geometry of produced items.

Those items are not required to be individual scalars.

For example, a producer iterating over rows of a matrix has a one-dimensional traversal geometry, even though each produced item is itself a one-dimensional array view.

This makes the following computation possible:

```
use ndarray::{array, Array1, Zip};

let matrix = array![
    [1, 2, 3],
    [4, 5, 6]
];

let row_sums: Array1<i32> =
    Zip::from(matrix.rows())
        .map_collect(|row| row.sum());

assert_eq!(row_sums, array![6, 15]);
```

The output has one element per produced row, not one element per scalar in the original matrix.

This distinction generalizes to chunk producers, axis iterators, and window producers.

It explains why producers yielding multidimensional views can be composed with producers yielding scalar references, provided their producer traversal shapes agree.

Such composition can replace manually coordinated outer loops over blocks or subarrays.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.Zip.html`

### 4.3 `Zip` producer arity and output-layout behavior

The published `Zip` implementations support tuples containing up to six producers.

A `Zip::indexed()` traversal consumes one of those producer positions for the coordinate producer.

Consequently, operations requiring more than six independent producer inputs cannot be represented by indefinitely chaining `.and()`.

This is an interface constraint rather than a general computational limitation. Larger computations can instead be decomposed into multiple traversals, encapsulated within custom producers, or expressed using higher-level operations.

A separate, easily overlooked characteristic concerns `Zip::map_collect()`.

The documentation states that when all inputs have the same C-order or F-order layout, the corresponding order is preserved in the output.

Thus, creating a result through `Zip` is not necessarily equivalent to allocating a standard-row-major array and filling it in logical order.

This behavior can preserve layout compatibility and improve locality when producer layouts agree, but it should not be mistaken for a guarantee that every mixed-layout operation produces a particular storage order.

`Zip::indexed()` also carries documented overhead relative to non-indexed traversal.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.Zip.html`

### 4.4 `map_mut()`: simultaneous source mutation and result construction

`map_mut()` occupies a distinct position among the mapping operations.

Its signature is conceptually:

```
pub fn map_mut<'a, B, F>(
    &'a mut self,
    f: F,
) -> Array<B, D>
where
    F: FnMut(&'a mut A) -> B,
    A: 'a;
```

Unlike `map()`, the closure receives mutable references to source elements.

Unlike `map_inplace()`, it constructs a separate output array.

This allows an operation to update its input while simultaneously producing a derived output.

Potential uses include updating per-element state while collecting diagnostics, performing stateful transformations that retain the mutated input, or deriving a new representation from mutable records without a second traversal.

The original and derived values remain distinct representations, and the result allocation still exists.

The documented visitation order is arbitrary. Algorithms using `map_mut()` should therefore avoid depending on a particular sequence of side effects across elements.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`

### 4.5 Integer, Boolean, and bitwise element operations

The numerical discussion in the original reference primarily emphasizes floating-point and conventional arithmetic.

`ndarray` also provides elementwise operator implementations covering remainder, bitwise operations, shifts, and corresponding assignment variants.

These include `%`, `&`, `|`, `^`, `<<`, and `>>`, subject to the element type's trait implementations and the supported array-operand combinations.

This is useful for:

- Integer grids and discrete numerical algorithms.
- Bitfield transformations and packed flags.
- Boolean predicate combinations.
- Elementwise index and mask calculations.

The usual array broadcasting and allocation considerations continue to apply. Bitwise array operators do not introduce a separate bit-packed tensor representation; a Boolean `Array` remains an array of Boolean elements.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



`https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayBase.html`

### 4.6 Approximate equality through the `approx` feature

The optional `approx` feature integrates `ndarray` arrays with the approximate-equality traits from the `approx` crate.

Relevant comparison models include:

| Comparison   | Numerical interpretation                                                           |
| ------------ | ---------------------------------------------------------------------------------- |
| `AbsDiffEq`  | Absolute difference within an epsilon tolerance                                    |
| `RelativeEq` | Relative-error comparison with an absolute-tolerance component                     |
| `UlpsEq`     | Comparison based on representable floating-point separation, with epsilon handling |

These address different numerical error models and should not be treated as equivalent.

Approximate equality is especially useful for testing algorithms whose results are affected by floating-point rounding, summation order, or alternative numerical backends.

Unlike elementwise predicate generation, array approximate-equality operations answer whether the compared arrays satisfy the specified tolerance relation.

The feature integrates trait implementations; it does not automatically choose domain-appropriate tolerances for a numerical computation.

The supported bounds and behavior also depend on the element type's `approx` implementations.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



`https://docs.rs/ndarray/0.17.2/ndarray/struct.ArrayRef.html`

`https://docs.rs/approx/0.5.1/approx/`

## 5. Serde serialization: logical contents versus physical representation

The original guide identifies the `serde` feature but does not describe the serialization contract.

This distinction matters because `ndarray` arrays have both a logical multidimensional representation and an underlying physical layout.

### 5.1 Serialized structure

The published 0.17.2 implementation serializes an array into a versioned structure containing three fields:

| Field  | Meaning                                               |
| ------ | ----------------------------------------------------- |
| `v`    | Array serialization format version                    |
| `dim`  | Dimensional shape                                     |
| `data` | Sequence of array elements in logical iteration order |

The serializer obtains the elements through `self.iter()`.

It does not serialize the original stride vector, base-pointer offset, memory-allocation identity, or ownership-sharing relationships.

Consequently, two arrays with the same logical shape and elements but different physical layouts have equivalent array-level serialized contents.

The exact byte representation also depends on the chosen Serde serializer; the field structure does not imply that every serialization format produces identical bytes.

### 5.2 Deserialization and reconstruction

The deserializer validates the serialized format version and reconstructs an owned array using the dimensions and element sequence, including a size-consistency check.

It does not reconstruct the original physical layout.

For example, serializing a transposed view and subsequently deserializing it can recover the transposed logical values while producing an array with standard contiguous storage rather than the original transposed stride geometry.

Similarly, serializing a reference-counted array records its contents, not the original shared-allocation relationship.

This yields a useful distinction between three types of persistence:

| Requirement                                                 | Relevant mechanism                                                           |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------- |
| Preserve shape and logical element values                   | Serde                                                                        |
| Preserve NumPy-compatible array contents and dtype metadata | `.npy` / `.npz` through `ndarray-npy`                                        |
| Preserve or share compatible existing physical buffers      | Explicit raw-buffer, mapped-memory, or specialized interoperation mechanisms |

Serde is appropriate for logical data persistence and structured exchange. It is not a promise of zero-copy reconstruction or restoration of original physical layouts.

These conclusions are derived directly from the 0.17.2 serialization implementation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



Source:

`https://docs.rs/ndarray/0.17.2/src/ndarray/array_serde.rs.html`

## 6. Additional companion libraries and architectural integration boundaries

The following packages extend the computations and representations available around `ndarray`.

They are independent crates with their own versioning, features, dependency constraints, and runtime behavior.

### 6.1 General tensor contractions: `einsum-ndarray`

Verified release: 0.1.1

The `einsum-ndarray` crate provides Einstein-summation operations on dynamically shaped `ndarray` arrays.

Einstein notation expresses combinations of axis permutation, elementwise multiplication, reduction, diagonal extraction, and tensor contraction through labeled dimensions.

For example:

```
ij,jk->ik
```

represents a matrix product.

More general expressions can describe operations over higher-rank tensors without requiring the caller to manually implement every combination of reshaping, axis permutation, and pairwise reduction.

The crate supports explicit and implicit outputs, repeated-index diagonals, ellipsis-based broadcasting, scalar operands, and zero-length axes.

This extends the native `dot()` capability, which does not provide general higher-rank tensor contraction.

#### Planning and reuse

The interface distinguishes immediate evaluation from preparing reusable contractions.

`einsum()` parses, plans, and evaluates an expression.

`einsum_path()` computes a contraction path from operand shapes.

`EinsumPlan` binds a parsed contraction to fixed operand shapes, permitting its execution to be reused without repeatedly interpreting the original expression.

Its planning structures include `ContractionPath`, `ContractionStep`, and a selectable `Strategy`.

This matters for repeated tensor operations with invariant shapes, where the preparation stage can be separated from numerical execution.

#### Arithmetic and numerical semantics

The documented implementation uses wrapping arithmetic for integer addition and multiplication in all build profiles. Other supported element types use their `ndarray` arithmetic behavior.

This is a consequential difference from assuming all contraction operations inherit the usual debug-build integer-overflow behavior of arbitrary Rust arithmetic.

Furthermore, a reusable plan binds operand shapes. Passing incompatible runtime shapes is an error, not an automatic plan adaptation.

Because the package is in its early `0.1.x` series, its API stability and contraction-strategy behavior should be treated separately from the maturity of core `ndarray`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source:

`https://docs.rs/einsum-ndarray/0.1.1/einsum_ndarray/`

### 6.2 Sparse matrix representations and operations: `sprs`

Verified release: 0.11.6

`ndarray`'s native representation is dense and strided. `sprs` supplies complementary sparse matrix data structures and algorithms.

Its principal representations are compressed sparse row (CSR), compressed sparse column (CSC), coordinate/triplet matrices, and sparse vectors.

These representations store nonzero elements and associated indexing structures rather than representing every logical matrix element as an ordinary dense array entry.

The library provides sparse matrix/vector and matrix/matrix products, sparse addition and subtraction, sparse/dense operations, and iteration over nonzero structures.

It also includes selected numerical algorithms, such as sparse triangular solves with dense right-hand sides and a simple sparse Cholesky implementation.

The latter requires an explicit LGPL-related licensing opt-in.

#### Interoperability boundary

The important architectural distinction is that sparse matrix multiplication is not generally equivalent to applying a dense `ndarray` kernel to a specially strided view.

Sparse storage changes the data structure and computational complexity.

Consequently, `sprs` supplies a separate sparse model rather than merely adding a sparse option to `ArrayBase`.

The declared dependency range in `sprs` 0.11.6 accepts `ndarray >=0.15.0, <0.18`, which includes the target `ndarray` 0.17.2.

This compatibility matters because older releases of sparse-array companions have historically depended on earlier semver-incompatible `ndarray` versions.

The availability of sparse/dense operations makes `sprs` relevant for algorithms containing both sparse system operators and dense vectors or intermediate arrays. However, data exchange still requires matching dimensions, scalar types, and the relevant operation implementations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Source:

`https://docs.rs/sprs/0.11.6/sprs/`

### 6.3 Spectral computation: `ndrustfft`

Verified release: 0.6.2

`ndrustfft` provides multidimensional-array interfaces to the separate RustFFT, RealFFT, and RustDCT computational engines.

Its functional coverage includes:

| Transform family            | Principal operations                                   |
| --------------------------- | ------------------------------------------------------ |
| Complex FFT                 | `ndfft`, `ndifft`                                      |
| In-place complex FFT        | `ndfft_inplace`, `ndifft_inplace`                      |
| Real-to-complex FFT         | `ndfft_r2c`                                            |
| Complex-to-real inverse FFT | `ndifft_r2c`                                           |
| Discrete cosine transforms  | DCT types I–IV                                         |
| Parallel execution          | Corresponding `_par` variants for supported transforms |

A key mechanism is the use of reusable transform handlers such as `FftHandler`, `R2cFftHandler`, and `DctHandler`.

These encapsulate transform preparation and can support repeated transformations without rebuilding the underlying planning state for every array.

#### Layout and copying

Although input arrays use `ndarray` representations, the underlying transform engines impose their own access and layout requirements.

The `ndrustfft` documentation explicitly notes that some axis choices require temporary copies.

This means that a perfectly valid strided `ndarray` view may still incur materialization during a specialized FFT operation.

It is therefore important to distinguish the following:

- A transformation expressed as an `ndarray` view can be zero-copy.
- An FFT consuming that view may have additional layout requirements.
- The complete operation is zero-copy only when both representations and algorithms permit it.

#### Normalization conventions

`ndrustfft` also introduces its own normalization behavior.

The underlying RustFFT/RealFFT/RustDCT engines do not uniformly normalize their transforms. `ndrustfft` applies SciPy-like normalization conventions by default and provides configurable normalization behavior.

Numerical results can therefore differ by scale from a direct call to an underlying transform engine or another numerical library using a different convention.

The `parallel` feature uses `ndarray`/Rayon parallelism and is enabled by default. The package also exposes architecture-related features forwarded to the underlying FFT implementation.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source:

`https://docs.rs/ndrustfft/0.6.2/ndrustfft/`

### 6.4 Apache Arrow interoperation: `ndarrow`

Verified release: 0.0.4

`ndarrow` provides an explicit bridge between the Apache Arrow memory model and `ndarray`.

Its primary capabilities are:

Arrow to ndarray: Borrow compatible Arrow buffers as `ndarray` views without copying element data.

ndarray to Arrow: Transfer compatible owned `ndarray` buffers into Arrow arrays through ownership movement.

The distinction is between borrowing Arrow-owned memory and transferring ownership of an `ndarray` allocation.

For simple compatible primitive arrays, both can avoid allocating and copying a second element buffer.

#### Conversion interfaces

The central public traits include `AsNdarray` and `IntoArrow`.

```
use arrow_array::Float64Array;
use ndarrow::{AsNdarray, IntoArrow};
use ndarray::Array1;

let arrow = Float64Array::from(
    vec![1.0, 2.0, 3.0]
);

let view = arrow.as_ndarray().unwrap();

assert_eq!(view[1], 2.0);

let owned = Array1::from_vec(
    vec![4.0, 5.0, 6.0]
);

let arrow_owned: Float64Array =
    owned.into_arrow().unwrap();
```

The first operation borrows Arrow storage. The second transfers a compatible owned buffer.

Both differ from serializing array contents to an intermediate format.

#### Null semantics

Apache Arrow arrays can contain a separate validity bitmap. Ordinary `ndarray` arrays do not inherently have such a bitmap.

`ndarrow` therefore exposes explicit choices:

| Conversion path          | Null behavior                                                                       |
| ------------------------ | ----------------------------------------------------------------------------------- |
| `as_ndarray()`           | Validates the absence of nulls and returns an error when invalid values are present |
| `as_ndarray_unchecked()` | Unsafe path where the caller guarantees that null-handling preconditions hold       |
| `as_ndarray_masked()`    | Returns the values view alongside validity information                              |

This enables nullable columnar data to participate in numerical computations without necessarily replacing nulls with sentinel values or allocating a second numeric array.

The masked result does not make ordinary `ndarray` arithmetic automatically null-aware. The validity information must still be incorporated into the algorithm.

For nested representations, validity can be more complicated. In particular, a fixed-size-list conversion may preserve outer-row validity while rejecting unsupported nulls within individual list elements.

Thus, Arrow null handling remains an explicit integration contract rather than a native `ndarray` feature.

#### Tensor and complex representations

The package goes beyond primitive one-dimensional arrays.

Its documented capabilities include fixed-size-list-to-matrix views, fixed-shape tensor representations, variable-shape tensor batches, complex-valued extension representations, and CSR sparse-array exchange facilities.

These provide representation-level interfaces for multidimensional numerical data within Arrow-based systems.

For example, an Arrow fixed-shape tensor representation can communicate tensor geometry while exposing compatible values through `ArrayViewD`.

Variable-shaped tensor batches require different treatment because each row can carry different tensor dimensions.

#### Copying boundaries

The crate separates zero-copy conversion mechanisms from explicit helpers that may allocate.

These include numerical type casts, null filling, non-null compaction, CSR densification, and layout normalization.

For multidimensional owned arrays, zero-copy outbound conversion depends on compatible physical layout; nonstandard layouts may require reordering.

This prevents a common misunderstanding: zero-copy interoperability does not imply that every preparation, conversion, or downstream numerical operation is zero-copy.

`ndarrow` is an early `0.0.x` project. Its own documentation warns that its API remains under active development and should be version-pinned where a stable interface contract is important.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Sources:

`https://docs.rs/ndarrow/0.0.4/ndarrow/`

`https://docs.rs/ndarrow/0.0.4/ndarrow/inbound/index.html`

`https://docs.rs/ndarrow/0.0.4/ndarrow/helpers/index.html`

### 6.5 Native-Rust linear algebra interoperation: `faer` and `faer-ext`

Verified packages: `faer` 0.24.4; `faer-ext` 0.8.0

`faer` provides a substantial native-Rust numerical linear algebra system, with dense and sparse matrix representations and decomposition algorithms.

It is a distinct numerical library rather than an additional backend automatically selected by `ndarray`.

The companion `faer-ext` supplies interoperability with several external matrix representations.

Its published conversion traits include:

| Trait          | Role                                                                  |
| -------------- | --------------------------------------------------------------------- |
| `IntoFaer`     | Convert compatible external matrix views to `faer` representations    |
| `IntoNdarray`  | Convert compatible external matrix views to `ndarray` representations |
| `IntoNalgebra` | Convert compatible views to `nalgebra` representations                |

These interfaces make it possible to separate array-oriented data transformations from numerical routines that operate through another library's preferred matrix abstractions.

The main architectural benefit is that a system need not necessarily standardize all matrix operations on one owned representation merely to access a particular numerical algorithm.

However, view conversion and owned-data conversion are different operations. Representation compatibility depends on element types, dimensions, strides, ownership, and the enabled integration features.

The availability of a view conversion trait does not establish that every possible owned matrix can be transferred without copying.

`faer` also has independent parallel-execution configuration, so its threading behavior should not be conflated with `ndarray`'s Rayon traversal.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



`https://docs.rs/faer-ext/0.8.0/faer_ext/`

`https://docs.rs/faer/0.24.4/faer/`

### 6.6 Extended ndarray-native numerical methods: `nabled`

Verified release: 0.0.11

`nabled` is an independently developed numerical library organized around `ndarray` representations.

It provides a wider algorithmic surface than the basic native numerical operations in core `ndarray`.

Its documented functional families include:

| Family                       | Examples                                                                      |
| ---------------------------- | ----------------------------------------------------------------------------- |
| Matrix decompositions        | SVD, QR, LU, Cholesky, eigen, Schur, polar                                    |
| Matrix functions             | Exponential, logarithm, powers, matrix sign                                   |
| Linear systems               | Triangular solves, Sylvester and Lyapunov equations                           |
| Iterative methods            | Krylov-style solvers, preconditioning, sparse numerical workflows             |
| Statistical computation      | Covariance, correlation, PCA, regression                                      |
| Numerical differentiation    | Jacobian, gradient, and Hessian-related methods                               |
| Vector and tensor operations | Norms, similarities, pairwise and batched computations                        |
| Embedding computation        | Exact search, ranking, normalization, quantization, and evaluation primitives |

The 0.0.11 release uses a modular architecture in which the facade crate defaults principally to the linear algebra domain. Other capabilities require enabling the relevant feature or using specific component crates.

#### Backend selection

`nabled` documents optional support for BLAS/LAPACK providers, selected Rayon-accelerated kernels, WGPU execution, and MAGMA-backed operations.

These are not universal acceleration mechanisms.

Their availability is dependent on the selected feature configuration, supported element type, target architecture, and operation family.

For example, its documented WGPU pathways include native `f32` execution, while native `f64` execution depends on shader capabilities and can fall back to CPU execution.

This is an important boundary: `nabled` can extend or accelerate computations involving `ndarray` data, but does not change core `ndarray` into a general GPU tensor library.

#### Arrow integration

An optional `arrow` feature integrates with `ndarrow`, allowing Arrow-facing inputs to reach selected numerical workflows while preserving the separation between the columnar representation and the ndarray-based execution layer.

This illustrates a compositional architecture:

Arrow memory representation → compatible `ndarray` view → numerical algorithm → compatible output conversion.

The numerical execution layer need not implement its own Arrow parser or buffer ownership model.

However, the overall workflow remains subject to the same compatibility and materialization constraints discussed for `ndarrow`.

`nabled` explicitly identifies itself as under active development and warns that its pre-1.0 APIs may change. Its extensive documented capability surface should therefore not be confused with a guarantee of mature, stable behavior across all algorithm families or backends.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs



Source:

`https://docs.rs/nabled/0.0.11/nabled/`

## 7. Cross-cutting implications of the additional capabilities

The mechanisms in this addendum reveal several further architectural distinctions that complement the original reference.

### 7.1 Shape transformation and physical execution are separate concerns

`merge_axes()`, `squeeze()`, `collapse_axis()`, and the various view-oriented slicing operations allow array geometry to change without necessarily moving elements.

This can eliminate custom reshaping loops and redundant array allocations.

However, a subsequent computation might impose stricter requirements.

A BLAS routine, FFT implementation, Arrow conversion, or other specialized numerical kernel may require contiguous or otherwise compatible buffers.

Therefore, a useful evaluation of an array transformation considers not only its immediate allocation cost, but also the resulting layout and the requirements of subsequent consumers.

A metadata-only transformation is advantageous when its resulting layout is accepted by the rest of the computation.

### 7.2 Logical multidimensional equivalence is not storage equivalence

Two arrays can have identical shapes and element values while differing in strides, allocation ownership, and physical memory order.

This distinction appears in several newly covered features:

| Mechanism                      | Preserved semantics                                 | What is not necessarily preserved           |
| ------------------------------ | --------------------------------------------------- | ------------------------------------------- |
| `squeeze()`                    | Represented elements                                | Original rank                               |
| `merge_axes()`                 | Element traversal along merged axes                 | Original axis extents                       |
| Serde                          | Logical shape and values                            | Original strides and storage ownership      |
| Arrow conversion               | Compatible element values and agreed representation | Arbitrary nonstandard layout                |
| Numerical companion algorithms | Mathematical result under documented semantics      | Input layout, temporary-allocation behavior |

This is why representation-preserving transformations and format conversions should not be treated interchangeably.

### 7.3 Mutable array geometry does not imply element destruction

`remove_index()` demonstrates that removing a value from an array's logical indexing space can differ from removing its ownership from the backing allocation.

This distinction is most important for arrays with nontrivial element types.

Generic array algorithms should therefore separate assumptions about shape changes from assumptions about element lifetime, especially when using owned arrays, mutable views, or shared representations.

### 7.4 Specialized engines complement, rather than replace, multidimensional traversal

The additional ecosystem packages cover capability families deliberately outside the primary `ndarray` abstraction.

`einsum-ndarray` addresses general tensor contractions. `sprs` supplies sparse representations and sparse numerical kernels. `ndrustfft` handles spectral computation. `faer` and `nabled` provide larger numerical algorithm families. `ndarrow` supplies columnar-memory interoperability.

Their value is not solely additional mathematical functionality.

They can also eliminate separate implementations of contraction planning, sparse indexing, transform preparation, buffer interoperability, and specialized numerical execution.

At the same time, every integration introduces its own contracts concerning layout, supported types, allocation behavior, numerical semantics, threading, and dependency versions.

The strongest general architectural opportunity is to preserve `ndarray` as a flexible multidimensional representation and traversal layer while using specialized packages for computations or representations whose requirements extend beyond that layer.

This combination can retain the benefits of native views, generic array references, and fused traversal without requiring every advanced numerical operation to be implemented through bespoke `ndarray` loops.

## 8. Version and compatibility considerations

The following versions were verified during this research.

| Package          | Verified version | Relationship to ndarray 0.17.2                                |
| ---------------- | ---------------- | ------------------------------------------------------------- |
| `ndarray`        | 0.17.2           | Target core library                                           |
| `einsum-ndarray` | 0.1.1            | Built for the 0.17-era API; early-stage contraction interface |
| `sprs`           | 0.11.6           | Declared `ndarray` dependency range includes 0.17.2           |
| `ndrustfft`      | 0.6.2            | Declares `ndarray` 0.17 compatibility                         |
| `ndarrow`        | 0.0.4            | Built around `ndarray` 0.17 and Apache Arrow representations  |
| `faer`           | 0.24.4           | Separate numerical representation and execution library       |
| `faer-ext`       | 0.8.0            | Supplies feature-dependent matrix-view interoperability       |
| `nabled`         | 0.0.11           | Declares `ndarray` 0.17 compatibility; evolving modular API   |

Compatibility should be distinguished from the ability of every package to accept every array type or layout. Cargo dependency compatibility permits compatible versions to participate in the same build, but individual numerical and conversion interfaces retain their own type and representation requirements.

Some companions also introduce native backend dependencies or compiler requirements exceeding the minimum Rust version of core `ndarray`.

For early-stage companions in particular, published capabilities should be evaluated against the exact release rather than assuming a stable long-term API contract.

Taken together, these additions complete several meaningful areas left implicit by the original reference: exact NumPy-compatibility boundaries, more advanced metadata-only transformations, array-view conversion and lifetime interfaces, subtle execution constraints, persistence semantics, and the specialized numerical and data-interoperability ecosystem around `ndarray` 0.17.2.
