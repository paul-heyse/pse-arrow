# Semantic model and scientific authority

**Interface-checked, 2026-11-01, source `19e7b49a67df24c1bf73005a6186e145e930d53a`.**
The [collection baseline and limits](README.md) apply throughout.

Authored relations remain the scientific authority. Quantity admission, typed resolved syntax,
checked specialization and compiler publication form meaningful boundaries: quantity admission
is explicit (`pse-quantity/src/admission.rs`); resolved executable syntax has its own types
(`pse-modeling/src/resolved.rs`); a failed compiler candidate does not replace the previous
published head (`pse-compiler/src/workspace/modeling.rs`, candidate publication).
The findings concern residual interpretations and permission handoffs across those boundaries,
not an absence of scientific checking.

## <a id="da-003"></a>DA-003 — Lookup policy is repeated across checking and selected-source discovery

**DP-01, AP-04, DP-03, DP-04. Revalidated, narrowed.**

`CheckedPackage::resolve` and `resolve_with_members` in `pse-modeling/src/check.rs` implement
import alias tails, lexical children and ancestor lookup. The runtime's `SourceIndex` in
`pse-runtime/src/workflow/modeling/canonical.rs` implements guarded member, base and import
selection independently. Changing a language lookup rule therefore requires coordinated
interpretation across the checking and source-selection owners. The significance is selection
completeness: reuse premises depend on selection including everything checking can resolve.

The two owners share `LexicalLookup` and `selected_source::resolve_segments` traversal.
`CheckedLexical::member` delegates to `declared_member`; it is not a wholly independent third
guard algorithm. Storage-backed discovery deliberately admits a conservative superset before
checking (blueprint §14.4, ADR-0183). That difference is legitimate. No concrete missed member,
behavioral divergence or unsound reuse was established. The finding is repeated policy
ownership, with the superset relationship requiring coordinated maintenance.

Name-encoded `f_<hex>` identities are a related manifestation of
[DA-050](structure-and-invariants.md#da-050), not another lookup finding.

## <a id="da-006"></a>DA-006 — Runtime preparation reconstructs formulation obligations

**AP-01, AP-03, DP-01, DP-02, DP-03, PS-03, PS-07. Revalidated, narrowed.**

`pse-runtime/src/workflow/modeling/cases.rs` constructs numerical targets, materializes
engineering rules and interprets closure attribution and policy while building cases.
Its sequence preparation and `workflow/dynamics.rs` independently compose related preparation
steps. This places formulation interpretation in orchestration alongside the modeling and
compiler owners that already determine its source meaning. A formulation change can require
corresponding changes in those consumers rather than a change confined to the scientific owner.

This is not a claim that every runtime choice belongs in the compiler. Point values,
execution placement and requested numerical modes genuinely arrive later. Nor are fixtures
unchecked: `pse-modeling/src/check.rs` validates primitive options, unique names, uncertainty
and positive allowances. The older blanket fixture-validation claim is withdrawn.
One narrower late refusal remains: `workflow/strategies/declared.rs` rejects explicit presolve
before preparation although the general authored option can be checked earlier.
The breadth of `SpecializedModel` alone is not evidence of a missing contract.

## <a id="da-048"></a>DA-048 — Derived-product permission is composed by individual consumers

**AP-04, DP-01, DP-05, PS-12, PS-10; H19. Revalidated, narrowed.**

`workflow/completion.rs::capture_product_permissions` records combined permissions for fit
covariance, Wald intervals and profile intervals, but returns for non-fit outcomes.
`workflow/run.rs` independently combines step permission with sensitivity and Hessian
production when constructing invocation-product availability. `workflow/result_inspection.rs`
maps fit-statistical relations to recorded product permissions; modeling-derived products
retain `Unknown`. Lazy propagation in `workflow/local_analysis.rs` publishes mathematical
validity without producing the corresponding combined scientific permission fact.

These consumers must interpret the relationship between original step permission and a derived
product separately. A new consumer or new product can preserve one part and omit the other.
The diagnosis is uneven ownership of the combined fact, not missing sensitivity mathematics:
`pse-backend-native/src/kkt/sensitivity.rs` checks qualification, multipliers and complementarity.

Blueprint §19.2 deliberately separates local mathematical validity from original scientific
permission. Diagnostic results may be visible without use permission. An immutable invocation
also correctly reports a lazy product unavailable before its later computation; that is not
itself a false claim. The older suggestion that these facts necessarily promote unusable
science is therefore withdrawn.

Fitted-parameter publication uses `FitReport::estimate_qualified` through
`workflow/fitting/results.rs`, while covariance reuse consumes product permission through
`workflow/uncertainty.rs`. No reachable fit on which these predicates disagree was established.
That particular consequence remains a hypothesis, not an additional confirmed manifestation.

## <a id="da-049"></a>DA-049 — Review selections accept caller-supplied scientific usability

**DP-01, DP-02, DP-03, PS-12, AP-02. Revalidated, narrowed.**

`workflow/result_selection.rs::PackScientificState` says that usability is recorded by the
producer and publication never upgrades it. Public connected result readers nevertheless accept
that state as an argument to `pack_selection`. The shared implementation in
`workflow/connected_results.rs` copies it into the selection. `validate_pack_selection` then
reconstructs the selection using that same supplied state and compares coordinates, schema,
range, field, terminal class and predicate, without checking usability against original
permission. Review-pack publication in `tooling/review_pack.rs` relies on this validator.

Thus this boundary can accept a selection labelled `Usable` without establishing the producer
permission that the label promises. Exact result custody and coordinate validation do not
establish scientific permission. This is an enforcement gap at the exported meaning boundary;
no malicious caller or published incorrect scientific result was reproduced.

Study summaries still project richer assessments into `usable`, an optional singleton
`candidate_use` and seed permission (`workflow/study_operations.rs` and `workflow/study.rs`).
Absent and multiple assessments can both lack a singleton; qualifiers are omitted. Exact
completion remains reachable, and an aggregate boolean may answer a narrower legitimate
question. The summaries remain an investigation area; information loss alone does not establish
an improper promotion by their consumers.

## Earlier identities

These are historical dispositions, not newly repeated verification. Their explanations are
recoverable in the preceding account at source `19e7b49a6` and its Git history.

| Identity | Retained lineage |
|---|---|
| <a id="da-001"></a>DA-001 | Resolved 2026-10-25: typed resolved IR |
| <a id="da-002"></a>DA-002 | Resolved 2026-10-25: derived identity ownership |
| <a id="da-004"></a>DA-004 | Resolved 2026-10-25: generated artifact/owner enums |
| <a id="da-005"></a>DA-005 | Previously resolved: physical view ownership; string-key residue belongs to DA-007 |
| <a id="da-031"></a>DA-031 | Resolved 2026-10-25 for view choice |
| <a id="da-032"></a>DA-032 | Merged into DA-007 |
| <a id="da-038"></a>DA-038 | Previously resolved for covariance permission; uncovered products retain DA-048 |
