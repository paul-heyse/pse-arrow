---
title: "25b: Scientific knowledge and applicability"
status: done
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
| <a id="b1"></a>B1 Composition completeness | Existing authored relations | Distinguish complete, complete-empty and unknown composition; enforce conserved claims | done |
| <a id="b2"></a>B2 Reaction/material projection | B1; A2/A4 | Derive reaction sources and bind kinetics/heat to one extent convention | done |
| <a id="b3"></a>B3 Parameterization and pair selection | A1/A2 for affected physical declarations | Separate phase scope, parameter identity, provenance and coherence; migrate selections/data | done |
| <a id="b4"></a>B4 Applicability and data-use policy | B3; A3 for mapped family contracts | Compose evidence, consuming-model domains and explicit permission; export observations to E3 | done |

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

Completed 2026-10-01. Work began at `ef5bd2ec` with the completed 25a dirty baseline and
integrated into the shared checkout, now at `be469cc9`; concurrent governance changes and the
Plan 24 repository move are preserved. B1–B4 are complete. The Outcome below owns the
implementation, composite focused evidence and handoff; the series coordinator owns finding
dispositions. ADR-0140/0141 preceded implementation and remain proposed pending their decision PRs.

The [selection implementation review](../design_review/reviews/design_review_plan25b-selection-implementation_2026-10-01.md)
identified consumed-context and late-identity gaps (IR25B-01/02) and duplicate fit membership
authority (IR25B-03). Consumed function/instance-scoped closures and derived backlinks repair
those gaps. The [applicability implementation review](../design_review/reviews/design_review_plan25b-applicability-implementation_2026-10-01.md)
identified winning-union obligations, branch-hoisted claims, ungated direct reads and lost
attribution (IR25B4-01–04). Demanded source gates and complete typed observations repair them.
Subsequent numeric-default, Set-origin and local-alias gaps are also corrected. Focused
controls exercise the repaired obligations; neither historical review alone establishes acceptance.

No functional work remains in this child packet. Later packets consume the admitted conserved
products, immutable selections and full observations described below. Full integration and
qualification remain 25k; F12 retains its E3 result-qualification obligation.

## Outcome (recorded after implementation)

### What was built

**Implemented, 2026-10-01:** B1 distinguishes complete, complete-empty and unknown
composition, with independent charge evidence and guarded sparse lookup. Conservation and
explicit apparent/lumped translations require the evidence they claim; atomic weights remain
separate from completeness. B2 derives one immutable reaction/material projection from the
reaction's coefficients, checks all nonzero participants, allows inert extras and binds kinetic
and heat records to the same explicit extent convention. Both authored reactors consume it.

**Implemented:** B3 separates parameterization, family, subject tuple, variant and provenance;
phase is present only where relevant. Ordered and symmetric selections use existing records or
an explicit predictive rule. Required dependency/fit-group closure, declared subsystem
projections and convention identity survive consumption. Joint-fit membership has one authored
owner and derived record backlinks. Ternary independent sources compose without copied
coefficients, and fitted zero, missing pair and predictive zero remain distinct.

**Implemented:** B4 retains fitted/recommended/validated/reported evidence, unknown and
unrestricted claims, declared alternatives and required dependencies. Default use is strict;
exact-record or declared-family Unknown and extrapolation permissions remain independent.
Checked nominal ancestry supports family targeting without widening authored targets. The actual
read carries its gate through branches, cancellation, partials, parameter defaults, constructor
arguments and evidence-bearing aliases. Parameters remain case-value inputs; generated
prerequisite indices transfer effects without recomputing their numerical authority. Set-origin
receipts follow actual membership/reduction reads and local aliases without importing an
unused instance's selections. Hard mathematical and model domains remain unwaivable.

