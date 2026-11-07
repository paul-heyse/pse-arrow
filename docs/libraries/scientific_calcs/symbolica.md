# Symbolica 3.0.1 — Comprehensive Rust Library and Ecosystem Reference

Research date: October 21, 2026 | Target: Symbolica 3.0.1 (latest verified stable release, October 6, 2026)

## 1. Executive overview

Symbolica is a high-performance computer algebra system (CAS) written in Rust, designed for sophisticated symbolic mathematics, exact algebraic computation, expression transformation, and compilation of mathematical expressions into efficient numerical execution.

Unlike libraries focused primarily on interactive symbolic mathematics, Symbolica is explicitly engineered for programmatic integration, large mathematical expressions, efficient algebraic kernels, and converting symbolic mathematics into executable numerical computations.

Its capabilities span several areas traditionally handled by separate systems:

- Symbolic expression engine: Canonical expression representation, parsing, algebraic simplification, substitution, differentiation, and extensible mathematical functions.
- Pattern matching and rewriting: Structural matching, wildcards, conditional transformations, and configurable expression-transformation pipelines.
- Exact computational algebra: Multivariate and univariate polynomials, rational functions, factorization, GCD algorithms, Gröbner bases, finite fields, algebraic number fields, and exact equation solving.
- Numerical execution and code generation: Optimized evaluation, common-subexpression elimination, compiled evaluators, JIT compilation, SIMD, C++, assembly, and CUDA generation.
- Symbolic–numerical integration: Arbitrary-precision arithmetic, automatic differentiation, numerical integration, series expansions, and symbolic preprocessing of numerical workloads.
- Structured mathematics: Matrices, tensors, graphs, combinatorics, and specialized mathematical domains.
- Large-scale computation: Streaming expressions to disk, efficient expression representations, and facilities for processing algebra too large to retain conveniently in memory.
- Extensibility and interoperability: Rust and Python APIs, additional language bindings, user-defined symbolic functions, custom coefficient domains, and related numerical and graph libraries.

The most consequential architectural property is that Symbolica connects general-purpose symbolic manipulation to specialized exact algebra and optimized numerical execution within a common computational framework. It can therefore perform much more than expression simplification: it can serve as a mathematical transformation engine, an algebraic backend, or a frontend for generating high-performance numerical kernels.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



Primary documentation:

Official Rust API

