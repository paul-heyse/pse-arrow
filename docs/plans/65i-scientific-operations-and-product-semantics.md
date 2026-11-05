---
title: "Plan65i: Scientific operations and product semantics"
status: proposed
date: 2026-11-03
adrs: []
review_sources: [docs/library_leverage/reviews/proto/README.md, docs/plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md]
doc_role: plan
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, numerical-execution, preparation, validation]
---

# Scientific operations and product semantics

## Target, source and responsibility

**Proposed:** make scientific operation and product-use contracts govern native preparation,
admission, execution and consumption before their mechanical representations are generated.
This companion implements review [I01](../plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md#I01),
[I02](../plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md#I02),
[I03](../plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md#I03)
and the scientific early slice in [I11](../plan_improvement_reviews/plan65-integrated-proto-and-simulation-extensibility-2026-11-03.md#I11).
Those review IDs are source recommendations; I01–I06 in this document are implementation
packages with this plan as their sole progress owner.

The [coordinator](65-proto-integrated-target.md) owns combined decisions and completion.
[65e](65e-scientific-portability-and-execution.md) owns portable artifacts and execution
placement; [65f](65f-diagnostics-and-result-delivery.md) owns reporting and readback. This
document owns the scientific meaning those mechanisms consume. It neither moves scientific
behavior into the pure structural crate nor establishes a universal scientific ontology.

Source inspected at `5f4fa0246c0b9fac68493b608f0b2eefe79f116d` with concurrent dirty Plan64
source preserved. Mechanisms are **Interface-checked** by representative source reads; all
changes and benefits remain **Proposed**. Execution is authorized alongside incomplete Plan64
as of 2026-11-04 and may replace overlapping mechanisms. Each delivery consumes the actual
native capability it needs; whole-Plan64 completion is not a prerequisite. No completed scientific implementation, product test or
performance measurement is claimed here. Criteria: core 3.4, heuristics 1.0, process-simulator
1.5 and test-assessment through the selected standard. Process-system breadth is the horizon;
remote/distributed deployment and SurrealDB v4 are excluded from delivery.

## Execution checkpoint — 2026-11-04

The first native semantic slice starts from `d63fcbb11`. `DeclaredExecution` already joins
the admitted route, payload-derived `DeclaredProcedure::kind`, analysis, original model and
requested start policy, retaining private source/case/compiler preparation. Extend that
composition; do not introduce a parallel scientific-operation policy table. Its observation
projection must not construct a new admitted execution from recorded fields.

The first positive consumer is the existing integrated analytic model `dx/dt = 1`, `x(0) = 1 s`,
with an independently specified endpoint `x(1 s) = 2 s` and observable nominal scale. Native
`SimulationSampleAttempt` and observer completion keep attempt completion separate from final
selection. `Completion` remains the owner of original task/step/point scope, local product
validity and governing scientific use. These are the required meanings for the minimum A/B
projection; they do not require a new contract type for every concept.

Native step selection now uses the same unique-match operation for public step lookup and
derived point-product permission. Duplicate governing step facts remain readable raw evidence,
but cannot issue permission based on their order, including two individually accepted copies.
Local validity remains visible when governing permission is unavailable. The subsequent
completion-owner observation projections and independent readback are implemented and narrowly
tested on main; [65f's checkpoint](65f-diagnostics-and-result-delivery.md#execution-checkpoint--2026-11-04)
owns the exact command and limits. Task/step/point facts, coverage, qualifiers and local validity
are projected without an inverse permission constructor. Whole completion, selected-attempt
and complete scientific consumer integration remain required; this does not close I03.

Main `178ddaa2a` additionally binds a modeling failure observation to its actual recorded
step. Completed earlier results stay intact; a later unattempted step has no invented
observation, and aggregate failure/cleanup remains available separately. Missing step
attribution does not fall back to step zero. The native three-step regression passes with
the original mathematics/assertions; six selected summary controls also pass. Its initial
failure preceded execution because the non-capacity fixture lacked preparation headroom;
only that fixture pool was adjusted. Runtime default and KINSOL/root-isolation all-targets
receiving checks pass without errors/warnings. This corrects attribution, not scientific
permission or full I03 coverage.

**Tested, targeted:** the native `scripts/pse test pse-runtime --lib --features solver-diffsol`
selection covering `ambiguous_original_step_never_grants_product_permission`,
`local_validity_cannot_grant_permission_without_original_scope`,
`point_products_use_exact_original_point_and_independent_step_permission`,
`dynamic_observable_nominal_is_bound_from_the_compiler_inventory` and
`stopping_one_prepared_handle_preserves_an_independent_session` completed five tests with zero
failures against the zero-failure target on 2026-11-04. The run used original user-scope custody,
explicitly prepared compiler/KLU, explicit cache-off preparation/execution and consuming-closure
Arrow validation. Native execution/capture settled completely. This establishes the selected
native scope, including the independently specified analytic endpoint; property-CoolProp
transport, generated Shutdown publication and assembled qualification remain unexercised.

## Native semantic foundations and selected composition

Existing native and authored contracts already govern substantial science:

| Scientific decision | Present owner and selected continuation |
| --- | --- |
| Temporal route and independent procedure | `pse-modeling::analysis::declared_policy` resolves coherent route/procedure and endpoint requirements. Reuse this admission rather than infer procedure from a service method name. |
| Property/data meaning and applicability | Authored property slots, parameterizations, dependency selection, joint-fit projection and empirical applicability in `domain/models/properties.pse`. Reconstruct the chosen context, not only a map of numeric parameters. |
| Conservation and supported physics | Authored control-volume/equipment definitions govern balances, mesh, boundary conditions and closure. A spatial domain alone supplies no temporal inventory law. |
| Fitting roles and source context | Runtime fitting admission distinguishes global parameters, experiment-local ports and exact revision/physical context. Preserve those roles through portable overlays and observations. |
| Product validity and use | `Completion`, `ScientificPermission`, `ProductScope` and `ProductPermission` compose requested coverage, local mathematics and original decisions. Consumers read these facts rather than derive another conjunction. |
| Dynamic selection and provisional evidence | `SimulationSampleAttempt` and its observer separate native termination, achieved time, requested sample/check coverage, accuracy attempt and final selection. Transport delivery cannot select the final scientific attempt. |

The selected scientific-operation contract joins these native decisions for each actual
operation: authored phenomenon/formulation and material context; selected physical/data
interpretation; route/procedure; required initial, boundary and observation inputs; available
provider/derivative/smoothness/domain capabilities; preparation/evaluation; requested products
and governing assessments. Ordinary checked types/functions, authored guards and narrow
composition objects may express it. There is no requirement for one new type per concept.

Use one model across supported analysis modes while preserving legitimate numerical and
formulation differences. Switching from a steady solve to a transient fit cannot silently
inherit initial conditions, observation mapping, sensitivities or accuracy policy. Control
and optimization extensions must name the required manipulated/observed variables, objectives,
constraints and scientific support; a generic enum does not supply their implementation.

Keep distinct operation owners where their semantics differ. Consolidate repeated decisions
at the native owner and expose their composition. A universal task interpreter would obscure
scientific support and amplify orchestration. Keeping every private mode-specific contract
unrelated would repeat interpretation across clients and retained results. The selected middle
ground shares real scientific definitions and explicit translations without a global workflow
state machine or Proto mathematical IR.

## Product interpretation and use

Record and expose what the original scientific owner established, at its actual scope:

- A task product, an independently requested original step, and an exact producer-owned point
  remain different scopes. Row ranges and quantity names do not substitute for those identities.
- Present/raw candidates, provisional samples and locally valid derived mathematics are not
  automatically usable results. Governing coverage, qualifications and final attempt selection
  remain attributable native facts.
- A protected readable artifact may intentionally retain unusable data for diagnosis. Raw
  evidence access and scientific-result selection are different operations with different
  promises; neither operation upgrades the other's conclusion.
- A decoded historical permission reports its original decision. New execution, derivation
  or reuse needs the receiving owner's applicable context/admission, not a wire constructor
  that reissues the old native grant.

Extend existing completion and product selectors rather than create another permission table.
Preserve ambiguous lookup refusal, unknown/missing evidence and per-product qualifiers.
`Completion::product_permission` is the existing single-match lookup; `ProductPermission`
retains local validity separately from the governing `CandidateUse`. Generated observations
may expose these facts, while owner-sealed admission/permission constructors remain native.

## Concrete library realization at the boundary

The science remains native. [65a](65a-contract-foundations-and-generation.md) generates its
selected metadata and portable records using Buffa 0.9.2 `generate_with_diagnostics`, typed
custom options and compatible companions through `apply_companions`. A generation diagnostic
must identify an unsupported selected scientific facet; silently emitting a generic string
would create a second interpretation obligation in clients.

Buffa `DescriptorPool` and `DynamicMessage::from_options` let independent tools read declared
route/procedure, ordered physical roles, support/refusal metadata and product descriptions from
the matching immutable world. `ReflectMessage::for_each_set` can inspect generated messages
without an owned dynamic bridge. These APIs describe observations; they cannot compute
scientific eligibility or create a live `PreparedProviderRequest` from arbitrary bytes.

Use the selected Buffa 0.9.2 generation/reflection composition: vtable reflection and a shared
descriptor root at generation, `buffa-descriptor`'s `reflect` capability for dynamic inspection,
and JSON/text support only for their selected human/tool consumers. These feature selections
change representation availability, not scientific support. Preserve exact tagged/runtime
agreement when moving declarations into the pure structural foundation.

Use [65b](65b-representation-validation-and-interpretation.md)'s `buf.validate`/generated
`Validate` composition for locally decidable shape and value rules, with actual supported
owned/view coverage from the maintained generator. Physical applicability, coherent
route/procedure, derivative support and original product-use decisions stay native. Exact
byte preservation, ordinary ProtoJSON inspection and scientific adoption are separate promises.

Descriptor/policy identity travels with the observation where it affects interpretation.
Retain the original source/semantic dependencies independently. `DescriptorPool` linking,
wire compatibility or successful validation does not establish unchanged physics. Keep native
numerical buffers and Arrow bulk products rather than interpret descriptors in solver callbacks.
The precise library contracts and their conditions are supported by
[Buffa 0.9.2 generation](https://docs.rs/buffa-codegen/0.9.2/buffa_codegen/),
[reflection APIs](https://docs.rs/buffa-descriptor/0.9.2/buffa_descriptor/reflect/index.html),
[tagged reflection source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-descriptor/src/reflect/dynamic.rs)
and the [maintained validator](../../vendor/protoc-gen-protovalidate-buffa/README.md).

## Consuming dependencies and evolution

Use existing compiler read sets, granular physical scopes, artifact identity and native state
owners. Do not use the whole contract world as one scientific cache key. At each consuming
owner, make the following effects explicit:

| Changed fact | Intended consequence, subject to actual consumed dependencies |
| --- | --- |
| Wire packaging, documentation, Debug redaction or unconsumed result field | Regenerate affected client/presentation artifacts; do not rebuild unchanged mathematical structure. |
| Local validation policy | Renew applicable boundary admission and preserve historical-policy interpretation; recompile science only when its actual premises changed. |
| Database mapping/projection revision | Rebuild or migrate affected projections; retain original scientific meaning and promised historical reads. |
| Physical convention, selected correlation/data, applicability, formulation or derivative capability | Invalidate or explicitly readmit affected scientific products/preparation, even when wire shape is unchanged. |
| Compatible parameter values | Reuse valid structure while refreshing numerical work that consumes the values. |
| Attempt/session/handle generation | Retire mutable state and stale tokens according to the actual owner; immutable mathematics has its own validity lifetime. |

The compiler already separates numerical and validity/guard dependency edges. Preserve those
roles rather than making every validity dependency an arithmetic cycle. A narrower key is
safe only with complete consumed dependencies, including absent names and membership where
relevant. Conservative recomputation remains valid when dependency completeness is unknown.

## Early readiness and packages

The first package supplies **agreed native semantics and discriminators**, using existing
owners before the new structural foundation exists. A01/A02 can then co-design a narrow
declaration/metadata slice. I02/I03 consume working A/B slices for delivery; broad migration
does not precede that evidence. This avoids an A01 ↔ scientific-contract dependency cycle.
The matching developer-evidence slice is defined in 65j; neither slice waits for every
registry or storage projection. H01 retains final assembled-population reconciliation.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| I01 | in-progress | Actual native semantic owners and coordinator decision scope; no completed-Plan64 or new-crate barrier | First operation/product owner agreement is recorded above; complete the selected population and A01/A02 metadata co-design with supported dynamic/product and independent refusal/use cases. Buffa options expose native meaning without defining physics. |
| I02 | planned | I01 agreed meanings; narrow working A/B generation/adoption and existing native operation | Deliver authored source → admission/preparation → native execution → typed observation. Use generated `Validate` only for selected local rules and `DynamicMessage::from_options` for inspection. Preserve native route/procedure and physical guards. Migrate the selected producers/readers together and remove superseded mechanical interpretation. |
| I03 | in-progress | I01 product-use design; narrow working I02/A/B and F/G readback | Completion-owned permission/local-validity observations and independent readback are narrowly delivered. Complete task/step/point, coverage, qualifier and selected-attempt facts through portable/native selectors and actual consumers. Generated reflection is observational; no decoded permission issues new authority. Independently test raw versus scientific access, ambiguity and historical/current interpretation; delete replaced client-derived permission conjunctions. |
| I04 | planned | I01 consumed-dependency agreement and I02/I03 real products | Implement differential change controls at compiler/admission/artifact/projection/state owners. A Debug-only or unconsumed metadata change must not rebuild unchanged math; correlation/datum or derivative changes must invalidate/readmit affected science. Reuse existing dependency machinery, remove replaced blanket world-key invalidation and duplicate semantic-key derivation where present. |
| I05 | planned | Working I02–I04 slices and ready E/F/G consumers | Complete the scientific population declared in I01 across supported post64 steady/dynamic/fitting/study/analysis operations and explicit unsupported combinations. Buffa generated adapters/typed options carry one meaning; native reconstruction and governing product selectors enforce it. Remove replaced mode/client-specific interpretation and obsolete fixtures with all migrated consumers. |
| I06 | planned | I02–I05 functional scope; H01 agreed acceptance scenarios, not H02 qualification results | Establish extension locality and scientific product-use journeys with independent physical/semantic expectations; hand actual population and limits to H. Publish enduring contracts and delete remaining replaced selected routes. Measurements, if claimed, cover complete preparation/use/readback rather than wire-only timing. |

## Verification and closure

**Proposed controls:** reconstruct the spatial control volume and preserve its refusal of
dynamics without a temporal inventory law. Use an existing supported dynamic model for the
positive operation, including supported integrated/simultaneous and transient-fitting routes.
A new inventory closure is an explicitly selected scientific extension, not manufactured by
the transport to make the negative case pass.

Independently specify locally valid covariance whose governing request lacks required
coverage, and useful samples from a nonselected accuracy attempt. Raw access may expose them;
ordinary scientific selection must refuse or correctly qualify use. Cover task/step/point,
missing and ambiguous facts, partial publication and subsequent derived products.

For reuse, contrast diagnostic/redaction/interface-only changes with changed selected data,
datum, applicability, derivative profile and layout; compare compatible value-only refresh.
Source-generated expected records alone cannot establish physical correctness. Use analytic
cases, independently transcribed references and original-model checks appropriate to each claim.

Compile and run targeted owner/consumer controls during implementation, with explicit Arrow
validation where consumed. [65h](65h-integration-and-qualification.md) owns assembled scope-end
qualification and measurements. No test or measurement was run to author this document.
Required scientific/kernel/metadata changes follow the coordinator's decision route before
implementation; no accepted ADR is amended here.

## Native source references

- [Route/procedure admission](../../crates/pse-modeling/src/analysis.rs) and
  [declared preparation/execution](../../crates/pse-runtime/src/workflow/modeling/declared.rs).
- [Property/data selection](../../packages/reference/domain/models/properties.pse),
  [spatial balance/refusal](../../packages/reference/process/models/control-volume-1d.pse), and
  [fitting source binding](../../crates/pse-runtime/src/workflow/fitting/modeling.rs).
- [Completion and product permissions](../../crates/pse-runtime/src/workflow/completion.rs) and
  [dynamic sample/final-attempt interpretation](../../crates/pse-runtime/src/workflow/modeling/dynamics.rs).
- [Granular modeling](../../crates/pse-compiler/src/workspace/modeling/granular.rs) and
  [numerical/validity dependency roles](../../crates/pse-compiler/src/workspace/modeling/executable/dependencies.rs).
