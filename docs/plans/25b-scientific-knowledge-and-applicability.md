---
title: "25b: Scientific knowledge and applicability"
status: in-progress
date: 2026-09-30
adrs: [ADR-0140, ADR-0141]
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s01]
---

# 25b: Scientific knowledge and applicability

## Context and target

This plan addresses F11/F12 and FU01–FU03, incorporating R2. Checked chemistry currently does
not govern every executed reaction source; unknown composition can pass conservation vacuously;
pair selection and phase-key conventions obstruct ordinary scientific composition.

The target remains authored scientific knowledge over generic kernel mechanisms. Catalog entities
may be incomplete, but each admitted operation requires the evidence its claim needs. An unknown
formula is different from a complete empty composition. A reaction owns its coefficients and
extent convention; kinetics supplies rates on that convention. A parameterization owns scientific
identity and coherence; provenance records its evidence.

Plan 24 supplied useful boundary distinctions: known-empty compositions, extent conventions,
independent versus jointly fitted/conditional collections, explicit pair orientation and unknown
validity. Its database, readers, expression evaluator and production snapshot bridge are outside
this plan. Available source declarations and correspondence notes were inspected; committed final
kernel-gap/schema-delta reports were not available at authoring time.

## Decisions and interfaces

- **Composition evidence:** declare completeness for the composition used in a conservation
  claim. Sparse zeros are valid inside a complete composition. Empty complete composition is
  legal where scientifically appropriate; unknown composition cannot establish conservation.
  Molar-mass or atomic-weight availability is not the completeness test.
- **Reaction projection:** one checked reaction/material binding derives coefficients from
  the authoritative reaction. It verifies every nonzero participant is represented, allows inert
  extra components, and records the extent convention. Rates and reaction heat bind to that
  convention. No unrestricted callback can redefine coefficients.
- **Translations:** apparent/true-species or lumped mappings are explicit transformations with
  conservation checks for the claimed quantities. They cannot silently omit or rename a participant.
  This does not introduce a universal isotope/site ontology.
- **Parameter identity:** a parameter set identifies family, subject tuple and variant within
  a parameterization. Phase scope is attached only when scientifically relevant; species-only
  parameters use no vapor sentinel. Provenance is attached evidence, not a variant key.
- **Pair selection:** choose parameter sets for each required ordered or symmetric pair.
  Independent records may mix. Jointly fitted collections declare inseparable groups, dependencies
  and allowed subsystem projections. Require their closure for the selected model, subjects and
  applicability domain, not every unrelated record in a bank. Conditional sets declare the
  dependencies selection must satisfy. Scientific interchangeability
  is declared, not inferred from identical coefficient dimensions.
- **Applicability:** represent a known region, an explicit unrestricted claim, or unknown evidence.
  Regions may combine relevant state/composition/phase conditions where the data supports them.
  One evaluation operation returns applicable, outside-region or unknown-evidence observations.
  Mathematical-domain failure always refuses. Default data-use policy requires established
  applicability; an explicit recorded permission may allow unknown evidence or extrapolation,
  separately, without relabelling it validated. Permissions name selected records or declared
  families; both permissions default false. Universal constants need no invented regression
  interval. 25e consumes the observations and selected policy in result qualification.
- **Ownership:** packages own chemistry, forms, data and selection. Extend generic relational
  or expression mechanisms only when the chosen declarations require it; add no scientific
  branch on a species, phase or method name in Rust.

Merely adding a pair index to source selection would still conflate variants and coherent
collections. Requiring formulas everywhere or fabricating unbounded envelopes would destroy
legitimate knowledge. The selected target makes completeness local to the operation that needs it.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="b1"></a>B1 Composition completeness | Existing authored relations | Distinguish complete, complete-empty and unknown composition; enforce conserved claims | in-progress |
| <a id="b2"></a>B2 Reaction/material projection | B1; A2/A4 | Derive reaction sources and bind kinetics/heat to one extent convention | in-progress |
| <a id="b3"></a>B3 Parameterization and pair selection | A1/A2 for affected physical declarations | Separate phase scope, parameter identity, provenance and coherence; migrate selections/data | in-progress |
| <a id="b4"></a>B4 Applicability and data-use policy | B3; A3 for mapped family contracts | Compose evidence, consuming-model domains and explicit permission; export observations to E3 | in-progress |

### B1 — Complete composition and conserved claims

