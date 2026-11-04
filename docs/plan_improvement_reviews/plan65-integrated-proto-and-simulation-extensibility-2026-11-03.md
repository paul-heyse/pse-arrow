---
title: "Plan65: Integrated process-domain, lifecycle and Proto improvements"
status: proposed
date: 2026-11-03
doc_role: review
doc_owner: docs/plans/65-proto-integrated-target.md
doc_topics: [design, libraries, numerical-execution, validation]
---

# Integrated process-domain, lifecycle and Proto improvements

## Combined judgment

**Proposed:** strengthen Plan65 around executable scientific operations, scoped product use
and complete developer/production evidence lifecycles, with Proto supplying faithful generated
projections and independently usable interfaces. Retain its mixed native/Proto/Arrow design,
but establish those semantic contracts before broad adapter generation and crate extraction.

Plan65 is a sound contract-infrastructure proposal. It explicitly preserves native scientific
meaning, physical settlement, historical interpretation and useful numerical representations.
Its present packages nevertheless leave scientific-operation composition and developer test
evidence less concrete than message generation. Implementing every listed projection would
not by itself demonstrate the fully integrated, extensible process system requested here.

The most important amendments are a connected group: source-owned scientific operation and
product-use contracts (I01–I03), explicit developer evidence and observation/authority boundaries
(I04–I06), and early integrated delivery (I11). The library inquiry adds concrete leverage in
lossless descriptor acquisition, semantic option checking/evolution and generated assurance
inputs (I07–I09). I10 is an optional release-gated simplification, not an assumed current API.

These recommendations fit the undertaking because Plan65 already changes source declarations,
adoption, scientific portability, operation interfaces, retained products and independent
inspection. They can improve those changes coherently without a new universal workflow engine,
global result store, simulator IR or ontology. Proposed breaking changes are eligible. This
review neither revises the plans nor accepts implementation or architectural decisions.

## Baseline, scope and method

Reviewed the coordinator and all eight companions at
`d2d82e5fdcadcdfdd5de12d18ac2746e7426fff4` on 2026-11-03. All 55 packages are proposed
remaining work. The completed-Plan64 baseline is an assumption for their later execution,
not a current implementation or qualification claim. Concurrent dirty projection/codegen,
runtime launch/stack and structural-incidence changes and the supplied Proto references were
observed and preserved. They were not accepted as tested behavior.

The selected horizon is process-system breadth: thermodynamics, reactions, equipment,
steady/dynamic/distributed models, studies, fitting, control and optimization. No atomistic,
remote/deployment or universal supported-physics promise follows. SurrealDB v4 was excluded.
No older plan is assigned ownership and no existing plan is changed.

The standard is core 3.4, efficient-architecture heuristics 1.0, process-simulator profile 1.5,
test-assessment 1.2 and the repository binding. All applicable foundations inform the scoped
judgment; this is an improvement review, not a formal design verdict, completion audit or
refresh of either diagnostic account.

Three read-only contributions examined library compositions, scientific domain integration,
and lifecycle/test results. The coordinator reconciled their recommendations and inspected
decisive source behind their conclusions. The prior end-to-end reading of all eight reference
documents, including addenda, was combined with new focused Context7, primary documentation,
installed-source and authored/runtime source investigation. Earlier research retains its dates.
No product tests, experimental probes, numerical runs or performance campaigns were performed.
Mechanisms are **Interface-checked** under named conditions; amendments and benefits remain
**Proposed**. Quantitative improvements are not claimed.

## Recommendation map

