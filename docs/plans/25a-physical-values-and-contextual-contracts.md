---
title: "25a: Physical values and contextual contracts"
status: done
date: 2026-09-30
adrs: [ADR-0135, ADR-0136]
review_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md, docs/design_review/reviews/design_review_codebase-domain-alignment-follow-up_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_codebase-domain-alignment_2026-09-30.md#s09]
---

# 25a: Physical values and contextual contracts

## Context and target

F01/F03 identify physical meaning lost inside otherwise typed laws; FU10 identifies unchecked
overflow during canonical conversion; F34 identifies competing metadata/compatibility rules.
This plan also supplies F08's physical overlay contract and F35's datum/partial corrections.
Follow-up R1/R9 constrain the remedy. See the [series coverage](25-design-remediation.md#finding-dispositions)
for primary and contributing owners.

The target keeps complete declared quantity contracts at named boundaries, while allowing
intermediate expressions to carry semantic kind algebra without a new public declaration for
every product. Their basis, reference, point/difference status, subjects and index context survive
composition. Equal dimensions are necessary for compatibility but never sufficient evidence of it.

A lawful reduced-coordinate model is supported through an explicit physical mapping. Heat/work
are datum-free transfers; material enthalpy transport retains its reference datum. Sign belongs
to an owner-relative transfer contract. These changes let an ordinary scientific extension stay
in its authored package while the generic kernel enforces its meaning.

## Decisions and interfaces

1. **Semantic intermediates.** Extend the existing quantity inference operation to represent
   anonymous semantic products/quotients/powers and indexed reductions. Preserve declared
   equivalences and distinctions. An inferred product meets a named boundary only through its
   admitted semantic relation; dimension cancellation cannot erase kind, subject or datum.
2. **Coordinate maps.** A mapped-law contract names ordered physical inputs, reduced slots,
   forward expressions, references and validity obligations. Species/composition-dependent
   references are explicit dependencies. Its physical wrapper composes those expressions with
   the implementation. A bare Scalar does not satisfy a mapped-coordinate slot; neutral scalar
   multiplication remains lawful.
3. **Differentiation.** Lower the composed physical wrapper through existing independent partial
   slots and Symbolica differentiation. Preserve Delta(result)/Delta(argument) semantics,
   including coincident actual inputs. Do not add a project-owned chain-rule engine.
4. **Transfers.** Energy-transfer rates carry no enthalpy datum. Typed direction is relative to
   a named boundary/owner and is converted explicitly once. Enthalpy reference translation is a
   different operation requiring the relevant composition context. Do not mint equipment-specific
   quantity kinds or permit arbitrary cross-datum arithmetic.
5. **Physical admission.** One checked canonical-magnitude operation validates source and
   converted finiteness. Preserve multiply-then-add rounding; do not introduce FMA, saturation
   or arbitrary magnitude ceilings. It retains row/source attribution on refusal.
6. **Facet projections.** The schema owner declares each facet's purpose. Consumers request exact
   native observation, execution identity, value identity or the separate
   logical/storage-type projection; they cannot independently choose metadata keys to ignore.
   Physical admission determines quantity compatibility and permits lawful defined-unit
   conversion, including affine cases. Source-publication identity belongs to I.
7. **Overlay interface.** Export a quantity value with unit plus the full expected target
   contract/context. 25f resolves targets and composes overlays; this owner converts and checks
   their physical compatibility. 25i owns framing, 25j generated transport.

Plan 24 informs coordinate/reference distinctions; its dimension-only expression checker is not
adopted. Dimension-only intermediates, blanket bans on reduced laws and declarations for every
coefficient were rejected because they either lose meaning or preserve the current authoring
burden. No new quantity library or crate is required.

## Packets

| Packet | Prerequisites | Responsibility | Status |
|---|---|---|---|
| <a id="a1"></a>A1 Canonical values and facets | Existing quantity/schema owners | Establish checked conversion and purpose-specific physical projections; export the overlay value contract | implemented; focused checks passed |
| <a id="a2"></a>A2 Semantic intermediate algebra | A1; physical ADR decision | Extend one inference authority and migrate ordinary intermediate consumers | implemented; focused checks passed |
| <a id="a3"></a>A3 Mapped laws and derivatives | A2; I1 framing contract | Admit coordinate mappings and migrate physical law boundaries | implemented; focused checks passed |
| <a id="a4"></a>A4 Transfers and datum operations | A2 | Admit direction/reference operations and migrate energy bindings | implemented; focused checks passed |

### A1 — Canonical values and facets

**Final state.** Both inline quantities and declared numeric data columns produce the same
admitted canonical numeric value. It consists of finite canonical magnitude bits and the resolved
quantity identity, associated with its complete declared contract. It is not a naked float whose
unit, basis or datum the next consumer must guess. The existing internal Value::Number can remain
the compact carrier; the referenced admitted quantity contract supplies shared meaning rather
than copying a full descriptor into every cell.

**Admitted conversion plan.** Before converting values, resolve a plan containing the target
quantity identity/key, declared source representation unit, canonical representation and admitted
scale/offset operation. Resolve semantic compatibility before arithmetic: kind, dimension, basis,
point/difference status, reference datum, subjects and indices must meet the target contract.
A representation conversion changes units within that meaning. Changing basis, datum or subject
requires the separately admitted physical operation; multiplying by a convenient scale is not
permission to reinterpret an enthalpy or composition.

**Value flow.** The column adapter obtains the plan from its admitted quantity/storage declaration
instead of carrying detached scale_bits/offset_bits as an unchecked convention. Its document
unit must still agree with that storage declaration: accepting convertible units in other
contexts does not let a file silently override its declared storage unit. The inline adapter
obtains the same plan from the checked literal and expected quantity. Each adapter then:

1. Verifies the incoming numeric magnitude is finite.
2. Applies the admitted multiply-then-add operation with existing rounding semantics. Offsets
   apply according to the point/difference contract; a difference cannot accidentally acquire
   a point's origin shift.
3. Verifies the converted magnitude is finite.
4. Constructs the canonical value only after success. Missing optional values remain Missing
   and do not enter numeric conversion. Preserve the conversion scale for existing consumers
   such as uncertainty scaling. A failure distinguishes
   nonfinite input, incompatible representation and nonfinite converted result, carrying operands;
   the adapter attaches document/column/row attribution or the inline expression span.

For example, an admitted temperature point of 80 °C yields 353.15 K. A temperature difference
uses the interval conversion, and an enthalpy with the wrong datum refuses before numeric
conversion even if its unit matches. A finite extreme input whose multiplication overflows
refuses as conversion overflow, rather than storing infinity for a later evaluator to discover.

**Field comparison after the change.** Schema declarations own facet purposes and generate a
pure policy table usable by the lower columnar layer. Do not introduce a columnar → schema
dependency: schema already consumes columnar facilities. Preserve the existing distinct
equivalence questions, making their key classifications declaration-driven:

| Requested comparison | Resulting projection and boundary |
|---|---|
| PhysicalObservation | Exact native fields, storage, names, nullability and metadata remain observable |
| ExecutionIdentity | Exclude declared prose/structure presentation names; retain execution-relevant and unknown metadata |
| ValueIdentity | Exclude those presentation facets and declared usage-role/foreign-reference facets; normalize root alias/nullability as the existing value contract specifies, while retaining value-domain/quantity meaning |
| Physical quantity compatibility | Use the resolved quantity operation above; neither equal storage types nor ValueIdentity alone proves basis, datum or reference-binding compatibility |
| Logical/storage value type | Keep the separate FieldContract value-type projection for its storage question; its omission of quantity/identity facets cannot authorize semantic admission |

Unknown metadata stays observable/semantic by default until classified. Recurse through nested
field children so list/struct/union/dictionary carriers do not lose the rule. Exact source
publication identity remains I's separate contract; do not relabel ExecutionIdentity as source
identity or use one projection for every comparator.

Relational quantity validation consumes a projection of the selected admitted physical definition
closure: resolved unit identity/dimension/reference restriction and the complete quantity context.
A defined composite unit derives those facts from its admitted factors; it does not gain another
authored dimension declaration. Missing dependencies refuse rather than being filled from a
global registry. A compatible kJ/(kmol*K) representation can therefore satisfy a heat-capacity
contract, while a missing factor or wrong reference cannot.

The declaration covers domain and materialized semantic spellings, including root-versus-nested
behavior. Field restoration remains directional: annotations lost by execution may be restored
only from an established expression/input contract; contradictory annotations refuse. It is not
value equality or permission to invent a quantity. Root alias normalization does not erase
meaningful nested member names.

Migrate the actual comparator/admission consumers to request the appropriate purpose, then remove
their literal-key allowlists and the canonical-unit-only refusal where admitted conversion is
valid. Validation may still occur at several boundaries; all consume the same declared predicates.

Focused acceptance covers inline/column equivalence, pre- and post-conversion nonfiniteness,
lawful near-limit values, affine point versus difference handling, nested-field metadata,
unknown keys, display-only differences, role/reference usage and retained basis/datum distinctions.
A1 is complete when these consumers produce/compare the admitted products just described, not
merely when a new helper exists.

### A2 — Semantic intermediate algebra

**Implementation vision.** An inferred expression carries a semantic kind expression: canonical
factors over declared kinds with exact exponents, plus basis, reference, point/difference status,
subjects and bound indices. Inference takes an operation, its operand contracts and any expected
named result; it returns the resulting contract and admitted rule/equivalence, or an attributable
refusal. Normalize through declared kind relations rather than replacing kinds with dimension
vectors. For U·A·DeltaT, U·A can be an anonymous product until checked against the heat-rate
result; flow·z[j] retains its species index. Torque does not become energy by dimension equality.
The checker retains this admitted result and lowering consumes it, so the compiler does not
approximate or reconstruct the physical algebra.

Make modeling/checking and compiler lowering consume the same inferred contract. Include indexed
products, reductions and exponent handling already supported by the language; keep bound indices
and subjects explicit. Migrate representative UA·DeltaT, component-rate and potential expressions
as each operation becomes supported. Delete their unit-stripping/re-dressing helpers and parallel
inference branches immediately. These examples are first controls, not the completion boundary:
migrate every production strip/re-dress consumer in the affected constitutive and unit-model
scope. Record a bounded consumer inventory and justify each retained scalar operation by its
lawful neutral-scaling role.

Focused acceptance: valid indexed products and reductions, wrong basis/subject, same-dimension
wrong-kind refusals and affine-point multiplication refusal. Named public quantities do not become
interchangeable through anonymous intermediates. This packet amends the relevant ADR-0124
“never synthesized” restriction rather than silently bypassing it.

### A3 — Mapped laws and derivatives

**Implementation vision.** Admit a coordinate-map descriptor with physical input slots, reduced
slots identified by map/role, forward expressions, reference dependencies and domain obligations.
For a Helmholtz free-energy wrapper, independent inputs T, V and n[j] produce density=sum(n)/V,
composition x[j], Tc(x), rhoc(x), tau=Tc(x)/T and delta=density/rhoc(x). Bind alpha(tau,delta,x) through the physical wrapper
A=sum(n)·R·T·alpha. This wrapper is ordinary typed expression composition, so library
differentiation includes composition-dependent reference derivatives. Clients request physical
A or its partials, not an unlabelled reduced derivative they must rescale themselves. Partial requests state held-fixed coordinates: differentiating at fixed
density is not the chemical potential at fixed volume. A density-coordinate interface requires
an explicit coordinate transformation before reporting standard volume-based derivatives. Map role,
direction and dependencies enter admitted/preparation identity; no plain dimensionless slot is
interchangeable merely because it lowers to a number.

Migrate the existing Helmholtz, cubic, PC-SAFT and activity-law reduced implementations through
physical wrappers, retaining their scientific values and declared parameter conventions. Record
the actual coordinate direction and dependencies for each family. Remove obsolete unit constants
only where their role was stripping meaning; genuine scientific reference constants remain.

Focused acceptance compares direct physical and mapped formulations for values and first/second
derivatives. Bindings incompatible with declared coordinate slots/order/direction, references or
datum refuse. Both direct and reciprocal mappings are lawful when declared and composed
consistently. A scientifically wrong but type-correct mapping expression fails independent family
value/derivative controls; generic typing cannot infer its intended mathematics. Exercise indexed partials and repeated actual arguments. 25d consumes this physical
derivative contract; it does not reinterpret the mapping.

### A4 — Transfer and datum migration

**Implementation vision.** A transfer binding contains a datum-free rate expression, boundary
owner and positive-direction convention. Values remain signed; positive-into is a convention,
not a nonnegativity constraint. The conservation operation performs orientation conversion once:
dU/dt = sum(H_in) - sum(H_out) + Q_into + W_into. Opposite exchanger boundaries satisfy
Q_hot_into + Q_cold_into = 0. A heat-removal signal must explicitly change orientation before
binding to an into slot. In contrast, an enthalpy-datum translation consumes source/target datum
contracts and composition, and produces translated enthalpy with its reference provenance.
Heat addition cannot perform that translation accidentally.

Migrate heater, exchanger, reactor and control-volume energy bindings to datum-free transfer
rates and typed orientation. Contribution admission checks that sign conversion occurs once.
Derive or validate gauge conversion offsets against the single authored datum; retire unused
conversion helpers and repeated numeric authority. Replace the bt-ideal oracle-to-stock datum
translation with the explicit reference operation. Reactor extent/rate reports must expose their
actual physical flow quantity and metadata, not a scalar carrying only a convention label.

Focused acceptance: opposite exchanger boundaries cancel internal heat; an incorrectly bound
heat-removal convention refuses; unlike enthalpy datums require explicit translation; changing the
gauge datum changes every derived conversion consistently. 25c consumes these contracts when
forming connected boundaries, and 25b uses them for reaction extent/heat binding.

## Authority and handoff

Use an ADR plus design review for D5/ADR-0124 and metadata-convention changes, with updates to
blueprint §8, §9.8 and the relevant data-boundary sections. Widen the R-51 trigger so silent
unit stripping is a reason to revisit adequacy, not only a failed named-kind lookup.
No accepted record is changed during plan authoring.

The resulting public contracts are physical values, semantic intermediates, coordinate maps and
owner-relative transfers. The plan does not own study identity, scientific parameter selection,
workflow acceptance or Python validation. Those consumers link these meanings instead.

## Execution and evidence

The packet table records implementation progress; unimplemented contracts remain **Proposed**.
Focused evidence does not establish integrated product qualification. The [series coordinator](25-design-remediation.md)
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

Implementation completed on 2026-09-30 under the maintainer's instruction to implement this plan.
The work is integrated in the current checkout; concurrent Plan 24 and governance work is
preserved. The Outcome owns the focused verification and its limits. No publication,
Python-extension refresh or integrated qualification is implied.

A1's canonical-value and facet cutover is implemented. A2 records and consumes physical
admission by exact expression occurrence, including anonymous finite-reduction prototypes.
The independent review corrections for short-circuit occurrences, retained reductions and
integral-power bounds are implemented and their targeted controls pass.

A3's ideal, PR, PC-SAFT and NRTL controls pass, including independent direct/reciprocal maps
with composition-dependent references, abstract scientific witnesses and mapped cubic
polynomial front doors. Source consumers retain distinct species fugacity/activity contracts.
A4's stored-energy, material/element, separator, reaction-rate, selected-solvent and entropy
consumers use physical algebra or declared maps. Obsolete unit-adapter helpers are removed.
Transfer source admission exposed missing indexed-boundary and inherited-boundary handling;
the core now resolves actual indexed, inherited and child-instance boundaries. Generated
conservation rows retain the admitted accumulator type at their initial zero. Executable
preparation preserves the checked body and its occurrence spans together.

Targeted source controls retain production scientific declarations, excluding external dataset
transport and supplying explicitly synthetic keyed data. They do not establish a full
package-data, native-solver or Python journey.

The architectural amendment, blueprint revision and R-51 trigger widening are prepared through
ADR-0135/0136. Those records remain proposed; accepting or superseding decision records follows
the decision-PR route. The implementation amendment does not claim that route has completed.

### Handoff

No functional work remains in A1–A4. The series coordinator records the resolved physical
findings and the completed contributions to findings with other owners. Other lettered plans
remain proposed and require their own authorization. Full integration, solver/Python journeys,
lint and performance qualification remain in 25k. This completed record remains linked while
the active series depends on its evidence; enduring contracts live in the architecture sections.

### Bounded consumer inventory

| Consumer family | Physical boundary and deletion | Retained numerical role |
|---|---|---|
| Helmholtz, PR, PC-SAFT and NRTL | Physical T/V/amount wrappers; named species responses; composition-aware references; ordinary independent partials | Reduced algorithms consume their declared map slots; calibrated scientific constants remain |
| Cubic density and flash roots | Density and pressure-difference residuals; physical T/P/composition polynomial front doors; old scalar coordinate members removed | Polynomial helpers, dimensionless compressibility and solver start/bound multipliers relative to the declared density scale |
| Equilibrium and caloric states | Named log-component-fugacity and Gibbs products; shared pressure/composition entropy map; physical weighted-pressure sum; explicit Cp/R fixture reconstruction | Neutral composition weights, phase fractions and the existing complementarity slacks normalized by the explicit 1 K scale |
| Vessel, reactor, heater, exchanger and pressure changer | Direct stored-energy products, datum-free owner-relative transfer rates and composition-aware reference translations | Efficiencies, split fractions and numerical tolerances remain neutral scales |
| Material and element projections, separator | ComponentFlow/Flow composition, physical energy-flow products and indexed totals; explicit elemental-count map | Element counts are scientific coordinates; no component-flow/total-flow/pressure-difference unit helpers remain |
| Reaction and dilute-liquid laws | Declared Arrhenius, second-order and selected-solvent maps; physical reaction-extent reports | Calibrated coefficient, concentration and activation-energy references occur inside maps/reconstruction |

The obsolete `ChainTree`/`BodyBuilder::chain` inference path, syntactic-only witness acceptance,
and migrated stripping/re-dressing callers are removed. The existing library differentiation
and numeric lowering remain the sole execution path.

## Outcome (recorded after implementation)

### What was built

**Implemented:** canonical-magnitude admission checks source and converted finiteness, preserves
multiply-then-add rounding and carries row/source attribution. Declared field-purpose projections
replace consumer-owned metadata allowlists; resolved selected physical definitions govern unit
compatibility, including composite and affine representations.

**Implemented:** physical inference preserves semantic kind factors, basis, datum, subjects and
indices. Checked function occurrences retain admitted operations through specialization and
lowering, including anonymous finite reductions. Public named boundaries remain explicit; the
registered geometry and total/species-density products do not grant dimensional casts.

**Implemented:** declared coordinate maps and reconstructions serve the physical potential,
activity, cubic-root, reaction, caloric and selected-solvent consumers in the inventory above.
Physical partials use the composed wrapper and the existing library differentiation. Normalized
abstract scientific witnesses reject zero coefficients and cancellation while admitting a lawful
zero implementation and coincident actual arguments.

**Implemented:** datum-free transfer rates retain actual boundary owner, ordered coordinates and
direction through reflection, conservation and report projection. Scientific enthalpy/entropy
reference translation retains composition and paired anchor provenance. The architectural owners
and R-51 trigger are amended; ADR-0135/0136 remain proposed pending their decision-PR route.

**Tested, 2026-09-30:** the following focused controls pass against a **zero-failure baseline**.
Each table row uses `just unit-package <package> '<filter>' --test-threads 1`; that recipe
uses the pinned toolchain, the optimized test profile and explicit
`--features pse-relations/force-validate`. Filters deliberately exclude unrelated tests.

| Package | Exact filter | Result |
|---|---|---|
| pse-quantity | `test(chain_tests) \| test(contextual_tests) \| test(canonical_admission_) \| test(standard::tests)` | passed: 19; failed: 0 |
| pse-modeling | `test(canonical_numeric_) \| test(physical_operations::tests) \| test(contextual::tests) \| test(physical_admissions_) \| test(generic_physical_admission_) \| test(inherited) \| test(override) \| test(caloric_contract_tests)` | passed: 24; failed: 0, before the additional conservation control below |
| pse-modeling | `test(conservation_zero_retains_material_energy_type_with_directed_transfer)` | passed: 1; failed: 0 |
| pse-columnar | `test(native_field::tests) \| test(physical_execution_and_value_metadata_projections_answer_different_questions)` | passed: 5; failed: 0 |
| pse-relations | `test(validate::obligations::quantities::tests)` | passed: 5; failed: 0 |
| pse-math | `test(retained_physical_admission_) \| test(retained_finite_reduction_) \| test(normalized_scientific_witness) \| test(integral_power_growth_is_bounded)` | passed: 5; failed: 0 |
| pse-runtime | `test(report_transfer_context_retains_actual_owner_order_and_direction)` | passed: 1; failed: 0 |
| pse-authoring | `test(physical_maps_responses_translations_and_transfers_roundtrip)` | passed: 1; failed: 0 |

**Tested:** compiler verification is a composite result, not an initially clean run. All
commands below use the same test profile, explicit force-validation and zero-failure baseline.

```sh
just unit-package pse-compiler 'test(physical_potential_tests) | test(scientific_witness) | test(authored_transfer_tests) | test(checked_occurrence_handoff) | test(checked_compound_guards) | test(checked_finite_reduction) | test(direct_powers) | test(physical_inventory_identity_frames)' --test-threads 1
just unit-package pse-compiler 'test(authored_bt_ideal_translation) | test(authored_bt_pr_translations) | test(authored_cstr_extent_report) | test(authored_distributed_energy) | test(authored_exchanger_reflection) | test(authored_vessel_amount)' --test-threads 1
just unit-package pse-compiler 'test(physical_potential_tests) | test(scientific_witness) | test(authored_bt_ideal_translation) | test(authored_bt_pr_translations) | test(authored_cstr_extent_report) | test(authored_exchanger_reflection) | test(checked_occurrence_handoff) | test(checked_compound_guards) | test(checked_finite_reduction) | test(direct_powers) | test(physical_inventory_identity_frames)' --test-threads 1
```

The initial selection had 21 passed and 6 failed. Fixture-root and input-binding repairs
made the distributed-boundary and vessel controls pass; that six-test rerun had 2 passed
and 4 failed. Preserving checked function spans and the generated conservation zero's
type corrected the remaining failures. The final 19-test run passed all 19 with 0 failures,
including the shared physical-law, witness and retained-admission controls. The composite
selection therefore covers **27 distinct compiler controls with no unresolved failures**.
The other package controls above cover 61 distinct tests, for **88 focused controls** overall.

The scientific controls exercise ideal, asymmetric PR, PC-SAFT and NRTL values/partials;
composition-dependent direct and reciprocal maps; independent frozen cubic-polynomial values;
coincident arguments; and erased-witness refusals. Source controls exercise physical reaction
rates, stored energy, material/element projections, datum translations, actual CSTR report
quantity/unit, two distributed boundary coordinates, opposite exchanger owners and vessel
residuals. They retain authored scientific bodies while using explicit synthetic keyed data.
They do not run a native solve or qualify the complete packaged dataset.

**Implemented / passed:** `just codegen` regenerated all six schema targets and confirmed the
workspace-hack required no change. This is generation evidence, not a hygiene result.

**Interface-checked / passed:** `just check` (`cargo check --keep-going --workspace --all-targets
--locked`, pinned toolchain, optimized dev profile) compiled all workspace targets with
0 errors against a zero-error baseline. Cargo emitted an upstream future-incompatibility
warning for `proc-macro-error2 v2.0.1`; this is not a warning-free result.

**Not run:** full integration, native-solver, Python and performance campaigns remain 25k work.
Manual hygiene was not run; `just hygiene` belongs to 25k's scope-end qualification (ADR-0143).
No performance or complete-product qualification is claimed. ADR-0135/0136 acceptance and
supersession remain the separate decision-PR route.

### A mistake made and corrected

The initial energy-ledger cutover exposed that registered addition had no dimensional-result
case, although multiplication and division did; the registered Add/Sub admission was corrected.
The first residual molar response declarations accidentally copied a derived stock-datum
definition; they were corrected to distinct datum-free physical kinds. Scientific anchor
evaluation was moved out of the static selector into the ordinary composed mathematical path.
Translation execution then exposed post-admission span clearing, which broke exact occurrence
lookup; preparation now preserves the checked function unchanged. Generated energy conservation
also needed its existing accumulator contract when inferring the initial zero: a bare watt
literal is ambiguous among distinct physical meanings. That context is supplied only to the
derived conservation row, with authored ambiguity and wrong-direction refusals retained.

### Deviations from the plan, deliberate

The physical inventory framing and metadata comparator prerequisites were implemented with A1–A4
because leaving them until 25i/25h would create ambiguous identities or drop the new context.
The scope remains the minimal prerequisite described by this plan. Focused compiler controls
load actual scientific source declarations with explicit synthetic parameter rows and omit
external dataset transport. Full package-data, solver and Python execution remains in 25k;
these focused controls make no integrated qualification or performance claim.