**Implementation vision.** The target composition lookup returns either a complete coefficient vector with its claim/source,
or IncompleteComposition for the requested conserved quantities. Completeness belongs to the
composition meaning, not to the existence of a nonzero table cell. Missing cells become zero
only under a complete claim; a complete empty vector is representable. Charge evidence remains
explicit. Molar mass is a separate derivation requiring atomic-weight data. Reaction checking
first obtains complete participant vectors and only then sums stoichiometric conservation;
unknown catalog entities remain valid where no such claim is requested.

Require known participating composition when checking elemental reaction conservation and
apparent-species translation. Check charge and elements independently; absence of evidence is
a typed unresolved/refused claim, not an established imbalance or a zero vector. Preserve unknown
entities in catalog/model roles that do not claim their elemental conservation.

Focused acceptance: unknown neutral species do not satisfy conservation vacuously; complete-empty
species remain expressible; sparse complete formulas work; charge-balanced but element-unbalanced
dissociation refuses. Delete tests that use formula absence as evidence of zero content.

### B2 — One reaction source

**Implementation vision.** The checked binding is an immutable derived product: reaction identities, material component
identities, extent conventions, projected coefficient matrix and evidence references. It is not
another editable coefficient table. Admission reads every nonzero participant, checks composition
and material support, then derives the matrix. CSTR/PFR multiply that matrix by the admitted
rate vector. For 2H2 + O2 → 2H2O, a volume-based extent rate multiplied by volume gives extent/time;
coefficients (-2,-1,+2) then give component source rates. Reaction heat uses the same extent.
Omitting water refuses; adding inert nitrogen gives zero source. Explicit apparent/lumped
translation adds its own mapping and conservation obligations before producing the same product.

Replace the RateLaw coefficient callback and its forwarding bindings with the checked projection.
Move CSTR/PFR and their seed bindings in the same packet. The material support check covers the
reaction's actual participants rather than equality of two independently supplied subsets.
Translate a genuinely different reaction basis only through its declared conserved mapping.

Focused acceptance: changing authoritative coefficients changes both reactors through one
declaration; missing products refuse before solving; inert additions contribute zero; mismatched
rate/extent/heat bases refuse. Delete callback parameters, parallel source tables and obsolete
callback tests as their callers move. Preserve the independent chemical conservation checks.

### B3 — Parameterization and selection migration

**Implementation vision.** Selection requests name the consuming model's required family/subjects and chosen parameter
records. A record's identity distinguishes parameterization, family, subject tuple and variant;
NRTL (i,j) remains directional while a symmetric family's pair ordering is canonicalized by its
own rule. Admission closes explicit dependencies and atomic-fit groups for the selected scope,
then returns an admitted selection containing record identities, dependency closure, conventions
and applicability references. Coefficients stay in their owned records. This lets independent
A–B from source X and A–C/B–C from Y compose without copying values, while an inseparable fitted
block cannot be split accidentally. Phase is present only when the family makes it relevant. Selection also distinguishes a stored/
fitted zero, an absent required pair and a zero supplied by an explicitly selected predictive
rule. The last carries its rule identity and inputs, never an invented fitted record. Missing
required pairs refuse. Test all three outcomes and their provenance.

Migrate phase-independent pure data and pure/pair selectors together. The existing PC-SAFT
family already declares liquid and vapor applicability; remove its vapor-key sentinel without
claiming that the old family was scientifically vapor-only. Preserve explicit orientation for
asymmetric methods, and symmetric transposition only where declared.

Focused acceptance: a ternary selects existing independent pairs from two sources without copied
coefficients; two fits from one publication remain distinct; directional reversal cannot silently
reuse the wrong fit; incomplete or incompatible jointly fitted groups refuse. Delete composite-bank
copying, phase sentinels and replaced package-wide source selectors. Do not copy Plan 24's full
assembly machinery into production.

An explicitly allowed subsystem projection succeeds; a missing required joint-fit dependency
refuses. Scientific coherence does not mean loading an entire publication.

### B4 — Applicability composition

**Implementation vision.** Each form/set carries an evidence claim with scope and either a region/predicate, explicit
unrestricted applicability, or unknown evidence. Preserve whether the source calls a region
fitted, recommended or validated; being inside a fit range is not experimental validation.
State evaluation follows the admitted selection/dependencies and emits observations naming the
responsible form/set and outcome: inside, outside, unknown or mathematical-domain failure.
Alternative regions form a declared union; dependencies compose their required conditions.
B4 applies the low-level data-use gate; 25e separately qualifies results using the unchanged
observations and selected permissions. Allowed extrapolation remains visibly outside
its source range; permission neither edits the evidence nor removes its qualification from results.

