# Symbolica ecosystem discovery coverage and evidence

Inspection date 2026-11-09; source `d9e82627feec6cea5470dbcf1974fd4e8dfe19df`.
All three supplied overviews were read end to end during preparation. The execution then
checked selected consequential contracts against locked sources, representative consumers,
tagged candidate source and generated API evidence. Breadth below is a family-level account,
not a claim of equal implementation-level depth or semantic completeness of API extraction.

## Reference coverage

Section numbers below name the supplied references, not architecture section IDs.

| Symbolica overview territory | Disposition and destination |
| --- | --- |
| Original §§1–2: representations, symbols, normalization, attributes/hooks | Current strong leverage plus reuse/extension hypotheses in [transformations](symbolic-transformations.md). Native registration does not replace physical/provider semantics. |
| §3: matching, conditions, replacement and Transformer | Native callback replacement already used; reusable ordered transformation programs remain promising where preparation repeats. See transformations; termination/domain/growth remain case-specific. |
| §4: polynomials, domains, finite fields, Gröbner, rationals, algebraic extensions, coefficient rings | Stronger exact representations and elimination hypotheses in [exact algebra](exact-algebra-and-solving.md); do not confuse cancellation with preservation of excluded inputs. |
| §5: exact solutions, parameters, coverage and roots | Existing bounded roots recognized; solution branches, generic coverage and root identity remain distinct. See exact algebra. |
| §6: differentiation, symbolic integration, series | Shared derivative work in transformations/evaluators; analytical limits, integrals and remainder-aware series in exact algebra and [ecosystem](ecosystem.md). |
| §7: evaluator optimization, multiple outputs, functions, exported instructions, JIT and native targets | Current joint construction/direct translation recognized; composition/lifecycle/backend inquiries in [evaluators](evaluator-composition-and-execution.md). |
| §8: AD, precision, numerical integration | Existing dynamic dualization plus shape-reuse and new expectation hypotheses in [Numerica](../numerica/briefing.md). |
| §9: storage, aliases, streaming, parallelism, serialization | Conditional whole-operation hypotheses in [build/memory/transfer](build-memory-and-transfer.md); global normalization and state custody constrain composition. |
| §§10–11: Numerica, Graphica, matrices/tensors, Spenso, community, Typst, ODE | Dedicated companion accounts and ecosystem screening. Specialized physics wrappers have no established process-engineering fit here; retained as leads, not blanket exclusions. |
| §12: Rust/bindings/WASM/features/presentation | Native-first build and presentation hypotheses; binding exposure does not establish equivalent native APIs. See build/transfer and ecosystem. |
| §13: licensing | Not a rejection criterion. Actual package/features/runtime capabilities are established independently of licence comparisons. |
| §§14–16 and conclusion: interactions, limitations, sources | Cross-topic workflows integrated into [entry briefing](briefing.md); primary source map used for selected uncertainties. |
| Addendum §1: predicates, zero tests, numerical solving | Three-valued/identity/pointwise and approximate-root distinctions in exact algebra. Numerical Newton is a candidate for a named operation, not a native-solver replacement by name. |
| Addendum §2: typed views, visitors, callback mapping, ReplaceBuilder, Flat | Transformation hypotheses; pure mathematical attributes distinguished from invocation-specific effects. |
| Addendum §3: layouts/orders, EEA/division, resultants, reduction, interpolation/Padé, CRT/Hensel, sparse GCD, roots, contexts | Exact preprocessing/tooling hypotheses; algebraic preconditions and coefficient growth stated. Context ownership in build/memory. Low-level algorithm implementation contracts are not all independently revalidated. |
| Addendum §4: merge/export/vectorize, functions, optimizer, stack, real specialization, ring mapping | Lifecycle constraints checked in selected source; newer composer/import treated as separate candidates. See evaluator account and technical contract. |
| Addendum §5: rational reconstruction/rationalization, matrices, tensor indices, aliases, special functions | Exact/numerical distinction retained. Rationalization changes the model's coefficients; no scientific literal rewriting is proposed. Tensor versus scalar reuse in ecosystem. |
| Addendum §§6–7/conclusion: compositions and corrections | Composition ordering incorporated; overview availability claims are not assumed valid for every locked feature/backend. |