**Implemented:** full typed assessments survive refusals and generated
`runtime.modeling_checks` version 5 observations. They retain actual owner/ancestry, consuming
instance, required/alternative status, reasons, typed inputs, records/dependencies and complete
matched permission identity, scope, targets and flags. Authored declarations are version 22.
Changed preparation preimages use parameter-read v1, finite-function v9, dispatch-body v6,
typed-definition v9 and observation v2 frames. Twenty-three concrete seed/campaign scopes name
only the two water constant fits whose standalone parameter reads lack an applicable form
signature; extrapolation stays disabled and generic liquid models stay strict.

Deleted the RateLaw coefficient callback and forwarding source tables, source-wide pair-bank
selectors and vapor-key sentinel, editable fit backlinks, blanket extrapolation declarations,
hard-range permission fields and the static optional-envelope observer path with its obsolete
fixtures/tests. Rust, generated transports and Python decoders consume the replacement meaning.

### Verification

**Tested, 2026-10-01:** 100 distinct focused Rust controls have successful receipts against a
**zero-failure baseline**. The recipe for each table row is
`just unit-package <package> '<filter>' --test-threads 1`, using the pinned toolchain,
optimized test profile and explicit `--features pse-relations/force-validate`.

| Package | Exact filter | Result |
|---|---|---|
| pse-modeling | `test(scientific_composition_tests)` | 5 passed, 0 failed |
| pse-modeling | `test(scientific_selection_tests)` | 12 passed, 0 failed |
| pse-modeling | `test(static_reduction_and_fold_resolve_lexical_entity_member_domains) \| test(domain_schema)`; then `test(domain_schema_refusals_name_the_violated_constraint)` | composite: 5 distinct passed, 0 unresolved failures |
| pse-model | `test(applicability_tests)` | 13 passed, 0 failed |
| pse-math | `test(applicability_tests)` | 7 passed, 0 failed |
| pse-runtime | `test(applicability_tests)` | 4 passed, 0 failed |
| pse-authoring | `test(annotation_kinds_parse_typed_members) \| test(cell_spellings_parse_to_one_value) \| test(cell_render_parse_roundtrip)` | 3 passed, 0 failed |
| pse-compiler | source selections below | composite: 51 distinct passed, 0 unresolved failures |

Compiler source verification used these actual final selections:

```sh
just unit-package pse-compiler 'test(applicability_tests) | test(selection_identity_tests) | test(scientific_parameter_tests) | test(physical_potential_tests) | test(scientific_reaction_tests) | test(authored_material_projection_preserves_complete_component_support) | test(authored_cstr_extent_report_retains_quantity_and_unit_contract) | test(authored_reaction_extent_and_stoichiometric_component_rates_are_physical) | test(kernel_conservation_scatter_preserves_mixed_contracts_and_homogeneous_control) | test(increment_guards_its_integration_interval)' --test-threads 1
just unit-package pse-compiler 'test(applicability_tests) | test(authored_cstr_extent_report_retains_quantity_and_unit_contract) | test(scientific_composition_elemental_control_volume) | test(scientific_reaction_authoritative_coefficients) | test(scientific_reaction_inert_extra_material) | test(scientific_reaction_missing_products) | test(authored_material_projections_keep_component_and_element_contracts)' --test-threads 1
just unit-package pse-compiler 'test(scientific_composition_elemental_control_volume) | test(scientific_reaction_authoritative_coefficients) | test(scientific_reaction_inert_extra_material) | test(scientific_seed_constant_parameter_reads)' --test-threads 1
```

The 49-control run had 44 passed and 5 failed: source tracking incorrectly parsed a literal
Set default as a numerical expression. After correction, the 27-control rerun had 24 passed
and 3 data-use refusals, which required the exact seed record permissions described above.
The new permission control initially declared component density as total density, making all
four selected tests fail admission; correcting that fixture's nominal type gave 4 passed and
0 failed. The first command's material filter matched no test; the second command explicitly
exercised the correctly named material projection control. These are **composite receipts**,
not an initially clean run. Earlier domain/reaction integration failures were repaired and
rerun; none are treated as a permitted baseline.

