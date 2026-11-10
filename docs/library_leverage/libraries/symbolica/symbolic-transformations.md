# Shared expressions and native transformations

Inspected 2026-11-09 at `d9e82627`; **Interface-checked** representative source, **Proposed** research directions. No probe or timing measurement. This is contextual discovery, not adoption selection.

## Already effective

`pse-math::typed_partial` uses `replace_map` to lift shared bindings into unary function calls, delegates differentiation to Symbolica, then uses native replacement to rebind materialized values and partials. `class_evidence` uses a checked visitor followed by `replace_map` for substitutions, and aggregates original contributions before library normalization. `execution::LocalPartials::new` retains each original definition and local first/second partial as named aliases. The numerical builder receives these aliases jointly and uses direct translation. These paths already preserve useful sharing; they are the comparison baseline.

## Promising directions

**Preserve one shared transformation product across compatible consumers.** Native `count_subexpressions` and `alias_subexpressions` identify repeated nontrivial expression structures before evaluator preparation; native visitors avoid a second owned tree. They could expose reuse across currently separate construction requests. The important question is whether a named pair of consumers has equal qualified definitions, axes, domain guards and derivative demand. Aliasing itself does not make every consumer share numerical work, and broader retained products may cost more than rebuilding a small expression.

**Prepare a reusable transformation rather than repeated walks.** `Transformer`, conditional/multiple replacement, `ReplaceBuilder` traversal restrictions, and collection callbacks can express a whole mathematical pass. The current native callback uses are evidence that replacement machinery is already delegated, not a reason to replace each callback with a rule object. Investigate where multiple stages rebuild the same match/replacement preparation or repeatedly expand/substitute the same definitions. Native rule preparation has value when reused; one-off or source-attributed guarded operations may legitimately remain thin callbacks.

**Move from structural inspection into the strongest applicable algebraic domain.** `collect_multiple_mapped`, coefficient rings, and polynomial conversion can combine coefficient extraction and arithmetic instead of repeatedly scanning general atoms. Conversion must preserve the parameter context and must not hide nonlinear relationships by treating subexpressions as independent indeterminates. Current request-local exact objective reuse is already present; a new hypothesis needs a named additional consumer.

## Function definitions and hooks

Symbol attributes and derivative/normalization/series/evaluation hooks can encode genuine mathematical properties once. They do not automatically encode invocation-specific provider failures, validity, or instance-dependent physical context. Current plain formal-pool registration deliberately checks exportable identity and incompatible callbacks. Investigate a pure reusable mathematical head separately from a fallible host provider. A different target design may revise this boundary, but must explain its new semantic owner.

## Next discriminator

Trace one repeated shared-definition product from authored preparation through symbolic derivatives and evaluator construction. Identify actual duplicate construction rather than merely several evaluator instances, then inspect whether a reusable native transformer or aliased product removes that construction without broadening derivative demand. Use the current source/domain/guard consumer as contrary evidence. No benchmark is required to establish that a redundant construction disappears.

Sources: `vendor/symbolica/src/atom/core.rs` (`count_subexpressions`, `alias_subexpressions`, `collect_multiple_mapped`, `replace_map`, `replace_map_bottom_up`); `crates/pse-math/src/{typed_partial.rs,class_evidence.rs,execution.rs,library.rs}`. Broad pattern and Transformer candidates are overview/API-supported leads; exact matched pass remains unselected.
