---
title: "25a: Physical values and contextual contracts"
status: in-progress
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
| <a id="a2"></a>A2 Semantic intermediate algebra | A1; physical ADR decision | Extend one inference authority and migrate ordinary intermediate consumers | in progress |
| <a id="a3"></a>A3 Mapped laws and derivatives | A2; I1 framing contract | Admit coordinate mappings and migrate physical law boundaries | in progress |
| <a id="a4"></a>A4 Transfers and datum operations | A2 | Admit direction/reference operations and migrate energy bindings | in progress |

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

The maintainer requested a restart checkpoint on 2026-09-30, with compile and targeted-test
repairs before stopping. **This plan remains in progress.** Changes are in the shared checkout;
no commit, push, full integration run or series qualification is implied. Concurrent work under
`thermo-knowledge/` belongs to Plan 24 and must be preserved when resuming.
The existing editable Python extension predates the changed generated contracts; refresh it
through the normal `just py-sync` recipe when a Python execution scope is resumed. Rust-focused
checks below do not qualify that extension.

**A1 — implemented core admission and comparison cutover.** Inline values, data columns,
reference conditions, smoothing inputs and mathematical literals/bindings use checked canonical
magnitudes. Conversion admission and application retain full quantity meaning, finite source/result
checks and multiply-then-add rounding. Schema declarations generate comparison-purpose policies
downward into columnar code, including nested and unknown metadata. Relational obligations consume
the exact selected physical declaration closure and admit only encountered quantity/unit pairs.
The runtime-only decoder, raw-dimension/canonical-unit-only obligation branch, unchecked scalar
conversion and detached column coefficients have been removed. Transfer report context and the
remaining alias/output comparison consumers were added during A4; their focused checks belong to
the same completion boundary.

**A2 — implemented inference and lowering, consumer closure still being established.**
`ResolvedPhysicalContract` retains anonymous semantic factors, contextual obligations, actual
binders, ordered axes and numerical representation. Closed schemes and polymorphic substitutions
can retain that product without a fabricated quantity ID. Registered matches/refusals are final;
the former maximal-chain rescue and competing named multiplication/division implementation are
removed. Mathematical lowering retains admitted operands, selected operations, numerical scales
and exact output authorizations. Ordinary qualified cancellation does not erase its context.
New frame versions cover the changed physical inventory, admissions, typed definitions,
mathematical bodies, finite functions and dispatch bodies.
The source checker does not yet persist a per-expression admission map for direct handoff to
lowering: lowering recomputes admission through the same quantity authority. That remaining
handoff is part of A2's stated final state, even though the competing algorithms are removed.

**A3 — implemented authoring and specialization; scientific verification is a separate exit.**
Coordinate maps, nominal slots, reduced laws, reconstruction families and scientific responses are
declared in the modeling schema. `reconstruct(family, selected_law, physical_arguments...)` evaluates
that map's coordinates from those exact arguments. A response retains the selected function and
requires its closure to reach a reconstruction. Its narrowly scoped scientific formula authority
is retained through compilation; it cannot rescue a registered refusal or leak into ordinary
arithmetic. The PR, PC-SAFT, NRTL and Helmholtz source interfaces and dependent potential signatures
have moved to physical wrappers. The former generic Scalar potential/derivative path is removed.
Residual/excess potentials are distinct physical contracts; residual molar enthalpy/entropy are
datum-free responses, while material DeltaH/DeltaS retain their datum. New focused compiler controls
exercise current authored documents rather than copies of their formulas.
Response witness admission currently requires an explicit authored call/partial dependency and
a selected closure that reaches a reconstruction. It does not prove that the normalized formula
still depends on the potential after simplification, such as when an authored term multiplies
the potential by zero. The stronger dependency check and its negative control remain open.

**A4 — implemented contextual mechanisms and authored migration, with remaining closure work.**
Boundaries resolve to actual instance, declaration and ordered coordinates. Reorientation and paired
reflection retain the owner and apply the convention factor once; directed ledger contributions
consume Into, and final contextual equation admission checks actual owners before numerical
lowering. The old contribution-pair ID/side mechanism and its primitive fixture are removed after
the replacement's focused model controls passed. EnergyTransferRate, ReactionExtentRate and typed
stoichiometric coefficients replace the energy/extent Scalar adapters in the affected process
models. The distributed spatial coordinate remains dimensionless. Both BTIdeal enthalpy and BT_PR
enthalpy/entropy bindings now name explicit reference translations and scientific anchors, including
the BT_PR entropy pressure correction. Translation templates retain anchor calls and composition
in ordinary library expressions; they do not evaluate scientific functions in the static selector.
Gauge conversion derives its origin from the selected datum after unit representation; affine
conversion declarations must agree with that authority. The duplicated gauge rule and unused
runtime conversion-expression helper/test are removed.
Report rows now have a declared transfer context carrying the actual instance, boundary,
ordered coordinate identities and direction. Unbound and unreconstructed roles refuse at that
boundary. This is a generated transport addition; it does not establish execution of a complete
process report journey.

**Decision/document boundary.** ADR-0135 and ADR-0136 remain proposed and describe the selected
target, including the bounded design assessment. Proposal status is not implementation acceptance.
The enduring architecture amendment, revision row and any successor treatment of ADR-0124 still
belong to the decision/design route once the implementation boundary is settled. Full series
qualification remains in 25k.