The controls exercise guarded composition, source vectors in both reactors, missing products,
inert zero sources, extent/heat mismatches, elemental flow, actual ternary NRTL and
symmetric/directional values, fit closure, convention-only identity, branch/alias/source demand,
union obligations, both permissions, interval coverage, nominal family matching, full refusal
and observation transport, and mixed-contract conservation scatter. The prerequisite math
control returns input 7 while observing source input 999, preserves its physical type and
first/second derivatives, and refuses missing permission even after numerical cancellation.
They execute authored scientific expressions with bounded source-selected inputs, not solves
or storage journeys. `just nrtl-reference` generated the ordered-pair source input without
fabricated diagonal rows; it is input-generation evidence.

**Tested, 2026-10-01:** `just py-unit-native python/pse/tests/test_generated_contracts.py -k applicability_observation`
passed 1 isolated unit control, with 15 deselected and 0 failures against zero, on Linux,
Python 3.14.7 and the refreshed editable dev native-solvers extension. It round-trips the
generated version 5 observation, checked owner ancestry and complete scoped permission and
rejects malformed ancestry identity. The initial unlinked `just py-unit` invocation failed
import because the MKL library was unavailable in that recipe's environment; the first
native invocation then failed import because the extension held the old registry. Neither
executed a test. `just py-sync-native` refreshed the extension and actual compiled API stubs;
the native unit rerun above passed. No solver was invoked by this codec control.

**Interface-checked, 2026-10-01:** `just codegen` completed all six schema targets and found
no hakari changes; `just check` compiled the workspace's all-target dev selection with
0 errors against zero. Cargo reported an upstream future-incompatibility notice for
`proc-macro-error2 v2.0.1`; it is not a functional test result. Full integration/native solves,
linked Python journeys, parity, feature powersets and performance campaigns belong to 25k.
Static hygiene (`just hygiene`) belongs to 25k's scope-end qualification (ADR-0143); no new product
qualification or measurement is claimed here.

### A mistake made and corrected

A numeric default retained its value while erasing the scientific read that supplied it.
The first replay correction then reused an owner's aggregate selections and could import an
unused context's dependency. Exact lexical source replay, demanded Set/alias receipts and an
explicit effect-only prerequisite correct both mistakes. Runtime inactive-branch, same-record
unused-context and constructor/alias controls exercise the correction. Literal Set defaults
remain structural values, and the density fixture now preserves its component contract.

### Deviations from the plan, deliberate

**Implemented:** the authored target required small generic structural tuple/set, optional
reference and keyed-reference-set mechanisms, transitive typed derived-attribute ordering and
numerical prerequisites. They carry the existing owned records and effects rather than adding
scientific-name dispatch or another coefficient authority. No quantity-valued authored function
static evaluator or thermodynamic database/snapshot bridge was added.

Standalone parameter coefficients and source range bounds may lack a consuming form signature
even when their record reports a region. Their observations remain Unknown; individually named
seed permissions permit those reads without relabelling the region or waiving hard domains.
The generic mixed-contract conservation lowering falls back to a complete typed closure when
terms cannot be safely scattered under one ledger contract; independent contribution outputs
remain available. Its homogeneous control retains the scatter path.

ADR-0140/0141 remain proposed pending the decision-PR route. Enduring meaning lives in blueprint
§6.15.2, §9.1/§9.3/§9.7/§9.9/§9.10, §14.3 and §19.3/§19.5, amended explicitly under
`PSE_DESIGN_EDIT=1` and revision 88. The completed child record stays linked while later series
packets consume its handoff; retirement follows series closure. 25c consumes the conserved
products, 25e/25f consume observations and declared policy, 25g owns persisted evolution and
25j owns the remaining public inventory. The coordinator retains F12's E3 qualification work.