Numerica's [complete grouped section coverage](../numerica/briefing.md#capability-coverage)
accounts for §§1–17, including polynomial kernels, adaptive-width integers, reconstruction,
PSLQ/lattices, modular representations, precision, AD, dense/sparse matrices, adaptive grids,
combinatorics, features and persistence. Graphica's
[complete grouped coverage](../graphica/briefing.md#complete-capability-disposition)
accounts for §§1–14 and its conclusion, including labeling, canonical maps, orbits, generation,
partial filters, abort/progress, weak connectivity, mutation, export and feature limitations.

## Baseline and source interpretation

The dependency owners select maintained path-overridden Symbolica 3.0.1, Numerica 3.0.1,
Graphica 3.0.1 and optional native code generation (locked SymJIT 2.26.0). Current source reads
cover `pse-math`, selected compiler/runtime consumers and structural flowsheet/incidence owners.
Spelling searches across non-generated crate Rust found direct Symbolica integration concentrated
in `pse-math`; this is representative navigation, not a resolved absence claim for other facades.

Core contracts inspected include AtomCore mapping/aliases/solving, evaluator optimization,
vectorization/function maps/instructions, bounded roots, collocation and local partial construction.
Numerica checks resolve overview gaps about balls, CSR validation, underdetermined solving and
shared derivative shape. Graphica checks resolve canonicalization/generation, partial pruning,
result retention and cancellation semantics. Upstream 3.0.2 composer/import sources are linked
in the technical account. Companion source gaps remain explicit in ecosystem screening.

Current Plan66 implementation and acceptance are linked from the entry briefing, not maintained
here. In particular the original review's supplier-provenance repair suggestion is historical;
the current owner records subsequent reproduction of the tracked supplier.

## Generated evidence and limitations

Retained evidence directory:
`~/.local/share/pse-arrow/library-evidence/2026-11-09-symbolica-discovery/`.
`retained-api-navigation.json` records the original November2 Symbolica manifest, module-level
path distribution and selected spelling leads. The original export has 14,694 declarations,
23,213 public paths and 36 traits under integer-gmp/float-mpfr on Linux. This is historical API
navigation, not a refreshed reference census. Facade aliases and blanket trait entries multiply
paths; individual item IDs are artifact-local.

Fresh `pse api numerica --prepare` and `pse api graphica --prepare` succeeded, followed by
`pse inspect api PACKAGE --export-jsonl NEW_DIR` into `numerica-api/` and `graphica-api/`.
Both original acquisition/export responses are retained beside their manifests. Numerica's
context selects integer-gmp/float-mpfr; Graphica's acquired documentation context lists no direct
features. Both target x86_64-unknown-linux-gnu using rustdoc JSON format61.

| Fresh defining-package export | Declarations | Public paths | Traits | Implementations |
| --- | ---: | ---: | ---: | ---: |
| Numerica 3.0.1 | 7,916 | 6,599 | 28 | 4,177 |
| Graphica 3.0.1 | 665 | 534 | 2 | 493 |

All twelve companion table digests and row counts were checked with a stdlib Python read.
These exports confirm public paths for native Graph canonization/generation, CSR validation,
HyperDual and RealBall; the latter is publicly re-exported directly under `domains::float`.
Blanket methods visible on RealBall illustrate why path exposure alone cannot certify numerical
operations or prove use-site trait satisfaction. Source contracts, not path counts, settle the
non-certifying transcendental limitation.

`pse code session status`, live preparation and frozen-project native-snapshot preparation all
encountered `host.ownership: recorded original manager incarnation changed`. No fresh resolved
reference census was obtained. The original November2 census at frozen `ad69066e7` remains
historical (1,629 Symbolica observations); none of its counts is relabeled current. The frozen
retry used a Git archive of this inspection revision, excluding unrelated dirty Plan72 changes.
The failed attempts establish an evidence gap, not an unused library or failed scientific result.

An initial API request with `--budget 1` was refused because limited views require a checkout-owned
evidence directory; rerunning the full view succeeded. `just ready` passed skills-sync and doctor
after host preparation. Generated evidence does not select adoption and does not replace source
inspection or independent scientific verification. Product tests and benchmarks were not run:
this engagement changed research documents only, and the selected contract questions were settled
by source rather than requiring new execution experiments.

## Fresh Symbolica bundle

`just library-evidence --family symbolica --acquire --output NEW_DIR` subsequently produced
`symbolica-bundle/` in the retained directory. It returned exit1: **one failed step against
the zero target**, the resolved dependency-usage capture (`host.ownership`). Package inspection,
API acquisition and API export succeeded independently. The manifest retains
`capture_complete: false`; this is not a complete evidence census.

The fresh maintained-3.0.1 API export contains 15,318 declarations, 23,687 public paths,
36 traits and 9,538 implementations, with the same selected GMP/MPFR features and Linux
target. Its six table digests and row counts also passed verification.
`fresh-symbolica-api-navigation.json` retains selected public-path leads and the full manifest.
Packing inspection and numerical restore paths are exposed; the selected composer/import
spellings are not in this export and remain tagged newer-source research. That spelling
observation is not a general capability-absence proof.

The difference from November2 includes maintained supplier evolution; larger item/path counts
are not a benefit measure. No cross-artifact numeric item IDs were joined. API exposure still
includes documentation-context and blanket-implementation conditions rather than a proved
ordinary-build or scientific execution contract.

## Publication verification

**Tested:** `just docs` in the documentation environment published 1,207 chapters and scoped
search with zero failures against a zero target. The check covers the contemporaneous checkout,
including parallel documentation. Source-only final consistency edits received a scoped local-link
and whitespace check; no product qualification is claimed. The fresh-reader challenge found no
blocking omission or overclaim and corrected API-versus-census wording and ecosystem evidence scope.
The earlier `just turn-end` finished successfully before the maintainer's request to omit
repository-level health checks; no further such checks were run.
