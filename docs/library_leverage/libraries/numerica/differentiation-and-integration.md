# Differentiation and adaptive integration

Reusable type: `numerica/differentiation-and-integration`.
Research date 2026-11-09. **Interface-checked** against installed Numerica 3.0.1, checksum
`836accdb415b7667573bd12676e0b2eb4440e3f60b110038d96a9086f5651c15`.
No runtime probe or benchmark. Source locations refer to that exact package.

## Derivative preparation is a reusable structural product


[`HyperDual<T>`](https://docs.rs/numerica/3.0.1/numerica/domains/dual/struct.HyperDual.html)
contains a public values vector and a shared immutable shape. In `dual.rs`,
`from_values` validates nonempty/equal-width shape, value count and immediate-ancestor closure,
then builds a component lookup and multiplication table with a nested component-pair loop.
Invalid shape construction panics. `variable` derives a new value from existing shape and
floating settings; rebuilding a shape for every independent variable is unnecessary.

`create_hyperdual_from_components!` checks ancestor closure at compile time and generates
fixed-layout arithmetic. Single-derivative and depth macros select common shapes. These
mechanisms can reduce runtime structure work when layouts are stable, at the cost of Rust
specialization and inflexibility when requested axes change. There is no demonstrated general
compile-time or runtime superiority. Retaining only a high-order multi-index without its
required ancestors is not a valid shape reduction.

## Adaptive integration and combinatorial output


[`Grid`](https://docs.rs/numerica/3.0.1/numerica/numerical_integration/enum.Grid.html)
separates sampling, `add_training_sample`, compatibility-checked merging and update.
`StatisticsAccumulator` separately accumulates samples and iteration estimates.
`clone_without_samples` permits working grid copies without carrying their observations;
merging still requires compatible grids. `MonteCarloRng::new(seed, stream_id)` and explicit
32-byte state export/import support reproducible streams. Parallel scheduling, update order,
weighted measure semantics and reproducible accumulation remain the caller's responsibility.

`CombinationIterator` and `CombinationWithReplacementIterator` yield borrowed current indices;
`unique_permutations` instead returns exact multiplicity plus an owned vector of permutations.
Choose according to needed output and live-memory shape rather than assuming every combinatorial
API is streaming.


## Interpretation for integrated numerical workflows

A reusable shape can serve many scalar values without reconstructing its multiplication table. This is a native sharing seam before any fixed-type code generation. In an adopter already using evaluator dualization, using Numerica hyperduals is not a new differentiation engine; the relevant improvement is preserving shape and product reuse across preparation. A compiled multi-output integrand can serve grid sampling, but adaptation, sample weights and probability-law identity remain distinct from mathematical expression evaluation. Neither an ordinary quadrature nor a stochastic expectation automatically supplies a derivative of an adaptive estimator. Fixed component types are sensible only for truly stable shapes; otherwise specialization may add build work without removing useful dynamic flexibility.
