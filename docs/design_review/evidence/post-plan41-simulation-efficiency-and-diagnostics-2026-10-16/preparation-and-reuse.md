---
doc_provenance: authored
doc_retention: while-dependent
title: Preparation and reuse inquiry
doc_owner: docs/design_review/reviews/design_review_post-plan41-simulation-efficiency-and-diagnostics_2026-10-16.md
---

# Preparation and reuse evidence

Read-only code-mapper inquiry, reconciled by the coordinator against the baseline in
[the evidence README](README.md). All new remedy claims are **Proposed**; source observations
are **Interface-checked**. No tests or measurements were run in this inquiry.

## Retained and detached composition

`benches/benches/native_process.rs::process` discards the `DeclaredExecution` prepared model
by selecting `.analysis`, then calls `prepare_analysis`. In runtime modeling `declared.rs`,
`declared_execution_with_demand` prepares and retains that model; `prepare_declared` consumes
it through `resolve_prepared_case`. `cases.rs::resolve_case` adds case value/variable paths
to demand before another preparation. `math/preparation.rs::BasisKey` includes that demand.
Different query keys therefore legitimately cause work, while this composition discards an
already available product. The 266.38-second cold public phase includes both preparations
and start; it does not isolate the second specialization's time. The warm 122.77-second phase
likewise does not quantify each proposed optimization's contribution.

Python `prepare_solve`, declared studies and declared dynamics already carry their prepared
model. Python diagnose variants in `pse-py/src/workflow/modeling.rs` detach the model, then
`modeling/diagnostics.rs::prepare_diagnostics` calls `resolve_case`. Diagnostics intentionally
permits missing free values, whereas `prepare_declared` resolves with `allow_missing_free=false`.
Consequently a retained diagnostic composition must preserve that policy; simply replacing
every call with the solve entry point is not sound. Fitting experiment instances/demands and
editable horizon analyses are not automatically interchangeable preparations.

## Scalar checking and lexical environment

`pse-modeling/src/expression.rs::check_member_indices` checks arity, builds
`declaration_environment`, then iterates the indices. With both lists empty it builds an unused
map. The helper calls `named_types`, resolves ancestral scope parameters and coordinate/function
arguments. The specialization-local `LexicalBases` inventory does not cover this helper.
The unindexed `Type::Indexed` export is handled earlier in `path_type` and continues before
this helper; it is a preservation case, not the cause of the unused environment.

A narrow empty-index path can remove work without a package cache. Confirm that the checked
package already establishes the relevant enclosing declaration validity; preserve nonempty
sequential index domains, contextual/refined references and error behavior for invalid input.
These checker controls require the checker and its real semantic dependencies, not a store
or runtime. Captured `named_types`/resolver frames support investigation, not exclusive timing.

## Source member expansion

`workflow/modeling/canonical.rs::SourceIndex::{member,member_candidates,guarded}` allocates
owner/name probing keys, clones direct/base/guard candidate vectors and repeats guarded/base
expansion. `pse-modeling/src/selected_source.rs::resolve_segments` repeats member lookup across
lexical ancestors/candidates/path segments and preserves `Pending`. Index instances are
operation-local in the inspected ingress/local/delta/replay consumers; no long-lived cross-run
cache opportunity is established.

Borrowed-key/nested-map access is a narrow alternative. Memoizing recursive calls is different:
an active probing key returns `Absent`, and nested results can depend on the active recursion
path even when they contain positive candidates. A root-result memo must define both insertion
and consultation context, frontier/completeness invalidation, and the meaning of a recursion cut.
An empty-context-only memo would reuse the same root resolver observation, not certify an exact
graph closure. Do not consult a root result inside arbitrary nested recursion or cache pending
absence. No SCC redesign is selected by the evidence.

The mapper's focused followup found no behavioral counterexample when both storage and
consultation require an empty probing set and the indexed frontier is unchanged. The earlier
warning is therefore narrowed to reuse across different recursive contexts, not a claim that
every completed root-result memo is unsound. Frontier/completeness invalidation remains required.

Preserve direct precedence, candidate order/multiplicity, inherited/guarded sort/dedup,
all potential guard representatives, lexical/import shadowing, protected absence and pending
source acquisition. Compare each current consumer rather than introducing a global cache.

## Work before warm hits

The correct runtime owner is `workflow/modeling/views.rs`, not `math/views.rs`.
`bound_case` constructs `bound_structure` and its key before solver-view lookup.
`pse-compiler/src/workspace/modeling/executable/solve.rs::bound_structure` copies variables,
applies state/domain overlays, filters/clones rows/instances and builds the full structure.
The hit then legitimately rebinds values. Consider a compact structural lookup identity from
immutable base plus result-affecting state overlay, with miss-only expansion, while retaining
required domain admission and bound tightenings. The existing cache is useful; this is work
performed before reaching it.

`math/products.rs::attribute_modeling_view` reserves and rebuilds the occurrence map before
equality comparison. Mathematical-view equality does not establish current-revision spans;
attribution needs its own exact retained occurrence identity. Observation preparation already
removed duplicate construction on a miss; do not repeat that retired diagnosis.

`pse-modeling/src/specialize/value.rs::typed_result` builds type/presence/path context and types
the subtree. Additive nodes evaluate/type children before typing the whole node again. Repeated
subtree work is a concrete source lead, but quadratic exclusive cost or dominance is unmeasured.
Preserve integer overflow/integrality, quantity typing, optional/refined members and fold/reduction
binders in any later correction. Fresh permission/protection on a warm physical cache hit is
not redundant merely because the content is equal.
