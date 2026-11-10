# Numerica (Rust): Comprehensive Library Capability Reference

Research date: November 9, 2026 | Target version: 3.0.1 | Language: Rust | License: MIT

Numerica is an open-source mathematical computation library developed by the Symbolica project. Its central purpose is to provide efficient, composable numerical and algebraic primitives for applications requiring exact arithmetic, controlled floating-point precision, automatic differentiation, finite-field computation, matrix algorithms, and numerical integration.

Unlike general-purpose numerical libraries organized primarily around arrays and linear algebra routines, Numerica is fundamentally organized around mathematical coefficient domains and their algebraic operations.

This distinction is important. Numerica allows algorithms to operate over integers, rational numbers, modular fields, arbitrary-precision floating-point numbers, and other mathematical structures while preserving the algebraic properties appropriate to each domain.

Its capabilities are especially relevant to computational algebra, exact and hybrid numerical algorithms, mathematical modeling, scientific computation, and the construction of higher-level mathematical software.

Primary resources

- Source repository: [https://github.com/symbolica-dev/numerica](https://github.com/symbolica-dev/numerica?utm_source=chatgpt.com)
- Rust API documentation: [https://docs.rs/numerica/latest/numerica/](https://docs.rs/numerica/latest/numerica/?utm_source=chatgpt.com)
- Crate distribution: [https://crates.io/crates/numerica](https://crates.io/crates/numerica?utm_source=chatgpt.com)
- Symbolica project: [https://symbolica.io/](https://symbolica.io/?utm_source=chatgpt.com)
- Symbolica 3.0 release documentation: [https://symbolica.io/posts/symbolica_3_0_release/](https://symbolica.io/posts/symbolica_3_0_release/?utm_source=chatgpt.com)

Numerica was extracted from Symbolica and initially released separately in November 2025. Version 3.0 introduced significant improvements to arithmetic backends, polynomial kernels, interval arithmetic, and domain abstractions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra+2



## 1. Architecture and capability map

Numerica consists of several closely related mathematical layers.

Mathematical applications and algorithms

Exact computation · Scientific modeling · Computational algebra · Numerical evaluation

Automatic differentiation

Dual and hyperdual numbers

Linear algebra

Dense and sparse matrices

Number theory

Reconstruction, factoring, PSLQ, LLL

Numerical integration

Adaptive VEGAS and multichannel grids

Algebraic domain framework

Set · Ring · EuclideanDomain · Field · OrderedRing · SampleableRing

Integer / Rational

Finite fields

Float / Complex

Arithmetic backends: GMP / Malachite and MPFR / Astro-float

This is a conceptual diagram, not a strict dependency graph. Individual algorithms impose more specific trait bounds, and some number types operate directly through Rust arithmetic traits.

### Capability summary

| Functional family         | Principal capabilities                                                                                          |
| ------------------------- | --------------------------------------------------------------------------------------------------------------- |
| Algebraic abstraction     | Generic rings, fields, Euclidean domains, ordered domains, sampling, and coefficient-specific operation kernels |
| Exact arithmetic          | Adaptive-width integers, arbitrary-precision arithmetic, exact fractions, and modular reconstruction            |
| Number theory             | GCD, prime testing, factorization, Chinese remainder theorem, integer relations, lattice reduction              |
| Finite fields             | Modular arithmetic, specialized representations, prime generation, primitive roots                              |
| Floating-point arithmetic | Multiprecision, compensated double precision, complex numbers, error propagation, interval arithmetic           |
| Automatic differentiation | Multivariable dual numbers, higher derivatives, mixed partials, configurable derivative structures              |
| Linear algebra            | Dense and sparse matrices, exact solving, inversion, determinants, basis reduction                              |
| Polynomial kernels        | Specialized multiplication, checked division, geometric-sequence evaluation                                     |
| Numerical integration     | Adaptive Monte Carlo integration, continuous and discrete sampling, multichannel integration                    |
| Combinatorics             | Combinations, distinct permutations, constrained partitions with multiplicities                                 |

The most distinctive architectural capabilities are the domain abstraction system, automatically resizing integer representation, modular reconstruction, high-order dual number generation, and precision-aware computation. Together, these make Numerica more than a collection of isolated arithmetic utilities.

## 2. Mathematical domain abstraction

### 2.1 Rings and elements are separate concepts

Numerica's most consequential design decision is to distinguish a ring object from its element type.

For example:

| Domain               | Mathematical structure | Element type              |
| -------------------- | ---------------------- | ------------------------- |
| Integers             | `Z`                    | `Integer`                 |
| Rational numbers     | `Q`                    | `Rational`                |
| Modular arithmetic   | `FiniteField<T>`       | `FiniteFieldElement<T>`   |
| Floating-point field | `FloatField<T>`        | Floating-point scalar `T` |

The ring object defines how elements are manipulated. This is particularly useful when the relevant arithmetic context is not intrinsic to an element's representation.

For instance, the integer value `3` does not need an accompanying domain parameter. But the modular element `3 mod 7` behaves differently from `3 mod 11`. The modulus belongs to the algebraic context.

Numerica's generic algorithms can therefore receive both a domain and elements belonging to that domain, rather than requiring every element to implement all arithmetic independently.

The domain documentation explicitly describes this separation and notes that most Symbolica mathematical structures are generic over the ring type.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/numerica/latest/numerica/domains/](https://docs.rs/numerica/latest/numerica/domains/?utm_source=chatgpt.com)

### 2.2 Core traits and their semantic differences

The central traits are:

| Trait              | Meaning and significance                                                            |
| ------------------ | ----------------------------------------------------------------------------------- |
| `Set`              | Defines the element domain                                                          |
| `Ring`             | Addition, multiplication, additive identities, and related ring operations          |
| `RingOps`          | Operations over owned and borrowed ring elements                                    |
| `EuclideanDomain`  | Quotients, remainders, and greatest common divisors                                 |
| `Field`            | Multiplicative inverses and division by nonzero elements                            |
| `OrderedRing`      | A mathematically meaningful total order compatible with arithmetic                  |
| `RealEmbedding`    | Supports comparisons through a distinguished real embedding                         |
| `SampleableRing`   | Adds domain-specific random sampling                                                |
| `Derivable`        | Supports differentiation within the algebraic structure                             |
| `UpgradeToField`   | Describes structures that can be extended to fields, such as integers to rationals  |
| `SelfRing`         | Represents domains whose elements already carry sufficient arithmetic context       |
| `InternalOrdering` | Provides an internal ordering, including where mathematical ordering does not exist |

These distinctions prevent several common mathematical modeling mistakes.

For example, a complex number can be ordered internally for deterministic data structures, but that does not make the complex numbers an ordered field. Similarly, being able to draw random elements from a domain does not mean that random sampling is inherent to its ring operations.

Numerica 3.0 explicitly separated mathematical ordering and random sampling from the basic ring abstraction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



### 2.3 Why the abstraction matters

Consider a matrix algorithm expressed over a field. Its implementation can manipulate rational numbers, finite-field elements, or suitable floating-point values through the corresponding arithmetic context.

This provides several architectural possibilities:

- Exact calculations can be performed with rational numbers rather than floating-point approximations.
- Modular variants of algorithms can reuse algebraic structure without introducing ad hoc modulus handling.
- Specialized coefficient domains can supply more efficient arithmetic implementations.
- Generic mathematical algorithms can express algebraic preconditions through trait bounds.

However, genericity does not imply that every algorithm works with every domain. Gaussian elimination, for example, requires suitable division behavior; an arbitrary ring does not guarantee inverses for nonzero elements.

Nor does it guarantee identical numerical behavior. An algorithm that is exact over rational numbers can become susceptible to conditioning and rounding over floating-point types.

### 2.4 Domain-specific polynomial kernels

Numerica 3.0 adds an important optimization layer through `Ring::kernels()` and associated kernel interfaces.

The `kernels` module exposes structures and traits such as:

- `RingKernels`
- `PolynomialKernels`
- `CheckedPolynomialDivisionKernels`
- `GeometricSequenceKernels`
- `DensePolynomialMulRequest`
- `ChunkedDensePolynomialMulRequest`
- `TotalDegreePolynomialMulRequest`
- `DensePolynomialExactDivisionRequest`
- `UnivariatePolynomialDivisionRequest`
- `GeometricSequenceStepRequest`
- `DivisionAttempt`

These provide a mechanism for mathematical coefficient domains to expose specialized bulk algorithms rather than requiring every higher-level polynomial implementation to assemble operations from individual scalar additions and multiplications.

The distinction matters for performance.

For ordinary elementwise operations, a generic `Ring` implementation is sufficient. For dense polynomial multiplication, however, the coefficient domain may have specialized approaches that avoid generic multiplication loops or exploit representation-specific optimizations.

A higher-level polynomial system can invoke those capabilities through the domain's kernel interface.

The checked-division interface is also consequential: polynomial division may fail to yield an exact quotient, and the kernel API represents that outcome rather than requiring callers to assume divisibility.

Architectural implication: Numerica allows arithmetic domains to carry optimized polynomial machinery without turning Numerica itself into a complete symbolic polynomial system.

Source: [https://docs.rs/numerica/latest/numerica/kernels/](https://docs.rs/numerica/latest/numerica/kernels/?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 3. Adaptive-width integers and exact number theory

Numerica's integer system is designed to combine inexpensive arithmetic for small values with arbitrary precision when values outgrow machine-sized representations.

### 3.1 Automatically upgrading and downgrading integers

The `Integer` type represents values using progressively larger storage:

- `Single`: machine-width signed integers, conventionally `i64`.
- `Double`: wider integers using `i128`.
- `Large`: arbitrary-precision integer storage.

Instead of requiring the caller to select a fixed-width integer or arbitrary-precision type in advance, arithmetic can move between representations.

The crucial benefit is avoiding arbitrary-precision allocation and operation costs for values that remain small, while preserving correctness when exact arithmetic generates large intermediate results.

This is useful for algorithms in which numbers are initially small but can grow unpredictably, such as rational elimination, polynomial arithmetic, combinatorial counting, determinant computation, and modular reconstruction.

The automatically resizing representation also introduces some considerations. Runtime cost becomes dependent on operand magnitude, intermediate arithmetic can change storage representation, and memory requirements are not constant.

Numerica 3.0 redesigned compact integer storage and introduced selectable arbitrary-precision backends.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



Source: [https://docs.rs/numerica/latest/numerica/domains/integer/](https://docs.rs/numerica/latest/numerica/domains/integer/?utm_source=chatgpt.com)

### 3.2 Number-theoretic operations

The integer facilities extend substantially beyond addition, multiplication, and division.

| Capability               | Operations and role                                                         |
| ------------------------ | --------------------------------------------------------------------------- |
| Divisibility             | GCD, extended GCD, LCM, quotient and remainder                              |
| Modular arithmetic       | Modular inverse, symmetric modular reduction                                |
| Reconstruction           | Chinese remainder theorem using Garner's algorithm                          |
| Prime testing            | Miller–Rabin with deterministic handling for supported machine-sized values |
| Factorization            | Integer factorization and elliptic-curve-method routines                    |
| Integer powers and roots | Exponentiation and truncated integer roots                                  |
| Combinatorics            | Factorials, binomial coefficients, multinomial coefficients                 |
| Arithmetic functions     | Euler's totient                                                             |
| Relation discovery       | PSLQ integer-relation solving                                               |

Some methods and signatures were documented in earlier releases and may have changed in 3.0.1. Their presence is supported by published API documentation, but the latest release's exact signatures should be checked before implementation.

Source: [https://docs.rs/numerica/latest/numerica/domains/integer/enum.Integer.html](https://docs.rs/numerica/latest/numerica/domains/integer/enum.Integer.html?utm_source=chatgpt.com)

### 3.3 Modular reconstruction with the Chinese remainder theorem

The Chinese remainder theorem (CRT) is particularly important for high-performance exact computation.

Suppose an algorithm needs an integer result \\(N\\), but computing directly with large integers is expensive.

Instead, it may calculate:

\\[ N \bmod p_1,\quad N \bmod p_2,\quad \ldots,\quad N \bmod p_k \\]

for pairwise coprime moduli.

The residues can then be combined to reconstruct an integer modulo

\\[ M=\prod\_{i=1}^{k}p_i. \\]

Once sufficient bounds on the original result are known, the reconstruction can uniquely identify the desired integer.

Numerica provides CRT-related arithmetic, including symmetric modular representatives and Garner-style reconstruction.

This can be combined with finite-field operations to implement exact algorithms using predominantly machine-sized modular arithmetic.

Important distinction: CRT reconstructs a residue class, not necessarily the correct unbounded integer. Recovering the intended exact result requires sufficient modulus coverage, reconstruction bounds, or independent verification.

### 3.4 Integer relation discovery with PSLQ

PSLQ searches for integer coefficients \\(a_i\\) such that

\\[ a_1x_1+a_2x_2+\cdots+a_nx_n\approx 0. \\]

For example, given approximate numerical values, it may discover a small-integer relationship among them.

Numerica exposes `Integer::solve_integer_relation`, with controls including the input values, numerical tolerance, iteration limit, coefficient bound, and an optional algorithm parameter.

Potential uses include recognizing rational or algebraic relationships from numerical results, discovering constants, reconstructing analytic formula candidates, and identifying linear dependencies.

A significant caveat is that numerical relation discovery is not mathematical proof. Spurious relations can emerge from insufficient precision or permissive tolerances. Candidate relationships should be validated independently, ideally with greater precision or exact arithmetic.

Source: [https://docs.rs/numerica/latest/numerica/domains/integer/enum.Integer.html](https://docs.rs/numerica/latest/numerica/domains/integer/enum.Integer.html?utm_source=chatgpt.com)

### 3.5 Lattice basis reduction

Numerica also provides integer lattice basis reduction through its vector facilities.

Lattice reduction seeks a shorter, more useful basis for the same lattice. Given basis vectors \\(b_1,\dots,b_n\\), reduction algorithms replace them with vectors that span the same integer lattice but typically have smaller lengths and improved geometric properties.

The upstream examples demonstrate `Vector::basis_reduction`.

Numerica 3.0 additionally introduced approximate integer lattice basis reduction.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub+1



Lattice reduction and PSLQ address related but distinct needs. PSLQ directly targets integer relations among numerical values, while lattice reduction provides a more general transformation of integer vector bases.

These mechanisms support rational approximation, integer-relation discovery, certain Diophantine constructions, and algebraic reconstruction algorithms. They do not constitute a general mixed-integer optimization solver.

## 4. Rational numbers and reconstruction

Numerica provides exact rational arithmetic through `Rational`, `Fraction`, and `FractionField`.

A fraction represents

\\[ \frac{a}{b}, \qquad b\ne 0, \\]

where the numerator and denominator belong to an appropriate underlying ring.

For ordinary rational arithmetic, both are integers, and operations preserve exactness rather than rounding to floating-point approximations.

### 4.1 Generic fraction fields

The generalized `FractionField<R>` abstraction is important because it is not limited conceptually to fractions of integers.

For an integral domain, fractions may also represent rational functions formed from polynomial numerators and denominators.

The construction allows algorithms to distinguish arithmetic in a ring from arithmetic in its field of fractions.

For example, polynomial division over a finite field and polynomial division over a fraction field can have materially different capabilities.

Numerica documents `FractionNormalization` as a separate trait, reflecting the need to consider representation normalization rather than assuming arbitrary fractions are automatically in canonical form.

Source: [https://docs.rs/numerica/latest/numerica/domains/rational/](https://docs.rs/numerica/latest/numerica/domains/rational/?utm_source=chatgpt.com)

### 4.2 Rational reconstruction

One of Numerica's less obvious capabilities is reconstructing exact rational results from modular information.

Suppose an unknown rational number is

\\[ r=\frac{a}{b}. \\]

For a modulus \\(p\\), assuming \\(b\\) is invertible modulo \\(p\\), its modular representation satisfies

\\[ r \equiv a b^{-1}\pmod p. \\]

Rational reconstruction attempts to recover a suitable numerator and denominator from a modular residue.

The documented API family includes `maximal_quotient_reconstruction` and `rational_reconstruction`.

The former is associated with Monagan's maximal-quotient rational reconstruction algorithm. The latter supports a process involving evaluations over finite fields, potentially using additional primes to recover an exact rational value.

These methods make possible a powerful architecture:

Exact rational problem

Evaluate over several finite fields

Combine modular results using CRT

Reconstruct rational coefficients

Verify the exact result

This can substantially reduce expensive fraction arithmetic when intermediate rational expressions would otherwise experience large numerator and denominator growth.

However, reconstruction can encounter unlucky primes, insufficient modulus coverage, or unsuitable denominator bounds. Numerica's reconstruction facilities should therefore be understood as algorithmic components with correctness conditions, rather than automatic proof of every reconstructed candidate.

Sources:

[https://docs.rs/numerica/latest/numerica/domains/rational/struct.Fraction.html](https://docs.rs/numerica/latest/numerica/domains/rational/struct.Fraction.html?utm_source=chatgpt.com)

[https://symbolica.io/posts/symbolica_3_0_release/](https://symbolica.io/posts/symbolica_3_0_release/?utm_source=chatgpt.com)

## 5. Finite-field and modular arithmetic

Finite-field computation is a major part of Numerica's identity.

The `finite_field` module includes specialized field implementations, modular element types, prime iterators, and helpers for primitive roots.

Source: [https://docs.rs/numerica/latest/numerica/domains/finite_field/](https://docs.rs/numerica/latest/numerica/domains/finite_field/?utm_source=chatgpt.com)

### 5.1 Modular rings versus finite fields

The mathematical distinction is important.

For a prime \\(p\\),

\\[ \mathbb{F}\_p=\mathbb{Z}/p\mathbb{Z} \\]

is a field, meaning every nonzero element has a multiplicative inverse.

For a composite modulus \\(m\\),

\\[ \mathbb{Z}/m\mathbb{Z} \\]

is generally only a ring. Some nonzero elements are not invertible.

Numerica's generic modular structures support arithmetic with odd moduli, including composite cases in some implementations. That does not mean field-dependent algorithms are valid for every modulus.

The `FiniteField` abstraction, `FiniteFieldCore` trait, and associated element types therefore need to be used with attention to the modulus requirements of the particular algorithm.

### 5.2 Specialized modular representations

Published types include:

| Type or alias           | Purpose                                   |
| ----------------------- | ----------------------------------------- |
| `FiniteField<T>`        | General modular arithmetic structure      |
| `FiniteFieldElement<T>` | Domain-associated modular element         |
| `Zp`                    | 32-bit modular field representation       |
| `Zp64`                  | 64-bit modular field representation       |
| `Z2`                    | Two-element field                         |
| `Mersenne32`            | Specialized arithmetic for \\(2^{31}-1\\) |
| `Mersenne64`            | Specialized arithmetic for \\(2^{61}-1\\) |

Specialized modulus representations can reduce the cost of modular arithmetic compared with generic large-integer operations.

The underlying implementation may exploit fixed-width arithmetic, Montgomery representations, or the structure of a particular prime.

The documented arbitrary-precision finite-field implementation uses Montgomery-form elements. Such representations keep repeated multiplication and reduction efficient, but they require conversion between internal and conventional integer representations.

Numerica supplies conversion facilities for this purpose.

### 5.3 Prime generation and roots

The module also provides:

- `PrimeIterator` and `PrimeIteratorU64`, for generating primes.
- `PrimitiveRootIterator`, for primitive-root computations.
- `SmoothPrimeIterator`, for generating primes whose \\(p-1\\) factors over a selected set of small primes.
- Constants describing selected smooth primes and their associated parameters.

Smooth primes are valuable in algorithms that need particular multiplicative group structures, including transform-oriented modular computation.

The presence of smooth-prime generation does not itself establish that Numerica provides a complete standalone FFT or NTT framework. Instead, it offers useful mathematical infrastructure for such algorithms.

### 5.4 Composition with exact reconstruction

Finite-field arithmetic is especially valuable when combined with:

1. Modular evaluation of a computationally expensive exact operation.
2. CRT reconstruction.
3. Rational reconstruction where denominators are involved.
4. Exact verification.

This technique supports modular approaches to determinants, polynomial arithmetic, rational function interpolation, and exact linear algebra.

Numerica's combination of finite fields, reconstruction, and coefficient-specific kernels makes it suitable as a foundation for such systems without requiring a separate implementation of each mathematical primitive.

## 6. Floating-point arithmetic and precision control

Numerica provides several distinct floating-point abstractions rather than treating machine `f64` as the universal numerical representation.

| Type                       | Main purpose                                                                         |
| -------------------------- | ------------------------------------------------------------------------------------ |
| `F64`                      | Wrapper around machine `f64`, with total-order and hashing semantics                 |
| `DoubleFloat`              | Approximately 106-bit significand precision using a compensated pair of `f64` values |
| `Float`                    | Arbitrary-precision floating-point arithmetic                                        |
| `Complex<T>`               | Complex arithmetic over an underlying numeric type                                   |
| `ErrorPropagatingFloat<T>` | Floating-point values carrying an estimated numerical error                          |
| `FloatField<T>`            | Algebraic field interface for floating-point types                                   |

Source: [https://docs.rs/numerica/latest/numerica/domains/float/](https://docs.rs/numerica/latest/numerica/domains/float/?utm_source=chatgpt.com)

### 6.1 `F64` and deterministic equality semantics

Raw IEEE-754 floating-point values have behaviors that complicate their use as keys in ordered and hashed collections.

In particular, NaN is normally unequal to itself, and positive and negative zero have distinct bit patterns.

Numerica's `F64` wrapper implements `Eq`, `Ord`, and `Hash` with explicit semantics:

- NaN values are treated as equal.
- Positive and negative zero are treated as equal.

This is useful for deterministic collections, canonicalized numerical values, and internal algorithmic ordering.

It is not equivalent to ordinary IEEE-754 comparison semantics. Applications that distinguish NaN payloads, signed zero, or particular bit patterns should not assume that these distinctions are preserved by `F64` equality.

### 6.2 `DoubleFloat`: compensated precision

`DoubleFloat` represents a number using two `f64` components.

Conceptually,

\\[ x=x\_{\mathrm{hi}}+x\_{\mathrm{lo}}, \\]

where the low component retains additional information beyond the high component.

This provides roughly twice the significand precision of a conventional double, without requiring a general arbitrary-precision representation for every operation.

The documented effective precision is approximately 106 bits.

Such arithmetic can be attractive when machine precision is insufficient but arbitrary precision introduces more complexity or overhead than desired.

However, compensated double precision is not interchangeable with arbitrary precision, and its accuracy depends on the implemented operations. It should not be assumed to deliver correctly rounded 106-bit results for all functions.

### 6.3 Arbitrary-precision `Float`

Numerica's `Float` supports multiprecision floating-point calculations and operations that track precision metadata.

Unlike a fixed-width floating-point number, a multiprecision value can be constructed with a selected number of bits.

This enables computations at hundreds or thousands of bits when an algorithm requires additional numerical resolution.

Typical reasons for higher precision include severe cancellation, reconstruction of small differences, numerically difficult special functions, and validation of mathematical results.

An important subtlety is that the floating-point value's configured precision is not the same thing as the accuracy of the result.

A calculation at 200-bit precision does not necessarily contain 200 accurate bits if its inputs or intermediate calculations have already lost information.

Numerica's documentation explicitly cautions that ordinary `Float` precision tracking can overestimate the precision of a result.

This motivates the separate error-propagating type.

### 6.4 `ErrorPropagatingFloat<T>`

`ErrorPropagatingFloat<T>` combines a numerical value with an estimate of its accuracy.

The published interface includes facilities such as:

- `new(value, precision)`
- `new_with_accuracy(value, accuracy)`
- `get_absolute_error()`
- `get_relative_error()`
- `get_precision()`
- `get_accuracy()`
- `truncate()`

The distinction between relative precision and absolute accuracy is especially important near zero.

For a nonzero quantity, relative error is approximately

\\[ \epsilon\_{\mathrm{rel}}=\frac{|\Delta x|}{|x|}. \\]

For a quantity close to zero, relative error may become very large even when absolute error remains small.

Numerica therefore permits accuracy to be expressed in absolute terms. Its documentation treats the precision parameter differently for zero-valued inputs.

#### Example: catastrophic cancellation

Consider

\\[ f(x)=\frac{e^x-1}{x} \\]

for very small \\(x\\).

Mathematically,

\\[ \lim\_{x\to 0}f(x)=1. \\]

But evaluating \\(e^x-1\\) by ordinary subtraction can eliminate many significant digits.

The upstream example starts with an extremely small \\(x\\) and approximately 60 accurate decimal digits. After evaluating the expression, the tracked precision falls to roughly 10 digits.

This demonstrates a useful property: numerical accuracy becomes information carried through the computation rather than a property assumed from the storage type.

An illustrative Rust example based on the upstream repository is:

```

use numerica::domains::float::{
    ErrorPropagatingFloat, Float, FloatLike, Real
};

let x = ErrorPropagatingFloat::new(
    Float::parse("1e-50", Some(200)).unwrap(),
    60.0,
);

let result = (x.exp() - x.one()) / x;

println!("{result}");
println!("Estimated digits: {:?}", result.get_precision());

```

The distinction between this mechanism and certified interval arithmetic is essential.

`ErrorPropagatingFloat` performs error propagation according to its numerical model; it should not be interpreted automatically as a rigorous enclosure of every possible true value.

Source: [https://docs.rs/numerica/latest/numerica/domains/float/struct.ErrorPropagatingFloat.html](https://docs.rs/numerica/latest/numerica/domains/float/struct.ErrorPropagatingFloat.html?utm_source=chatgpt.com)

### 6.5 Real and complex interval arithmetic

Numerica 3.0 introduced certified real and complex interval arithmetic for enclosures and root certification. This was a substantive expansion of the library's numerical capabilities.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra+1



An interval representation stores an enclosure such as

\\[ x\in[\underline{x},\overline{x}]. \\]

For suitably implemented interval operations, the calculated result contains the true result of the operation for all values in the input intervals.

This differs fundamentally from an error estimate.

| Mechanism                   | What it represents                                 | Primary purpose                                     |
| --------------------------- | -------------------------------------------------- | --------------------------------------------------- |
| `Float`                     | Approximate numerical value with working precision | High-precision computation                          |
| `ErrorPropagatingFloat`     | Value plus propagated error estimate               | Detecting and quantifying numerical degradation     |
| Real interval arithmetic    | Bounds enclosing real values                       | Validated numerical computation                     |
| Complex interval arithmetic | Regions enclosing complex values                   | Root certification and rigorous complex computation |

Interval arithmetic introduces challenges of its own, including interval widening and dependency overestimation when the same uncertain variable appears repeatedly.

For example, evaluating \\(x-x\\) with two independently propagated copies of the same interval may produce a nonzero-width enclosure, despite the exact mathematical result being zero.

Interval arithmetic is therefore not automatically tight, and rigorous root certification requires algorithms that correctly maintain enclosure guarantees.

The release documentation confirms the availability of certified interval arithmetic. However, I could not establish the complete public interval type signatures for Numerica 3.0.1 from the accessible documentation; those should be checked directly rather than assuming API names from other Rust interval crates.

### 6.6 Complex numbers and precision-aware transcendental functions

`Complex<T>` provides complex arithmetic using an underlying floating-point type.

Numerica's numerical traits also distinguish general arithmetic from real-valued mathematical functions.

The `Real` trait describes mathematical functionality such as constants \\(\pi\\) and \\(e\\), along with trigonometric and related operations.

The `FloatLike`, `RealLike`, `FixedPrecision`, `Constructible`, and `SingleFloat` traits express different numerical requirements.

These distinctions help generic algorithms avoid assuming that every numerical type supports the same conversions, transcendental functions, or precision behavior.

Numerica 3.0 included improvements to complex arithmetic accuracy, decimal parsing, precision validation, complex powers, and handling of mixed-precision calculations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 7. Automatic differentiation with dual and hyperdual numbers

Numerica includes a particularly flexible system for forward-mode automatic differentiation.

The key mechanism is to replace an ordinary scalar with a value that carries derivative components, then propagate those components through arithmetic.

### 7.1 First-order dual numbers

A first-order dual number can be written as

\\[ a+b\varepsilon,\qquad \varepsilon^2=0. \\]

If a function is evaluated at

\\[ x=x_0+\varepsilon, \\]

then its result satisfies

\\[ f(x_0+\varepsilon)=f(x_0)+f'(x_0)\varepsilon \\]

for the corresponding dual-number extension of \\(f\\).

The derivative is obtained directly from the coefficient of \\(\varepsilon\\).

This is not finite differencing. There is no small perturbation parameter whose selection affects truncation and rounding error.

Instead, derivatives are propagated algebraically as part of the arithmetic.

### 7.2 Multivariable differentiation

Numerica supports multiple independent derivative variables.

The macro `create_hyperdual_single_derivative!` creates a type that carries first derivatives with respect to a specified number of variables.

The following example illustrates the upstream API:

```

use numerica::{
    create_hyperdual_single_derivative,
    domains::{float::FloatLike, rational::Rational},
};

create_hyperdual_single_derivative!(Dual, 3);

fn main() {
    let x = Dual::<Rational>::new_variable(0, 1.into());
    let y = Dual::new_variable(1, 2.into());
    let z = Dual::new_variable(2, 3.into());

    let result = (x * y * z).inv();
    println!("{result}");
}

```

This evaluates

\\[ f(x,y,z)=\frac{1}{xyz} \\]

at \\((1,2,3)\\), producing

\\[ f=\frac16 \\]

and

\\[ \nabla f= \left( -\frac16,-\frac1{12},-\frac1{18} \right). \\]

An important capability is that the coefficient type can be rational. The derivatives in this example are exact fractions, not floating-point approximations.

Source: [https://docs.rs/numerica/latest/numerica/domains/dual/](https://docs.rs/numerica/latest/numerica/domains/dual/?utm_source=chatgpt.com)

### 7.3 Higher-order and mixed partial derivatives

Numerica is not restricted to one derivative per variable.

Its three main construction macros are:

| Macro                                 | Purpose                                                          |
| ------------------------------------- | ---------------------------------------------------------------- |
| `create_hyperdual_single_derivative!` | First derivatives in a specified number of variables             |
| `create_hyperdual_from_depths!`       | Derivative structures with specified maximum depths per variable |
| `create_hyperdual_from_components!`   | Explicitly selected multi-index derivative components            |

The last option is particularly expressive.

A derivative component can be identified by a multi-index:

\\[ \alpha=(\alpha_1,\ldots,\alpha_n), \\]

corresponding to

\\[ \frac{\partial^{|\alpha|}f} {\partial x_1^{\alpha_1}\cdots\partial x_n^{\alpha_n}}, \qquad |\alpha|=\sum_i\alpha_i. \\]

Instead of calculating all derivatives to a given order, a generated type can encode a particular set of derivative components.

For example, a system might require first derivatives in several variables, selected second derivatives, and one specific third-order mixed partial.

An explicitly specified derivative structure can avoid storing derivative components that are unnecessary for that calculation.

However, the selected component structure must be compatible with the algebra of truncated multivariate series. Numerica provides shape helpers, including `is_dual_shape_ancestor_closed`, to reason about these conditions.

### 7.4 Compile-time multiplication tables

One of Numerica's most distinctive implementation mechanisms is the generation of multiplication tables for fixed dual-number structures.

A product of two multivariate dual numbers requires combining derivative components according to their exponent indices.

Naively, this can involve runtime lookup and repeated interpretation of a derivative structure.

Numerica's macros can precompute these relationships and generate arithmetic specialized to a known shape.

The upstream documentation describes precomputed multiplication tables and the advantages of the generated types over a dynamically shaped `HyperDual`.

This establishes a useful tradeoff:

| Representation             | Benefit                                                | Tradeoff                                                     |
| -------------------------- | ------------------------------------------------------ | ------------------------------------------------------------ |
| Macro-generated dual type  | Fixed derivative layout and specialized multiplication | Shape known at compile time; potentially more generated code |
| Runtime-shaped `HyperDual` | Dynamically selected derivative structure              | Additional runtime structure handling                        |

Fixed derivative structures are attractive when many evaluations share the same differentiation requirements. Dynamic structures are more flexible when variable counts or required derivatives are determined at runtime.

The actual performance difference depends on derivative count, sparsity, arithmetic type, and code generation.

### 7.5 How Numerica's automatic differentiation differs from autograd frameworks

Numerica's documented dual-number system is primarily a forward-mode differentiation mechanism.

It is not the same architecture as a reverse-mode autodiff engine that records a computational graph and performs a backward pass.

The distinction is significant.

For a function

\\[ f:\mathbb{R}^{n}\rightarrow\mathbb{R}^{m}, \\]

forward-mode differentiation is often attractive when the input dimension is small or when only a few directional derivatives are required.

Reverse mode can be more attractive for scalar-valued functions with very large input dimensions.

Numerica's higher-order dual numbers provide direct access to higher derivatives, which can be valuable for Hessians, Taylor expansions, sensitivity calculations, and derivatives of numerically implemented functions.

But the presence of hyperdual arithmetic does not establish that Numerica supplies a reverse-mode tape, neural-network autograd engine, or automatic differentiation of arbitrary Rust programs without compatible numerical operations.

Architectural opportunity: A mathematical model expressed generically over supported scalar types can use ordinary floats for production evaluation and dual-number scalars for derivative evaluation. This can eliminate manually maintained derivative implementations for suitable portions of a model.

The requirement is that the model's operations are supported by the dual-number type and that control flow or discontinuities do not invalidate the desired derivatives.

## 8. Dense and sparse linear algebra

Numerica includes generic matrix and vector facilities with algebraic-domain-aware computation.

Its linear algebra is particularly interesting because exact arithmetic and modular arithmetic are first-class use cases, rather than incidental alternatives to machine floating point.

### 8.1 Dense matrices

The `Matrix` abstraction stores elements belonging to a ring or field.

Documented construction and operation families include:

- Construction from linear or nested collections.
- Construction of column or row vectors.
- Matrix arithmetic.
- Linear system solving.
- Matrix inversion.
- Determinant-related operations.
- Solving underdetermined systems.

The API includes methods such as `Matrix::solve`, `Matrix::solve_any`, and `Matrix::inv`.

The documentation demonstrates exact rational linear solving and finite-field inversion.

Source: [https://docs.rs/numerica/latest/numerica/tensors/matrix/](https://docs.rs/numerica/latest/numerica/tensors/matrix/?utm_source=chatgpt.com)

### 8.2 Exact linear system solving

For a system

\\[ Ax=b, \\]

the choice of coefficient domain changes the meaning of the solution.

Using rational arithmetic, a nonsingular system with rational coefficients has an exactly representable rational solution.

Using floating-point arithmetic, the result is approximate and subject to rounding and conditioning.

Using finite-field arithmetic, the system is solved according to modular field operations.

Numerica's generic design makes these different semantics available through the choice of domain.

An example based on the public API:

```

use numerica::{
    domains::rational::Q,
    tensors::matrix::Matrix,
};

let a = Matrix::from_linear(
    vec![
        1.into(), 2.into(), 3.into(),
        4.into(), 5.into(), 16.into(),
        7.into(), 8.into(), 9.into(),
    ],
    3,
    3,
    Q,
).unwrap();

let b = Matrix::new_vec(
    vec![1.into(), 2.into(), 3.into()],
    Q,
);

let solution = a.solve(&b).unwrap();
assert_eq!(solution.into_vec(), [
    (-1, 3),
    (2, 3),
    (0, 1),
]);

```

The solution is

\\[ x=\begin{pmatrix}-1/3\\\2/3\\\0\end{pmatrix}. \\]

This example illustrates exactness, but does not by itself establish which internal elimination or factorization method is chosen.

### 8.3 Underdetermined systems

Numerica also documents `solve_any`, including an example with fewer equations than unknowns.

An underdetermined system may admit infinitely many solutions.

The existence of `solve_any` is therefore important: it provides a route to obtain a compatible solution without requiring the matrix to be square and nonsingular.

However, the returned solution is not automatically a minimum-norm solution, a particular pseudoinverse solution, or a canonical symbolic parameterization. Those are separate mathematical specifications.

The exact selection behavior should be verified before depending on it.

### 8.4 Sparse linear algebra

Numerica exposes a separate sparse linear algebra module:

[https://docs.rs/numerica/latest/numerica/tensors/sparse/](https://docs.rs/numerica/latest/numerica/tensors/sparse/?utm_source=chatgpt.com)

Sparse matrices matter when the number of nonzero entries is much smaller than the full matrix size.

A dense \\(n\times n\\) representation stores \\(O(n^2)\\) entries. A suitable sparse representation can store entries in proportion to its number of nonzeros, together with structural metadata.

The upstream release notes confirm that Numerica validates sparse matrix structure and corrected sparse identity-check behavior in 3.0.

This is particularly relevant to correctness: a sparse matrix representation is not just a collection of values. Its index structure and dimensions must remain consistent for arithmetic to be meaningful.

The accessible documentation establishes the existence of sparse matrix functionality, but did not provide enough detail to confirm the complete set of sparse storage formats, decomposition algorithms, or iterative solvers in version 3.0.1.

Consequently, it would be inappropriate to assume support for CSR/CSC interoperation, sparse LU, Krylov methods, or preconditioner frameworks without additional source verification.

### 8.5 Linear algebra and integer reconstruction

Numerica's linear algebra becomes more distinctive when combined with its number-theory facilities.

For exact problems, possible workflows include:

- Solving over rational fields.
- Solving modular counterparts over finite fields.
- Reconstructing integer or rational results.
- Discovering integer relations.
- Reducing integer lattice bases.

These operations are mathematically complementary, even though they are not all exposed as one automatic high-level solver.

For very large numerical systems, other Rust linear algebra libraries may offer broader dense decomposition families, specialized BLAS/LAPACK integration, or advanced sparse iterative solvers.

Numerica's comparative advantage is the availability of exact arithmetic and algebraic-domain-aware primitives in the same library.

## 9. Adaptive Monte Carlo integration

Numerica includes a numerical integration subsystem based on adaptive Monte Carlo sampling, including the VEGAS algorithm.

This subsystem is conceptually different from deterministic one-dimensional quadrature.

Rather than evaluating a function at a prescribed sequence of quadrature nodes, it constructs a sampling distribution that adapts to observed function behavior.

Source: [https://docs.rs/numerica/latest/numerica/numerical_integration/](https://docs.rs/numerica/latest/numerica/numerical_integration/?utm_source=chatgpt.com)

### 9.1 Continuous VEGAS grids

For an integral

\\[ I=\int\_{\Omega} f(x)\\,dx, \\]

importance sampling draws points from a probability density \\(p(x)\\) and computes weighted contributions

\\[ w(x)=\frac{f(x)}{p(x)}. \\]

The estimator is

\\[ \widehat I=\frac{1}{N}\sum\_{i=1}^{N}\frac{f(x_i)}{p(x_i)}. \\]

The purpose of importance sampling is to allocate more samples to regions that contribute substantially to the integral, thereby reducing variance when the sampling distribution is well matched to the integrand.

Numerica's `ContinuousGrid` uses a factorized adaptive grid inspired by VEGAS.

Each dimension is partitioned into bins, and the grid adapts using accumulated training samples.

This makes it suitable for multidimensional integrals where a simple uniform sampling distribution would spend excessive computational effort on unimportant regions.

A limitation of factorized VEGAS grids is that correlated or non-axis-aligned features may be difficult to capture efficiently using independent dimensionwise adaptation.

### 9.2 Continuous and discrete grids

Numerica's integration system exposes:

| Type                    | Function                                                         |
| ----------------------- | ---------------------------------------------------------------- |
| `ContinuousGrid`        | Adaptive grid over continuous dimensions                         |
| `ContinuousDimension`   | Partitioning information for one continuous dimension            |
| `DiscreteGrid`          | Sampling over discrete bins or channels                          |
| `Grid`                  | Unified continuous/discrete grid abstraction                     |
| `Bin`                   | Discrete bin, potentially containing a nested grid               |
| `Sample`                | Encapsulates sampled values and associated weighting information |
| `Probe`                 | Access to sampling/Jacobian weighting at a point or region       |
| `StatisticsAccumulator` | Accumulation of integral estimates and uncertainty information   |
| `MonteCarloRng`         | Reproducible RNG for Monte Carlo sampling                        |

An especially important feature is that a discrete bin may contain a nested grid.

This allows an integration problem to combine discrete alternatives with continuous variables.

For example, a mathematical expectation involving a discrete state \\(s\\) and continuous variables \\(x\\) can be written as

\\[ I=\sum_s \int\_{\Omega_s} f(s,x)\\,dx. \\]

The discrete and continuous parts may require different sampling distributions.

Nested grids provide a representation for this structure without forcing all channels into one shared continuous coordinate system.

### 9.3 Multichannel integration

Multichannel sampling is useful when an integrand contains several distinct regions of high importance.

A single adaptive distribution may struggle to represent all of them.

A collection of grids can separately approximate different channels, potentially improving sampling efficiency for functions with different localized features.

The upstream integration documentation explicitly describes a `DiscreteGrid` containing multiple grids for multichannel methods.

This structure can avoid the need to implement a custom channel-selection and continuous-sampling framework.

### 9.4 Adaptive integration lifecycle

The documented workflow involves:

1\. Initialize

Create grid and RNG

Set dimensions, bins, and sampling settings

2\. Sample

Draw weighted points

Obtain integration coordinates and sampling weights

3\. Train

Evaluate the integrand

Accumulate observations to adapt the distribution

4\. Update

Adapt and estimate

Update grid partitions and review statistics

Sampling, training, and updating repeat until the integration procedure terminates.

Importantly, the integration API supports iterative training rather than presenting adaptation as an opaque operation.

This permits external applications to control the number of samples, iteration count, stopping conditions, and evaluation scheduling.

The statistics machinery tracks quantities including averages, estimated errors, and chi-squared statistics across iterations.

The numerical reliability of those estimates still depends on the sampling process and the assumptions underlying the estimator.

### 9.5 Reproducibility and parallel sampling

`MonteCarloRng` is documented as a fast, reproducible, noncryptographic random number generator suitable for parallel Monte Carlo workloads.

The seed is explicitly configured.

This supports repeatable sampling experiments, but reproducibility must be interpreted carefully.

The same random seed does not guarantee that arbitrarily reordered parallel computations will produce bit-for-bit identical accumulated floating-point results.

Parallel sampling also requires correct handling of random streams, sample weights, grid updates, and aggregate statistics.

Numerica provides the sampling infrastructure; it does not establish that every caller-created parallel execution strategy is statistically or numerically equivalent.

### 9.6 Version 3 changes

Numerica 3.0 tightened validation for grid dimensions, bin counts, and sampling settings.

Continuous and discrete Rust grid constructors now return `Result`, so code written against older constructors may require explicit error handling.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra



This is an example of a version-sensitive API change where copying earlier documentation verbatim would be misleading.

For implementation, use the exact 3.0.1 constructor signatures rather than older examples that assume infallible initialization.

## 10. Combinatorial utilities

The `combinatorics` module supports generating combinations, permutations, and partitions.

Its documented facilities include:

| API                                  | Capability                                                        |
| ------------------------------------ | ----------------------------------------------------------------- |
| `CombinationIterator`                | Generate combinations without replacement                         |
| `CombinationWithReplacementIterator` | Generate combinations with repeated selections                    |
| `unique_permutations`                | Generate distinct permutations when inputs contain duplicates     |
| `partitions`                         | Partition multisets into specified bins, retaining multiplicities |

Source: [https://docs.rs/numerica/latest/numerica/combinatorics/](https://docs.rs/numerica/latest/numerica/combinatorics/?utm_source=chatgpt.com)

The partitioning functionality is more sophisticated than ordinary combination generation.

Given an unordered multiset and a specification of named bins, it can enumerate distinct partitions together with their multiplicities.

For example, when the input contains repeated objects, many permutations represent the same mathematical partition. Returning a unique partition alongside its multiplicity avoids treating those equivalent assignments as independent cases.

This is useful for coefficient counting, combinatorial enumeration, symmetry-aware computations, and construction of mathematical expressions.

Its value becomes particularly clear when combined with exact integers: combinatorial multiplicities can grow beyond ordinary machine integer limits, while remaining exactly representable.

The main constraint is combinatorial growth. Generating only unique arrangements can reduce redundant work, but the number of genuinely distinct outputs may still be exponential or factorial in the problem size.

## 11. Arithmetic backends, features, and build integration

Version 3.0 made the selection of arbitrary-precision integer and floating-point backends independently configurable.

This is an important architectural change because Numerica no longer has to depend on the same native arithmetic backend configuration for every deployment.

### 11.1 Selectable arithmetic backends

| Feature             | Arithmetic engine           | Characteristics                                               |
| ------------------- | --------------------------- | ------------------------------------------------------------- |
| `integer-gmp`       | GMP, commonly through `rug` | Mature native arbitrary-precision integer arithmetic          |
| `integer-malachite` | Malachite                   | Rust-native arbitrary-precision integers                      |
| `float-mpfr`        | MPFR                        | Mature native multiprecision floating-point arithmetic        |
| `float-astro`       | Astro-float                 | Rust-native arbitrary-precision floating-point implementation |

The default configuration uses GMP and MPFR.

The two axes are independent: integer arithmetic and floating-point arithmetic can be configured using different backends.

The Symbolica project's release documentation explains that the Rust-native alternatives were introduced partly to facilitate WebAssembly compilation. Its October 2026 discussion reports that Malachite and Astro-float were slower than GMP/MPFR in the workloads considered there, but this is not a universal performance ranking.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+1



### 11.2 Why backend independence matters

The choice affects several dimensions.

Runtime performance. Arithmetic performance depends on operand size, operation type, precision, and allocation behavior. GMP and MPFR are highly optimized, but alternative backends may be competitive under particular conditions.

Build portability. Pure-Rust arithmetic can avoid some native dependency requirements associated with GMP and MPFR.

Deployment targets. WebAssembly and other constrained targets can benefit from arithmetic backends that do not depend on native C libraries.

Behavioral compatibility. Different multiprecision implementations may differ in rounding behavior, supported operations, exceptional-value semantics, and performance. A common Numerica API reduces integration complexity but does not justify assuming bitwise-identical behavior across backends.

Dependency management. Rust feature unification matters. A downstream project must ensure that the intended combination of backends is selected consistently across its dependency graph.

### 11.3 Installation and feature selection

The default dependency is straightforward:

```
cargo add numerica
```

An illustrative configuration for explicitly selected backends is:

```
[dependencies]
numerica = {
    version = "3.0.1",
    default-features = false,
    features = ["integer-malachite", "float-astro"]
}
```

This illustrates feature selection; it is not a tested build recipe. The exact target support and any additional required features should be checked against the 3.0.1 Cargo manifest.

The upstream Symbolica release confirms these backend feature names and the default GMP/MPFR selection.

Source: [https://github.com/symbolica-dev/numerica/blob/main/Cargo.toml](https://github.com/symbolica-dev/numerica/blob/main/Cargo.toml?utm_source=chatgpt.com)

### 11.4 Rust and platform requirements

Symbolica 3.0 declares Rust 1.89 as its minimum compiler version and uses Numerica 3.0.

This provides useful compatibility context, but the parent package's minimum compiler version should not automatically be treated as an independently verified MSRV guarantee for every Numerica 3.0.1 feature combination.

Numerica is written in Rust but can use native arithmetic dependencies, depending on the selected backend.

It is also important to distinguish the following:

- Numerica's availability as an ordinary Rust crate.
- Numerica's optional integration facilities, including Python-related functionality.
- Symbolica's own Python package and Python bindings.
- Symbolica's WebAssembly distribution and optimized evaluator backends.

A capability exposed by the Symbolica Python package does not necessarily imply an equivalent standalone Python interface to Numerica.

The upstream source also contains optional Python-related modules and dependencies, but Numerica should primarily be understood as a Rust library.

Sources:

[https://docs.rs/crate/numerica/latest](https://docs.rs/crate/numerica/latest?utm_source=chatgpt.com)

[https://symbolica.io/docs/installation/](https://symbolica.io/docs/installation/?utm_source=chatgpt.com)

### 11.5 Serialization and data interchange

Serialization support can matter when mathematical structures need to cross process boundaries or persist beyond an execution.

Numerica's published dependency information includes optional serialization-related packages such as `serde` and `bincode` in earlier releases.

These are useful integration points, but the latest manifest must be consulted to determine which particular numerical types implement serialization and under which feature flags.

The distinction is consequential for types containing an explicit ring context. Serializing only a numerical representative may be insufficient to reconstruct an element's mathematical meaning.

For modular elements, for example, the modulus must be recoverable somewhere in the serialized representation or surrounding schema.

Likewise, arbitrary-precision floating-point values may need precision and exceptional-value metadata to survive a round trip without semantic changes.

## 12. Relationship with Symbolica and neighboring libraries

Numerica occupies a particular position in the Rust mathematical ecosystem.

### 12.1 Numerica versus Symbolica

Numerica originated within Symbolica, but the projects have distinct responsibilities.

| Capability                                  | Numerica                              | Symbolica                                        |
| ------------------------------------------- | ------------------------------------- | ------------------------------------------------ |
| Arbitrary-precision integers and rationals  | Native                                | Available through its numerical infrastructure   |
| Finite-field arithmetic                     | Native                                | Uses and extends numerical infrastructure        |
| Error-propagating numerical values          | Native                                | Integrated with symbolic-numeric evaluation      |
| Hyperdual numbers                           | Native                                | Can exploit numerical differentiation machinery  |
| Generic matrices                            | Native                                | Supports broader symbolic matrix operations      |
| Polynomial coefficient kernels              | Native                                | Uses these in higher-level polynomial algorithms |
| Symbolic expression trees                   | Not its central responsibility        | Core capability                                  |
| Pattern matching and expression rewriting   | Not a general symbolic rewrite engine | Core capability                                  |
| Multivariate polynomial-system solving      | Not a general-purpose symbolic solver | Higher-level capability                          |
| Expression optimization and code generation | Not its primary scope                 | Includes JIT and code-generation facilities      |

Symbolica 3.0 introduced substantial exact polynomial-system solving, algebraic-number support, and evaluator functionality. Those capabilities should not be attributed wholesale to Numerica just because Symbolica uses its numerical infrastructure.

Conversely, Numerica provides a way to use much of that numerical infrastructure under an MIT license without incorporating the full Symbolica symbolic engine.

Symbolica itself is distributed under its own source-available licensing terms, which differ from Numerica's MIT license.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra+2



### 12.2 Comparison with other Rust numerical libraries

The following comparison focuses on architectural emphasis rather than attempting to rank libraries by performance.

| Library                               | Main architectural orientation                             | Distinction from Numerica                                                                                                         |
| ------------------------------------- | ---------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| Numerica                              | Exact and approximate arithmetic over algebraic domains    | Combines number theory, modular arithmetic, precision tracking, hyperdual AD, and generic linear algebra                          |
| `num` / `num-bigint` / `num-rational` | General numeric types and traits                           | More focused on reusable scalar types and conventional numeric trait abstractions                                                 |
| `rug`                                 | High-performance arbitrary-precision numbers               | Provides underlying integer, rational, and floating-point primitives without Numerica's complete domain and algorithmic framework |
| Malachite                             | Rust-native arbitrary-precision arithmetic                 | Can serve as Numerica's integer backend rather than replacing all its mathematical abstractions                                   |
| `nalgebra`                            | Matrix and vector algebra with generic scalar types        | Strong emphasis on geometric and numerical linear algebra                                                                         |
| `faer`                                | High-performance dense and sparse numerical linear algebra | Focuses on numerical matrix computation rather than exact algebraic domains                                                       |
| `ndarray`                             | N-dimensional array structures and operations              | General array computation rather than ring-oriented mathematical algorithms                                                       |
| Symbolica                             | Full symbolic-numeric computer algebra                     | Operates above Numerica's numerical primitives with expressions, rewriting, polynomial systems, and code generation               |

The distinctions should guide investigation rather than establish a universal winner.

For example, a numerical workload dominated by large floating-point matrix factorizations may be better matched to a library with specialized numerical linear algebra implementations.

An exact computation involving rational reconstruction, modular fields, and integer relations may benefit more directly from Numerica's integrated mathematical domain facilities.

General-purpose array libraries and Numerica can also be complementary: an application may use arrays for storage and iteration while using Numerica types for selected computations.

### 12.3 Interoperability is not automatically zero-cost

Even when two libraries use conceptually equivalent numbers, their representations may differ.

Possible conversion boundaries include:

- Numerica matrices versus `nalgebra` or `ndarray` storage.
- Numerica `Float` versus `rug::Float` or another multiprecision type.
- Numerica `Rational` versus `num_rational::BigRational`.
- Numerica finite-field elements versus other modular arithmetic representations.

Conversions may involve allocation, normalization, copying, precision changes, or altered ownership.

For arbitrary-precision numbers, it is especially important to preserve precision and rounding semantics during conversion.

Numerica's use of an underlying backend does not necessarily make all public representations transparent wrappers over that backend.

These costs can influence whether an application should use Numerica throughout an algorithm or only at an exact-computation boundary.

## 13. Composing Numerica capabilities into larger algorithms

Several of Numerica's most valuable architectural possibilities emerge from combining capabilities.

### 13.1 Modular exact computation

A general modular exact-computation architecture can combine:

1. Fixed-width finite-field arithmetic.
2. Modular evaluation of an algebraic operation.
3. CRT-based reconstruction.
4. Rational reconstruction when necessary.
5. Exact verification of the reconstructed answer.

This can avoid large intermediate rational coefficients and exploit relatively inexpensive modular operations.

The gain is conditional: it depends on the cost of modular computations, the number of primes required, reconstruction bounds, and the expense of verification.

Numerica supplies important pieces of this architecture, although a caller may still need to implement the problem-specific reconstruction procedure.

### 13.2 Exact differentiation

Numerica's dual-number system can use rational coefficients.

This makes it possible to evaluate derivatives of suitable arithmetic expressions exactly.

For a rational function,

\\[ f(x)=\frac{P(x)}{Q(x)}, \\]

where \\(P\\) and \\(Q\\) have rational coefficients, dual-number evaluation at a rational point can produce exact derivative coefficients, provided the expression is defined at that point.

This avoids finite-difference error entirely and can also avoid manually derived analytic derivatives.

The resulting exact arithmetic can, however, cause substantial numerator and denominator growth.

### 13.3 Precision-aware differentiated computation

A further possibility is combining differentiation with precision-aware numerical representations.

For compatible scalar operations, dual arithmetic can propagate derivatives while an error-propagating scalar tracks accuracy within its numerical coefficients.

This offers a conceptual route to obtaining both derivative information and indications of numerical degradation.

The important qualification is that composition of two mathematical type systems must actually be supported by the relevant trait implementations. The existence of dual numbers and error-propagating floats separately does not guarantee that every nested combination compiles or maintains all desirable error guarantees.

### 13.4 Numerical discovery followed by exact verification

Another natural combination is:

Numerical discovery

Use multiprecision computation, PSLQ, or lattice reduction to identify a candidate relationship.

Exact reconstruction

Convert the candidate to rational or integer structures using reconstruction algorithms.

Independent validation

Verify the candidate using exact arithmetic or a rigorously justified enclosure method.

This separates numerical evidence from mathematical certainty.

Numerica's facilities help implement the stages without requiring the numerical-discovery and exact-verification portions to use unrelated arithmetic frameworks.

### 13.5 Adaptive simulation and sensitivity evaluation

For a function requiring expensive numerical evaluation, two Numerica capabilities can play complementary roles:

- Adaptive Monte Carlo integration controls which points are evaluated.
- Dual-number arithmetic can provide derivatives of those evaluations where the underlying function supports the relevant scalar operations.

This may be useful in sensitivity analysis of multidimensional numerical models.

However, differentiation of an adaptive stochastic estimator raises additional questions, including how sampling distributions depend on parameters and whether differentiation through random sampling produces the intended estimator.

Numerica's individual primitives do not automatically resolve those statistical issues.

## 14. Important limitations and engineering considerations

### Exactness is conditional on the arithmetic domain

An integer or rational calculation can be exact, while a floating-point calculation using a mathematically similar operation is approximate.

A generic algorithm does not erase this difference.

### Error estimates are not necessarily validated enclosures

`ErrorPropagatingFloat` and certified interval arithmetic address different questions. An estimated number of reliable digits is not a guaranteed bound on the mathematical result.

### Advanced algebraic algorithms have mathematical preconditions

Field inversion, modular reconstruction, rational reconstruction, and polynomial division all depend on particular algebraic assumptions.

Type-level guarantees can help enforce some conditions, but not every mathematical property is necessarily encoded in the Rust type system.

### Dynamic precision does not automatically recover lost information

Increasing working precision after severe cancellation may not restore information already lost in earlier calculations.

The input representation and the full computation path matter.

### Runtime-sized and compile-time-sized derivative structures have different costs

Macro-generated hyperdual types can specialize arithmetic, but increase code specialization. Dynamically structured dual numbers can offer greater flexibility with additional runtime overhead.

### Generic linear algebra is not equivalent to a complete numerical solver stack

Numerica's matrix operations do not imply complete support for nonlinear optimization, ODE/DAE integration, large-scale sparse factorizations, GPU linear algebra, or general-purpose scientific simulation.

Those are separate capabilities requiring additional algorithms or libraries.

### Parallelism is workload-specific

The presence of Rayon and parallel-friendly random sampling mechanisms does not establish that every Numerica algorithm is internally parallelized.

Likewise, efficient numerical algorithms do not automatically imply GPU acceleration.

### Backend choice changes build and deployment behavior

GMP/MPFR and the Rust-native alternatives have different platform and dependency considerations.

Selecting a backend should be treated as a build-level architectural choice, not merely a performance toggle.

### Performance claims require workload-specific evidence

The upstream materials describe high-performance arithmetic and specialization mechanisms, but they do not justify asserting a general speedup versus `nalgebra`, `faer`, `rug`, or other libraries.

In particular, performance findings for Symbolica's full symbolic engine or JIT evaluators cannot be transferred directly to Numerica's standalone arithmetic operations.

## 15. Version 3.0 and 3.0.1: capabilities and compatibility

The 3.0 release deserves special attention because it is not simply an incremental update to the original Numerica design.

The upstream changelog identifies the following changes:

| Area                       | Important 3.0 change                                                    |
| -------------------------- | ----------------------------------------------------------------------- |
| Integer arithmetic         | Redesigned compact storage and improved factorization                   |
| Arithmetic backends        | Independent GMP/Malachite and MPFR/Astro-float selection                |
| Interval arithmetic        | Certified real and complex interval capabilities                        |
| Algebraic traits           | Ordering and random sampling separated from core ring operations        |
| Polynomial algorithms      | Faster integer, rational, and finite-field kernels; checked division    |
| Finite fields              | Corrections to zero powers and sampling behavior                        |
| Matrices                   | Corrections to inversion, determinant signs, and sparse identity checks |
| Lattice operations         | Approximate integer basis reduction                                     |
| Floating-point computation | Improved complex accuracy, decimal parsing, and precision tracking      |
| Numerical integration      | Stronger grid validation and fallible constructors                      |

Source: [https://symbolica.io/posts/symbolica_3_0_release/](https://symbolica.io/posts/symbolica_3_0_release/?utm_source=chatgpt.com)

The current published crate documentation identifies Numerica 3.0.1, released September 29, 2026, while the broader Symbolica project has continued publishing related improvements.

The distinction matters: newer features mentioned in Symbolica release material are not necessarily available in the standalone Numerica 3.0.1 API.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs+2



### Verification boundaries

This overview is grounded in the published crate documentation, upstream repository, and official Symbolica release documentation. Context7 was checked first but did not return an appropriate Numerica documentation entry.

Some documentation pages were accessible only through older indexed versions. Where those pages supported an established capability but not its latest signature, the distinction has been noted.

The following details deserve direct verification against the versioned 3.0.1 source before implementing advanced integrations:

- Complete interval arithmetic type signatures and certification interfaces.
- Sparse matrix storage formats and supported sparse solving algorithms.
- Exact behavior and coverage of matrix factorizations.
- Backend feature combinations and target-specific build requirements.
- Serialization implementations for individual numerical types.
- Which recursive combinations of dual numbers, multiprecision floats, and error-tracking types satisfy the necessary traits.

These are meaningful documentation-verification gaps, not evidence that the capabilities are absent.

## 16. Primary source map

The following direct URLs provide the most useful routes into the library.

| Area                           | Primary documentation                                                                                                                                     |
| ------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Crate overview and exports     | [https://docs.rs/numerica/latest/numerica/](https://docs.rs/numerica/latest/numerica/?utm_source=chatgpt.com)                                             |
| Algebraic domains and traits   | [https://docs.rs/numerica/latest/numerica/domains/](https://docs.rs/numerica/latest/numerica/domains/?utm_source=chatgpt.com)                             |
| Integer arithmetic             | [https://docs.rs/numerica/latest/numerica/domains/integer/](https://docs.rs/numerica/latest/numerica/domains/integer/?utm_source=chatgpt.com)             |
| Rational arithmetic            | [https://docs.rs/numerica/latest/numerica/domains/rational/](https://docs.rs/numerica/latest/numerica/domains/rational/?utm_source=chatgpt.com)           |
| Finite fields                  | [https://docs.rs/numerica/latest/numerica/domains/finite_field/](https://docs.rs/numerica/latest/numerica/domains/finite_field/?utm_source=chatgpt.com)   |
| Floating-point types           | [https://docs.rs/numerica/latest/numerica/domains/float/](https://docs.rs/numerica/latest/numerica/domains/float/?utm_source=chatgpt.com)                 |
| Dual and hyperdual numbers     | [https://docs.rs/numerica/latest/numerica/domains/dual/](https://docs.rs/numerica/latest/numerica/domains/dual/?utm_source=chatgpt.com)                   |
| Polynomial coefficient kernels | [https://docs.rs/numerica/latest/numerica/kernels/](https://docs.rs/numerica/latest/numerica/kernels/?utm_source=chatgpt.com)                             |
| Matrix and vector arithmetic   | [https://docs.rs/numerica/latest/numerica/tensors/matrix/](https://docs.rs/numerica/latest/numerica/tensors/matrix/?utm_source=chatgpt.com)               |
| Sparse linear algebra          | [https://docs.rs/numerica/latest/numerica/tensors/sparse/](https://docs.rs/numerica/latest/numerica/tensors/sparse/?utm_source=chatgpt.com)               |
| Adaptive integration           | [https://docs.rs/numerica/latest/numerica/numerical_integration/](https://docs.rs/numerica/latest/numerica/numerical_integration/?utm_source=chatgpt.com) |
| Combinatorial algorithms       | [https://docs.rs/numerica/latest/numerica/combinatorics/](https://docs.rs/numerica/latest/numerica/combinatorics/?utm_source=chatgpt.com)                 |
| Upstream implementation        | [https://github.com/symbolica-dev/numerica/tree/main/src](https://github.com/symbolica-dev/numerica/tree/main/src?utm_source=chatgpt.com)                 |
| Example implementations        | [https://github.com/symbolica-dev/numerica/tree/main/examples](https://github.com/symbolica-dev/numerica/tree/main/examples?utm_source=chatgpt.com)       |
| Package and build features     | [https://docs.rs/crate/numerica/latest](https://docs.rs/crate/numerica/latest?utm_source=chatgpt.com)                                                     |
| 3.0 release changes            | [https://symbolica.io/posts/symbolica_3_0_release/](https://symbolica.io/posts/symbolica_3_0_release/?utm_source=chatgpt.com)                             |

## 17. Overall assessment

Numerica's distinctive contribution to Rust numerical computing is the integration of mathematically explicit algebraic domains with high-performance exact arithmetic, numerical precision management, and reusable numerical algorithms.

Its most valuable characteristics are not limited to individual scalar implementations.

The more consequential capabilities are the combinations it makes possible: exact and modular computation through one domain model; reconstruction algorithms alongside efficient finite-field arithmetic; derivative propagation with configurable higher-order structures; numerical uncertainty handling; and extensible coefficient-domain kernels for higher-level polynomial algorithms.

Several aspects are particularly noteworthy.

First, automatic integer representation resizing addresses a practical efficiency problem in exact computation without placing the responsibility for magnitude management on every algorithm.

Second, finite-field arithmetic, CRT, rational reconstruction, PSLQ, and lattice reduction form a substantial toolkit for exact algorithm development that would otherwise require combining several specialized libraries.

Third, the hyperdual-number generation system is unusually flexible. Compile-time specialization of selected derivative structures provides a means of obtaining higher-order derivatives without building a general symbolic differentiation engine or a reverse-mode computation graph.

Fourth, the 3.0 polynomial-kernel interfaces provide an extension mechanism through which higher-level algorithms can exploit coefficient-specific arithmetic rather than relying entirely on generic scalar operation loops.

Fifth, Numerica explicitly distinguishes working precision, propagated error estimates, and certified interval enclosures. These support different levels of numerical assurance and should remain separate in mathematical software design.

The principal boundary is equally important: Numerica is a numerical and algebraic foundation, not a complete scientific-computing platform or symbolic computer algebra system. Its value is strongest where applications require precise mathematical semantics, exact or hybrid arithmetic, and the ability to construct specialized algorithms from reusable primitives.

Within that scope, Numerica 3.0.1 provides a broad and technically sophisticated foundation, with several advanced capabilities that may eliminate the need for custom integer arithmetic, modular reconstruction, dual-number implementations, or adaptive sampling infrastructure.

# Numerica (Rust) — Advanced Capability Addendum

Research date: November 9, 2026 | Target: Numerica 3.0.x, particularly 3.0.1 | Scope: functionality omitted or insufficiently developed in the original overview

## Scope and verification

This addendum extends the original Numerica reference with additional numerical representations, algebraic algorithms, optimization mechanisms, interoperability facilities, and important semantic distinctions. It is intended to be read alongside the original document rather than replacing it.

The most consequential additions are:

- SIMD-compatible numerical abstractions and their operation coverage.
- Fraction-free linear algebra and additional algebraic matrix transformations.
- Certified ball arithmetic and directed-rounding primitives.
- Precision management, stable elementary operations, and exceptional-value semantics.
- Specialized polynomial kernels, coefficient conversion, and geometric-sequence acceleration.
- Ring-aware printing, ownership-aware arithmetic, and interoperability boundaries.

Version qualification: The upstream Symbolica 3.0.1 API provides evidence for several Numerica-derived capabilities, but some independently indexed Numerica pages still resolve to 2.x documentation. Where a capability is verified only in an older source snapshot or through Symbolica's re-exported interface, this is identified rather than treated as definitive evidence of its exact standalone 3.0.1 signature.

Primary sources:

- Numerica repository: [https://github.com/symbolica-dev/numerica](https://github.com/symbolica-dev/numerica?utm_source=chatgpt.com)
- Numerica documentation: [https://docs.rs/numerica/latest/numerica/](https://docs.rs/numerica/latest/numerica/?utm_source=chatgpt.com)
- Symbolica 3.0 release notes: [https://symbolica.io/posts/symbolica_3_0_release/](https://symbolica.io/posts/symbolica_3_0_release/?utm_source=chatgpt.com)
- Symbolica's published numerical API: [https://docs.rs/symbolica/latest/symbolica/](https://docs.rs/symbolica/latest/symbolica/?utm_source=chatgpt.com)

The upstream release documentation confirms the redesigned arithmetic backends, specialized polynomial kernels, interval arithmetic, enhanced domain traits, and additional exact-algebra functionality in Numerica 3.0.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra+1



## 1. SIMD numerical types and generic vectorized execution

### 1.1 SIMD integration through numerical traits

Numerica's floating-point implementation includes SIMD support built around the `wide` crate.

The published implementation provides `FloatLike` and `Real` implementations for:

| Type          | Lane representation          | Main role                              |
| ------------- | ---------------------------- | -------------------------------------- |
| `wide::f64x2` | Two double-precision values  | Two independent numerical evaluations  |
| `wide::f64x4` | Four double-precision values | Four independent numerical evaluations |

Unlike a standalone SIMD math library, Numerica integrates these types into its broader numerical trait hierarchy.

That distinction allows suitably generic mathematical code to accept a vector-valued scalar rather than requiring a completely separate implementation of the mathematical algorithm.

For example, consider a numerical function:

\\[ f(x)=\exp(-x^2)\sin(x). \\]

Its scalar implementation evaluates one input per invocation. A SIMD representation can evaluate independent inputs in separate lanes using the same mathematical operations.

Conceptually:

```
fn evaluate<T: Real>(x: T) -> T {
    // Illustrative mathematical structure;
    // exact trait/operator requirements are omitted.
    (-x.clone() * x.clone()).exp() * x.sin()
}
```

The same underlying algorithm can potentially operate on a scalar or packed SIMD type when its full operation set is available.

This is distinct from vectorizing a general computation graph. SIMD operations run independently over corresponding lanes, with the underlying implementation determining instruction selection and fallback behavior.

### 1.2 Supported mathematical operations

The published SIMD implementation includes the following categories:

| Category              | Operations                                                  |
| --------------------- | ----------------------------------------------------------- |
| Arithmetic            | Addition, subtraction, multiplication, division, negation   |
| Algebraic             | Integer exponentiation, reciprocal                          |
| Elementary            | Square root, exponential, logarithm                         |
| Trigonometric         | Sine, cosine, tangent                                       |
| Inverse trigonometric | Arcsine, arccosine, two-argument arctangent                 |
| Numerical constants   | \\(\pi\\), \\(e\\), Euler–Mascheroni constant, golden ratio |
| Sampling              | Lane-valued random sampling from a unit interval            |
| Precision metadata    | Fixed 53-bit significand precision                          |

Source: [https://docs.rs/numerica/latest/src/numerica/domains/float/simd.rs.html](https://docs.rs/numerica/latest/src/numerica/domains/float/simd.rs.html?utm_source=chatgpt.com)

The retrieved source shows explicit implementations for `f64x2` and `f64x4`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

source



### 1.3 Important limitations of the SIMD abstraction

The indexed Numerica 2.1 implementation contains `unimplemented!` branches for six hyperbolic operations:

- `sinh`, `cosh`, `tanh`
- `asinh`, `acosh`, `atanh`

These limitations are verified for that source snapshot, not independently for 3.0.1.

This is an important Rust API distinction: implementing a mathematical trait does not guarantee that every method has a complete runtime implementation when the trait permits default methods or explicit runtime failure.

The SIMD implementation also constructs constants from ordinary `f64` constants, so using packed values does not increase numerical precision.

Its integer-power implementation uses a floating-point power operation in the inspected version; it should not automatically be assumed to have the same integer-exponent semantics or performance as an exact repeated-squaring implementation.

### 1.4 Composition with numerical algorithms

SIMD's most useful role is potentially as an alternative scalar representation within generic algorithms.

A function evaluated over many independent inputs can benefit from packed operations without requiring its mathematical definition to be duplicated.

The approach is especially relevant when:

- Most operations are supported by the SIMD numeric implementation.
- Evaluations have similar control flow.
- Independent inputs can be processed in batches.
- Data can be packed and unpacked without excessive overhead.

It is less appropriate for algorithms dominated by irregular branching, per-lane divergent work, or operations unavailable on the packed type.

An additional distinction is that SIMD vectorization does not imply multithreading. The two can be combined, but SIMD parallelism operates within a processor's vector instructions, whereas task parallelism distributes independent work across threads.

Architectural significance: Numerica's scalar abstraction can support numerical representation polymorphism across ordinary floating-point values, higher-precision types, and SIMD lanes, although actual composability depends on the complete trait implementations.

## 2. Fraction-free elimination and exact linear algebra

The original overview discussed exact matrix solving but did not adequately cover Numerica's separate fraction-free algorithm family.

This is one of the most important additions.

### 2.1 Field elimination versus Euclidean-domain elimination

Ordinary Gaussian elimination frequently introduces intermediate divisions.

For a matrix over the integers, this can require representing intermediate entries as rational numbers.

Consider

\\[ A= \begin{pmatrix} 2&3\\\ 5&7 \end{pmatrix}. \\]

A conventional elimination step produces a multiplier \\(5/2\\), creating fractions even though the original matrix contains only integers.

For larger matrices, intermediate fractions can accumulate large numerators and denominators. This is often called coefficient swell.

Fraction-free elimination reorganizes arithmetic so that suitable computations remain within an exact integral domain, using exact division where the mathematical algorithm guarantees divisibility.

The associated Numerica interfaces include:

```
partial_row_reduce_fraction_free
back_substitution_fraction_free
solve_fraction_free
```

These are implemented for matrices whose ring satisfies `EuclideanDomain`, rather than requiring an arbitrary field.

The published 3.0.1 Symbolica API confirms their presence and semantics.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Source: [https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html](https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html?utm_source=chatgpt.com)

### 2.2 Fraction-free echelon reduction

`partial_row_reduce_fraction_free` transforms selected matrix columns into a non-reduced echelon form.

The operation also returns the rank computed during elimination.

The important property is that the algorithm does not require arbitrary field division of matrix coefficients.

Its applicability to Euclidean domains makes it useful for integer and other suitable exact coefficient representations.

The method's `max_col` parameter allows elimination to be restricted to a specified number of leading columns. This is useful when processing augmented matrices:

\\[ [A\mid b], \\]

where the final columns contain right-hand-side information rather than coefficients used for pivot selection.

### 2.3 Fraction-free back-substitution

`back_substitution_fraction_free` transforms an echelon matrix into a more reduced form.

The documentation explicitly warns that the result is not necessarily normalized.

That distinction matters.

In ordinary field-based elimination, reduced row-echelon form often normalizes pivots to one.

Over an integer domain, dividing a row by its pivot may introduce fractions.

A fraction-free algorithm can instead preserve a scaled representation whose rows remain in the original domain.

Thus the resulting matrix can encode the same algebraic relationships without satisfying every conventional normalization condition associated with field-based reduced row-echelon form.

### 2.4 Fraction-free solving

`solve_fraction_free` attempts to solve

\\[ Ax=b \\]

within the original coefficient domain.

The API's documentation specifies that a solution is returned when one exists in that domain.

This is narrower than solving over the associated field of fractions.

For example,

\\[ 2x=1 \\]

has a solution over the rationals but not over the integers.

An integer-domain fraction-free solver should not be expected to return \\(1/2\\) as an integer solution.

Conversely,

\\[ 2x=4 \\]

admits an integer-domain solution.

This separation allows applications to express an important mathematical requirement: the answer must remain in a particular algebraic domain.

### 2.5 Relationship to modular exact solving

Fraction-free elimination and modular reconstruction can both avoid the costs of unrestricted rational intermediate arithmetic, but they use fundamentally different strategies.

| Mechanism                     | Arithmetic strategy                           | Principal consideration                                  |
| ----------------------------- | --------------------------------------------- | -------------------------------------------------------- |
| Rational Gaussian elimination | Work directly in a field of fractions         | Intermediate fractions can grow                          |
| Fraction-free elimination     | Preserve operations within a Euclidean domain | Exact divisions and coefficient growth still matter      |
| Modular elimination           | Solve over finite fields and reconstruct      | Requires sufficient modular information and verification |

Fraction-free elimination is especially attractive when matrix entries belong to a domain with efficient exact arithmetic and the intermediate coefficients remain manageable.

Modular algorithms may become attractive when exact intermediate coefficients grow substantially, particularly if independent prime-modulus calculations can be performed efficiently.

Neither is universally superior. The appropriate approach depends on matrix dimensions, coefficient growth, rank behavior, arithmetic costs, and reconstruction overhead.

Numerica's value is that it provides relevant primitives for both algorithmic families.

## 3. Additional algebraic matrix operations

Numerica's generic `Matrix<F>` abstraction supports several operations with semantics beyond ordinary numerical linear algebra.

### 3.1 Matrix content and primitive part

For an integer matrix

\\[ A= \begin{pmatrix} 6&12\\\ 18&24 \end{pmatrix}, \\]

the content is the greatest common divisor of all entries:

\\[ \operatorname{content}(A)=6. \\]

Its primitive part is

\\[ \operatorname{primitive}(A)= \begin{pmatrix} 1&2\\\ 3&4 \end{pmatrix}. \\]

Numerica provides:

- `content()`
- `primitive_part()`
- `div_scalar()`

under the appropriate Euclidean-domain constraints.

These facilities are useful when matrices are manipulated as exact algebraic objects rather than merely numerical operators.

Removing common factors can reduce the magnitude of intermediate coefficients and simplify subsequent calculations.

The exact normalization of content depends on domain conventions. Over the integers, signs and unit factors can produce equivalent primitive representations; over other Euclidean domains, the set of units may be more complicated.

### 3.2 Matrix differentiation

`Matrix::derivative` is available for coefficient domains implementing `Derivable`.

For a matrix-valued function

\\[ A(x)= \begin{pmatrix} x^2 & x^3\\\ x & 1 \end{pmatrix}, \\]

elementwise differentiation gives

\\[ A'(x)= \begin{pmatrix} 2x & 3x^2\\\ 1 & 0 \end{pmatrix}. \\]

The important feature is that differentiation belongs to the coefficient domain.

Numerica does not have to parse a symbolic expression tree itself. It delegates differentiation to an appropriate `Derivable` implementation.

This allows matrix-level differentiation to compose with algebraic coefficient types that provide their own derivative semantics.

It is separate from differentiating a complete matrix algorithm through forward-mode dual numbers.

### 3.3 Changing coefficient domains

`Matrix::map` applies a user-provided function to every matrix entry and constructs a matrix over another ring.

Conceptually,

\\[ \phi: R\rightarrow S \\]

can be applied entrywise to

\\[ A\in R^{m\times n}, \\]

producing

\\[ \phi(A)\in S^{m\times n}. \\]

Examples of potentially useful transformations include converting integer coefficients to rational coefficients, mapping integer values into a finite field, or converting exact values into a chosen approximate representation.

The mapping function is not automatically a ring homomorphism.

For example, an arbitrary function might not preserve

\\[ \phi(a+b)=\phi(a)+\phi(b) \\]

or

\\[ \phi(ab)=\phi(a)\phi(b). \\]

If a higher-level algorithm relies on preserving algebraic structure, the caller is responsible for choosing a valid mapping.

This is an important distinction between a generic element conversion API and a mathematically certified domain homomorphism.

### 3.4 Matrix transformations and structural operations

The additional published interfaces include:

| Method               | Capability                                                         |
| -------------------- | ------------------------------------------------------------------ |
| `rank`               | Compute matrix rank                                                |
| `row_reduce`         | In-place Gaussian elimination and rank computation                 |
| `partial_row_reduce` | Echelon reduction of selected columns                              |
| `augment`            | Construct a matrix with additional columns                         |
| `split_col`          | Split a matrix at a specified column                               |
| `into_transposed`    | Ownership-consuming transpose                                      |
| `into_vector`        | Convert a row- or column-shaped matrix into a vector               |
| `norm_squared`       | Compute a squared Euclidean-style norm over the coefficient domain |
| `eye`                | Construct a diagonal matrix from specified diagonal entries        |

These capabilities can avoid bespoke matrix-construction and transformation utilities in algebraic algorithms.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html](https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html?utm_source=chatgpt.com)

### 3.5 Error semantics and domain requirements

Not all matrix operations are available for arbitrary rings.

For example, inversion requires field operations, whereas fraction-free methods use Euclidean-domain operations.

The published APIs also distinguish several failure behaviors:

- Matrix inversion can return `MatrixError::Singular`.
- Construction from flat or nested arrays returns errors for invalid structure.
- Augmentation rejects incompatible row counts.
- Conversion to a vector can panic when the matrix does not have vector shape.

These distinctions matter for generic mathematical code. A `Result`-returning operation and an operation with a structural precondition requiring the caller to avoid a panic impose different obligations.

The 3.0 release also corrected matrix inversion behavior and strengthened dense/sparse structural validation, making version-specific testing particularly important when upgrading older mathematical algorithms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 4. Certified ball arithmetic and directed rounding

The original overview discussed interval arithmetic conceptually. Numerica 3.0's concrete ball representations and directed-rounding machinery deserve a more detailed treatment.

### 4.1 Real balls

A real ball represents a closed interval as a center and nonnegative radius:

\\[ B(c,r)=\\{x\in\mathbb R:|x-c|\leq r\\}. \\]

Equivalently,

\\[ B(c,r)=[c-r,c+r]. \\]

The published floating-point API exposes `RealBall` with this representation.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/domains/float/index.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/float/index.html?utm_source=chatgpt.com)

This is different from storing an approximate result and its estimated number of significant digits.

A certified enclosure is intended to contain the mathematically exact result. Its arithmetic must therefore account for rounding and all other errors included in the certification model.

For example, adding two balls gives the mathematical enclosure

\\[ B(a,r_a)+B(b,r_b) \subseteq B(a+b,r_a+r_b). \\]

In finite-precision implementation, the center and radius calculations must also account for rounding. Merely applying this formula using ordinary unguarded floating-point arithmetic is insufficient to guarantee a certified result.

### 4.2 Complex balls

The published API defines `ComplexBall` as a rectangular complex enclosure, with real-ball components for the real and imaginary parts.

Conceptually,

\\[ Z=B(a,r_a)+iB(b,r_b). \\]

The enclosure is a rectangle in the complex plane.

This distinction matters because it differs from a circular complex ball of the form

\\[ \\{z\in\mathbb C:|z-c|\leq r\\}. \\]

Rectangular enclosures permit separate bounds on real and imaginary components. They can be convenient for arithmetic over the component intervals, but may introduce overestimation when a computation involves rotations or strong coupling between real and imaginary parts.

A rectangular complex ball also should not be confused with a single uncertain complex value whose real and imaginary components are statistically correlated.

### 4.3 Directed rounding

The 3.0.1 floating-point API includes `RoundingDirection` and explicit operations:

```
Float::add_round
Float::sub_round
Float::mul_round
Float::div_round
Float::from_rational_round
```

These are particularly important because directed rounding provides the numerical foundation needed to construct rigorous enclosures.

For an exact arithmetic result \\(x\\), directed rounding aims to compute representable values satisfying

\\[ \operatorname{roundDown}(x)\leq x \leq\operatorname{roundUp}(x). \\]

For two exactly represented floating-point inputs \\(a\\) and \\(b\\), a directed addition can produce lower and upper bounds for their exact mathematical sum.

The published documentation explicitly describes the `Down` and `Up` modes as delivering lower and upper bounds for addition, subtraction, multiplication, and division, subject to the operation's documented conditions. Division requires a nonzero denominator.

`from_rational_round` also allows exact rational values to be rounded into multiprecision floating-point representations with a selected direction.

This is a particularly useful bridge between exact and approximate mathematics.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 4.4 Exact rationals and certified floating-point enclosures

Consider an exact rational

\\[ r=\frac{a}{b}. \\]

At a chosen finite precision \\(p\\), compute

\\[ \ell=\operatorname{roundDown}\_p(r), \qquad u=\operatorname{roundUp}\_p(r). \\]

Then

\\[ r\in[\ell,u]. \\]

This provides a way to introduce exact rational data into a validated numerical computation without pretending every rational number has an exact finite binary floating-point representation.

The same principle applies to arithmetic operations on representable operands.

An exact integer or rational computation can therefore provide inputs or verification targets for certified numerical evaluation.

The broader limitation is that rigorous outward rounding of basic operations does not automatically establish rigorous error bounds for arbitrary numerical algorithms. Iteration error, approximation error, termination criteria, and the mathematical model itself must also be accounted for.

### 4.5 Ball arithmetic versus propagated error estimates

The important distinction can be summarized as follows.

| Property            | `ErrorPropagatingFloat<T>`                         | `RealBall` / `ComplexBall`                     |
| ------------------- | -------------------------------------------------- | ---------------------------------------------- |
| Core representation | Approximation with propagated accuracy information | Geometric enclosure of possible values         |
| Primary objective   | Track loss of accuracy                             | Bound the true mathematical value              |
| Error semantics     | Numerical propagation model                        | Certified inclusion under supported operations |
| Typical output      | Approximate value and accuracy estimate            | Interval or complex region                     |
| Main limitation     | Accuracy estimate is not automatically a proof     | Bounds may widen or become uninformative       |

The two systems are complementary rather than interchangeable.

### 4.6 Interval dependency and conditioning

Suppose an uncertain variable satisfies

\\[ x\in[1,2]. \\]

Mathematically,

\\[ x-x=0. \\]

But independent interval evaluation gives

\\[ [1,2]-[1,2]=[-1,1]. \\]

The interval encloses the true result but is far from tight.

This illustrates the dependency problem. Ordinary interval evaluation does not necessarily preserve the fact that two occurrences refer to the same variable.

A more sophisticated validated computation may require algebraic simplification, interval Newton methods, affine arithmetic, Taylor models, or subdivision.

Numerica provides the relevant numerical representations and rounding primitives, but the existence of those primitives does not demonstrate that Numerica implements every such higher-level validated numerical method.

## 5. Precision control and numerical semantics

### 5.1 Working precision versus trustworthy precision

The multiprecision `Float` abstraction distinguishes the configured storage precision from the accuracy that can reasonably be attributed to the result.

The published documentation states that its precision tracking is loose and can overestimate the actual precision.

This is particularly relevant after cancellation.

For example, suppose two approximately represented quantities satisfy

\\[ a\approx 1.000000000000001, \qquad b\approx 1.000000000000000. \\]

Their difference is much smaller than either operand.

Even if the result is stored in a high-precision floating-point object, it cannot automatically recover uncertainty already present in the operands.

The API provides both `prec()` and `set_prec()` at the concrete multiprecision level, and `get_precision()` and `set_precision()` through `FloatLike`.

The latter has an important semantic caveat: for loosely tracked floating-point types, resetting precision may discard tracked information about previous loss of accuracy. For certified enclosures, the documentation specifies that the enclosure remains valid.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html?utm_source=chatgpt.com)

### 5.2 Decimal parsing and explicit precision

Numerica's multiprecision floating-point API can derive precision from the textual representation of a number or accept explicit precision.

The published interface includes:

```
Float::parse(s, precision)
Float::decimal_digits_to_bits(digits)
```

The parser supports decimal notation, scientific notation, and backtick precision suffixes.

An explicitly supplied precision is expressed in binary digits, whereas a backtick suffix can express decimal precision.

For example, a mathematical input can conceptually request 20 decimal digits using:

```
1.234`20
```

The distinction between decimal and binary precision matters because \\(d\\) decimal digits require approximately

\\[ p=d\log_2(10)\approx 3.32193d \\]

binary significand bits.

Input parsing is also a correctness boundary. Parsing a decimal number directly at high precision can retain information that would already have been lost if the value were first parsed as an ordinary `f64` and subsequently converted.

### 5.3 Conversion back to exact rational form

The published `Float` API exposes:

```
to_rational()
try_to_rational()
```

These functions support conversion of a finite binary floating-point representation to a rational value, subject to their documented exceptional-value behavior.

The critical distinction is between recovering the exact rational value of the stored floating-point number and recovering the intended mathematical quantity.

For example, the decimal input `0.1`, when first rounded into a conventional binary `f64`, does not store the exact rational \\(1/10\\).

Converting the stored binary value to a rational yields an exact fraction representing the approximation, not necessarily the original decimal rational.

This makes exact round-trip representation and recovery of mathematical intent different operations.

### 5.4 Numerical stability primitives

The `Real` trait provides stability-oriented operations that may be preferable to composing elementary expressions manually.

| Operation   | Numerical issue addressed                                          |
| ----------- | ------------------------------------------------------------------ |
| `hypot`     | Avoids unnecessary overflow or underflow in magnitude calculations |
| `log1p`     | Retains accuracy when evaluating \\(\log(1+x)\\) for small \\(x\\) |
| `copy_sign` | Preserves sign semantics, including signed zero where supported    |
| `sech`      | Avoids overflow from directly constructing \\(\cosh(x)\\)          |
| `csch`      | Handles challenging behavior near zero and at large magnitudes     |

These functions matter because mathematically equivalent expressions can have very different numerical stability.

For small \\(x\\),

\\[ \log(1+x) \\]

may lose the increment \\(x\\) when \\(1+x\\) rounds to one.

A specialized `log1p(x)` can retain meaningful information about the small increment.

Likewise,

\\[ \operatorname{sech}(x)=\frac{1}{\cosh(x)} \\]

can underflow or encounter an infinite intermediate result when \\(\cosh(x)\\) overflows, even though the final result is finite and close to zero.

The trait-level availability of stable alternatives can reduce the need for application-specific numerical special cases.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 5.5 Floating-point classification and exceptional values

The numeric abstraction includes mechanisms for distinguishing scalar floating-point values from more general mathematical representations.

Relevant interfaces include:

- `real_cmp`
- `real_classify`
- `needs_rescaling`
- `nan`
- `is_fully_zero`

`real_classify` can decline classification when the value does not have a meaningful single scalar floating-point classification, such as a complex number or SIMD vector.

`real_cmp` similarly addresses comparison through a real embedding rather than assuming every numeric type possesses an ordinary total mathematical order.

`needs_rescaling` is intended to identify when guarded arithmetic is appropriate, including cases involving zero, subnormal, or non-finite intermediates.

These mechanisms are useful for generic numerical algorithms because they avoid incorrectly applying ordinary `f64` assumptions to exact numbers, complex numbers, and composite numeric types.

### 5.6 Complex branch cuts and signed zero

The distinction between `is_negative()` and `is_sign_negative()` is also significant.

For ordinary numeric comparison, negative zero is not strictly less than zero.

But its sign bit can still affect the limiting direction used by complex elementary functions.

For example, the principal complex logarithm has a branch cut along the negative real axis.

The sign of an infinitesimal imaginary component determines which side of that cut is approached, and IEEE-754 signed zero can encode that distinction.

Preserving signed-zero information can therefore affect the correct branch of complex logarithms, square roots, and related functions.

The 3.0.1 API explicitly documents sign-bit inspection, including negative zero, as useful for signed-zero branch-cut conventions.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



This is a consequential capability for numerical complex analysis, not merely an implementation curiosity.

## 6. Polynomial coefficient kernels and specialized arithmetic

Numerica's polynomial-kernel infrastructure deserves more detailed explanation because it exposes optimization opportunities at the boundary between generic algebraic algorithms and coefficient-specific arithmetic.

### 6.1 Separation of polynomial structure and coefficient arithmetic

A polynomial library typically manages exponent vectors, monomial ordering, term storage, normalization, and algebraic transformations.

Its coefficient domain controls addition, subtraction, multiplication, division, and related mathematical operations.

In a purely generic implementation, polynomial multiplication can repeatedly invoke scalar operations without exploiting the representation of the coefficients.

Numerica's kernel abstraction introduces another option.

Through `Ring::kernels()` and `RingKernels`, a coefficient domain can advertise specialized bulk operations that receive a structured request describing the polynomial operation.

This allows polynomial infrastructure to retain responsibility for exponents and storage while delegating selected computationally intensive operations to the coefficient implementation.

The current Symbolica 3.0.x API documents implementations for multiple finite-field representations, including machine-word and arbitrary-precision modular elements.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



Sources:

[https://docs.rs/symbolica/3.0.1/symbolica/kernels/](https://docs.rs/symbolica/3.0.1/symbolica/kernels/?utm_source=chatgpt.com)

[https://docs.rs/symbolica/3.0.1/symbolica/domains/finite_field/struct.FiniteField.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/finite_field/struct.FiniteField.html?utm_source=chatgpt.com)

### 6.2 Dense-indexed multiplication

`DensePolynomialMulRequest` represents coefficients whose exponents have already been mapped to additive dense indices.

This layout can make exponent combination simpler.

For a univariate polynomial,

\\[ A(x)=\sum_i a_ix^i,\qquad B(x)=\sum_j b_jx^j, \\]

the product coefficients satisfy

\\[ c_k=\sum\_{i+j=k}a_ib_j. \\]

The index of a result coefficient is obtained by adding exponent indices.

For appropriate multivariate dense layouts, mixed-radix indexing can similarly map exponent combinations into arithmetic on indices, provided the layout and bounds are handled correctly.

This allows a specialized implementation to focus on coefficient convolution rather than repeatedly manipulating higher-level monomial objects.

`PolynomialKernels::try_dense_mul` is an optional specialized path for this operation.

### 6.3 Chunked dense multiplication

`ChunkedDensePolynomialMulRequest` describes multiplication organized one outer coefficient chunk at a time.

The primary motivation is memory locality.

A naive multiplication strategy may repeatedly access coefficient arrays whose working set does not fit well into processor caches.

Chunked processing can organize work around smaller blocks, potentially reducing repeated movement of data between main memory and cache.

Its benefit depends on the coefficient representation, polynomial dimensions, term density, and whether the specialized kernel can exploit the chosen block structure.

A chunked kernel is not automatically faster than a generic kernel for small polynomial operands.

### 6.4 Total-degree simplex multiplication

Multivariate polynomial algorithms commonly impose a total-degree bound:

\\[ |\alpha|=\alpha_1+\cdots+\alpha_n\leq d. \\]

The set of allowable exponent vectors forms a discrete simplex.

For \\(n\\) variables and maximum total degree \\(d\\), its number of monomials is

\\[ \binom{n+d}{d}. \\]

This can be substantially smaller than a full rectangular exponent grid containing

\\[ (d+1)^n \\]

positions.

Numerica provides `TotalDegreePolynomialMulRequest` and the corresponding optional specialized multiplication method.

The performance implications concern both the arithmetic and representation.

A total-degree-aware algorithm may avoid allocating or processing coefficient positions that cannot occur within the permitted total-degree support.

However, the result of multiplying two degree-\\(d\\) polynomials may have degree as large as \\(2d\\); the algorithm must account for the intended output bound rather than assuming the input simplex is sufficient.

### 6.5 Workspace heuristics

The `PolynomialKernels` interface also includes:

```
preferred_total_degree_mul_workspace_ratio
```

The published documentation describes this as a way for a coefficient implementation to specify a stricter mixed-radix-to-simplex workspace ratio when considering a compact multiplication strategy.

This is a subtle but important extension mechanism.

The appropriate multiplication algorithm depends not only on coefficient arithmetic speed, but also on how much temporary storage a particular dense representation requires relative to the actual output.

The coefficient domain can therefore influence an algorithm-selection heuristic based on its own arithmetic characteristics.

### 6.6 Exact polynomial division

The specialized polynomial interfaces include:

- `CheckedPolynomialDivisionKernels`
- `DensePolynomialExactDivisionRequest`
- `UnivariatePolynomialDivisionRequest`
- `DivisionAttempt`

Exact division has different semantics from ordinary field division.

Given polynomials \\(A\\) and \\(B\\), an exact quotient \\(Q\\) satisfies

\\[ A=BQ \\]

with no remainder.

When coefficient arithmetic is performed in a ring rather than a field, quotient coefficients may also need to remain inside the original domain.

The specialized checked-division interface provides a mechanism for attempting a fast exact-division computation without assuming that the operation succeeds.

A high-level caller can retain a generic division algorithm for cases not handled by a specialized kernel.

### 6.7 Optional kernels and fallback semantics

A key aspect of Numerica's kernel design is that specialized operations may return `None`.

That does not necessarily mean the underlying mathematical operation is impossible.

It can mean that the specialized kernel does not support the current operand layout, coefficient values, or bounded-accumulator requirements.

The caller may then use ordinary ring operations.

This makes kernel availability different from mathematical capability.

For example, a finite-field multiplication may be mathematically valid even if its optimized coefficient convolution kernel declines a particular request.

This separation makes it possible to introduce specialized algorithms without changing the mathematical semantics of the higher-level polynomial operation.

It also requires the caller to honor any documented rule about whether a failed specialized attempt leaves its input buffers unchanged.

## 7. Geometric-sequence kernels for sparse interpolation

`GeometricSequenceKernels` is an especially distinctive part of Numerica's kernel system.

### 7.1 Mathematical mechanism

A geometric sequence has the form

\\[ s_k=cr^k. \\]

For several sequences,

\\[ s\_{j,k}=c_jr_j^k, \\]

their sum is

\\[ S_k=\sum_jc_jr_j^k. \\]

Suppose a sparse polynomial is

\\[ P(x)=\sum\_{j=1}^{t}c_jx^{e_j}. \\]

Evaluating at geometric points \\(x=\alpha^k\\) gives

\\[ P(\alpha^k) =\sum\_{j=1}^{t}c_j(\alpha^{e_j})^k. \\]

Thus repeated evaluations can be represented as sums of geometric sequences.

This structure is important in sparse polynomial interpolation, where the goal is to recover polynomial coefficients and exponents from relatively few evaluations.

### 7.2 Specialized update operation

Numerica exposes:

```
try_sum_and_advance_geometric_sequences
```

through `GeometricSequenceKernels`.

The operation combines two tasks:

1. Sum the current values of several geometric sequences.
2. Advance the sequence states by one step.

Conceptually:

\\[ S_k=\sum_j s\_{j,k}, \qquad s\_{j,k+1}=r_js\_{j,k}. \\]

A specialized implementation can fuse the summation and state updates.

This may reduce intermediate allocations, repeated domain-method dispatch, or unnecessary conversion work.

### 7.3 Finite-field specialization

The published API documents implementations for:

- `FiniteField<u32>`
- `FiniteField<u64>`

These are particularly natural environments for sparse interpolation because arithmetic occurs in a finite field and can often use efficient machine-word representations.

The API documentation also specifies an important failure behavior.

When the specialized implementation cannot handle a request using its bounded accumulator, it returns `None`. The caller must then use generic ring operations, beginning with the original, unchanged sequence values.

This means the fallback contract includes state preservation, not simply optional acceleration.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/kernels/trait.GeometricSequenceKernels.html](https://docs.rs/symbolica/3.0.1/symbolica/kernels/trait.GeometricSequenceKernels.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 7.4 Architectural significance

The geometric-sequence kernel demonstrates a broader principle of Numerica's design.

A coefficient domain can expose optimizations that operate on a mathematical pattern of computation, rather than simply providing faster scalar multiplication.

This can eliminate specialized arithmetic loops that would otherwise be duplicated in every sparse polynomial interpolation implementation.

However, Numerica's geometric-sequence kernel should not be confused with a complete interpolation package. Recovering unknown exponents and coefficients still requires the corresponding high-level interpolation algorithm and its mathematical conditions.

## 8. Exact coefficient conversion and ownership-aware ring operations

### 8.1 Rational coefficient conversion

`RationalCoefficientConversion` provides a capability for lossless conversion between supported coefficient representations and rational numbers.

This is different from arbitrary numerical casting.

A lossless conversion preserves the mathematical value, whereas converting a large rational coefficient into an ordinary machine floating-point value may introduce rounding.

The distinction matters when a higher-level algorithm needs to move between specialized coefficient formats while maintaining exactness.

For example, polynomial computation might use a compact coefficient domain for part of its execution and then recover a rational representation for exact result construction.

A lossless conversion contract can support that transition without requiring bespoke numerical validation for every coefficient.

The full set of source and destination coefficient representations is implementation-dependent; the trait should not be interpreted as a universal exact conversion facility for arbitrary mathematical domains.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/kernels/](https://docs.rs/symbolica/3.0.1/symbolica/kernels/?utm_source=chatgpt.com)

### 8.2 Borrowed and owned arithmetic

Numerica's algebraic abstractions also distinguish operations on borrowed and owned elements.

`RingOps` supports arithmetic through implementations involving element values and references.

The domain API additionally exposes operations such as:

```
try_div_owned
exact_div_owned
sub_mul_assign_many
```

The owned division interfaces allow implementations to reuse storage from an operand that is being consumed.

For arbitrary-precision values, this can matter because the storage holding a large numerator or coefficient may be much more expensive to allocate than a machine-width scalar.

An owned operation may be able to mutate or reuse internal buffers instead of constructing an entirely new allocation.

This is an opportunity for lower allocation overhead, not a universal guarantee of allocation-free execution.

### 8.3 Accumulating multiple products

`sub_mul_assign_many` subtracts several coefficient products from an accumulator.

Conceptually,

\\[ a\leftarrow a-\sum\_{i=1}^{n}b_ic_i. \\]

This pattern occurs frequently in polynomial division, elimination, and other algebraic algorithms.

A domain-specific implementation may be able to combine the arithmetic more efficiently than a generic loop that repeatedly creates independent product objects.

The benefit is especially plausible for arbitrary-precision and modular arithmetic, where intermediate coefficient representations or reductions can be costly.

The exact performance depends on the implementation. The existence of a bulk operation does not automatically establish fused execution for every coefficient domain.

### 8.4 Domain ownership and independent elements

`Ring::has_independent_elements` addresses whether a ring's elements contain enough information to be interpreted independently of the ring object.

Integers and rationals can generally be interpreted independently.

Finite-field elements may instead rely on the ring for their modulus.

This distinction is important for generic ownership, caching, and serialization.

A numerical value whose meaning depends on an external ring context cannot safely be moved into an unrelated domain merely because its underlying integer representation is identical.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/domains/finite_field/struct.FiniteField.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/finite_field/struct.FiniteField.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



## 9. Ring-aware printing and mathematical formatting

Numerica's `printer` module and the formatting facilities of its domain abstraction are another omission from the original reference.

The important capability is not merely printing a number. It is formatting an element in the context of the mathematical ring that defines its meaning.

### 9.1 `RingPrinter`

The published domain interface includes `RingPrinter`, which wraps a ring-associated element in a formatting object suitable for Rust's `format!` and related formatting facilities.

The ring is responsible for determining the mathematical representation of the element through its formatting methods.

This allows generic structures to format their coefficients without requiring every coefficient representation to implement an independently sufficient `Display` interface.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/domains/index.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/index.html?utm_source=chatgpt.com)

### 9.2 `WrappedRingElement`

`WrappedRingElement` associates an element with its ring.

This also enables ordinary arithmetic operators to be used with the wrapped values, delegating operations through the corresponding domain.

The distinction is significant because Numerica deliberately separates mathematical elements from the domains defining their operations.

For example, the internal integer representation of a modular element may be meaningless without its modulus.

A wrapper can supply the missing context for formatting and arithmetic.

This reduces the amount of custom adapter code needed when generic algebraic elements are used in ordinary Rust expressions.

### 9.3 Distinction from Symbolica's expression printer

Numerica's printing functionality should not be conflated with Symbolica's full symbolic expression printing facilities.

Symbolica provides a broader formatting system for expressions, polynomials, mathematical notation, and output modes.

Numerica's central responsibility is formatting ring elements and ring-aware mathematical structures.

The standalone Numerica `printer` module is confirmed in the published crate index, but a complete 3.0.1 inventory of its public types was not available from the retrieved sources. More extensive Symbolica print options should therefore not be attributed to Numerica without direct verification.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+1



## 10. Sparse matrix storage and conversion

The previous reference left the specific sparse storage representation unclear.

The published matrix interface confirms that `Matrix::to_sparse()` converts a dense matrix into compressed sparse row (CSR) format.

Source: [https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html](https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html?utm_source=chatgpt.com)

### 10.1 CSR representation

CSR commonly uses three arrays:

- Row pointers indicating where each row begins.
- Column indices of stored nonzero entries.
- Values corresponding to those column indices.

For a matrix with \\(n\\) rows and \\(k\\) stored nonzeros, typical storage is proportional to

\\[ O(n+k) \\]

rather than the \\(O(mn)\\) entries of a dense \\(m\times n\\) matrix.

CSR is particularly suitable for row traversal and sparse matrix-vector multiplication.

### 10.2 Dense-to-sparse conversion

Numerica's conversion facility allows a matrix initially constructed in dense form to be transformed into its sparse representation without separately implementing CSR assembly.

For example, a mathematical algorithm may begin with dense construction because its entries are easier to generate in row-major form, then move into a sparse representation once the sparsity pattern is established.

The conversion does not imply that all subsequent operations preserve sparsity efficiently.

Some matrix transformations can create fill-in, and a sparse algorithm may become less efficient when the number of nonzero entries grows substantially.

### 10.3 Sparse algorithms and version boundaries

The 3.0 release notes specifically mention sparse structural validation and corrections to sparse identity checks.

These are relevant because sparse storage contains structural information beyond the stored values.

An invalid row-pointer array, inconsistent column indices, or incorrect dimension metadata can corrupt mathematical results even when every stored coefficient is valid.

The public evidence confirms CSR conversion and sparse matrix infrastructure, but the complete standalone Numerica 3.0.1 decomposition and iterative-solver inventory remains unverified.

In particular, it would be inappropriate to infer the presence of general sparse LU, Cholesky, Krylov solvers, or preconditioners merely from CSR support.

## 11. Additional exact-rational approximation utilities

The further source review also revealed rational approximation operations not covered in the original overview or the first gap analysis.

The published rational interface includes:

| Method                 | Capability                                                               |
| ---------------------- | ------------------------------------------------------------------------ |
| `truncate_denominator` | Find a rational approximation under a denominator bound                  |
| `round`                | Find a simpler rational within a relative-error interval                 |
| `round_in_interval`    | Find a rational with a small numerator or denominator inside an interval |

Source: [https://docs.rs/symbolica/3.0.1/symbolica/prelude/type.Rational.html](https://docs.rs/symbolica/3.0.1/symbolica/prelude/type.Rational.html?utm_source=chatgpt.com)&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



### 11.1 Denominator-bounded approximation

Given an exact rational

\\[ r=\frac ab, \\]

a denominator-bounded approximation seeks

\\[ \frac pq \\]

subject to a specified bound on \\(q\\).

This is useful when a mathematically exact value has a very large denominator but a simpler rational approximation is sufficient.

For example,

\\[ \frac{333333}{1000000} \\]

is close to \\(1/3\\).

A denominator-limited approximation can prefer the simpler representation when permitted by the selected bound.

This differs from conventional decimal rounding because it optimizes over rational representations rather than fixed decimal places.

### 11.2 Relative-error rational simplification

For a target rational \\(r\\) and relative error \\(\epsilon\\), `round` searches for a simpler rational inside the permitted interval

\\[ [r(1-\epsilon),r(1+\epsilon)]. \\]

The documented constraints require an appropriate positive relative error less than one.

The objective is a simpler exact rational satisfying an approximation requirement, not a floating-point result.

This can be useful when downstream calculations benefit from small exact coefficients.

### 11.3 Rational selection inside an interval

`round_in_interval` takes a lower and upper rational bound and selects a comparatively simple rational within them.

This provides an interesting complement to interval arithmetic.

Suppose a numerical calculation yields a rigorous interval

\\[ x\in[\ell,u]. \\]

A rational simplification algorithm may identify a compact candidate rational inside the interval.

That candidate is not necessarily the unknown exact value merely because it lies within the enclosure. Additional mathematical knowledge is needed to establish identity.

Nevertheless, interval-constrained rational selection can be valuable for constructing simple rational approximations to validated numerical results.

This is a distinct capability from CRT-based rational reconstruction: one recovers rational values from modular information, while the other chooses rational approximations under size or error constraints.

## 12. Python interoperability and embedding

Numerica's optional Python integration includes more than generic PyO3 compatibility.

The published numerical API contains dedicated Python-facing floating-point representations and input adapters.

| Type or function              | Role                                                         |
| ----------------------------- | ------------------------------------------------------------ |
| `PythonFloat`                 | Immutable real number with arbitrary binary precision        |
| `PythonComplexFloat`          | Immutable complex value using arbitrary-precision components |
| `PythonMultiPrecisionFloat`   | Adapter for Python numeric inputs                            |
| `PythonMultiPrecisionComplex` | Adapter for complex inputs and real/imaginary pairs          |
| `register_python_floats`      | Registers the numeric types in a Python module               |

Source: [https://docs.rs/symbolica/3.0.1/symbolica/domains/float/index.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/float/index.html?utm_source=chatgpt.com)

### 12.1 Representations versus adapters

The distinction between an exposed numeric object and an input adapter matters.

An object such as `PythonFloat` supplies an externally visible high-precision numerical type.

An adapter such as `PythonMultiPrecisionFloat` focuses on accepting Python input representations and converting them into the corresponding Rust-backed numeric value.

Separating these responsibilities helps maintain explicit conversion semantics without requiring every Rust numerical operation to accept arbitrary dynamic Python values.

### 12.2 Module registration

The `register_python_floats` function allows an embedding application to register the floating-point types in a Python module.

This can support Rust extensions that need to expose Numerica-compatible high-precision values across the Python boundary.

It does not imply that the standalone Numerica crate is distributed as an independently installable Python package.

Likewise, numerical Python registration does not make Symbolica's broader symbolic expression functionality a part of Numerica.

### 12.3 Precision and conversion boundaries

Python interoperability raises several concerns:

- Conversion from Python `float` can import binary rounding already present in the input.
- Conversion from a string may retain decimal input information unavailable from a pre-rounded float.
- Complex values require maintaining appropriate precision semantics for their components.
- Python-facing immutable values may differ in ownership and mutation behavior from internal Rust numerical objects.
- Repeated crossings of the Python/Rust boundary can introduce overhead even when the Rust arithmetic itself is efficient.

These distinctions matter when implementing precision-sensitive numerical interfaces.

## 13. Integration and utility infrastructure: remaining boundaries

The previous gap analysis identified Numerica's `utils` module and incomplete detail about Monte Carlo integration controls.

The current evidence supports several relevant conclusions, but not an exhaustive API inventory.

### 13.1 Utility abstractions

The crate index identifies `utils` as providing utility traits and structures.

In the broader Symbolica API, a `BorrowedOrOwned` abstraction is publicly visible, but that alone does not establish its exact standalone Numerica export path or version-specific implementation.

Such an abstraction can be useful when APIs need to accept either borrowed data or values whose ownership has been transferred.

For numerical computation, this may affect allocation behavior and buffer reuse.

However, the available source material is insufficient to describe a complete set of Numerica utility types or guarantees.

Source: [https://docs.rs/numerica/latest/numerica/](https://docs.rs/numerica/latest/numerica/?utm_source=chatgpt.com)

### 13.2 Monte Carlo sampling and statistics

Numerica's integration abstractions include `ContinuousGrid`, `DiscreteGrid`, `Sample`, `Probe`, `StatisticsAccumulator`, and `MonteCarloRng`.

Their compositional role was covered in the original document. The remaining details worth distinguishing are primarily operational semantics:

Sampling weights. A sample's mathematical contribution depends on the density from which it was drawn. The Jacobian or inverse-density weight must agree with the actual sampling procedure.

Adaptive training. Data used to adapt a sampling distribution is not automatically equivalent to an independent sample drawn from a fixed final distribution. Statistical analysis must respect the estimator's actual sampling scheme.

Discrete and continuous composition. When discrete channels have different continuous domains, their contributions must be combined using the correct channel probabilities and within-channel weights.

Statistics accumulation. Combining iterations involves more than averaging their reported integral estimates if those estimates have different variances or correlations.

Parallel execution. Correct aggregation of independently generated samples requires preserving the estimator's weighting and random-stream assumptions.

These are mathematical constraints on the use of Numerica's infrastructure. They should not be interpreted as claims that Numerica itself automatically handles every combination correctly.

The 3.0 release also introduced stronger validation for grid dimensions, bin counts, and sampling settings, with Rust continuous and discrete constructors returning `Result`.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra



## 14. Consequential interactions between the added capabilities

The new functionality substantially expands the architectural possibilities visible in the original reference.

### 14.1 Exact matrix solving with minimized coefficient growth

An exact algebraic workflow can combine:

- Matrix content extraction.
- Fraction-free row reduction.
- Exact integer arithmetic.
- Modular arithmetic when appropriate.
- Rational reconstruction.
- Verification over the original domain.

The importance lies in reducing unnecessary arithmetic complexity.

Content extraction may reduce common factors. Fraction-free elimination can avoid generating fractions during elimination. Modular methods may offer an alternative when direct exact arithmetic becomes expensive.

These are complementary techniques rather than one automatic Numerica solver.

### 14.2 Validated exact-to-floating computation

Exact rational values can be passed into multiprecision floating-point computation through directed conversion.

The resulting numerical calculation can use directed rounding or ball arithmetic to preserve enclosures where the supported operations provide the necessary guarantees.

A rational approximation can subsequently be selected within a numerical interval, subject to its size and accuracy constraints.

This supports a general separation between:

1. Exact representation of mathematical input.
2. Efficient approximate numerical calculation.
3. Rigorous enclosure of calculated values.
4. Construction and verification of simpler exact output candidates.

Not every Numerica numerical operation is necessarily implemented with certified enclosure semantics, so this composition must be restricted to operations with the required guarantees.

### 14.3 One mathematical function, multiple execution representations

Numerica's numerical traits can potentially support several representations of the same mathematical operation:

| Representation                   | Primary purpose                      |
| -------------------------------- | ------------------------------------ |
| Ordinary scalar floating point   | Conventional numerical evaluation    |
| SIMD scalar abstraction          | Batched independent evaluations      |
| `DoubleFloat`                    | Increased fixed-width precision      |
| Multiprecision `Float`           | Configurable working precision       |
| Error-propagating floating point | Track estimated accuracy degradation |
| Ball arithmetic                  | Obtain certified enclosures          |
| Dual/hyperdual numbers           | Propagate derivatives                |

This is not automatic universal interchangeability. An algorithm must use operations supported by each desired type, and some operations may have incompatible semantic requirements.

Nevertheless, the trait architecture can reduce duplication when the same mathematical model needs to be evaluated under different numerical objectives.

### 14.4 Exact interpolation and specialized modular kernels

Sparse polynomial interpolation can combine:

- Machine-word finite-field arithmetic.
- Geometric-sequence kernels.
- Specialized polynomial multiplication.
- Exact division checks.
- Modular reconstruction.
- Rational coefficient conversion.

The distinctive advantage is that optimized coefficient operations can be reused by higher-level algorithms without embedding the optimization directly into the polynomial's structural representation.

An implementation can retain a generic fallback while using specialized kernels where the coefficient domain supports them.

## 15. Version-specific caveats and source map

The most important remaining documentation limitation is the mismatch between some indexed Numerica versions and the current 3.0.x release.

The following distinctions should accompany the addendum when incorporated into a durable library reference.

| Capability                                               | Evidence status                                                                                           |
| -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------- |
| Fraction-free matrix methods                             | Confirmed in published 3.0.1 Symbolica-facing matrix API                                                  |
| Matrix content, primitive part, mapping, differentiation | Confirmed in published 3.0.1 Symbolica-facing matrix API                                                  |
| CSR dense-to-sparse conversion                           | Confirmed in matrix API                                                                                   |
| `RealBall`, `ComplexBall`, directed rounding             | Confirmed in 3.0.1 floating-point API and 3.0 release notes                                               |
| Specialized polynomial and geometric-sequence kernels    | Confirmed in 3.0.x API                                                                                    |
| SIMD `f64x2` and `f64x4` trait implementations           | Verified in indexed Numerica 2.1 source; exact 3.0.1 behavior not independently confirmed                 |
| Ring-aware printers and wrappers                         | Confirmed in published domain API                                                                         |
| Rational approximation methods                           | Confirmed in published 3.0.1 Symbolica-facing rational API                                                |
| Optional Python numerical types                          | Confirmed in published 3.0.1 Symbolica-facing API; standalone feature configuration requires verification |
| Complete `utils` functionality                           | Not exhaustively verified                                                                                 |
| Complete sparse solver family                            | Not exhaustively verified                                                                                 |

### Direct primary sources

- Numerica crate and module index: [https://docs.rs/numerica/latest/numerica/](https://docs.rs/numerica/latest/numerica/?utm_source=chatgpt.com)
- Numerica source repository: [https://github.com/symbolica-dev/numerica](https://github.com/symbolica-dev/numerica?utm_source=chatgpt.com)
- Published SIMD implementation: [https://docs.rs/numerica/latest/src/numerica/domains/float/simd.rs.html](https://docs.rs/numerica/latest/src/numerica/domains/float/simd.rs.html?utm_source=chatgpt.com)
- Matrix API: [https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html](https://docs.rs/symbolica/3.0.1/symbolica/tensors/matrix/struct.Matrix.html?utm_source=chatgpt.com)
- Floating-point types: [https://docs.rs/symbolica/3.0.1/symbolica/domains/float/index.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/float/index.html?utm_source=chatgpt.com)
- Multiprecision arithmetic and directed rounding: [https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/float/struct.Float.html?utm_source=chatgpt.com)
- Polynomial kernels: [https://docs.rs/symbolica/3.0.1/symbolica/kernels/](https://docs.rs/symbolica/3.0.1/symbolica/kernels/?utm_source=chatgpt.com)
- Geometric-sequence kernels: [https://docs.rs/symbolica/3.0.1/symbolica/kernels/trait.GeometricSequenceKernels.html](https://docs.rs/symbolica/3.0.1/symbolica/kernels/trait.GeometricSequenceKernels.html?utm_source=chatgpt.com)
- Finite-field kernel implementations: [https://docs.rs/symbolica/3.0.1/symbolica/domains/finite_field/struct.FiniteField.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/finite_field/struct.FiniteField.html?utm_source=chatgpt.com)
- Domain formatting: [https://docs.rs/symbolica/3.0.1/symbolica/domains/index.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/index.html?utm_source=chatgpt.com)
- Rational approximation: [https://docs.rs/symbolica/3.0.1/symbolica/prelude/type.Rational.html](https://docs.rs/symbolica/3.0.1/symbolica/prelude/type.Rational.html?utm_source=chatgpt.com)
- Numerica 3.0 release changes: [https://symbolica.io/posts/symbolica_3_0_release/](https://symbolica.io/posts/symbolica_3_0_release/?utm_source=chatgpt.com)

## Addendum conclusion

The additional capabilities reveal three important dimensions of Numerica that were not sufficiently visible in the original overview.

First, Numerica's numerical type system supports more than a choice of scalar precision. Its abstraction encompasses SIMD lanes, dynamic precision, directed rounding, certified enclosures, exceptional-value semantics, and automatic differentiation. This can make numerical representation an independently selectable dimension of an algorithm's design, within the constraints of supported operations.

Second, Numerica's exact linear algebra extends beyond conventional field-based matrix operations. Fraction-free elimination, content extraction, primitive normalization, domain mapping, and exact solving provide mathematical machinery particularly valuable when intermediate arithmetic complexity matters.

Third, Numerica's kernel interfaces are substantive algorithmic extension points. Specialized dense and total-degree multiplication, geometric-sequence advancement, exact division, and coefficient conversion make it possible for higher-level computational algebra algorithms to exploit arithmetic-domain-specific optimizations without duplicating their entire implementations.

Taken together with the original reference, these additions provide a more complete view of Numerica as a reusable numerical and algebraic substrate. The remaining gaps are concentrated in detailed version-specific API coverage, particularly the standalone crate's utilities, sparse algorithms, SIMD completeness, and some integration internals, rather than in the fundamental mathematical architecture.