[https://docs.rs/symbolica/3.0.1/symbolica/](https://docs.rs/symbolica/3.0.1/symbolica/?utm_source=chatgpt.com)

Official capability guide

[https://symbolica.io/docs/](https://symbolica.io/docs/?utm_source=chatgpt.com)

Upstream repository

[https://github.com/symbolica-dev/symbolica](https://github.com/symbolica-dev/symbolica?utm_source=chatgpt.com)

### Version and licensing distinctions

Symbolica 3.0.1 is a significant current-version target. The 2.x releases introduced major changes to evaluation, symbolic integration, specialized functions, pattern matching, and numerical compilation. The 3.x documentation additionally covers functionality such as exact solution sets, specialized numerical and algebraic kernels, and newer APIs that should not be assumed to exist in older versions.

Symbolica is source-available, not conventionally open-source software. Its source license permits inspection and certain authorized modifications but restricts redistribution and use. Hobbyist usage can be free; professional academic and commercial use generally requires an appropriate license. This is particularly important when evaluating redistribution, embedding, CI execution, and deployment of applications built around Symbolica.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Current source license

[https://github.com/symbolica-dev/symbolica/blob/main/License.md](https://github.com/symbolica-dev/symbolica/blob/main/License.md?utm_source=chatgpt.com)

Licensing and plans

[https://symbolica.io/license/](https://symbolica.io/license/?utm_source=chatgpt.com)

## 2. Core architecture: expression representation and algebraic domains

### 2.1 General expressions versus specialized mathematical structures

Symbolica distinguishes between general symbolic expressions and specialized algebraic representations. This distinction is fundamental to using the library effectively.

| Representation                   | Principal Rust API                       | Role                                                                |
| -------------------------------- | ---------------------------------------- | ------------------------------------------------------------------- |
| General expressions              | `Atom`, `AtomView`, `AtomCore`           | Arbitrary symbolic structures, functions, algebraic transformations |
| Multivariate polynomials         | `MultivariatePolynomial<R, E>`           | Polynomial arithmetic over a selected coefficient ring              |
| Univariate polynomials           | `UnivariatePolynomial`                   | Specialized single-variable polynomial algorithms                   |
| Rational functions               | `RationalPolynomial`                     | Ratios of polynomials, cancellation, exact arithmetic               |
| Algebraic numbers and extensions | `AlgebraicExtension`, algebraic contexts | Exact arithmetic involving roots of polynomials                     |
| Power series                     | Series types in `poly`                   | Truncated expansions, including noninteger and negative powers      |
| Numerical and algebraic domains  | `Ring`, `Field`, `EuclideanDomain`       | Reusable algebraic operations and domain-specific algorithms        |
| Evaluators                       | `ExpressionEvaluator`                    | Optimized numerical programs derived from expressions               |

A general `Atom` is flexible enough to represent expressions such as

\\[ \sin(x^2)+\frac{f(x,y)}{1+x}, \\]

where `f` may be an entirely user-defined symbolic function.

A polynomial object instead provides an explicit coefficient domain, indeterminates, and an algebraic structure. It can use algorithms that would be unnecessarily expensive or difficult to apply directly to arbitrary expression trees.

This separation offers an important design opportunity: retain general symbolic structures where expressiveness matters, and convert into specialized algebraic domains for algorithms requiring stronger mathematical guarantees.

It also imposes a conversion boundary. Treating a nonlinear subexpression as a polynomial variable is not equivalent to understanding that subexpression's internal algebraic properties.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



[https://docs.rs/symbolica/3.0.1/symbolica/atom/index.html](https://docs.rs/symbolica/3.0.1/symbolica/atom/index.html?utm_source=chatgpt.com)

[https://docs.rs/symbolica/3.0.1/symbolica/domains/index.html](https://docs.rs/symbolica/3.0.1/symbolica/domains/index.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/polynomials.html](https://symbolica.io/docs/polynomials.html?utm_source=chatgpt.com)

### 2.2 `Atom`, `AtomView`, and normalization

The expression engine's central representation consists of numbers, symbols, functions, sums, products, and powers.

The most important interfaces are:

- `Atom`: An owned mathematical expression.
- `AtomView`: A borrowed view into an expression, avoiding an owned copy for many operations.
- `AtomCore`: A trait exposing much of the common expression functionality.
- `Symbol`: A registered identity that can represent either a variable or a function head.
- `AtomOrView`: A representation allowing owned or borrowed operands.
- `AliasedAtom`: A facility for representing repeated subexpressions through aliases.

Normalization happens during expression construction. For example, `x*x` is represented as `x^2`, and numerical sums are combined.

However, canonical expression normalization is not complete algebraic equivalence testing. An unexpanded product and its expanded polynomial can represent the same mathematical function without having the same structural representation. Operations such as `expand`, `factor`, `cancel`, or conversion to a rational polynomial address different forms of that equivalence problem.

This is particularly important for symbolic equality, caching, and deduplication: structural equality is inexpensive but weaker than proving a general mathematical identity.

Rust illustration, following the documented 3.0 API:

```
use symbolica::prelude::*;

fn main() {
    let x = symbol!("x");
    let expression = parse!("x^2*log(2*x + y) + exp(3*x)");

    let derivative = expression.derivative(x);
    println!("{derivative}");
}
```

### 2.3 Symbol registration and mathematical attributes

Symbolica's symbol registry is more consequential than a simple mapping from names to identifiers.

Symbols can have:

- Explicit namespaces to prevent collisions between mathematical packages.
- Symmetric, antisymmetric, and cyclic symmetry attributes.
- Linear and scalar behavior affecting algebraic normalization.
- Assumptions and properties used in symbolic reasoning.
- Tags and aliases for classification and identification.
- Custom callbacks for normalization, formatting, differentiation, series, and numerical evaluation.
- Typed user data associated with symbolic definitions.

For example, declaring a function symmetric allows normalization to recognize argument permutations as representing the same symbolic value. Declaring a function linear allows scalar extraction and distribution to be handled as properties of the symbolic head rather than as repeated application-specific rewrite rules.

This gives Symbolica some of the characteristics of an extensible mathematical language: user-defined mathematical objects can participate in native expression operations.

There are two important constraints.

First, symbol attributes are registered globally and cannot safely be changed after expressions using that symbol exist. Definitions should therefore be established before parsing expressions that depend on them.

Second, formatted names can omit namespaces. Two identically printed symbols may have distinct fully qualified identities. Namespace-aware formatting is important when exchanging or debugging expressions.

In 3.0, property queries can also have an unknown result. Unknown must not silently be interpreted as false, particularly in conditional algebraic rewrites.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+2



[https://symbolica.io/docs/expressions.html](https://symbolica.io/docs/expressions.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/symbols.html](https://symbolica.io/docs/symbols.html?utm_source=chatgpt.com)

## 3. Pattern matching, rewriting, and mathematical transformation

This is one of Symbolica's strongest and most architecturally significant capability families.

### 3.1 Structural and algebra-aware matching

Symbolica's pattern language supports named wildcards, repeated wildcard consistency, sequences, alternative patterns, and conditions.

The principal wildcard forms are:

| Form                 | Meaning                                                   |
| -------------------- | --------------------------------------------------------- |
| `x_`                 | Match a single expression                                 |
| `x__`                | Match one or more arguments                               |
| `x___`               | Match zero or more arguments                              |
| Optional patterns    | Permit omission under an operation-specific identity      |
| Alternative patterns | Accept several pattern shapes                             |
| Restricted patterns  | Constrain matches by mathematical properties or structure |

Matches are not limited to fixed-position syntax trees. The matcher also understands relevant properties of commutative operations and symbol attributes. This enables algebraic rules to operate across sums, products, and selected user-defined functions without first manually enumerating operand permutations.

Repeated wildcard occurrences must bind consistently. Conditional matching can restrict rules to, for example, particular integer values, sign properties, or structural forms.

The system exposes match iterators as well as replacement operations, allowing inspection of multiple possible matches rather than forcing immediate substitution.

Important distinction: a match is still governed by Symbolica's normalized representation and the matcher's specified semantics. It is not an unrestricted theorem prover. Associative or commutative matching with multiple sequence wildcards can also produce a large search space.

The 3.0 matcher changed the handling of antisymmetric function matching to a more positional interpretation. Rewriting systems that rely on signed argument permutations should follow the migration documentation rather than assuming earlier behavior.

### 3.2 Rules, replacement, and conditions

Representative operations include `replace`, `replace_multiple`, pattern-based substitutions, and constrained replacements.

These mechanisms can encode mathematical laws such as:

\\[ f(x,0)\rightarrow x,\qquad g(x,x)\rightarrow 0, \\]

or domain-specific reductions involving special functions, indexed objects, and algebraic relations.

Conditions are particularly useful because many valid symbolic transformations have nontrivial preconditions. For example,

\\[ \frac{x^2-y^2}{x-y}=x+y \\]

requires attention to the original expression's domain and the condition \\(x\ne y\\).

A conditional rewrite system should not equate "not proved true" with "proved false." Symbolica's three-valued condition handling helps preserve this distinction.

### 3.3 Transformers: composable rewriting programs

A `Transformer` represents a reusable transformation program.

Rather than repeatedly crossing the Python/Rust boundary or constructing ad hoc recursive walkers, transformations can be composed from native operations such as expansion, replacement, filtering, conditional execution, repetition, and term mapping.

An illustrative transformation sequence might:

1. Expand selected products.
2. Apply a set of algebraic identities.
3. Repeat until no further changes occur.
4. Normalize resulting terms.
5. Apply a final collection or factorization pass.

Transformers support conditional execution and repeated passes, and they can map operations over terms using multiple cores.

They are useful where a mathematical subsystem needs an explicit, inspectable normalization procedure or domain-specific simplification pipeline.

The documentation recommends constructing reusable transformers once rather than rebuilding them for each expression. Thread creation, callback overhead, and repeated rule preparation may otherwise erase much of the benefit of a native execution engine.

A further distinction is that Symbolica's `Transformer` is an ordered rewriting facility, not a general equality-saturation engine. It should not be assumed to perform globally optimal rewrite exploration or automatically find the lowest-cost expression among all equivalent forms.

### 3.4 Extensibility without a separate rewrite engine

Combining symbol attributes with pattern matching can eliminate substantial categories of bespoke mathematical infrastructure.

For instance, a mathematical application can define symbolic heads with symmetry or linearity properties, implement a small number of specialized canonicalization rules, and compose those with native expansion, coefficient collection, and differentiation.

This provides a common expression language for multiple algebraic operations rather than requiring each operation to implement its own parser, tree representation, traversal, and substitution system.

The main engineering tradeoff is controlling intermediate-expression growth and rewrite termination. Repeated expansion and unconstrained rewriting can generate much more work than an equivalent computation performed in a specialized polynomial domain.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+2



[https://symbolica.io/docs/pattern_matching.html](https://symbolica.io/docs/pattern_matching.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/transformers.html](https://symbolica.io/docs/transformers.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/upgrade/migration_to_3_0.html](https://symbolica.io/docs/upgrade/migration_to_3_0.html?utm_source=chatgpt.com)

## 4. Polynomial arithmetic, rational functions, and exact algebra

### 4.1 Multivariate polynomial kernel

Symbolica implements specialized polynomial types and algorithms rather than performing every polynomial operation on general symbolic expressions.

Its polynomial facilities cover:

- Univariate and multivariate polynomials over configurable coefficient domains.
- Polynomial addition, multiplication, division, differentiation, GCD computation, and factorization.
- Finite-field arithmetic and modular algorithms.
- Rational-function normalization and cancellation.
- Polynomial systems and Gröbner bases.
- Algebraic extensions and polynomials with algebraic-number coefficients.
- Variable ordering, coefficient extraction, and conversion between algebraic representations.

The coefficient ring is not incidental metadata: it determines which operations are mathematically valid and which optimized algorithms are available.

For example, polynomials over \\(\mathbb Z\\), \\(\mathbb Q\\), \\(\mathbb F_p\\), and an algebraic extension field may have the same visible monomials but different factorization results.

Rust's `MultivariatePolynomial<R, E>` exposes the coefficient-domain type `R` and exponent-storage type `E`. The latter can matter for compactness and arithmetic costs; a narrow exponent type should be used only when it safely represents the required degrees.

### 4.2 Algebraic domain abstraction

The domain architecture provides interfaces such as `Ring`, `Field`, `EuclideanDomain`, `OrderedRing`, `RealEmbedding`, and `SampleableRing`.

A distinguishing design choice is that a ring instance can own the algebraic operations and contextual information, independently of its element representation. For example, a finite-field element does not necessarily carry its modulus.

This supports operations over parameterized domains while avoiding the assumption that all algebraic values behave like self-contained Rust numbers.

It also allows algorithms to use stronger constraints when available. A general ring may support addition and multiplication, while a field supports inversion; a Euclidean domain supports division with remainder and GCD-related algorithms.

Symbolica 3.0 also exposes specialized coefficient-domain kernels through its `kernels` module for selected bulk polynomial and interpolation operations. These are optimized internal or extensibility mechanisms, not a reason to assume that every custom coefficient type automatically receives identical performance.

### 4.3 Finite fields and modular computation

Prime fields and Galois fields are first-class polynomial coefficient domains.

For example, the Rust API provides `Zp` for a prime field and `AlgebraicExtension::galois_field` for constructing an extension field.

These facilities are useful beyond finite-field applications themselves. Modular polynomial algorithms can perform work over several suitable finite fields before reconstructing a result over the integers or rationals.

That is an important source of Symbolica's exact-algebra capabilities: large integer coefficients need not be manipulated naively throughout every intermediate polynomial operation.

The upstream 3.0 release specifically identifies improvements to modular GCD reconstruction, polynomial multiplication, resultants, and factor reconstruction. Such performance improvements are algorithm-dependent, not uniform guarantees for arbitrary workloads.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+3



### 4.4 Gröbner bases and polynomial systems

`GroebnerBasis` provides a representation and algorithms for polynomial ideals.

Supported ordering choices include lexicographic and graded reverse lexicographic (`grevlex`) order.

These bases can support operations such as ideal reduction, solving algebraic systems, elimination, and testing polynomial consequences of a set of equations.

The ordering matters substantially:

- Lexicographic order can yield elimination-friendly structures.
- Grevlex frequently makes basis construction less expensive.
- The best ordering for computing the basis need not be the best ordering for consuming its result.

Polynomial system complexity can increase dramatically with the number of variables, degrees, and ideal structure. Gröbner-basis availability does not imply that every nonlinear system can be handled cheaply.

### 4.5 Rational polynomials and rational-function transformations

`RationalPolynomial` represents ratios of polynomials over a chosen domain.

This creates a more specialized computation model for expressions such as

\\[ \frac{x^4-1}{x^2-1}, \\]

whose numerator and denominator have common algebraic factors.

Relevant capabilities include cancellation, rational simplification, factoring, combination of rational terms, and partial fraction decomposition.

Symbolica 3.0 adds improved multivariate partial fractions: the expression-level `apart` functionality can accept selected variables or choose variables automatically.

However, cancellation in a rational-function field and equality of partially defined functions are not always interchangeable. Removing a common factor can remove information about excluded input values. Any application requiring exact domain preservation must track those conditions separately.

### 4.6 Algebraic number fields: a particularly important 3.0 enhancement

Symbolica 3.0 supports polynomial computation in fields extended by specific algebraic numbers.

Consider:

\\[ p(x)=x^2-2. \\]

Over the rationals this polynomial is irreducible. Over \\(\mathbb Q(\sqrt 2)\\), it factors into

\\[ (x-\sqrt 2)(x+\sqrt 2). \\]

The distinction is represented explicitly by the coefficient field.

The Rust API includes `AlgebraicContext` and `to_polynomial_in_algebraic_extension`, allowing a calculation to preserve an algebraic extension and its embedding.

A critical semantic distinction is between:

- A formal field extension, specified by a minimal polynomial.
- An embedded algebraic number, which identifies a particular root and therefore an interpretation in the real or complex numbers.

A minimal polynomial alone does not identify which root a generator denotes. When converting back to a symbolic expression, a preserved algebraic context is needed to reconstruct the intended embedded value.

The new system can also discover algebraic generators such as radicals, combine generators into a common field, and factor polynomials within that field.

These capabilities are valuable for exact solutions, algebraic identities, symbolic spectral problems, and avoiding premature floating-point approximations.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+2



### 4.7 Coefficient rings and controlling intermediate expression growth

An easily overlooked Symbolica capability is the ability to treat selected symbolic variables as part of a coefficient ring rather than as independent term-level variables.

For example,

\\[ 3x\varepsilon+\frac{x}{2} \\]

can instead be represented as

\\[ \left(3\varepsilon+\frac12\right)x. \\]

This is more than a formatting preference.

If an algorithm repeatedly transforms expressions involving a parameter \\(\varepsilon\\), treating the parameter as a coefficient can prevent repeated expansion into separate terms.

Symbolica provides `set_coefficient_ring` and coefficient-conversion facilities for this purpose.

There is a tradeoff: once an expression is embedded within an opaque coefficient, the general pattern matcher cannot freely inspect its internal structure. Conversion back to an ordinary expression is available when required.

This creates a useful division of labor between compact coefficient arithmetic and structural symbolic rewriting.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+2



[https://symbolica.io/docs/coefficients.html](https://symbolica.io/docs/coefficients.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/polynomials.html](https://symbolica.io/docs/polynomials.html?utm_source=chatgpt.com)

[https://docs.rs/symbolica/3.0.1/symbolica/poly/index.html](https://docs.rs/symbolica/3.0.1/symbolica/poly/index.html?utm_source=chatgpt.com)

## 5. Exact equation solving and solution semantics

Exact solving is one of Symbolica 3.0's most important additions.

### 5.1 Solving systems over explicit domains

The solver supports selected linear and nonlinear algebraic equation systems.

A system is formulated using equations or expressions required to equal zero, with specified unknowns. Symbols not included among the unknowns can act as parameters.

The default domain is complex numbers, with options for real, rational, or integer solutions.

The principal result is not simply a vector of roots. It is a `SolutionSet` composed of solution branches, enabling more expressive outcomes.

For example, the system

\\[ x+y=3,\qquad x-y=1 \\]

has a unique solution, \\(x=2,\ y=1\\).

By contrast,

\\[ x+y=1 \\]

has a family of solutions with a free variable.

The solver's solution representation can distinguish those cases rather than necessarily manufacturing arbitrary numeric parameter values.

### 5.2 Parameter conditions and coverage

This is a particularly consequential feature.

For the parameterized equation

\\[ ax-1=0, \\]

the solution \\(x=1/a\\) is valid only when \\(a\ne0\\).

Symbolica exposes coverage metadata and conditions for such cases:

- `coverage`: Whether the result establishes complete coverage or only generic coverage.
- `coverage_guard`: Conditions under which generic coverage holds.
- Branch conditions: Additional restrictions associated with individual solutions.
- Free variables and dimension: Information about the structure of solution families.

A result with generic coverage is not an unconditional classification of every parameter assignment.

Exceptional parameter values may produce additional, different, or no solutions. They must be analyzed under the corresponding specialized problem.

Similarly, a reported branch count is not necessarily a count of individual solution points.

The API deliberately distinguishes unsupported problems from proven absence of solutions. Operations such as emptiness testing may raise `IncompleteCoverage` if the available information is insufficient to establish the answer.

This is much stronger semantic modeling than a solver interface that always returns either a list of values or an empty list.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+1



### 5.3 Exact algebraic roots

Symbolica uses exact root objects for algebraic values that cannot conveniently be represented using radicals.

A symbolic root such as

\\[ \operatorname{root}(p,k) \\]

identifies a selected root of a defining polynomial using Symbolica's canonical root ordering.

This preserves exact information while allowing high-precision numerical approximation when needed.

For some problems, roots simplify automatically into rationals, integers, or radicals; for others, they remain symbolic algebraic roots.

These objects can then participate in subsequent computations, including defining coefficient fields for polynomial factorization.

Important caveat: the root index is tied to the canonical representation of the defining polynomial, rather than being a universal identifier independent of all transformations. When Symbolica changes a polynomial representation, it can change the corresponding index while preserving the selected algebraic value.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+2



[https://symbolica.io/docs/solving.html](https://symbolica.io/docs/solving.html?utm_source=chatgpt.com)

[https://symbolica.io/posts/symbolica_3_0_release/index.html](https://symbolica.io/posts/symbolica_3_0_release/index.html?utm_source=chatgpt.com)

## 6. Calculus, integration, and series

### 6.1 Symbolic differentiation

Symbolica supports differentiation of general symbolic expressions, including the product and chain rules and derivatives of supported mathematical functions.

The combination of differentiation and user-defined symbol behavior is important: custom function symbols can register derivative rules rather than requiring the caller to manually expand every occurrence.

Differentiation also combines with normalization, term streaming, and evaluator construction. For example, the symbolic derivatives of a residual vector can be generated and then optimized together into a numerical Jacobian evaluator.

### 6.2 Symbolic integration: `symbolica-integrate`

Symbolic integration belongs to a separate Rust crate, `symbolica-integrate`, rather than being a standalone feature implemented entirely within the main crate.

The current verified companion release is 2.0.1, compatible with Symbolica 3.x.

It implements a native Rust version of the Rubi rule-based integration system, with more than 7,000 integration rules.

Its primary Rust integration point is the `Integrate` trait.

```
use symbolica::prelude::*;
use symbolica_integrate::Integrate;

fn main() {
    let x = symbol!("x");
    let integrand = parse!("1/(x^2+1)");

    let primitive = integrand.integrate(x).unwrap();
    println!("{primitive}");
}
```

The companion crate also supports `integrate_with_steps`, with the `steps` Cargo feature, to retain a rule-by-rule explanation of how an antiderivative was derived.

The ecosystem additionally supports compressed integration-step metadata, reducing the amount of rule-description material that must be distributed in certain builds.

Upstream reports successful execution of its 72,944-case Rubi test corpus, and publishes comparative timings on a specified hardware and benchmark setup. These results establish the scope of upstream testing, not a general claim that every integral has an elementary closed form or that the integrator will always be faster than alternatives.

Rule-based integration can also produce expressions with implicit assumptions or branch-cut considerations. In domains where those distinctions matter, results and exceptional parameter cases require appropriate validation.

`symbolica-integrate` is MIT licensed, but using it does not eliminate the separate license conditions of the Symbolica dependency.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



[https://github.com/symbolica-dev/symbolica-integrate](https://github.com/symbolica-dev/symbolica-integrate?utm_source=chatgpt.com)

[https://docs.rs/crate/symbolica-integrate/2.0.1](https://docs.rs/crate/symbolica-integrate/2.0.1?utm_source=chatgpt.com)

[https://symbolica.io/posts/symbolic_integration/](https://symbolica.io/posts/symbolic_integration/?utm_source=chatgpt.com)

### 6.3 Series: Taylor, Laurent, and fractional-power expansions

Symbolica supports formal power-series expansions, including negative and fractional powers.

This covers Taylor expansions as well as Laurent- and Puiseux-like representations.

A significant semantic detail is that the series representation tracks what coefficients are known, including the order of the omitted remainder.

A missing term below the truncation boundary may represent a known zero coefficient. A term beyond that boundary is unknown.

These are not equivalent states.

Converting a series to a general expression through `to_expression()` discards the remainder information. As a result, an algorithm concerned with truncation accuracy should retain the series object for as long as it needs to distinguish known zeros from unknown higher-order coefficients.

Series can be combined with derivatives, special functions, and symbolic parameter calculations. Custom function hooks can also provide special expansion behavior.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+1



[https://symbolica.io/docs/series.html](https://symbolica.io/docs/series.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/automatic_differentiation.html](https://symbolica.io/docs/automatic_differentiation.html?utm_source=chatgpt.com)

## 7. Optimized numerical evaluation, compilation, and code generation

Symbolica's numerical execution capabilities deserve particular attention because they extend the library beyond a conventional symbolic algebra engine.

The central idea is to use symbolic mathematics to construct and optimize a numerical program, rather than repeatedly interpreting the original expression.

### 7.1 Three levels of numerical execution

Symbolica distinguishes several operations that are superficially similar but serve different purposes.

| Mechanism                | Purpose                                                 | Main distinction                                             |
| ------------------------ | ------------------------------------------------------- | ------------------------------------------------------------ |
| `to_float`               | Convert known exact constants to approximate values     | Returns another symbolic expression                          |
| `evaluate`               | Perform a numerical substitution or one-off calculation | Convenient, but not a reusable optimized program             |
| `evaluator(...).build()` | Construct a reusable numerical evaluator                | Optimizes repeated evaluation                                |
| `.jit_compile(...)`      | Generate native machine code at runtime                 | Removes much of the interpreter overhead                     |
| `export_cpp`             | Generate and compile a separately loadable library      | Supports native deployment and specialized numerical targets |

An evaluator can be parameterized over a particular numerical representation rather than being permanently tied to ordinary `f64` arithmetic.

The Rust API uses `map_coeff` to convert its coefficients to the target numerical domain.

A representative 3.0.1 example is:

```
use symbolica::prelude::*;

fn main() {
    let params = [parse!("x"), parse!("y")];

    let mut evaluator = parse!("x*y + sin(x)^2")
        .evaluator(&params)
        .build()
        .unwrap()
        .map_coeff(&|c| c.re.to_f64());

    let mut output = [0.0];
    evaluator.evaluate(&[1.0, 2.0], &mut output);

    println!("{}", output[0]);
}
```

This separates a potentially expensive preparation phase from repeated low-overhead numerical execution.

### 7.2 Algebraic optimization and common-subexpression elimination

Evaluator construction includes algebraic and computational optimizations, including Horner-style polynomial restructuring, common-subexpression elimination (CSE), and temporary-value reuse.

For example,

\\[ f(x,y)=x^4+2x^3y+3x^2y^2+4xy^3 \\]

may be restructured to reduce multiplication or other operation counts rather than directly evaluating the original expanded form.

CSE detects repeated intermediate results that can be evaluated once and reused.

The benefit depends on the workload. An expression with extensive reusable structure may improve substantially, while an expression consisting mostly of unrelated operations offers fewer opportunities.

Optimization also has a preparation cost, so the economically relevant quantity is total compilation plus execution work, rather than a single evaluation benchmark.

### 7.3 Multi-output evaluators

`Atom::evaluator_multiple` is particularly important for vector-valued mathematics.

Instead of separately compiling

\\[ f_1=x^2+y,\qquad f_2=x^2+z,\qquad f_3=x^2+w, \\]

one evaluator can recognize that \\(x^2\\) is shared by all three outputs.

This applies to much larger structures: residual vectors, Jacobians, gradients, matrix entries, and other collections of related symbolic expressions.

The optimizer sees all outputs together, allowing common work to be eliminated across expression boundaries.

That is a meaningful advantage over generating one independently optimized function per output.

### 7.4 Nested functions and selective inlining

User-defined symbolic functions can be supplied to the evaluator builder using `add_function` or `add_tagged_function`.

The optimizer supports function inlining decisions, conceptually corresponding to:

- Always inline.
- Never inline.
- Automatically choose.

Inlining can expose additional opportunities for cross-function optimization, but it can also duplicate large function bodies and increase evaluator or generated-code size.

Keeping a reused function as a separate sub-evaluator can reduce generated program size at the expense of additional invocation overhead and possibly fewer optimization opportunities.

This makes function boundaries an explicit optimization parameter rather than an unavoidable consequence of the symbolic expression's original structure.

### 7.5 Portable evaluator instructions: an important extension point

Symbolica exposes an evaluator's compiled instruction representation through `get_instructions()`.

`EvaluatorInstructions` includes information such as:

- Input and output counts.
- Temporary storage requirements.
- Exact numerical constants.
- Instruction sequences.
- Non-inlined nested function bodies.

This is significant because it allows an optimized symbolic expression to be consumed by another execution system without requiring that system to reproduce Symbolica's algebraic optimizer.

For example, a specialized numerical runtime could interpret or translate these instructions into its own execution representation.

This is an architectural extension point, not a guarantee that every possible target backend is already supported. A custom emitter must preserve the instruction semantics, numeric-domain behavior, and nested-function structure.

### 7.6 SymJIT runtime compilation

Symbolica integrates SymJIT, an independent Rust JIT compiler specialized for mathematical expressions.

For Rust, JIT compilation is explicit:

```
let compiled = evaluator
    .jit_compile(
        JITCompilationSettings::new()
            .direct_translation(true)
    )
    .unwrap();
```

`direct_translation(true)` lowers the already optimized evaluator instructions directly to SymJIT's intermediate representation, avoiding an additional tree-based analysis.

Python's evaluator interface enables JIT compilation lazily by default; this is a behavioral distinction from the Rust builder.

SymJIT itself supports multiple CPU architectures and SIMD facilities, but the supported platforms and numerical representations of a particular Symbolica integration should be verified independently of the broader SymJIT project.

The performance tradeoff includes JIT startup, compilation costs, architecture availability, and the characteristics of the numerical workload. A JIT backend is not always preferable to a prepared interpreter for short-lived computations.

SymJIT is separately MIT licensed. Symbolica 3.0's manifest declares it as an optional dependency in the `2.25` compatibility series.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+2



### 7.7 C++, assembly, SIMD, and CUDA generation

The compiled evaluator backend supports generating C++ code, building shared libraries, and reloading compiled numerical functions.

Available representations include:

- Scalar real arithmetic.
- Scalar complex arithmetic.
- SIMD real and complex arithmetic.
- CUDA real and complex evaluators.

Rust exposes `export_cpp`, `compile`, and `load`. Python provides an analogous compilation interface.

The documented default code-generation path can emit optimized inline assembly to avoid excessive C++ compiler work for very large expressions.

The SIMD and CUDA paths are especially relevant when the same mathematical function must be evaluated at many different inputs. Their usefulness depends on available hardware, batching opportunities, memory-transfer overhead, supported mathematical operations, and the target numeric types.

GPU code generation does not make arbitrary symbolic operations execute on a GPU: the symbolic transformation and evaluator construction are separate from numerical execution.

A notable 3.0 change is that compiled libraries carry their own input and output dimensions. Compiled artifacts generated by earlier releases must be regenerated for the current loader format.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+2



### 7.8 Branches, functions, and numerical semantics

Numerical evaluators support conditional expressions, including the built-in `if(condition, true_branch, false_branch)`.

Its documented truth semantics are specific: the condition is false when its numerical value is zero and true otherwise. This is not a general symbolic three-valued logical expression evaluator.

Custom function callbacks, inlined symbolic definitions, and numerical mappings also have distinct performance properties. Unresolved or externally evaluated functions may limit optimization opportunities compared with functions whose expressions are known at build time.

Sources for the full evaluation pipeline:

[https://symbolica.io/docs/numerical_evaluation.html](https://symbolica.io/docs/numerical_evaluation.html?utm_source=chatgpt.com)

[https://docs.rs/symjit/latest/symjit/](https://docs.rs/symjit/latest/symjit/?utm_source=chatgpt.com)

[https://github.com/siravan/symjit](https://github.com/siravan/symjit?utm_source=chatgpt.com)

## 8. Automatic differentiation, numerical precision, and integration

### 8.1 Three differentiation mechanisms

Symbolica's mathematical stack permits three distinct approaches to derivatives.

| Approach                               | Mechanism                                   | Principal characteristic                                                         |
| -------------------------------------- | ------------------------------------------- | -------------------------------------------------------------------------------- |
| Symbolic differentiation               | Transform an `Atom` using `derivative`      | Produces an explicit derivative expression                                       |
| Compile-time dual/hyperdual arithmetic | Numerica-generated Rust number types        | Computes designated derivative components while executing arithmetic             |
| Evaluator dualization                  | `dualize` on a prepared numerical evaluator | Evaluates values and specified derivatives through a jointly optimized evaluator |

These are complementary rather than interchangeable.

Symbolic differentiation permits subsequent algebraic simplification, factorization, and reuse of derivative expressions.

Dual arithmetic is valuable when the desired derivatives can be carried through numerical operations, including mathematical functions whose complete symbolic expression may not be available in advance.

Evaluator dualization allows Symbolica to optimize requested derivative components jointly with the original numerical calculation.

### 8.2 Custom higher-order hyperduals

The Rust numerical stack provides macros including:

- `create_hyperdual_single_derivative!`
- `create_hyperdual_from_components!`
- `create_hyperdual_from_depths!`

These generate number types with a specified derivative structure.

For example, a type may retain only a function's value, three first derivatives, and particular mixed second derivatives rather than allocating a dense collection of every possible partial derivative.

This is a potentially significant optimization because the derivative structure is known at compile time. Loops can be unrolled and components not requested by the user need not be carried through each operation.

The tradeoff is compile-time specialization and the growth of the required coefficient structure as derivative depth and variable count increase.

The compile-time hyperdual type-generation machinery belongs to Numerica, although it is exposed conveniently through Symbolica's Rust prelude. Separately, prepared evaluator dualization is available through both the Rust and Python interfaces.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+2



### 8.3 Arbitrary precision and error-aware arithmetic

Symbolica's numerical domains extend beyond `f64`.

Through Numerica, relevant types and mechanisms include exact rationals, arbitrary-precision real and complex values, estimated numerical-error propagation, and interval arithmetic.

It is important to distinguish estimated significant-digit tracking from mathematically certified interval enclosures.

An error-propagating float may estimate how many digits are reliable after a nearly cancelling subtraction. That does not automatically constitute a proof that the exact result lies within a given interval.

Interval arithmetic instead maintains enclosures with appropriate outward-rounding semantics, making it a different tool for rigorous numerical work.

Symbolica 3.0 also introduced richer precision-aware Python scalar types and improved interactions between exact algebraic values and numerical approximations.

Precision is not purely a formatting option: coefficient types, algorithms, and evaluation backends may differ materially between machine precision and arbitrary precision.

### 8.4 Numerical integration

For integrals that cannot conveniently be solved symbolically, Symbolica exposes adaptive numerical integration functionality based on the VEGAS family of Monte Carlo algorithms.

The underlying numerical infrastructure comes from Numerica.

Capabilities include multidimensional continuous integration, discrete integration dimensions, adaptive importance-sampling grids, iterative refinement, multiple outputs, and user-controlled integration loops.

The principal mechanism is to adapt the sampling distribution toward regions contributing greater variance, potentially reducing sampling work relative to uniform Monte Carlo sampling.

This approach has important limitations:

- It produces statistical estimates, not generally certified exact answers.
- Adaptation quality depends on integrand geometry.
- Sharp correlations between dimensions can defeat simple separable grids.
- Rare events, narrow peaks, and cancellation can complicate convergence.
- The uncertainty estimate should be interpreted in the context of sampling assumptions and convergence diagnostics.

The Rust and Python interfaces allow customization, including batch evaluation to avoid repeated language-boundary overhead.

One useful combination is to compile a vector-valued symbolic expression into a numerical evaluator and use that evaluator inside an adaptive integration loop. The symbolic optimizer then reduces the per-sample cost, while the integrator controls where samples are spent.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+2



Sources:

[https://symbolica.io/docs/automatic_differentiation.html](https://symbolica.io/docs/automatic_differentiation.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/numerical_integration.html](https://symbolica.io/docs/numerical_integration.html?utm_source=chatgpt.com)

[https://docs.rs/numerica/3.0.1/numerica/](https://docs.rs/numerica/3.0.1/numerica/?utm_source=chatgpt.com)

## 9. Large expressions, memory management, and term streaming

### 9.1 Compact expression representation

Symbolica uses a compact expression representation rather than depending entirely on independently allocated, pointer-heavy tree nodes.

Together with normalized multi-argument sums and products, this can reduce per-node overhead and improve locality for operations that process large algebraic expressions.

`AtomView` further permits operations over borrowed expression data, reducing unnecessary ownership changes and allocations.

For repeated expressions, `AliasedAtom` offers a mechanism to refer to shared subexpressions through aliases.

These are distinct optimizations: compact storage reduces representation overhead, borrowing limits copies, and aliasing reduces repeated storage or processing of common structures.

None prevents algebraic expansion from producing intrinsically large results.

### 9.2 `TermStreamer`: bounded-memory algebra

Symbolica's `TermStreamer` addresses workloads where a mathematical expression contains an enormous number of relatively small terms.

Instead of retaining the entire expression, it can process individual terms and write intermediate results to disk.

`TermStreamerConfig` exposes operational controls including:

- `n_cores`: Number of worker cores for eligible operations.
- `path`: Working location for disk-backed data.
- `max_mem_bytes`: Memory threshold.

The streaming interface can map transformations over terms in parallel, with a compression-backed writer used in the documented Rust example.

A key distinction is between term-local and global operations.

Term-local operations can often include individual-term substitutions, differentiation, and expansion.

Global operations such as factorization, polynomial GCD, Horner-form construction, or matching a pattern across a sum require relationships between terms and cannot simply be performed independently.

Also, mapping a streamer does not automatically merge like terms. Calling `normalize` triggers the required global sorting and merging, potentially using an external merge sort.

That means a streaming pipeline must carefully choose where it incurs global normalization and where it can preserve independent term processing.

The benefit is potentially much smaller peak memory usage; the cost is additional disk traffic, serialization, sorting, and coordination. Term streaming is not inherently a faster alternative to in-memory calculation when the expression already fits comfortably in memory.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+1



### 9.3 Parallel execution boundaries

Symbolica supports multicore execution for several transformation and streaming workloads.

However, it does not imply that all algebraic operations automatically parallelize optimally.

Parallel scaling depends on term granularity, work distribution, output size, temporary allocations, merging requirements, and the applicable license.

In particular, the free execution tier may impose restrictions on usable cores or instances independently of the host machine's hardware capabilities.

### 9.4 Serialization and persistent mathematical state

Symbolica exposes serialization-related support through features and state-management functionality, including optional Serde and binary-serialization infrastructure.

This makes it possible to preserve mathematical representations across execution phases instead of always regenerating expressions by reparsing text.

There are important limits to portability and completeness. Mathematical data may depend on symbol identities, namespace registrations, attributes, and user-defined callbacks. A serialized expression should not be assumed to reproduce arbitrary executable closures or every global symbol definition on a different process without explicit state handling.

Consequently, textual formulas, serialized expression objects, and compiled evaluators are three separate persistence formats with different fidelity and deployment implications.

Sources:

[https://symbolica.io/docs/expressions.html](https://symbolica.io/docs/expressions.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/streaming.html](https://symbolica.io/docs/streaming.html?utm_source=chatgpt.com)

[https://docs.rs/symbolica/3.0.1/symbolica/state/index.html](https://docs.rs/symbolica/3.0.1/symbolica/state/index.html?utm_source=chatgpt.com)

## 10. Associated Rust libraries and mathematical infrastructure

Symbolica has a growing ecosystem of independent libraries and specialized extensions. Some functionality is native to `symbolica`; other functionality is re-exported from or implemented on top of separate crates.

Understanding these boundaries is important for both licensing and architecture.

### 10.1 Ecosystem map

| Package                  | Verified version or status                 | Principal responsibility                                                            | Relationship to Symbolica                                |
| ------------------------ | ------------------------------------------ | ----------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `symbolica`              | 3.0.1                                      | General symbolic algebra, exact computation, evaluators, rewriting                  | Core engine; source-available                            |
| `numerica`               | 3.0.1                                      | Numeric domains, exact arithmetic, differentiation, numerical integration, matrices | Independent MIT-licensed mathematical foundation         |
| `graphica`               | 3.0.1                                      | Graph generation, isomorphism, canonicalization                                     | Independent MIT-licensed graph engine used by Symbolica  |
| `symbolica-integrate`    | 2.0.1                                      | Rule-based symbolic integration                                                     | Independent MIT-licensed integration extension           |
| `symjit`                 | 2.25 compatibility series in Symbolica 3.0 | Native machine-code JIT compilation                                                 | Independent MIT-licensed compiler backend                |
| `symbolica-community`    | 3.0-series source distribution             | Combined Python bindings and specialized extensions                                 | Companion distribution, not equivalent to the core crate |
| `spenso`                 | 0.6.0                                      | Symbolic/numeric tensors and tensor networks                                        | Specialized independent Rust crate                       |
| `symbolica-typst-plugin` | Published 0.1.0                            | Symbolic mathematics inside Typst documents                                         | Official application-specific integration                |
| `symbolica-ode`          | Early-stage upstream repository            | Symbolically optimized ODE systems using SUNDIALS                                   | Experimental companion integration                       |

Versions refer to the verified release or documentation state, not a claim that all packages are simultaneously pinned to those exact versions in every downstream distribution.

### 10.2 Numerica: exact and numerical mathematics without the symbolic layer

Numerica is arguably the most important companion crate.

It provides a substantial numerical and algebraic toolkit without requiring the general Symbolica expression engine.

Its capabilities include:

Exact arithmetic

- Integers with automatic transitions between small and arbitrary-precision representations.
- Exact rational numbers.
- Prime-field and extension-field arithmetic.
- Rational reconstruction.
- Ring and field abstractions supporting generic algebraic algorithms.

Floating-point and numerical analysis

- Arbitrary-precision real and complex arithmetic.
- Error-propagating numeric representations.
- Interval arithmetic.
- SIMD-oriented numerical types.
- Higher-order automatic differentiation.

Linear algebra and mathematical algorithms

- Dense and sparse matrices.
- Exact and approximate linear-system solving.
- Selected lattice and integer-relation algorithms.
- Numerical integration.
- Combinatorial utilities.
- Specialized coefficient-domain kernels.

Numerica's integer representation is particularly useful for exact computation: operations can remain in efficient smaller representations when possible and transition to arbitrary precision when necessary.

Its `ErrorPropagatingFloat` is also distinctive because it tracks estimated numerical significance, helping identify loss of precision from operations such as subtracting nearly equal values.

#### Architectural significance

There are several reasons to consider Numerica independently of Symbolica.

First, it supplies much of the mathematical-domain machinery behind Symbolica. The numerical stack therefore has reusable abstractions beyond the symbolic engine.

Second, Numerica is an appropriate standalone foundation when a computation operates entirely on known algebraic or numerical data and does not require symbolic expressions or pattern rewriting.

Third, combining Numerica's coefficient types with Symbolica's expression and polynomial representations allows arithmetic semantics to remain consistent across the symbolic/numerical boundary.

The separation also matters for licensing: Numerica is MIT licensed, whereas the Symbolica core has more restrictive terms.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust

+1



Primary sources:

[https://github.com/symbolica-dev/numerica](https://github.com/symbolica-dev/numerica?utm_source=chatgpt.com)

[https://docs.rs/numerica/3.0.1/numerica/](https://docs.rs/numerica/3.0.1/numerica/?utm_source=chatgpt.com)

### 10.3 Graphica: graph generation, canonicalization, and isomorphism

Graphica is another substantial standalone Rust library.

It supports graphs containing:

- Directed and undirected edges in the same structure.
- Multiple edges between vertices.
- Self-loops.
- Arbitrary node and edge data.
- Half-edge representations.
- Graph canonicalization and isomorphism testing.
- Constraint-based graph generation.

A particularly important capability is producing canonical graph representations, allowing structurally equivalent graphs to be recognized despite differences in node numbering or traversal order.

Graphica includes algorithms related to automorphisms, symmetry, and generation of constrained graph families.

Its graph-generation facilities can restrict the number of loops or bridges and specify permissible vertex structures. This is particularly useful in diagrammatic mathematics, but the graph representation and algorithms are not exclusive to physics applications.

Graphica can export diagrams to representations such as Mermaid, and Symbolica's graph-facing interfaces integrate these capabilities.

The important boundary is that Graphica owns the generic graph algorithms, while Symbolica supplies algebraic expressions that may be attached to or derived from graph structures.

Examples of capabilities enabled by this distinction include canonical graph identifiers, deduplication of generated topologies, and applying symbolic transformations to mathematical data associated with graph nodes or edges.

Canonicalization can itself be computationally expensive for highly symmetric or difficult graph families, so the cost of obtaining a canonical representation must be included when evaluating its usefulness.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+2



Primary sources:

[https://github.com/symbolica-dev/graphica](https://github.com/symbolica-dev/graphica?utm_source=chatgpt.com)

[https://docs.rs/graphica/latest/graphica/](https://docs.rs/graphica/latest/graphica/?utm_source=chatgpt.com)

[https://symbolica.io/docs/graphs.html](https://symbolica.io/docs/graphs.html?utm_source=chatgpt.com)

### 10.4 Matrices and tensors

Symbolica's matrix capabilities sit at the intersection of several layers.

Numerica provides much of the underlying numerical and algebraic matrix infrastructure, including matrix operations over different coefficient domains.

Symbolica can represent and manipulate matrices containing symbolic expressions, enabling operations such as exact linear algebra and symbolic matrix calculations.

The core also exposes indexed tensor-related functionality, including tensor canonicalization.

A mathematical matrix is not necessarily the same as a symbolic tensor network. Symbolica's native matrix abstractions address structured arrays and algebraic linear operations, while separate libraries such as Spenso provide richer indexed tensor structures, heterogeneous tensor data, and contraction networks.

This distinction is important when choosing between a conventional linear algebra representation and a network of tensors connected by symbolic indices.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://symbolica.io\&sz=32)

Modern Computer Algebra

+1



[https://symbolica.io/docs/matrices.html](https://symbolica.io/docs/matrices.html?utm_source=chatgpt.com)

[https://docs.rs/symbolica/3.0.1/symbolica/tensors/index.html](https://docs.rs/symbolica/3.0.1/symbolica/tensors/index.html?utm_source=chatgpt.com)

## 11. Specialized extensions and integrations

### 11.1 Symbolica Community

`symbolica-community` is an upstream-maintained, community-enhanced distribution that bundles Symbolica with additional functionality.

It should not be confused with the ordinary Rust `symbolica` crate.

The 3.0-series community repository includes integrations for symbolic integration and domain-specific computational physics libraries.

Important families include:

| Library or extension          | Primary role                                                                                 |
| ----------------------------- | -------------------------------------------------------------------------------------------- |
| Spenso                        | Indexed tensors, dense/sparse tensor data, tensor networks                                   |
| Idenso                        | Specialized symbolic tensor and indexed algebra, including Dirac/color algebra               |
| FeynKit / HEPKit              | Particle-physics models, Feynman diagrams, graph generation, amplitudes and kinematics       |
| Vakint                        | Symbolic and numerical processing of vacuum integrals                                        |
| `symbolica-amflow`            | Numerical transport of integral systems, with arbitrary precision and reusable boundary data |
| Other pinned HEP dependencies | Specialized reduction, integration, and related computational physics functionality          |

These are not ordinary features that become available simply by importing the `symbolica` crate in an unrelated Rust project. Availability depends on the particular companion package, its revision, enabled features, and corresponding language bindings.

The community repository pins specific revisions of several extensions. Functionality exposed by a development checkout should therefore not automatically be attributed to a released Rust or Python package.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



Source:

[https://github.com/symbolica-dev/symbolica-community](https://github.com/symbolica-dev/symbolica-community?utm_source=chatgpt.com)

### 11.2 Spenso: structured tensor computation

Spenso is especially notable for workloads involving indexed objects or higher-rank tensors.

It provides:

- Arbitrary-rank tensors.
- Tensor structures defined by slots and indices.
- Dense and sparse tensor storage.
- Symbolic, parametric, and numerical tensor representations.
- Tensor contraction.
- Tensor networks and contraction algorithms.
- Metric-related facilities, including Minkowski and configurable diagonal metrics.

A tensor network can represent a structured contraction of many tensors rather than forcing the entire operation into one expanded scalar expression.

This creates opportunities to choose contraction orders, preserve sparsity, and postpone expansion.

The distinction from Symbolica's scalar expression engine is important: a network of tensors may be far more efficient to manipulate in its native structural form than after expanding every indexed operation into scalar sums and products.

Spenso can use Symbolica expressions as components of its symbolic tensor calculations, connecting tensor-level structure to the general algebraic engine.

Source:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust



[https://docs.rs/spenso/0.6.0/spenso/](https://docs.rs/spenso/0.6.0/spenso/?utm_source=chatgpt.com)

### 11.3 FeynKit, Idenso, Vakint, and related scientific tools

The community distribution contains a more specialized ecosystem oriented toward high-energy physics.

FeynKit provides facilities for representing mathematical models, generating and manipulating Feynman diagrams, managing kinematics, and constructing amplitudes.

Idenso supports specialized indexed algebra, including structures important in quantum field theory.

Vakint addresses selected classes of vacuum integral calculations.

The community's numerical integration interfaces also expose differential-system transport, boundary caches, and operations involving arbitrary precision.

For a broader mathematical software architecture, these projects illustrate a useful pattern: Symbolica supplies reusable algebra, while specialized packages retain the structural semantics of their mathematical domain.

However, these extensions should be investigated individually for maturity, API stability, external dependencies, supported platforms, and license conditions. Their capabilities should not be generalized to arbitrary symbolic integration, tensor calculation, or nonlinear equation solving.

Sources:

[https://github.com/symbolica-dev/symbolica-community](https://github.com/symbolica-dev/symbolica-community?utm_source=chatgpt.com)

[https://hepkit.org/](https://hepkit.org/?utm_source=chatgpt.com)

[https://symbolica.io/docs/python_api/community/spenso/](https://symbolica.io/docs/python_api/community/spenso/?utm_source=chatgpt.com)

[https://symbolica.io/docs/python_api/community/hepkit/vakint/index.html](https://symbolica.io/docs/python_api/community/hepkit/vakint/index.html?utm_source=chatgpt.com)

### 11.4 Official Symbolica Typst plugin

The official Typst plugin enables symbolic mathematics directly inside technical documents.

Its capabilities include mathematical expression construction, simplification, differentiation, integration, evaluation, and equation solving.

The published Typst Universe release is 0.1.0.

There is a significant version distinction: the plugin's upstream repository contains an evolving API, including renamed functions that are not present in the published 0.1.0 interface.

The plugin uses a WebAssembly build of Symbolica.

It also has a specific licensing exception permitting free usage inside Typst, including commercial Typst usage. This should not be interpreted as a general exemption for arbitrary commercial software embedding Symbolica.

This integration is particularly useful when mathematical derivations must be computed as part of document generation rather than separately calculated and manually copied into a report.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source:

[https://github.com/symbolica-dev/symbolica-typst-plugin](https://github.com/symbolica-dev/symbolica-typst-plugin?utm_source=chatgpt.com)

### 11.5 Symbolica-ODE and SUNDIALS

`symbolica-ode` demonstrates a direct connection between Symbolica's optimized expression engine and established differential-equation solvers.

The repository combines symbolic ODE definitions with the SUNDIALS numerical integration system.

Its example constructs an `ODESystem` and uses an `ARKodeSolver` to evolve a Brusselator reaction system.

Conceptually, this separates two responsibilities:

- Symbolica constructs and optimizes the mathematical right-hand-side expressions.
- SUNDIALS performs numerical time integration.

This is a valuable integration model because a CAS is not automatically a replacement for specialized stiff and nonstiff ODE integrators.

At the research date, `symbolica-ode` is an early-stage repository with limited upstream development history. It should be treated as a demonstrated integration direction rather than as evidence of a mature, comprehensive native ODE-solving product.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub



Source:

[https://github.com/symbolica-dev/symbolica-ode](https://github.com/symbolica-dev/symbolica-ode?utm_source=chatgpt.com)

## 12. Language bindings, developer tooling, and build configuration

### 12.1 Rust API

Rust provides Symbolica's broadest low-level integration and extension surface.

The crate exposes internal mathematical representations, algebraic-domain traits, owned and borrowed expressions, pattern-matching machinery, evaluator builders, custom callbacks, and compile-time numerical specialization.

Installation:

```
cargo add symbolica
```

For the separately distributed integration crate:

```
cargo add symbolica-integrate
```

The Rust API is generally the most appropriate reference when investigating low-level extension mechanisms or ownership and performance semantics.

Important entry points include `symbolica::prelude`, `atom`, `domains`, `poly`, `evaluate`, `transformer`, `tensors`, and related modules.

### 12.2 Python API

The Python package wraps the same fundamental symbolic engine, with interfaces appropriate for interactive computation and scientific Python applications.

```
pip install symbolica
```

It supports Pythonic construction and manipulation of expressions, evaluation, visualization, and connections to array-oriented numerical workflows.

The Python ecosystem additionally exposes notebook formatting, LaTeX and Typst representations, compiled evaluators, and community-extension interfaces.

Python and Rust APIs do not necessarily have identical method names or execution behavior. For example, Rust evaluator construction uses the builder pattern, while Python offers a higher-level evaluator interface.

Python-level callback overhead can also matter for large numerical loops; batch evaluation and native compiled evaluators can avoid repeated crossings into Python.

A Python `symbolica` package release and a Rust `symbolica` crate release are distinct artifacts. Their version availability and bundled companion features should be verified independently.

### 12.3 WebAssembly

Symbolica can run symbolic computations in browser environments through WebAssembly.

The website itself provides runnable examples and browser-based mathematical execution.

WebAssembly generally requires a different dependency and compilation configuration from a native Linux build. In particular, native machine-code JIT and CUDA execution should not be assumed to be available in browser deployments.

Portable numerical backends allow meaningful symbolic and exact operations without depending on native GMP/MPFR configurations.

The community distribution also publishes browser-compatible capabilities, although some native-only extensions and compiled evaluators are unavailable in that environment.

### 12.4 C, C++, and Wolfram/Mathematica integration

Symbolica exposes additional language-facing integration facilities and relevant Cargo features, including Wolfram LibraryLink support.

An older upstream getting-started page describes the C/C++/Mathematica bindings as being narrower than the Rust and Python interfaces, focused on multivariate polynomial arithmetic.

Because that page predates 3.0, its exact coverage should not be treated as an exhaustive description of today's bindings. Nevertheless, there is insufficient evidence to assume that every current Rust API is also available through those language interfaces.

The important distinction is between:

- Generating C++ numerical code, which is an evaluator feature.
- Calling Symbolica's symbolic functions from C++, which requires a language binding and may expose a narrower surface.

These are separate interoperability mechanisms.

### 12.5 Cargo features and arithmetic backends

Symbolica 3.0 requires a recent Rust toolchain; its manifest specifies Rust 1.89 as the minimum supported version.

Consequential feature and dependency choices include:

| Option                   | Implication                                               |
| ------------------------ | --------------------------------------------------------- |
| `integer-gmp`            | GMP-backed integer arithmetic                             |
| `float-mpfr`             | MPFR-backed arbitrary-precision floating-point arithmetic |
| `integer-malachite`      | Alternative portable integer backend                      |
| `float-astro`            | Alternative floating-point backend                        |
| `native_code_generation` | Native numerical compilation support                      |
| `faster_alloc`           | Allocator-related performance configuration               |
| `python_api`             | Python binding support                                    |
| `mathematica_api`        | Wolfram integration support                               |
| `serde` / `bincode`      | Optional serialization-related functionality              |
| `wasm`                   | WebAssembly-oriented build configuration                  |

The GMP/MPFR configuration is not interchangeable with a pure-Rust arithmetic backend in every performance, compatibility, or deployment respect.

Similarly, native evaluator compilation may require a suitable compiler toolchain, and CUDA output needs the relevant NVIDIA compilation and runtime environment.

For reproducibility, the actual selected features, target architecture, numerical backend, and external compiler version can be as important as the top-level `symbolica` version.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Docs.rs

+2



[https://docs.rs/crate/symbolica/latest/source/Cargo.toml.orig](https://docs.rs/crate/symbolica/latest/source/Cargo.toml.orig?utm_source=chatgpt.com)

[https://symbolica.io/docs/get_started.html](https://symbolica.io/docs/get_started.html?utm_source=chatgpt.com)

[https://symbolica.io/docs/upgrade/migration_to_3_0.html](https://symbolica.io/docs/upgrade/migration_to_3_0.html?utm_source=chatgpt.com)

### 12.6 Formatting, parsing, and mathematical presentation

Symbolica supports expression parsing and multiple output representations.

These include conventional algebraic formatting, LaTeX, Typst, and Mathematica-oriented syntax.

This functionality can replace independently maintained mathematical pretty-printers in systems where Symbolica already owns the expression representation.

However, presentation formats should not be confused with lossless serialization protocols.

A formatted string may omit symbol namespaces, attributes, or other information needed to reconstruct the exact original mathematical state.

Likewise, successful parsing of a representation does not imply that every syntax extension from the source language is supported.

## 13. Licensing, redistribution, and restrictions affecting engineering use

The licensing model is unusually consequential and warrants a separate discussion.

### 13.1 Core license versus companion licenses

The main Symbolica crate is governed by Symbolica Source-Available License 1.0.

Numerica, Graphica, SymJIT, and Symbolica-integrate are independently MIT licensed.

Using an MIT-licensed companion directly does not impose Symbolica's restrictions unless the resulting product also includes or uses Symbolica itself.

Conversely, including an MIT-licensed companion in a product does not make the Symbolica core MIT licensed.

### 13.2 Distribution and runtime licensing are separate

Under Section 3 of the current source-available license, another product may link, embed, bundle, and distribute an unmodified copy of Symbolica as a dependency, subject to notice requirements.

However, this does not automatically grant runtime rights to every recipient.

Recipients must independently qualify under free-use provisions or be covered by an appropriate license or written agreement.

The official pricing page indicates that user licensing can be addressed differently when Symbolica is only a small part of a product's exposed API, but those arrangements are determined by applicable terms and, in some cases, separate agreements.

It would therefore be inaccurate to conclude either that redistribution is universally prohibited or that distribution automatically grants unrestricted execution rights.

### 13.3 Free, academic, and commercial execution

The official pricing page distinguishes several arrangements.

These include free restricted academic or noncommercial execution, qualified hobbyist licensing, negotiated startup arrangements, academic licenses, and standard commercial licenses.

The publicly described free academic/noncommercial tier allows one core and one instance per device.

Paid arrangements can change those limits, but interactive developer-machine authorization should not automatically be interpreted as authorization for servers, CI systems, or distributed services.

The website specifically identifies server and CI usage as requiring separate arrangements under its standard plan.

License keys can be provided through the `SYMBOLICA_LICENSE` environment variable or the relevant API.

### 13.4 Source inspection, AI tools, and competing products

The current license permits specified source inspection, internal changes, bug investigation, and contributions, but imposes material restrictions.

In particular, it restricts distribution of modified Symbolica versions, use of implementation-specific information to develop competing symbolic-mathematics functionality, certain reverse-engineering activities, and use of source or derived implementation information for AI training or retrieval systems.

It contains a narrowly defined allowance for AI coding assistants performing otherwise permitted changes, subject to restrictions on provider retention, training, and competing products.

This matters for programming-agent workflows. Ordinary use of documented Symbolica APIs should not be conflated with unrestricted automated extraction, indexing, profiling, or reconstruction of its implementation.

The exact terms of the version being used, rather than an assumption about conventional open-source practice, should govern such activities.

Sources:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://github.com\&sz=32)

GitHub

+1



[https://github.com/symbolica-dev/symbolica/blob/main/License.md](https://github.com/symbolica-dev/symbolica/blob/main/License.md?utm_source=chatgpt.com)

[https://symbolica.io/license/](https://symbolica.io/license/?utm_source=chatgpt.com)

## 14. Capability interactions and architectural opportunities

Symbolica is most distinctive when several of its native capabilities are composed.

The following combinations illustrate possibilities supported by the architecture, rather than recommendations for a particular codebase.

### 14.1 Symbolic models to optimized numerical kernels

Symbolic expressions

Algebraic optimization

Multi-output evaluator

JIT / C++ / SIMD / CUDA

This is the central symbolic-to-numerical workflow.

It can avoid building separate systems for symbolic parsing, algebraic simplification, CSE, numerical code generation, and compiled function loading.

It is especially attractive when an expression is constructed once but evaluated many times.

The limiting factors are compilation overhead, generated-program complexity, hardware suitability, and numeric-domain requirements.

### 14.2 Exact algebra combined with certified numerical results

Polynomial and algebraic-number representations can preserve exactness through transformations that would otherwise introduce rounding error.

Numerica then supplies high-precision and interval arithmetic for operations where numerical approximations are required.

This separation can support workflows where algebraic structure remains exact until a controlled numerical evaluation boundary.

However, a floating-point approximation to an exact algebraic value is not itself an exact value. Precision, rounding, root selection, and enclosure guarantees must remain explicit.

### 14.3 Pattern rewriting combined with specialized coefficient domains

Custom symbolic rules provide expressiveness, while polynomial domains and coefficient rings provide algorithmic efficiency.

A mathematically sophisticated system can therefore perform only those rewrites that require general expression structure, then use specialized polynomial or rational arithmetic for large algebraic operations.

This is often more scalable than expressing every mathematical transformation as a general tree rewrite.

The key decision is how much mathematical structure is preserved or hidden at each conversion boundary.

### 14.4 Tensor networks combined with scalar code generation

Spenso provides higher-level tensor structure and contraction algorithms.

Symbolica can simplify or optimize the scalar expressions associated with components of those contractions.

Combined with numerical compilation, this creates a progression from structured mathematical operations to optimized scalar numerical kernels.

The potential benefit comes from preserving tensor topology and sparsity before scalar expansion.

The associated risk is prematurely expanding a tensor network into expressions too large for efficient downstream algebraic manipulation.

### 14.5 Large-expression transformations with controlled global work

Term streaming and transformers can be composed to process algebraic expressions without retaining the entire intermediate result in memory.

Global normalization, collection, or factorization can then occur at selected boundaries.

This allows the system to distinguish operations that can scale by independent term processing from those requiring a whole-expression view.

The corresponding tradeoff is additional disk and sorting work.

## 15. Important limitations and distinctions

The principal constraints are best understood as boundaries between otherwise powerful mechanisms.

| Topic                 | Important distinction                                                                                    |
| --------------------- | -------------------------------------------------------------------------------------------------------- |
| Symbolic equality     | Canonical structural representation is not complete mathematical equivalence testing                     |
| Simplification        | Not every mathematically valid transformation is valid without assumptions or domain restrictions        |
| Exact solving         | Generic coverage is not unconditional coverage of every parameter specialization                         |
| Algebraic roots       | Root identity depends on defining polynomial, embedding, and canonical root representation               |
| Numerical precision   | Estimated significant digits and certified interval enclosures are different guarantees                  |
| Series                | Converting to an expression may discard remainder-order information                                      |
| Polynomial conversion | Treating a subexpression as an indeterminate can hide mathematical relationships                         |
| Pattern matching      | Algebra-aware matching is not an unrestricted theorem prover or global equality-saturation engine        |
| Symbolic integration  | Rule-based antiderivatives have algorithmic coverage and branch-condition limitations                    |
| Code generation       | Optimization and JIT compilation introduce preparation cost and platform constraints                     |
| CUDA/SIMD             | Acceleration depends on numeric representation, data parallelism, batching, and hardware                 |
| Term streaming        | Most term-local operations stream well; global polynomial and algebraic operations remain more expensive |
| Language bindings     | Python, Rust, C/C++, Wolfram, and Typst interfaces do not expose identical capabilities                  |
| Licensing             | Source availability and distributability do not imply unrestricted runtime or derivative-work rights     |

These boundaries do not diminish Symbolica's usefulness. They identify the places where a highly capable engine still requires explicit mathematical or engineering decisions.

## 16. Primary-source resource map

The following is a consolidated, copy-friendly index of the most useful documentation and source locations. All URLs are given in plain text.

| Area                                 | Primary source                                                    |
| ------------------------------------ | ----------------------------------------------------------------- |
| Rust 3.0.1 API reference             | `https://docs.rs/symbolica/3.0.1/symbolica/`                      |
| Official documentation index         | `https://symbolica.io/docs/`                                      |
| Upstream code                        | `https://github.com/symbolica-dev/symbolica`                      |
| Expression engine                    | `https://symbolica.io/docs/expressions.html`                      |
| Symbols and attributes               | `https://symbolica.io/docs/symbols.html`                          |
| Pattern matching                     | `https://symbolica.io/docs/pattern_matching.html`                 |
| Transformers                         | `https://symbolica.io/docs/transformers.html`                     |
| Polynomial arithmetic                | `https://symbolica.io/docs/polynomials.html`                      |
| Coefficient domains                  | `https://symbolica.io/docs/coefficients.html`                     |
| Exact solving                        | `https://symbolica.io/docs/solving.html`                          |
| Series expansions                    | `https://symbolica.io/docs/series.html`                           |
| Numerical evaluation and compilation | `https://symbolica.io/docs/numerical_evaluation.html`             |
| Automatic differentiation            | `https://symbolica.io/docs/automatic_differentiation.html`        |
| Numerical integration                | `https://symbolica.io/docs/numerical_integration.html`            |
| Matrices                             | `https://symbolica.io/docs/matrices.html`                         |
| Graphs                               | `https://symbolica.io/docs/graphs.html`                           |
| Streaming                            | `https://symbolica.io/docs/streaming.html`                        |
| Symbolica 3.0 release                | `https://symbolica.io/posts/symbolica_3_0_release/index.html`     |
| 2.2-to-3.0 migration guide           | `https://symbolica.io/docs/upgrade/migration_to_3_0.html`         |
| Numerica                             | `https://github.com/symbolica-dev/numerica`                       |
| Numerica Rust API                    | `https://docs.rs/numerica/3.0.1/numerica/`                        |
| Graphica                             | `https://github.com/symbolica-dev/graphica`                       |
| Graphica Rust API                    | `https://docs.rs/graphica/latest/graphica/`                       |
| Rule-based integration               | `https://github.com/symbolica-dev/symbolica-integrate`            |
| SymJIT                               | `https://github.com/siravan/symjit`                               |
| Spenso                               | `https://docs.rs/spenso/0.6.0/spenso/`                            |
| Community extensions                 | `https://github.com/symbolica-dev/symbolica-community`            |
| Typst plugin                         | `https://github.com/symbolica-dev/symbolica-typst-plugin`         |
| ODE integration                      | `https://github.com/symbolica-dev/symbolica-ode`                  |
| License text                         | `https://github.com/symbolica-dev/symbolica/blob/main/License.md` |
| Licensing plans                      | `https://symbolica.io/license/`                                   |
| Upstream community discussions       | `https://zulip.symbolica.io/`                                     |

## Conclusion

Symbolica is best understood as a collection of interconnected mathematical execution capabilities, not merely a symbolic-expression manipulation library.

Its greatest distinguishing characteristics are the combination of a compact general expression representation, configurable symbolic rewriting, specialized exact polynomial algebra, explicit solution semantics, and the ability to transform symbolic mathematics into efficient numerical programs.

The surrounding ecosystem expands those capabilities considerably. Numerica provides the independently useful exact and numerical mathematical foundation. Graphica supplies graph structures and canonicalization. The integration crate adds a large Rubi-derived rule system. SymJIT supplies machine-code compilation. Spenso and the specialized scientific extensions retain mathematical structures that would be inconvenient or inefficient to represent solely as scalar expressions.

For advanced engineering use, the central insight is that the strongest capabilities arise from choosing the appropriate representation at each stage of computation: general expressions for structural reasoning, specialized algebraic domains for exact algorithms, tensor or graph representations for structured mathematical objects, and optimized evaluators for repeated numerical execution.

The most important qualifications are that these transformations have meaningful semantic and computational boundaries, companion packages vary in maturity and API coverage, and the core Symbolica license is materially more restrictive than the licenses of several associated libraries.

# Symbolica 3.0.1 — Rust API Capability Addendum

Research date: November 6, 2026 | Scope: Native Rust functionality and independently usable Rust companion crates

## Scope and relationship to the original reference

This addendum supplements Symbolica 3.0.1 — Comprehensive Rust Library and Ecosystem Reference. It documents additional native Rust functionality, particularly advanced algebraic algorithms, expression-analysis mechanisms, nonlinear numerical solving, evaluator composition, mathematical-domain operations, and extension interfaces.

The reference version is Symbolica 3.0.1, matching the original document. A newer Rust crate, 3.0.2, has been published, but the APIs discussed here are verified against the versioned Rust 3.0.1 documentation unless explicitly stated otherwise.

Every API identifier below refers to a native Rust type, trait, method, or function. Python-only methods and Python Community wrappers are excluded. Companion-crate functionality is identified separately from functionality owned by `symbolica` itself.

Several distinctions from the previous addendum are particularly important:

- The native Rust identity-testing interface is `AtomCore::zero_test()`, not Python's `Expression.check_zero()`. Their documented behavior must not be equated.
- Native Rust evaluator composition uses `ExpressionEvaluator::merge()`, and evaluator instruction export uses `export_instructions()`. The Python-specific reconstruction and serialization methods should not be presumed to exist as corresponding Rust methods.
- Rust's specialized polynomial types expose algorithms and constraints that differ from the simplified Python polynomial interface.
- High-energy-physics functionality belongs to separately maintained Rust components; its presence in the Python Community distribution does not establish that it is part of the `symbolica` Rust crate.

The discussion below is organized around those native capabilities and their mathematical and architectural implications.

## 1. Mathematical property reasoning, zero testing, and nonlinear solving

Supplements original §§2.2, 3.2, 5, and 8.

### 1.1 Native mathematical property queries

The `AtomCore` trait exposes mathematical property queries that return `ConditionResult`, rather than reducing every mathematical condition to a Boolean.

Relevant operations include:

| Rust method        | Mathematical property           |
| ------------------ | ------------------------------- |
| `is_real()`        | Expression is real-valued       |
| `is_integer()`     | Expression takes integer values |
| `is_scalar()`      | Expression is scalar            |
| `is_positive()`    | Expression is positive          |
| `is_nonnegative()` | Expression is nonnegative       |

`ConditionResult` distinguishes `True`, `False`, and `Inconclusive`.

This is an important distinction for algebraic transformations. A symbol whose positivity has not been established must not automatically be treated as nonpositive.

The API also distinguishes expression-level mathematical properties from symbol attributes. For instance, `Symbol::is_real()` tests whether the `Real` attribute is registered on the symbol. `AtomCore::is_real()` instead reasons about the mathematical property of an expression.

The property queries are deliberately inexpensive; they do not attempt exhaustive simplification or general theorem proving. Their guarantees concern the values of expressions where those expressions are defined, not the absence of singularities.

For conditional transformations, `.is_true()` can establish whether a required property has been proved. A result of `Inconclusive` must be handled separately from a proved negation.

### 1.2 Rust `AtomCore::zero_test()`

The relevant native Rust method is:

```
fn zero_test(
    &self,
    iterations: usize,
    tolerance: f64,
) -> ConditionResult
```

It performs a test of whether an expression could be zero, potentially using numerical sampling.

The upstream documentation demonstrates an important limitation:

```
use symbolica::prelude::*;

let expr = parse!("(x+1)^2 - x^2 - 2x - 1");

let result = expr.zero_test(100, 1e-7);

assert_eq!(result, ConditionResult::Inconclusive);
```

The expression is algebraically identical to zero, yet `zero_test()` returns `Inconclusive`.

This illustrates why the method must not be characterized as a complete identity verifier.

Its principal value is as a conditional mathematical test or inexpensive filtering mechanism, rather than as a replacement for exact algebraic equivalence procedures.

For exact polynomial identities, a stronger approach is to convert to a suitable polynomial domain and test whether the polynomial difference is zero. For rational expressions, exact normalization and cancellation can be used under their corresponding algebraic semantics.

Even those exact calculations require care when rational-function cancellations can remove information about excluded input values.

Architectural implication: Symbolica provides complementary mechanisms for quick mathematical-property tests, structural comparison, and exact algebraic reasoning. Selecting among them depends on whether the required result is a heuristic indication, a proved property, or an exact identity within a specified algebraic domain.

Primary Rust reference:

`https://docs.rs/symbolica/3.0.1/symbolica/atom/trait.AtomCore.html`

### 1.3 Numerical nonlinear solving in Rust

The original guide explains exact algebraic equation solving but omits the native numerical Newton solvers.

The `AtomCore` trait includes:

```
fn nsolve<N, V>(
    &self,
    x: V,
    init: N,
    prec: N,
    max_iterations: usize,
) -> Result<N, SolveError>
```

where the numeric type satisfies the required real, floating-point, and evaluation-domain trait bounds.

It also includes the associated method:

```
fn nsolve_system<N, T>(
    system: &[T],
    vars: &[Indeterminate],
    init: &[N],
    prec: N,
    max_iterations: usize,
) -> Result<Vec<N>, SolveError>
```

The second operation addresses systems of nonlinear equations.

Given

\\[ F(x)=0, \\]

Newton iteration requires solving

\\[ J_F(x_k)\Delta x_k=-F(x_k) \\]

and then updating

\\[ x\_{k+1}=x_k+\Delta x_k. \\]

Unlike exact algebraic solving, these methods operate numerically and can address supported transcendental expressions.

The Rust documentation includes the following system-solving example:

```
use symbolica::prelude::*;

let expr1 = parse!("x^2 + y^2 - 1");
let expr2 = parse!("x^2 - y");

let system = &[expr1, expr2];
let vars = &[symbol!("x").into(), symbol!("y").into()];
let init = &[F64::from(0.5), F64::from(0.5)];

let roots = Atom::nsolve_system(
    system,
    vars,
    init,
    1e-7.into(),
    100,
).unwrap();
```

The Rust solver is generic over supported numeric domains, rather than being permanently restricted to primitive `f64`.

The important distinction from the exact solver is that Newton methods locate solutions according to the supplied initial conditions. They do not automatically identify every root or provide a complete parameterized solution set.

Convergence can fail because of singular or ill-conditioned Jacobians, inappropriate starting points, domain restrictions, or insufficient numerical precision.

The availability of both exact and numerical solvers over Symbolica's expression representation permits symbolic preprocessing before numerical solution. For example, mathematical transformations can remove redundant factors, generate derivatives, or expose algebraic constraints before the local numerical solve.

However, simplifications must preserve the domain and desired solution set.

Primary Rust references:

`https://docs.rs/symbolica/3.0.1/symbolica/atom/trait.AtomCore.html`

`https://docs.rs/symbolica/3.0.1/symbolica/solve/index.html`

## 2. Expression inspection and programmable rewriting in Rust

Supplements original §§2–3.

### 2.1 Structural access through `AtomCore`

The original guide explains the `Atom`/`AtomView` representation and basic pattern matching. It does not fully describe how much structural information native Rust callers can access without implementing a separate expression-tree abstraction.

`AtomCore` supplies several complementary interfaces:

| Rust API                                          | Function                                                           |
| ------------------------------------------------- | ------------------------------------------------------------------ |
| `get_atom_type()`                                 | Identify the expression variant                                    |
| `as_add_view()`, `as_mul_view()`, `as_fun_view()` | Access specialized borrowed views of sums, products, and functions |
| `as_num_view()`, `as_var_view()`                  | Access numerical or symbolic variants                              |
| `children()`                                      | Iterate over immediate expression components                       |
| `terms()`                                         | Iterate over additive terms                                        |
| `visitor()`                                       | Visit subexpressions recursively, with control over descent        |
| `position()`                                      | Locate matching subexpressions                                     |
| `get_all_symbols()`, `get_all_indeterminates()`   | Discover symbolic dependencies                                     |
| `to_canonical_string()`                           | Produce a canonical textual representation                         |

These interfaces serve different purposes. `children()` exposes immediate structure, whereas `visitor()` traverses recursively. `terms()` exposes additive structure rather than arbitrary tree nodes.

The visitor callback can decide whether the traversal should descend into a subexpression. This permits selective inspection without materializing a second full expression tree.

The typed views are especially useful because the representation of an addition, multiplication, or function carries mathematical structure that a generic sequence of independently allocated nodes would not necessarily preserve.

Architectural significance: native traversal, typed borrowed views, and symbol discovery can support structural analysis, dependency extraction, expression inspection, and custom transformations while retaining Symbolica's canonical expression representation.

Primary source:

`https://docs.rs/symbolica/3.0.1/symbolica/atom/trait.AtomCore.html`

### 2.2 Callback-based transformations

`AtomCore` provides callback-oriented operations in addition to declarative pattern replacement.

Relevant methods include `replace_map()`, `replace_map_bottom_up()`, `collect_mapped()`, `collect_multiple_mapped()`, and `collect_symbol_mapped()`.

The mapping interfaces receive borrowed expression data and a mutable output target, including `Settable<Atom>` in several APIs.

This gives native Rust callers more control than a simple expression-in/expression-out replacement callback.

For example, `collect_mapped()` can transform both the expression serving as a collection key and its coefficient. Consequently, collection need not consist solely of grouping identical powers into an ordinary polynomial-like representation.

`replace_map_bottom_up()` traverses from deeper subexpressions toward their parents, allowing a transformation to observe or replace already-transformed children.

This is different from pre-order replacement, where replacing an outer expression can prevent some internal subexpressions from being visited.

The choice matters for transformations such as constant folding, eliminating redundant nested functions, and propagating local simplifications through a larger expression.

The native APIs allow these operations to remain integrated with Symbolica's construction and normalization mechanisms. They do not, however, guarantee that arbitrary user callbacks terminate, preserve mathematical equivalence, or avoid substantial intermediate allocations.

### 2.3 Advanced `ReplaceBuilder` controls

The native `symbolica::id::ReplaceBuilder` exposes a broader range of matching and replacement controls than described in the original reference.

| Method                                        | Significance                                                |
| --------------------------------------------- | ----------------------------------------------------------- |
| `min_level()`, `max_level()`, `level_range()` | Restrict where a pattern may match                          |
| `level_is_tree_depth()`                       | Distinguish literal tree depth from function nesting        |
| `partial()`                                   | Control partial-subexpression matching                      |
| `non_greedy_wildcards()`                      | Prefer smaller sequence-wildcard matches                    |
| `when()`                                      | Apply mathematical or structural restrictions               |
| `once()`                                      | Replace one occurrence rather than all eligible occurrences |
| `bottom_up()`                                 | Prioritize deeper matches                                   |
| `nested()`                                    | Apply replacements through nested matched structures        |
| `repeat()`                                    | Repeat rewriting until no further match is available        |
| `rhs_cache_size()`                            | Bound cached replacement results                            |
| `with_map()`, `iter_map()`                    | Construct replacements through native Rust callback logic   |
| `match_iter()`, `iter()`                      | Inspect matches or alternative replacements lazily          |

The distinction between `bottom_up()` and `nested()` is particularly consequential.

For a rule

\\[ f(u)\longrightarrow u^2 \\]

applied to \\(f(f(x))\\), different traversal modes can produce materially different intermediate results.

A bottom-up replacement processes the inner expression first, while nested replacement can continue applying the rule to enclosing matches formed around the transformed result.

`rhs_cache_size()` offers another useful mechanism: repeated wildcard bindings can reuse prepared replacement results rather than recalculating an expensive right-hand-side transformation.

These controls give the matcher a programmable execution strategy. Nevertheless, they do not create a general equality-saturation system or guarantee an optimal expression under a global cost model.

### 2.4 Associativity as a registered symbol attribute

Symbolica's native Rust symbol API exposes the `Flat` attribute:

```
use symbolica::prelude::*;

let f = symbol!("f"; Flat);
assert!(f.is_flat());
```

For a flat function \\(F\\), nested applications are normalized according to associativity:

\\[ F(a,F(b,c))\longrightarrow F(a,b,c). \\]

This is separate from `Symmetric`, `Antisymmetric`, `Cyclesymmetric`, and `Linear` attributes.

In particular, associativity does not imply commutativity. The order of arguments can remain mathematically significant even when nested applications are flattened.

Combining flatness with other function attributes can express mathematical operators whose normalization would otherwise require repeated custom rewriting.

The attribute is appropriate only when the user-defined function genuinely obeys associativity. It should be registered before constructing expressions that depend on that behavior.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Primary sources:

`https://docs.rs/symbolica/3.0.1/symbolica/id/struct.ReplaceBuilder.html`

`https://docs.rs/symbolica/3.0.1/symbolica/atom/struct.Symbol.html`

## 3. Specialized polynomial algorithms and domain-aware computation

Supplements original §4.

The original reference identifies Symbolica's polynomial kernel, coefficient domains, and Gröbner bases. Several additional native algorithms and representation controls merit substantially more attention.

### 3.1 Three independent polynomial type parameters

The native Rust type is:

```
MultivariatePolynomial<
    F: Ring,
    E: Exponent = u16,
    O: MonomialOrder = LexOrder,
>
```

The third type parameter, `O`, explicitly selects a monomial-ordering implementation. The original overview shows only the coefficient and exponent type parameters.

The representation is also more specific than a generic sparse polynomial:

- Nonzero coefficients are stored in a contiguous vector.
- Exponents are stored as flattened, dense blocks for each monomial.
- Terms follow the selected monomial ordering.
- The variable map is shared through a reference-counted context.

This is a sparse-term representation with dense exponent vectors, rather than a sparse map for every variable exponent.

Direct manipulation of the public `coefficients` or `exponents` vectors must preserve ordering and nonzero-coefficient invariants.

The distinction is consequential because polynomial algorithms depend on monomial order, coefficient-domain arithmetic, and the representation's structural invariants.

### 3.2 Exact division, polynomial congruences, and Bézout identities

Native polynomial methods include:

| Operation                                  | Rust interface              | Purpose                                                                         |
| ------------------------------------------ | --------------------------- | ------------------------------------------------------------------------------- |
| Extended Euclidean algorithm               | `eea_univariate()`          | Recover GCD and Bézout coefficients                                             |
| Multi-factor Diophantine solving           | `diophantine_univariate()`  | Construct coefficients for combinations of pairwise coprime polynomial products |
| Modular exponentiation                     | `exp_mod_univariate()`      | Compute powers modulo a polynomial                                              |
| Precomputed-inverse modular exponentiation | `exp_mod_univariate_fast()` | Reuse a prepared reversed-divisor inverse                                       |
| Exact divisibility                         | `try_div()`                 | Return a quotient only when division is exact                                   |
| Quotient and remainder                     | `quot_rem()`                | Separate divisible and residual polynomial components                           |

For univariate polynomials over a field, the extended Euclidean method computes

\\[ sf+tg=\gcd(f,g). \\]

When \\(f\\) and \\(g\\) are coprime, this provides a modular inverse:

\\[ sf\equiv1\pmod g. \\]

This is foundational for arithmetic in polynomial quotient fields and for constructing exact algebraic algorithms from smaller modular operations.

`diophantine_univariate()` generalizes the idea to a collection of pairwise coprime factors. Its coprimality precondition is essential; the method should not be interpreted as an unrestricted polynomial-equation solver.

The fast modular division and exponentiation methods also expose an important optimization strategy: prepare and reuse a divisor-related inverse across repeated operations, instead of recomputing the same division machinery for every exponentiation or reduction.

### 3.3 Resultants and algebraic elimination

The native `UnivariatePolynomial` API exposes resultant algorithms, including implementations based on different subresultant and Euclidean techniques.

These include `resultant()`, `resultant_brown()`, `resultant_primitive()`, and, where the coefficient domain supports the necessary field operations, `resultant_euclidean()`.

For univariate polynomials \\(f\\) and \\(g\\), their resultant vanishes when they have a common root over an appropriate algebraic closure, subject to the usual polynomial-degree conventions.

Resultants are useful for eliminating variables from algebraic relationships.

For example, given

\\[ f(x,y)=x^2-y,\qquad g(x,y)=x-1, \\]

eliminating \\(x\\) yields a condition proportional to

\\[ 1-y=0. \\]

One important native representation mechanism is `MultivariatePolynomial::to_univariate()`: it treats one variable as the univariate indeterminate and retains the remaining variables inside the polynomial coefficient ring.

That allows the specialized univariate algorithms to operate on a multivariate problem without naively expanding every coefficient into an unrelated scalar representation.

The tradeoff is that elimination can increase coefficient complexity substantially. A resultant also supplies a necessary algebraic condition that may require checking against the original system, especially under degenerate parameter specializations.

### 3.4 Polynomial reduction modulo algebraic relations

`MultivariatePolynomial::reduce()` reduces a polynomial using a collection of polynomial divisors.

The method is available under the required field and monomial-ordering constraints.

It provides the algebraic operation underlying computations modulo polynomial relations.

For example, with

\\[ x^2-1=0, \\]

a polynomial containing higher powers of \\(x\\) can be reduced to a simpler representative.

However, the uniqueness and ideal-membership interpretation of a remainder depend on the basis and ordering.

An arbitrary list of divisors does not automatically provide a canonical quotient-ring normal form. For general ideal-membership decisions, a Gröbner basis under the corresponding ordering provides the stronger relevant structure.

This is an important distinction between a polynomial reduction operation and a complete algorithm for testing consequences of arbitrary polynomial constraints.

### 3.5 Newton interpolation and rational approximation

The native Rust API includes:

```
MultivariatePolynomial::newton_interpolation(...)
```

Its inputs are sample points and polynomial-valued evaluations, together with the variable in which interpolation is performed.

The ability to interpolate with polynomial-valued outputs is particularly significant. It allows one variable's dependence to be reconstructed while retaining the algebraic dependence of coefficients on other variables.

The kernel also provides:

```
rational_approximant_univariate(
    &self,
    deg_n: u32,
    deg_d: u32,
)
```

This computes a Padé-type rational approximant

\\[ \frac{n(x)}{d(x)} \\]

subject to

\\[ d(x)f(x)-n(x)\equiv0 \pmod{x^{\deg_n+\deg_d+1}}, \\]

with the specified numerator and denominator degree bounds.

This operation is not ordinary floating-point curve fitting and is not the same as rational-number reconstruction from modular residues.

It recovers a rational representation agreeing with a given formal expansion through a specified order. This can capture function structure more compactly than a truncated polynomial when the function is genuinely rational or suitably approximated by one.

### 3.6 Modular reconstruction, Chinese remaindering, and Hensel lifting

The specialized integer-polynomial API includes `chinese_remainder()`, which combines polynomial coefficients that satisfy congruences modulo two specified integer moduli.

The associated principle is

\\[ a\equiv a_1\pmod m,\qquad a\equiv a_2\pmod p. \\]

Under the necessary coprimality conditions, these congruences identify a residue modulo \\(mp\\).

This is foundational for computing with several small modular images and reconstructing larger integer coefficients.

The native `hensel_lift()` method instead addresses lifting a polynomial factorization from a prime modulus to a higher prime power.

Conceptually, starting with

\\[ f\equiv uw\pmod p, \\]

it attempts to construct a lifted factorization modulo \\(p^k\\).

Its return type distinguishes an exact recovered integer factorization from an approximate lifted factorization that has not yielded exact factors.

That distinction matters: successful modular lifting does not necessarily mean a nontrivial factorization exists over the integers.

Together, CRT and Hensel lifting help explain how Symbolica's modular-algebra machinery can avoid performing every intermediate calculation using enormous characteristic-zero coefficients.

### 3.7 Sparse multivariate GCD algorithms

Symbolica exposes specialized GCD methods based on Hu–Monagan interpolation:

- `gcd_hu_monagan()`
- `gcd_hu_monagan_bivariate()`

These can reconstruct a sparse GCD or a smaller cofactor from sampled polynomial images.

The bivariate variant retains two variables while sampling the others along geometric sequences. It can reconstruct whichever of the GCD or cofactor has the smaller sampled support and then attempt to certify the result by exact division.

Its documented preconditions include compatible variable layouts, more than two variables, and suitable degree bounds.

An unlucky specialization or unsuccessful reconstruction returns `None`, rather than implying the true GCD is one.

This is a useful distinction from a generic GCD algorithm: sparse interpolation can exploit the shape of a result that would be expensive to obtain through a large dense intermediate computation.

Its advantage depends on input sparsity, result sparsity, degree bounds, and sampling behavior. It is not uniformly superior to every other GCD algorithm.

### 3.8 Root counting and numerical versus exact polynomial roots

`MultivariatePolynomial::count_positive_real_roots()` uses a Sturm sequence to count distinct roots in

\\[ (0,\infty) \\]

for a supported univariate polynomial over a field with a real embedding.

It returns a result carrying possible root-counting errors, rather than silently producing a count for unsupported inputs.

The method counts distinct positive real roots; it is not a general complex-root isolator or a method for determining every root's multiplicity.

Separate `UnivariatePolynomial` functionality supports numerical root approximation and, for suitable exact coefficient domains, isolated algebraic roots.

These operations answer different questions:

- Root counting establishes how many distinct roots lie in a specified region.
- Numerical approximation estimates root locations.
- Root isolation identifies mathematical regions containing individual roots.
- An exact algebraic-root representation retains the root's defining polynomial and isolating information.

A numerical root solver's convergence tolerance is not automatically a certified bound on the exact root-location error.

### 3.9 Polynomial-context ownership and parallel execution

An easily overlooked performance facility is the management of shared polynomial contexts.

`MultivariatePolynomial` provides:

- `zero_with_new_context()`
- `clone_with_context_of()`
- `compact()`

The documentation explains that ordinary polynomial cloning or destruction can update an atomic reference count associated with a shared context.

When many worker threads manipulate polynomials sharing the same context, these updates may cause cache-line contention.

`zero_with_new_context()` creates a mathematically equivalent but independently allocated context. `clone_with_context_of()` permits a polynomial to be cloned onto a selected compatible context instead of updating the reference count of the source context.

This can reduce contention when repeated polynomial operations run independently across threads.

There are important invariants: the target context must have an equal coefficient field and variable map, and `clone_with_context_of()` can panic if they differ.

`compact()` serves a different purpose, shrinking coefficient and exponent buffers for longer-lived stored polynomials. It is not intended to run repeatedly on every temporary because the resulting reallocations can outweigh the space savings.

These are concrete native facilities for managing ownership and memory traffic, not just abstract claims that Rust or Symbolica is efficient.&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



Primary Rust references:

`https://docs.rs/symbolica/3.0.1/symbolica/poly/polynomial/struct.MultivariatePolynomial.html`

`https://docs.rs/symbolica/3.0.1/symbolica/poly/univariate/struct.UnivariatePolynomial.html`

`https://docs.rs/symbolica/3.0.1/symbolica/poly/univariate/struct.IsolatedRoot.html`

## 4. Native numerical evaluator extension points

Supplements original §§7–8.

### 4.1 Composing already-built evaluators

`ExpressionEvaluator<T>` provides:

```
pub fn merge(
    &mut self,
    other: Self,
    cpe_rounds: Option<usize>,
) -> Result<(), String>
```

This combines evaluators that use the same input parameters, concatenating their outputs.

It is distinct from `Atom::evaluator_multiple()`, which constructs a multi-output evaluator from a collection of expressions before optimization.

With `merge()`, independently prepared evaluators can be composed and then subjected to additional common-pair elimination.

A consequential Rust type constraint is that merging requires:

```
T: Default + Clone + Eq + Hash
```

Thus, merging exact-rational evaluators before converting coefficients to primitive floating-point values is an important option. Ordinary `f64` does not implement `Eq`, so a floating-point evaluator cannot be assumed to support this method directly.

The optional `cpe_rounds` parameter limits the additional common-pair elimination performed after merging.

This allows computational modules to retain separate mathematical definitions while still exposing shared intermediate calculations during evaluator composition.

### 4.2 Exporting a structured evaluator instruction representation

The native method is:

```
pub fn export_instructions(&self) -> ExportedInstructions<T>
```

Its output includes:

| Field                | Significance                                                                            |
| -------------------- | --------------------------------------------------------------------------------------- |
| `input_count`        | Number of input parameters                                                              |
| `output_count`       | Number of numerical outputs                                                             |
| `temporary_count`    | Required temporary slots                                                                |
| `instructions`       | Optimized instruction stream                                                            |
| `constants`          | Numerical constants referenced by instructions                                          |
| `constant_functions` | Registered constants and fixed-argument functions requiring target-precision resolution |
| `sub_evaluators`     | Retained bodies for functions that were not inlined                                     |

This representation provides a useful boundary between Symbolica's algebraic optimization machinery and a separate numerical execution engine.

For example, a custom runtime could consume the optimized instructions without independently implementing Horner transformation, common-subexpression elimination, and function expansion.

The distinction between embedded sub-evaluators and ordinary external functions is important. An instruction invoking a function without a matching sub-evaluator still requires an external implementation.

Similarly, constants associated with symbolic mathematical functions may require evaluation at the target precision rather than copying a previously rounded literal.

The native Rust API does not document a direct `ExpressionEvaluator::from_instructions()` counterpart to the Python convenience interface. `export_instructions()` establishes access to the optimized representation, but it does not by itself establish a complete Rust save/reload workflow for an arbitrary evaluator.

The `serde` and `bincode` feature flags should not be interpreted as guarantees that every evaluator, associated external callback, and compiled artifact supports lossless independent serialization.

### 4.3 Generic evaluator vectorization

One of the more significant native extension points is:

```
pub fn vectorize<V: Vectorize<T>>(
    self,
    v: &V,
) -> Result<ExpressionEvaluator<T>, String>
```

The `Vectorize<T>` trait defines how evaluator coefficients and instructions are lifted from scalar slots to multiple components.

Each original slot can correspond to \\(n\\) component slots, and a scalar instruction can be transformed into a collection of instructions acting on those components.

The trait exposes methods including:

- `get_dimension()`: Number of components.
- `duplicate_constants()`: Whether constants are replicated across components or confined to the base component.
- `map_instruction()`: Convert an instruction into its component-wise representation.
- `vectorize_function()`: Optionally construct a scalar evaluator for the component-level behavior of an external function.

This mechanism is broader than simply applying a SIMD vector type to an evaluator.

The supplied `Vectorize<T>` implementation defines the mathematical semantics of the component expansion.

Symbolica's `Dualizer<T>` is a concrete implementation used for automatic differentiation. It lifts instructions into a dual-number component representation.

However, the trait itself is a lower-level program-transformation interface, potentially supporting other component-based numerical algebras when their operations can be expressed through the available instruction semantics.

An important restriction appears in the native source: the vectorization operation can reject an evaluator containing retained, non-inlined sub-evaluators.

Therefore, evaluator function-inlining policy is not merely a performance concern; it can determine whether later evaluator transformations are available.

The handling of external functions is similarly explicit. Vectorized function registrations or a supported automatic component expansion must be available.

### 4.4 Function-registration policies and scoped numerical programs

`EvaluatorBuilder` supports:

```
add_function_with_options(...)
add_tagged_function_with_options(...)
add_aliases(...)
```

`FunctionRegistrationOptions` can specify an `InliningPolicy`.

The native enum exposes `Always`, `Never`, and `Auto`.

The precise implementation detail matters:

- `Always` expands a function's body at call sites.
- `Never` keeps the body in a separate sub-evaluator.
- `Auto` is documented in version 3.0.1 as currently having the same behavior as `Always`.

The original guide suggests that the automatic policy independently decides whether to inline. That description should be corrected for the targeted release.

Non-inlined functions can reduce duplication when large function bodies are called repeatedly, but they introduce separate sub-evaluator execution and can restrict transformations that require a flattened instruction representation.

Inlining exposes more cross-function common-subexpression opportunities, while potentially increasing the evaluator's instruction count.

Function registration also has a scope dimension. Registered expressions may reference parameters of surrounding evaluator contexts; explicit function arguments must be distinguished from captured outer parameters.

The native implementation includes tests addressing these scope and capture interactions.

### 4.5 Optimization controls and resource constraints

`EvaluatorBuilder` exposes more optimizer configuration than the original reference describes.

Relevant Rust methods include:

| Method                            | Control                                            |
| --------------------------------- | -------------------------------------------------- |
| `optimization_settings()`         | Configure optimization settings together           |
| `horner_iterations()`             | Limit Horner-scheme search                         |
| `cpe_iterations()`                | Set common-pair elimination rounds                 |
| `max_horner_scheme_variables()`   | Bound variables considered for Horner optimization |
| `max_common_pair_cache_entries()` | Bound common-pair cache size                       |
| `hot_start()`                     | Supply prepared expressions for optimization       |
| `cores()`                         | Set optimization parallelism                       |
| `abort_check()`                   | Allow an external abort condition                  |
| `abort_level()`                   | Configure optimization abort polling               |

These settings recognize that optimization itself can be computationally expensive.

Horner search and common-pair elimination may substantially reduce repeated numerical work, but their exploration costs can grow with expression complexity.

Consequently, the builder's controls support a distinction between mathematical-program preparation time and execution time.

An optimization suitable for millions of evaluations may be unnecessarily expensive for a program executed only a few times.

### 4.6 Temporary storage reuse and operation accounting

After evaluator construction, `optimize_stack()` reduces temporary-storage requirements by reusing slots after their final use.

This is a separate optimization from algebraic common-subexpression elimination.

CSE reduces repeated mathematical work; stack optimization reduces the storage needed to hold the resulting intermediate computations.

The evaluator also exposes `count_operations()`, returning an `OperationCount` covering categories such as arithmetic operations and function calls.

These mechanisms support analysis of both operation count and temporary-storage demand without requiring full native compilation.

Operation counts are not direct performance measurements. They do not independently capture memory access costs, transcendental-function latency, branch behavior, SIMD utilization, or instruction scheduling.

### 4.7 Real-domain specialization and evaluator lifecycle

`ExpressionEvaluator<Complex<T>>::set_real_params()` allows selected evaluator inputs to be declared real.

The method also accepts `ComplexEvaluatorSettings`, which can express assumptions about real-valued behavior of supported operations.

These assumptions can permit more efficient real-arithmetic code generation.

However, real inputs do not guarantee real outputs. For example, \\(\sqrt{x}\\) is not real for negative real \\(x\\) under ordinary principal complex-square-root semantics.

The documented lifecycle requirement is explicit: real-parameter registration must happen after optimization and merging. Earlier registration can be lost when those operations reconstruct or modify the evaluator.

This is an example of an extension point whose use depends on correct ordering of evaluator-construction phases.

### 4.8 Evaluation over alternative rings

`ExpressionEvaluator<Complex<Rational>>` provides `map_to_ring()` for mapping evaluator coefficients into another supported ring.

The evaluator can then be used through `evaluate_in_ring()`.

This is not equivalent to converting every coefficient to `f64`.

For example, an evaluator can be mapped into a finite field such as `Zp`, permitting arithmetic under a specified modulus.

That creates a useful connection between symbolic program optimization and exact modular evaluation.

The generic ring boundary still matters: the target domain must support the operations required by the evaluator. An expression involving transcendental functions cannot automatically be executed in an arbitrary finite field merely because its polynomial arithmetic can be mapped there.

The evaluator also provides `map_coeff_with_prec()`, allowing the coefficient conversion to carry an explicit binary-precision specification.

This is significant when an evaluator is moved between exact, approximate, and arbitrary-precision numerical representations.

Primary Rust references for §4:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+4



`https://docs.rs/symbolica/3.0.1/symbolica/evaluate/struct.ExpressionEvaluator.html`

`https://docs.rs/symbolica/3.0.1/symbolica/evaluate/struct.EvaluatorBuilder.html`

`https://docs.rs/symbolica/3.0.1/symbolica/evaluate/trait.Vectorize.html`

`https://docs.rs/symbolica/3.0.1/symbolica/evaluate/enum.InliningPolicy.html`

`https://docs.rs/symbolica/3.0.1/symbolica/evaluate/struct.ExportedInstructions.html`

## 5. Additional exact and structured mathematical facilities

Supplements original §§4, 8, and 10.

### 5.1 Rational reconstruction from modular evaluations

The native `Rational` type exposes two distinct reconstruction mechanisms.

```
Rational::maximal_quotient_reconstruction(...)
Rational::rational_reconstruction(...)
```

`maximal_quotient_reconstruction()` reconstructs a rational number from a residue modulo a prime, using a maximal-quotient reconstruction algorithm.

The underlying problem is to recover

\\[ r=\frac{a}{b} \\]

from modular information satisfying

\\[ ab^{-1}\equiv v\pmod p, \\]

where the denominator is invertible modulo \\(p\\).

The residue alone does not uniquely identify an unrestricted rational number. Successful reconstruction therefore depends on the algorithm's admissibility conditions and sufficient modulus information.

`rational_reconstruction()` provides another important capability: it accepts a function that evaluates over a finite field, together with a sample point and optional starting-prime configuration.

The method attempts to reconstruct the rational result corresponding to the black-box evaluation.

This offers an alternative to computing with large rational numerators and denominators throughout an entire calculation.

An algorithm may instead evaluate over suitable finite fields and reconstruct the exact rational result afterward.

This is particularly relevant to algebraic calculations involving exact rational outputs, where intermediate rational arithmetic can cause substantial coefficient growth.

The operation returns an error when reconstruction fails. The documentation specifically notes unsuccessful or unlucky modular evaluations.

Scope distinction: these are native rational-number reconstruction methods. Their existence does not, by itself, establish a general public API for reconstructing arbitrary multivariate rational functions from black-box evaluations.

Primary source:

`https://docs.rs/symbolica/3.0.1/symbolica/domains/rational/type.Rational.html`

### 5.2 Rationalization of approximate symbolic coefficients

Rust's `AtomCore` also provides:

```
fn rationalize(
    &self,
    relative_error: &Rational,
) -> Self::Output
```

It replaces numerical coefficients with simpler rational approximations within a specified relative-error interval.

The Rust documentation includes:

```
use symbolica::prelude::*;

let expr = parse!("0.333");
let result = expr.rationalize(&(1, 100).into());

assert_eq!(result, Atom::num((1, 3)));
```

The resulting transformation changes the coefficient's exact mathematical value. It is not simply an alternative formatting of the same number.

For example, the exact decimal \\(0.333\\) differs from \\(1/3\\), despite their proximity.

This distinction is important when rationalization is followed by exact polynomial arithmetic. An exact computation over the rationalized expression proves properties of the new rational coefficients, not necessarily of the original approximate values.

Rationalization is therefore a model-reconstruction or approximation operation, unlike exact modular rational reconstruction.

### 5.3 Linear systems as exact coefficient-domain matrices

The Rust API exposes:

```
AtomCore::system_to_matrix(...)
```

Its return type is a pair of matrices over

```
RationalPolynomialField<Z, E>
```

where `E` is a positive exponent type.

The method transforms a supported linear equation system into a coefficient matrix and right-hand-side matrix.

For a system

\\[ \begin{aligned} 2x+y&=1,\\\ x-y&=-1, \end{aligned} \\]

the matrix representation is

\\[ \begin{bmatrix} 2&1\\\ 1&-1 \end{bmatrix} \begin{bmatrix} x\\\y \end{bmatrix} = \begin{bmatrix} 1\\\\-1 \end{bmatrix}. \\]

A particularly useful feature is the rational-polynomial coefficient domain. The original symbolic equations may contain parameters that are not among the unknowns, and their algebraic dependence can be retained in the matrix coefficients.

This creates a native bridge between general symbolic equations and specialized exact linear-algebra algorithms.

It is not an arbitrary symbolic-to-matrix converter: the selected unknowns must enter the supported system linearly, and the method can return `SolveError` when conversion fails.

### 5.4 Tensor-index canonicalization

The native interface is:

```
fn canonize_tensors<I, T, G>(
    &self,
    indices: I,
) -> Result<CanonicalTensor<T, G>, TensorCanonicalizationError>
```

where the index expressions implement `AtomCore` and the grouping type supports the required ordering and hashing operations.

This operates on products of tensor-like symbolic functions whose arguments represent indices.

It recognizes equivalent index structures under permitted relabeling of dummy indices and supported symmetry attributes.

For example,

\\[ A\_{ij}B\_{ji} \\]

and

\\[ A\_{kl}B\_{lk} \\]

represent the same formal contraction when the respective indices are interchangeable dummy indices from compatible spaces.

By canonicalizing these indices, Symbolica can bring equivalent terms into a common representation, enabling subsequent collection and cancellation.

The native return type retains more information than just the canonical expression. It also reports external indices and ordered dummy indices.

Index groups prevent renaming across incompatible classes, such as indices belonging to different dimensions or tensor representations.

The method also accounts for relevant tensor symmetries. Antisymmetric tensors can introduce sign changes under permutations, while cyclic symmetries permit selected cyclic reorderings.

This capability is distinct from full tensor-network computation.

It does not automatically construct contraction plans, allocate multidimensional tensor data, or execute numerical contractions. Those responsibilities belong to separate tensor abstractions, including the Spenso ecosystem discussed in the original guide.

The expression-level canonicalizer instead provides a focused mechanism for recognizing equivalence of indexed symbolic expressions.

Primary Rust reference:

`https://docs.rs/symbolica/3.0.1/symbolica/atom/trait.AtomCore.html`

### 5.5 Subexpression occurrence analysis and alias extraction

Two additional native methods connect structural expression analysis to reuse of repeated mathematical work:

```
count_subexpressions()
alias_subexpressions(...)
```

`count_subexpressions()` identifies repeated nonconstant, nonvariable subexpressions and their occurrence counts.

For example,

\\[ f(x+1)+x f(x+1) \\]

contains repeated occurrences of \\(f(x+1)\\).

Counting these structures can expose reuse opportunities before numerical evaluator optimization.

`alias_subexpressions()` supplies a callback that receives a subexpression, its occurrence count, and its index in the discovered collection.

The callback can select which subexpressions should receive aliases and provide the corresponding replacement expressions.

The result is an `AliasedAtom`.

This allows aliasing decisions to depend on more than whether an expression repeats. A caller could distinguish a trivial repeated addition from an expensive repeated nonlinear subexpression.

The performance consequences depend on how aliases are subsequently interpreted. An alias representation does not automatically guarantee that every consuming algorithm shares the underlying numerical computation.

It does, however, provide a native mechanism for discovering and representing potential reuse without implementing a separate structural hashing and expression extraction system.

### 5.6 Built-in transcendental and special functions in Rust

The native `symbolica::transcendental` module provides built-in function symbols and the `TranscendentalFunctions` trait.

Its documented built-ins include:

| Mathematical family                | Native Rust functions                                |
| ---------------------------------- | ---------------------------------------------------- |
| Gamma and related functions        | `gamma`, `polygamma`                                 |
| Zeta and polylogarithm             | `zeta`, `polylog`                                    |
| Bessel functions                   | `bessel_i`, `bessel_j`, `bessel_k`, `bessel_y`       |
| Error function                     | `erf`                                                |
| Hyperbolic functions               | `sinh`, `cosh`, `tanh`, `sech`, `csch`, `coth`       |
| Inverse hyperbolic functions       | `asinh`, `acosh`, `atanh`, `asech`, `acsch`, `acoth` |
| Additional trigonometric functions | `sec`, `csc`, `cot` and their inverses               |
| Mathematical constants             | `euler_gamma`                                        |
| Algebraic roots                    | `root`, `root_var`                                   |

These are recognized built-in symbolic functions, rather than arbitrary user-defined function heads.

This distinction can affect normalization, symbolic derivatives, special values, and numerical evaluation, depending on the implemented behavior of each function.

The advanced mathematical consideration is analytic-domain semantics.

Complex logarithms, inverse trigonometric functions, Bessel functions, and polylogarithms can have poles or branch cuts. Consequently, a transformation valid for positive real arguments may not remain valid for arbitrary complex arguments.

Recognizing a function as built-in does not establish that every symbolic identity, high-precision numerical method, series expansion, or compiled evaluator backend supports it.

A numerical backend must support the actual operations appearing in the generated evaluator.

Primary Rust references:&#x20;

[image](https://www.google.com/s2/favicons?domain=https://docs.rs\&sz=32)

Rust+2



`https://docs.rs/symbolica/3.0.1/symbolica/atom/trait.AtomCore.html`

`https://docs.rs/symbolica/3.0.1/symbolica/transcendental/`

## 6. Consequential interactions among the native Rust capabilities

The newly documented mechanisms combine in several ways that materially extend Symbolica's functionality.

### 6.1 Exact modular evaluation and reconstruction

A mathematical computation can preserve symbolic structure during construction, perform selected operations through finite-field evaluation, and reconstruct rational outputs afterward.

The relevant mechanisms include:

- Polynomial and rational coefficient domains.
- Chinese remainder reconstruction.
- Modular exponentiation and polynomial division.
- Hensel lifting.
- `Rational::rational_reconstruction()`.
- `ExpressionEvaluator::map_to_ring()`.

This composition can avoid maintaining large rational intermediate values at every step.

Its feasibility depends on the operation being defined in the selected modular domain, suitable prime choices, and sufficient information for reconstruction.

### 6.2 Structural transformation followed by specialized algebra

Native expression visitors, callback-based rewriting, coefficient collection, and selective expansion can retain the information needed for structural manipulations.

The resulting expressions can then be converted to specialized polynomial representations for:

- Exact division and modular arithmetic.
- Resultants and variable elimination.
- Interpolation and Padé approximation.
- Sparse GCD reconstruction.
- Algebraic-system reduction.

This is often more coherent than implementing the entire sequence as unrestricted expression-tree rewriting.

A crucial boundary remains: converting an arbitrary nonlinear subexpression into an independent polynomial indeterminate can obscure mathematical relationships unless they are modeled separately.

### 6.3 Composable mathematical programs and specialized execution

The evaluator architecture can support a sequence in which mathematical expressions are optimized independently, merged through compatible parameter conventions, and then subjected to additional common-pair elimination.

The resulting evaluator can be transformed through `Vectorize<T>`, mapped into a supported numerical domain, inspected through exported instructions, or passed to one of the native code-generation paths.

Not all operations commute.

For example, converting an evaluator to primitive floating-point coefficients before merging may violate `merge()`'s trait bounds. Retaining non-inlined sub-evaluators may prevent vectorization. Setting real-parameter assumptions before merging or optimization can lose those registrations.

These are not incidental API details. They are part of the evaluator's computational lifecycle and determine which combinations of optimization and execution machinery are available.

### 6.4 Parallel exact algebra with controlled representation overhead

Sparse polynomial algorithms can be combined with thread-local polynomial contexts and borrowed general-expression views to reduce unnecessary memory traffic.

In particular, `zero_with_new_context()` and `clone_with_context_of()` address reference-count contention that can otherwise arise when many workers share a polynomial's immutable context.

This illustrates a broader performance distinction: mathematical algorithm selection and representation-level resource management are separate optimization dimensions.

A more efficient GCD algorithm does not eliminate context-sharing overhead, and an optimized context layout does not eliminate the inherent complexity of a difficult polynomial problem.

## 7. Version-specific corrections and scope boundaries

Several earlier descriptions warrant explicit correction in the original reference or should be constrained when this addendum is incorporated.

| Topic                         | Rust-specific clarification                                                                                                                                         |
| ----------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Mathematical identity testing | `AtomCore::zero_test()` returns `ConditionResult` and is not equivalent to Python's `check_zero()`.                                                                 |
| Polynomial type               | `MultivariatePolynomial` has three type parameters, including `O: MonomialOrder`.                                                                                   |
| Automatic inlining            | In the documented 3.0.1 implementation, `InliningPolicy::Auto` behaves like `Always`.                                                                               |
| Evaluator composition         | `merge()` requires compatible parameters and coefficient types satisfying its Rust trait bounds.                                                                    |
| Evaluator persistence         | `export_instructions()` is verified; a general Rust equivalent of Python's evaluator reconstruction methods is not established by that API.                         |
| Evaluator vectorization       | `Vectorize<T>` is a native extension interface, with restrictions for retained sub-evaluators and external functions.                                               |
| Polynomial root analysis      | Numeric root approximation, Sturm-based root counting, and exact root isolation provide different guarantees.                                                       |
| Polynomial reduction          | A remainder computed from arbitrary divisors is not necessarily a unique ideal normal form.                                                                         |
| Rationalization               | `AtomCore::rationalize()` changes coefficients within a specified approximation tolerance; it does not prove their original exact values.                           |
| Companion ecosystem           | Python Community wrappers do not establish equivalent APIs in the native Rust crate. Only separately verified Rust crates should be described as Rust integrations. |

No Python-only Community functionality has been carried over into this revised Rust addendum.

In particular, the previous addendum's Python `HeldExpression`, `MathMLViewer`, `Evaluator.save/load`, Python-style tensor interfaces, and Community hyperlogarithmic and HEPkit wrappers are not presented as native Rust functionality.

They would require separate Rust-level verification before being included in a reference whose focus is the Rust library.

## Conclusion

The additional native capabilities reinforce an important distinction in Symbolica's architecture: it provides not only a general symbolic-expression system and a numerical compiler, but also specialized computational-algebra algorithms and programmable intermediate representations that can be combined under explicit mathematical and Rust type constraints.

The most consequential additions to the original Rust reference are advanced polynomial algorithms and reconstruction, nonlinear numerical solving, evaluator merging and generic vectorization, structural callback-based transformations, tensor-index canonicalization, and representation-level performance controls.

These mechanisms can reduce the need for bespoke exact-arithmetic algorithms, expression-analysis infrastructure, numerical-program transformation machinery, and selected low-level memory-management optimizations.

Their effectiveness depends on the mathematical domain, the representation used, and the order in which operations are composed. The Rust API makes many of these boundaries explicit through trait bounds, specialized types, error results, and evaluator lifecycle requirements.

The central architectural opportunity is therefore broader than symbolic simplification: Symbolica can serve as a common mathematical representation and transformation engine while exposing specialized exact-algebra and executable numerical forms at the points where those representations provide stronger guarantees or more efficient computation.