**Focused verification at the checkpoint.** The final core reruns passed the eight quantity
algebra controls, three datum/reference controls, four anonymous/contextual modeling controls,
five mapped-operation admission controls, and the mathematical UA/cancellation/derivative
control. Compiler controls passed transfer execution, composition-dependent reference
translation and its inverse, and the existing multiplicative-chain case. Symbolica evaluation
was serialized, and unit recipes enabled explicit force-validation. Generation passed after
repairing the invariant-evidence producer to declare its tagged alternative through the existing
checked-value operation.
The final schema-facet, columnar-facet and engine-output selections passed 3, 4 and 3 controls
respectively. The transfer-report generated-row roundtrip, tagged-evidence producer and two
authoring physical-syntax controls also passed. Their successful compilation and generation do
not substitute for the authored scientific acceptance still open below.
`just check` passed for the workspace and all targets after removing the last two test calls to
the deleted unchecked conversion helper. `just codegen` passed. The build still reports the
existing `proc-macro-error2` future-incompatibility notice; this is not a warning-free lint claim.
The updated independent source-package scientific-unit control also passed via
`just test-package xtask -E 'test(codegen::physical::tests::reference_package_admission_scientific_units)' --test-threads 1`
(one selected test, zero failures). All agents have stopped editing; no task-owned build or test
process remains running at this checkpoint.

The remaining failing command is
`just unit-package pse-compiler 'test(reference_physical_potential_sources_admit)' --test-threads 1`:
one failure against the zero-failure target, at the subject-erasure boundary described below.
The earlier five-test physical-potential selection was blocked before scientific evaluation;
none of those scientific cases is claimed to pass. There was no integration, solver, Python,
performance, formatting or lint campaign.

### Resume order and remaining work

1. Read this checkpoint and inspect the current diff without resetting the shared checkout.
   Preserve Plan 24 changes. Keep the current checkout/toolchain and serialize Cargo work.
2. Resolve the known A3 source-admission failure first. `helmholtz.fugacity_response` declares
   plain Scalar, but its partial with respect to component Amount retains the species subject.
   The physical-response boundary correctly refuses that erasure. Establish the explicit
   component response contract and migrate the matching fugacity/activity consumers, including
   `ln_phi`, NRTL `activity_response`/`ln_gamma` and `gibbs_duhem_response`, as applicable.
   Do not weaken the generic subject-preservation guard or add a scalar cast to make the test pass.
   Then rerun source admission and the five `physical_potential_tests` controls serially.
   No scientific response value/derivative acceptance is established by the current failed runs.
   Finish the other focused A2/A3/A4 exits. Regenerate from schema/physical sources whenever they
   change; never repair generated output by hand.
3. Close the authored consumer migration using actual package admission and isolated physical
   evaluations. In particular, verify the full process energy/stoichiometry changes, reference
   translation composition and inverse behavior, distributed boundary coordinates, actual report
   context and quantity/unit metadata. A helper test or successful compiler build does not establish
   those whole source-package claims.
   Retained adapters needing explicit disposition include the vessel amount/stored-energy
   closures and the BT_PR entropy anchor's increment/gas-constant representation. The PR-specific
   density/oracle reduced helpers serve a separate local cubic calculation; distinguish them
   from the removed generic reduced-potential interface before deciding what to delete.
4. Complete the remaining acceptance cases in A2–A4: qualified cancellation and empty/indexed
   reductions, coincident physical partial arguments, composition-dependent coordinate references,
   all physical potential responses against independent oracles, wrong-owner/double-sign refusals,
   gauge-datum perturbation and both caloric reference families. Keep targeted checks distinct from
   the deferred cross-owner journeys.
5. Reconcile the scoped deletion inventory and documentation with the completed behavior; remove
   any remaining replaced adapters/callers/fixtures once their replacements are demonstrated.
   Check that changes to physical context reach every touched consumer, not just numerical payload
   extraction. Complete architecture/blueprint and deferred-decision trigger updates through the
   owning route; update finding dispositions only when the required correction evidence exists.
6. Mark this plan complete only when all four packets and their focused exits are satisfied.
   Formatting, lint, integration, solver/Python journeys and performance qualification remain in
   25k under the maintainer's chosen execution rhythm.

## Outcome (recorded after implementation)

### What was built

Partial implementation is recorded in the execution checkpoint. This section will record the
completed behavior and final evidence labels when the packet exits are satisfied.

### A mistake made and corrected

The initial energy-ledger cutover exposed that registered addition had no dimensional-result
case, although multiplication and division did; the registered Add/Sub admission was corrected.
The first residual molar response declarations accidentally copied a derived stock-datum
definition; they were corrected to distinct datum-free physical kinds. Scientific anchor
evaluation was moved out of the static selector into the ordinary composed mathematical path.

### Deviations from the plan, deliberate

The physical inventory framing and metadata comparator prerequisites were implemented with A1–A4
because leaving them until 25i/25h would create ambiguous identities or drop the new context.
The scope remains the minimal prerequisite described by this plan. The restart checkpoint is a
pause in execution, not a reduced completion criterion or a claim of integrated acceptance.