Migrate all currently represented parameter/form families, not just temperature correlations.
Record existing evidence honestly; missing source ranges remain unknown. A data-use permission
is part of the admitted analysis/binding and lineage, never a mutation of the source claim.
Compose form, parameter and consuming-model applicability while retaining the responsible layer.

Focused acceptance covers known limits, unknown evidence, unrestricted constants, permitted
extrapolation and mathematical-domain excursions that remain forbidden under every policy.
Delete optional-envelope shortcuts that represent absence as established unrestricted use.
Migrate the authored permissions of seed analyses consuming unknown records: default refusal,
explicit unknown-evidence permission and extrapolation permission are different controls. Do not
grant every old case blanket permission to preserve old outcomes. Broader external-library
acquisition or qualification remains outside this series.

## Authority and handoff

Amend blueprint §9.1/§9.7/§9.9 and the relevant ADR-0127 rationale through the design route.
Use a short ADR for any new generic contract; ordinary package changes stay package changes.
The plans propose the required authority changes and do not reopen Plan 23's historical Outcome.

25c consumes conserved projections and transport context; 25e consumes applicability observations;
25f carries selected policy/typed bindings; 25g handles persisted representation evolution;
25j generates the public shape. Each consumes this single scientific meaning.

## Execution and evidence

Implementation is authorized by the maintainer on 2026-09-30. Unimplemented contracts remain
**Proposed**; no new product qualification is claimed. The [series coordinator](25-design-remediation.md)
owns finding dispositions and decision dependencies. Packets compile affected owners, run focused
behavioral checks with explicit force-validation, regenerate changed declarations, and immediately
delete replaced code, callers, obsolete tests and fixtures. No shims or parallel production paths remain.
Full integration, formatting, lint and performance qualification run once in
[25k](25k-integrated-qualification-and-closure.md), after the series' functional scope is complete.

Use current recipe-owned checks such as `just check-package <pkg>` and
`just unit-package <pkg> <filter>`; select isolated tests rather than broad suites hidden under
a unit label. The acceptance scenarios above define what those tests must establish, not claims
that tests with particular names already exist. Cross-owner scientific/storage journeys are authored
with the functional work and executed in 25k. Record state, decisions and next steps during work;
record actual commands, conditions and failures against zero in the final qualification evidence.

### Execution checkpoint

Work starts from the current checkout at `ef5bd2ec`, including the completed 25a dirty baseline.
Concurrent Plan 24 and governance changes are preserved. ADR-0140/0141 precede the new
contracts; their scoped target review accepted the proposal and their status remains proposed.

The selected approach uses authored complete/unknown composition and guarded sparse lookups,
a concrete nonoverrideable reaction/material projection, and record-mediated kinetic/heat
methods bound to explicit extent conventions. Parameterization/family/subjects/variant are
distinct from provenance. Existing separate keyed shapes avoid optional/tuple-key extensions;
joint-fit membership is separate from derivation lineage.

Applicability observations remain per claim. Required dependencies can be both outside and
unknown; their independent named permissions are both required. Declared alternatives use a
known applicable region when available, otherwise unknown evidence remains unknown. Mathematical
and hard model-domain requirements always refuse. Whole-interval coverage is explicit; arbitrary
predicate regions do not inherit an endpoint-only shortcut.

The [selection implementation review](../design_review/reviews/design_review_plan25b-selection-implementation_2026-10-01.md) found three material gaps. IR25B-01/02 are being corrected with consumed-owner/table retention and function/instance-scoped selection framing after consumption. IR25B-03 is corrected in source by deriving record backlinks from the sole authored group membership; focused execution remains pending. Authored selection controls are also exposing source syntax/layout defects during integration. The [applicability implementation review](../design_review/reviews/design_review_plan25b-applicability-implementation_2026-10-01.md) additionally found IR25B4-01–04: incomplete winning-union obligations, branch-hoisted implicit claims, ungated direct record reads and lost observation attribution. Corrections and regression controls are in progress; neither review establishes functional acceptance. The next steps are admission controls, actual numerical consumers, applicability refusal/export controls and review of the stable corrections.

Chemistry/reaction and parameter-selection implementation have disjoint worktree ownership.
The root integrates changes, owns shared declarations/generation and runs serialized compile
and focused functional checks. Generic applicability follows the selected identities. Immediate
deletion accompanies each replaced consumer; comprehensive qualification remains Plan 25k.

## Outcome (recorded after implementation)

### What was built

Not implemented; record actual behavior and evidence labels at closure.

### A mistake made and corrected

Record an actual implementation correction, not a hypothetical planning example.

### Deviations from the plan, deliberate

None recorded. A changed architectural decision follows its owning ADR/design route.