| Reference | Amendment | Principal affected packages | Character |
| --- | --- | --- | --- |
| [I01](#I01) | Scientific operations govern portable projections | A02/A05/A06, E01/E02/E06, F01, G02/G04, H01/H02 | Domain alignment/extensibility |
| [I02](#I02) | Scientific products expose governing use and scope | E02/E06, F01/F03/F04/F06, C03/C04, G02/G04, H02 | Integration |
| [I03](#I03) | Separate scientific dependency validity from interface evolution | A03/A06/A07, B01/B02, C03/C04, E01/E02/E04, H01/H03 | Execution fit/reuse |
| [I04](#I04) | Include developer test evidence as an actual migration population | A05, F01/F03, G01/G02/G04, H01/H02 | Missing target integration |
| [I05](#I05) | Prevent observation decoding from recreating native authority | A01/A05, B01/B02, D04/D05, E/F/G adoption, H02 | Preservation/alignment |
| [I06](#I06) | Close retained evidence over its required interpretation and original context | C03/C04, F03, G02/G04, H02 | Lifecycle integration |
| [I07](#I07) | Preserve custom-option meaning through complete descriptor acquisition | A02–A04, B01/B05/B07, G02/G04, H02 | Library leverage/assurance |
| [I08](#I08) | Share semantic option validation and evolution comparison | A02–A04, B01/B07, C01/C04, D03, G02, H02 | Library leverage/streamlining |
| [I09](#I09) | Use bounded generated inputs for representation/adoption assurance | A05/A07, B02/B05/B07, G01/G02, H01/H02 | Library leverage/test quality |
| [I10](#I10) | Compare released generated strict JSON with existing repeated parsing | B02/B05/B06, G02, H controls | Conditional simplification |
| [I11](#I11) | Establish semantic/lifecycle populations through early complete slices | P02/P03, A01/A02, E/F/G design, H01 | Sequencing/integration |

References identify recommended changes, not newly scheduled packets. Existing CBO-195/196,
042/040/046/066, 110/099/094, 008 and 045/084/013 retain their meanings in the
[Proto applicability account](../library_leverage/libraries/proto/applicability.md).
This review sharpens their integration; it does not close those broader occurrences.

## Scientific domain and extensibility

<a id="I01"></a>
### I01 — Make scientific operation contracts executable before projecting them

Plan65a specifies structural declarations and method effects; 65e specifies reconstructible
input, prepared demand and outcome. These are useful, but preservation statements do not yet
identify all scientific decisions that must stay linked when the same model serves several
analyses. The native domain already supplies those decisions; the improvement is to make their
composition explicit in the target and its acceptance, not to declare that science is absent.

**Recommend** a narrow source-owned scientific-operation contract connecting authored
phenomenon/formulation, material/quantity/data selection, route/procedure, required initial,
boundary and observation inputs, provider/derivative/validity capabilities, preparation and
evaluation, produced products and governing assessment. Checked native functions/types and
authored guards realize it. Generated options and clients expose the relevant views; generic
reflection must not reproduce physical admission. No new type is required for every concept.

Concrete foundations include
[`analysis::declared_policy`](../../crates/pse-modeling/src/analysis.rs), which resolves route
and procedure separately; [property selection/applicability](../../packages/reference/domain/models/properties.pse);
[spatial conservation and closure](../../packages/reference/process/models/control-volume-1d.pse);
and [fitting source binding](../../crates/pse-runtime/src/workflow/fitting/modeling.rs).
The spatial control volume explicitly refuses dynamics without temporal accumulation closure.
Matching descriptors, dimensions and fields cannot remove that scientific limitation.

Retaining separate operation owners is preferable where their semantics genuinely differ.
Consolidate repeated scientific decisions and expose explicit composition, rather than create
a workflow supertype. A single universal formulation would hide legitimate model differences.

**Closure:** preserve the spatial model's receiving-side refusal through portable reconstruction;
use an existing supported dynamic model for positive preparation, diagnostics and independent
inspection, including integrated/simultaneous routes and transient fitting. A new temporal-inventory
closure is a separately selected scientific extension: its support must originate at the physical
owner rather than generic transport. Broader control/optimization extensions must state
their required inputs and scientific support rather than inherit a generic operation enum.
This improves AP-04–AP-06, DP-02/DP-08/DP-15 and PS-01/PS-04/PS-06/PS-07/PS-11.

<a id="I02"></a>
### I02 — Expose the governing scientific use decision for every selected product

65f preserves diagnostic/result structure and 65h tests partial delivery. Strengthen this
into a positive result contract: independent readers consume the actual recorded product-use
decision and its scope rather than reconstructing eligibility from status, row presence,
quantity labels or transport ranges.

[`Completion`, `ProductScope` and `ProductPermission`](../../crates/pse-runtime/src/workflow/completion.rs)
already distinguish task/step/point, local validity and governing original decisions;
ambiguous permission lookup refuses. The
[dynamic observer](../../crates/pse-runtime/src/workflow/modeling/dynamics.rs) distinguishes
provisional samples, completed accuracy attempts, requested/check coverage and final selection.
These are stronger semantics than a generic successful response or available array.

**Recommend** projecting those decisions once through native selectors, portable outcomes,
retained products and independent clients. Keep raw candidate access, locally valid derived
products, incomplete governing checks, selected attempt, qualification and protected readability
distinct. A serialized old permission reports what its original owner decided; it grants no
new scientific authority in a changed context.

Raw diagnostic readers remain useful and may intentionally expose unusable data. Label that
operation explicitly instead of weakening ordinary scientific selection. This supports future
covariance, sensitivity, trajectory, study and optimization products without one whole-run flag.

**Closure:** independently specify a locally valid covariance product whose governing request
failed coverage, and samples from a nonselected accuracy attempt. Verify raw access retains
them while scientific selection refuses or qualifies use. Cover task/step/point, ambiguity,
partial publication and subsequent derived products. This improves AP-04/AP-05, DP-05/DP-19/
DP-21/DP-24, PS-10/PS-12 and independent evidence under test-assessment §1/§7.

<a id="I03"></a>
### I03 — Bind reuse and invalidation to the consuming scientific dependencies

Plan65 correctly distinguishes several identities, but does not yet specify which changes
invalidate mathematics, renew admission, regenerate a client or rebuild a stored projection.
Treating the entire descriptor world as a scientific key would overinvalidate; accepting an
unchanged wire shape after physical assumptions change would underinvalidate. Neither failure
is established in current execution by this review.

**Recommend** a small change/dependency account at existing consuming owners: distinguish
wire packaging/documentation/redaction/unused fields; validation policy; storage mapping;
physical convention/data/applicability/formulation/derivative support; compatible value refresh;
and mutable attempt/session lifetime. Reuse existing compiler dependencies and artifact identities,
not another global dependency registry or universal cache key.

The source already separates granular phases and physical scopes in
[`workspace/modeling`](../../crates/pse-compiler/src/workspace/modeling.rs),
[granular preparation](../../crates/pse-compiler/src/workspace/modeling/granular.rs), and
[numerical versus validity dependencies](../../crates/pse-compiler/src/workspace/modeling/executable/dependencies.rs).
Fitting bindings and worker generations supply further context-specific checks.

**Closure:** a diagnostic-only field or Debug-redaction change must not rebuild unchanged
mathematical structure; a selected correlation or enthalpy-datum change must invalidate or
readmit affected science even with identical field shapes. Contrast value sweeps with changed
layout/derivative profiles. Source and control evidence establish the dependency semantics;
quantitative speed benefits require measurements. This improves AP-01/AP-04/AP-07,
DP-04/DP-09–DP-11 and PS-07/PS-11.

## Complete lifecycle and developer evidence

<a id="I04"></a>
### I04 — Include developer test evidence in the actual contract population

G01/G02 name configuration, facts, graphs and investigations; F06 names scientific/host failures.
Neither explicitly migrates test workflows, reports, inventory, fixture context and exact reruns.
H01's later population enumeration cannot safely stand in for this design decision. This is a
target-integration omission, not evidence that the current test system is ad hoc throughout.

Native [`Action`, `Outcome`, `Report` and phase/group observations](../../crates/pse-host/src/test_workflow/execute/report.rs)
already distinguish list/test/check/rustdoc, zero match, pending/incomplete evidence, selected
population, child exits, captures, original inventory and fixture requirements. The
[runner](../../crates/pse-host/src/test_workflow/execute/runner.rs),
[fixture declarations](../../crates/pse-host/src/fixtures.rs),
[fixture inputs](../../crates/pse-host/src/fixtures/execution.rs) and
[exact failed rerun](../../crates/pse-host/src/test_workflow/source.rs) supply their real lifecycle.

**Recommend** an explicit developer-evidence slice through A/F/G/H. Generate useful observation
schemas/readers/projections from these owners and retain Nextest/correctness selection and
execution. Keep native JSON persistence where it remains the simplest complete representation;
independent boundaries, not uniformity, justify Proto adoption.

**Closure:** an independent reader distinguishes successful list/check/nonempty test execution,
a passing assertion that correctly expects physical refusal, actual assertion failure, zero
match, setup failure, incomplete capture and interruption. Rerun retains original report/input/
inventory identity and checks the present listing; old results never become current evidence.
This improves AP-04–AP-06, DP-05/DP-19/DP-21/DP-23 and test-assessment §§1/4/6.

<a id="I05"></a>
### I05 — Make observation-to-authority limits explicit in generated adoption

A05 proposes native→wire→native correspondence. B01/B02 retain contextual admission, but the
generic mechanism needs an explicit category for native capabilities that cannot have inverse
wire constructors. This is a preservation recommendation, not an observed current vulnerability.

The native test `Report` is serializable without general deserialization into private verdict
capabilities. [`validate_report`](../../crates/pse-host/src/managed/workflow.rs) compares the
waited helper exit with its terminal, checks original private file ownership/extent and hashes
the full report. [`TerminalResult`](../../crates/pse-host/src/managed/result.rs) retains private
control/stream/error ownership. Completion exporters similarly copy scientific decisions.

**Recommend** owner-declared direction and construction roles: requests require trusted native
admission; evidence may be decoded as observations; owner-sealed verdict, control, publication
and settlement capabilities have no generic wire adoption. Unknown or unsupported capability
conversion fails generation explicitly. Do not classify authority later by type-name strings.

Manual adapter review alone leaves a repeated high-consequence convention as generic adoption
expands. Explicit owner-local construction boundaries improve local reasoning without a new
certificate framework. Observational records can still be portable and independently readable.

**Closure:** compile/owner controls prevent decoded messages from constructing native passes,
readiness, publication grants or drain proof. Exercise foreign reports, mismatched waited status,
wrong extent/digest and valid historical observations without current authority. This improves
AP-04–AP-06, DP-03–DP-05/DP-18–DP-21 and PS-10.

<a id="I06"></a>
### I06 — Define the complete evidence and interpretation closure per result family

C/F/G describe retained worlds and protected pages. Complete their integration by defining what
each promised reader actually needs: original selection/context, report/inventory or immutable
manifest/payload, relevant interpretation/policy and distinct protection/retention relations.

Developer reports publish the full report before a digest-bound terminal. Scientific
[`ResultAcknowledgments` and close](../../crates/pse-operations/src/canonical_execution.rs)
reconcile producer/append/manifest state;
[`canonical_results`](../../crates/pse-operations/src/canonical_results.rs) protects immutable
terminal reads, including older attempts. Native invocation and
[inventory/failure references](../../crates/pse-host/src/test_inventory.rs) have different original
evidence dependencies. They should remain distinct authorities within a coherent reader design.

**Recommend** explicit absent/expired/incomplete/incompatible/corrupt read outcomes and reference
closure at existing owners. Retained observations do not pin live fixtures; protected scientific
payloads do not pin every evaluator/worker. Preserve failed release observations and active-reader
protection without introducing a global evidence store or permanent interpretation cache.

**Closure:** retain a report while removing its inventory; remove an old required policy; corrupt
a payload; expire protection; retire with an active reader; fail release; read after a newer
attempt. Distinguish exact rerun, effect recovery, forwarding and scientific adoption. This
improves AP-02/AP-04/AP-07, DP-04/DP-09/DP-19–DP-24 and PS-10/PS-12.

## Additional concrete library leverage

<a id="I07"></a>
### I07 — Preserve semantic options through the complete descriptor acquisition chain

A03 fixes runtime embedding, but required metadata preservation begins before embedding.
A linking descriptor set can already have lost custom extension options through a typed
intermediary. Plan65 makes those options executable interpretation, so field/tag preservation
alone is insufficient.

**Recommend** checking compiler output → image/descriptor conversion → import normalization →
generator request → effective/runtime/inspection publication. A concrete library capability is
[`protox::Compiler::encode_file_descriptor_set`](https://docs.rs/protox/0.9.1/protox/struct.Compiler.html#method.encode_file_descriptor_set):
the versioned documentation distinguishes its extension-option preservation from re-encoding
the returned `prost_types::FileDescriptorSet`. `include_imports` and `include_source_info` govern
different material. The supplied compiler reference describes the corresponding 0.10 route.

This does not require adopting Protox or establish that the current Buffa route loses options.
It supplies a real loss trap and a complete capability against which actual selected producers
must be checked. Logical preservation, intentional feature/source transformations and exact
archival byte identity are separate promises.

**Closure:** independently specify imported message/field/method options; challenge lossy typed
conversion, missing imports, unknown options and nondefault effective features. Observe required
meaning at each real consumer, including an independent client. This improves DP-02/DP-08/
DP-24 and PS-13 without demanding byte equality for intentionally transformed artifacts.

<a id="I08"></a>
### I08 — Share typed-option semantic checking and meaningful evolution comparison

A02 defines typed options; A/B/C/D/G consume them. Buf wire compatibility does not detect
changed option meaning, and separate producer/importer/inspector checks could reconstruct
the same policy independently.

**Recommend** one native semantic checker over the approved world, reused by generation,
import admission and offline inspection. Check required metadata, known versions, coherent
references and declared effect/interpretation relationships. Compare old/new worlds with
compatible/incompatible/unknown outcomes for meaningful option changes. Native scientific
owners still decide physical admissibility; a metadata checker cannot establish physics.

[Buf check plugins](https://buf.build/docs/bsr/checks/plugins/) provide descriptor handlers,
rule specifications and source-attached annotations, with local/Wasm/remote configurations.
They are distinct from standard protoc generator plugins. The documented SDK is Go; a native
adapter must honor Bufplugin/PluginRPC contracts. Start with the actual Rust checker and add
a Buf adapter only when an independent authoring/publication workflow earns its burden.
Do not introduce another rule authority or Go subsystem solely to check the same thing twice.

**Closure:** independently create wire-compatible descriptor pairs changing a scientific
reference, validation interpretation or effect declaration; require precise source attribution
and explicit decision scope. This completes the planned metadata composition and improves
AP-04/AP-05, DP-01/DP-24. It does not certify dynamic admission or scientific equivalence.

<a id="I09"></a>
### I09 — Use generated inputs for a defined assurance role, with independent expectations

A07/B07/G/H require tests but leave a concrete useful library mechanism unselected.
Installed Buffa-codegen 0.9.2 exposes `CodeGenConfig::generate_arbitrary`; output is gated
behind the consumer's `arbitrary` feature, including dedicated handling for bytes-backed fields.
See [versioned generator source](https://github.com/anthropics/buffa/blob/v0.9.2/buffa-codegen/src/lib.rs)
and the [Buffa reference](../libraries/proto/Buffa.md), generation/feature-gating sections.

**Recommend** bounded generated construction and controlled mutation for selected recursive,
alternative, presence, unknown-field and extent populations. Use owned/view/dynamic/independent
runtime comparisons, retaining input bytes/seed, exact world and existing test failure identity.
Integrate with I04's original reports and reruns rather than another result store.

Generated structural values may violate application rules; generated expected values can
repeat the same wrong declaration. Keep independent format, physical, derivative and historical
cases for their respective claims. Qualify the actual custom-storage/extern/view feature closure;
do not force Arbitrary everywhere or turn the amendment into an unbounded fuzzing campaign.

**Closure:** demonstrate expected acceptance, preservation or rejection of independently specified
field/order/presence/extent perturbations, including legal wire reordering and selected unknowns;
reproduce failures through native retained selection. Report codec agreement, semantic adoption,
lifecycle reconciliation and scientific correctness separately. This improves AP-06, DP-16/
DP-23 and test-assessment §§1–4/6.

<a id="I10"></a>
### I10 — Add a release-gated generated strict-JSON simplification branch

The supplied Buffa addendum distinguishes `deny_unknown_json_fields` on upstream main from
tagged 0.9.2, where it is absent. Current upstream documentation exposes global/per-message
selection, while extern-mapped types retain their own policy. See the
[upstream guide](https://github.com/anthropics/buffa/blob/main/docs/guide.md); this is development
documentation evidence, not an installed released capability.

**Recommend conditionally** comparing an actually released, verified generated strict-JSON
composition with the host's intermediate-tree/key-checking path. B02/G02 already allow removing
repeated parsing with equivalent protection; this amendment gives that investigation a concrete
library route and a release gate.

Keep duplicate logical aliases, trailing-data, external-type and pre-allocation controls where
required. Unknown-key rejection alone cannot replace them. If a supported release/composition
does not simplify the complete path, retain current strict inspection rather than depend on
main. Closure requires malformed/unknown/alias/trailing/size countercontrols before deletion;
any claimed allocation/latency improvement needs matched boundary measurement. This is an
optional AP-07/DP-14/DP-16 improvement, not a new required upgrade.

## Integrate the amendments early

<a id="I11"></a>
### I11 — Establish semantic populations before finalizing broad generation interfaces

Plan65 permits narrow slices, which should be retained. H01 nevertheless places its explicit
acceptance-population reconciliation after working A–G contracts. That is too late to be the
first concrete decision about developer evidence, scientific product meaning or sealed authority.

**Recommend** an early semantic-population/design slice co-designed with A01/A02, E/F/G and
the first real consumers. It defines required meanings, transitions, effects, owner construction,
observation versus live context, migration population and independent discriminators. H01 retains
final assembled population reconciliation. No permanent workflow inventory or universal state
machine is required.

First deliver a native test workflow through complete retained report/independent inspection,
and a scientific operation through partial results, governing qualification and protected readback.
Include cancellation or acknowledgement loss where readable evidence and physical settlement
differ. These cases must shape the common foundation before its broad extraction/migration;
they need not await every registry, database or provider projection.

**Closure:** both slices preserve original verdict, effect, publication and cleanup owners, and
adding another outcome/observation family changes its semantic owner and intended consumers
locally. Package-local checks remain targeted; broad campaigns stay at functional scope end.
This improves all applicable AP foundations, DP-05/DP-16–DP-19/DP-23 and test-assessment §§2/5/8.

## Interactions, priority and retained choices

I01/I02/I04/I05 define the meanings the shared foundation must preserve. I03/I06 define their
change and retention lifetimes. I07/I08 ensure required metadata reaches consumers intact and
is interpreted consistently. I09 strengthens selected mechanical assurance; it cannot independently
validate I01/I02's science. I10 is optional and may remain unselected. I11 orders the work so these
semantics inform the foundation instead of becoming late fixes around it.

Consequence priority is highest for scientific product use and observation/authority boundaries;
prerequisite order starts with the early consumer contracts and actual metadata acquisition.
The pure `pse-contract` proposal remains plausible if those consumers prove the shared need.
Do not move scientific or native verdict behavior into it to make sharing convenient. Reassess
its exact scope against the complete slices rather than rejecting extraction merely for size.

Retain native numerical/evaluator/solver representations, Arrow bulk products, external
Prost/Tonic/OTLP/Perfetto contracts, static generated validation for stable first-party schemas,
explicit per-handle/session/process custody, actual capability refusal and independent physical
expectations. Views/segmentation already have good conditional treatment; more blanket adoption
is not a new improvement. Dynamic CEL, database-local execution and replay remain consumer-led
choices. No new management daemon, broker, global evidence store, second canonical database,
remote/browser delivery or blanket generated runtime is justified by this review.

## Decision implications and adoption route

| Proposed impact | Consequence if selected |
| --- | --- |
| Scientific operation/product semantics and sealed construction directions | Amend metadata/adoption and affected scientific interface decisions; native guards and owner authority stay explicit. R01/R02/R03 in the coordinator may need a more precise scope. |
| Explicit developer evidence and retained interpretation population | Expand the actual selected migrations and acceptance; retain Nextest/correctness/native report authority and distinct storage lifetimes. |
| Semantic option checking/evolution and descriptor preservation | Add shared generation/import/inspection controls; any Buf adapter or new dependency follows actual pin/tooling and decision obligations. |
| Original reader/result contract breaks | Apply the coordinator's explicit migration/retained-read decision route; do not silently reinterpret history or retain dual production paths. |

These are proposed impacts for subsequent plan revision and applicable ADR/formal-review routes,
not applied authority changes. No standalone broader undertaking is selected. New scientific
algorithms/families, distributed deployment, general event sourcing or test-platform replacement
would need their own concrete value and target decision if later proposed.

The next useful revision should integrate I01–I09/I11 coherently and explicitly dispose I10,
preserving these anchors, source mechanisms and verification distinctions. It may amend existing
companions or add a lifecycle/domain companion if that makes the reasoning clearer; another
document is not the improvement itself. This review does not change package status or completion.

## Coverage and verification limits

All nine Plan65 documents and their combined remaining footprint were considered through the
three responsibility investigations and coordinator synthesis. Decisive source samples covered
authored properties/control volumes, analysis/fitting/dynamics, compiler dependencies, completion,
canonical publication/readback, native test reports/rerun/fixtures/inventory and managed invocation
authority. Relevant neighboring consumers were followed for specific questions. This was not an
exhaustive implementation audit of every solver, test runner phase or chemical workflow.

Library research combined the eight supplied references and earlier compiler/generator/Rust-first
material with new Context7 lookups, exact installed Buffa source and primary Protox/Buf documentation.
Sparse Context7 results were not treated as capability absence. Main-only strict-JSON documentation
was not promoted to released support. Default/patched local behavior remains distinct from upstream.

No runtime probe was necessary to establish these amendment proposals; unverified runtime
composition remains a named acceptance obligation. No new scientific support, performance benefit,
host/tool availability, product qualification or formal architecture acceptance is claimed.

A fresh synthesis check challenged the combined draft. It corrected the distinction between
new scientific implementation and an extension challenge, and made perturbation acceptance
format-specific rather than blanket rejection. These corrections preserve supported legal
wire behavior and keep newly selected physics behind its explicit target decision.
