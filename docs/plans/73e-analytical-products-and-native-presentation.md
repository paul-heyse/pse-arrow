---
title: "Plan73e: Analytical products and native presentation"
status: draft
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73e-analytical-products-and-native-presentation.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources:
  - docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md
  - docs/functional_reviews/symbolica-ecosystem/structure-and-research.md
---

# Analytical products and native presentation

## Owned outcome and baseline

This companion of [Plan73](73-symbolica-ecosystem-improvements.md) owns O11,
O13 and the indexed mathematical part of O08. A developer can inspect a selected admitted
caloric law's derivative, local series and exact identity, inspect a small indexed
constitutive contraction under legitimate dummy renaming, and read native mathematical
views with their original physical/source interpretation. The result is useful through the
existing protected reader, not only as a library demonstration.

This new series fully executes before Plan66 resumes. This plan owns the necessary analytical
and presentation migrations, deletions and acceptance even where older plans describe related
mathematics or native views. It neither edits their status nor waits for their outstanding
packages. Companion A owns mapped-instance binding transport and the shared admitted product;
companion G owns topology exploration. Neither graph-assisted motif matching nor a new tensor
execution stack is required to complete this plan's selected indexed analytical product.

Planning examines baseline `e9f4e7b7`, with the source review's earlier observations retained
at their original evidence dates. **Interface-checked:** maintained Symbolica 3.0.1 derivative,
series, indexed canonicalization and printer entry points; current representative authored
caloric and protected-reader owners. **Proposed:** all new behavior, migrations, decisions and
verification below. No scientific code, compilation, tests or measurements were produced in
plan authoring.

The governing context is blueprint §§7, 8, 14, 19, 21.7, 23: typed authored relations remain
model authority; Symbolica/Numerica own mathematics; workflow results own historical outcomes;
reading is observation of an original protected product. The focused foundation assessment
uses core 3.4, efficiency heuristics 1.0 and process-simulator 1.5. It schedules bounded product
composition rather than a new CAS service, a parallel authoring authority or scientific policy
inside the independent native host.

## What changes and what becomes possible

This document delivers new selected analytical operations and a new retained mathematical view.
It also changes the shared compiler/result path needed to make those operations real product
capabilities. A native supplier example or a formula printed from current source is insufficient.

| Scope | Resulting change | Owner |
| --- | --- | --- |
| New developer capability | Inspect a selected caloric definition's derivative, remainder-aware series and exact identity under its original scientific premises. | [E01](#e01), [E02](#e02) |
| New indexed capability plus compiler change | Preserve authored finite reductions/index references before scalar specialization, then compare supported contractions under legitimate dummy renaming. | [E03](#e03) |
| Cross-cut publication/reader change | Analytical and D's exact/assurance producers persist native formula views plus physical/source correspondence; protected readers observe that original stored product after source changes. | [E04](#e04) |
| Bounded extensions | Decide integration, approximation, relation discovery, exact special-function representation, Spenso, rich Typst and combinatorics against the concrete cases below; deliver any selected extension fully. | [E05](#e05) |

Existing derivative execution remains science, authored DSL rendering remains authoring, and
new analytical inspection has its own result meaning. The source review identifies possible
consolidation categories rather than existing implementations in every category. No baseline
general analytical inspector, dummy-index canonicalizer or derived-math pretty-printer is
claimed to exist. Necessary changes can span compiler, mathematics, registry, runtime producer
and reader; the caloric and indexed journeys establish acceptance without limiting that footprint
to one consumer. Delete actual superseded adapters when identified at these cutovers, never
invent a legacy pathway just to list a deletion.

## Target and enabling composition

### Selected analysis retains original scientific meaning

The compiler/model owner admits a request selecting a resolved expression, named definition
or small equation block, the operation and coordinates, expansion point/order when applicable,
and current physical/domain/provider support. It consumes companion A's qualified original
product and bindings without reconstructing names from printed formulas. The scientific
operation returns a typed derived result under the existing result custody path. D01 supplies
the shared early exact/assurance claim vocabulary; only that contract is required, not all of
D's numerical realizations.

Keep the selected source version, declaration/field/occurrence identities, variable and
derivative-axis maps, original guard and validity premises, units/basis/reference state,
supplier provenance, actual operation and support/refusal meaning. These are conceptual
requirements, not a requirement to create a separate object/service for each fact. Source
edits make new requests use new products; retained old artifacts continue to identify their
old premise. Equal expression text never merges physical occurrences. Unsupported whole-expression
analysis can yield an explicitly selected supported local region or refuse the request; it
cannot report success after silently dropping an opaque provider or branch.

The existing [compiler executable owner](../../crates/pse-compiler/src/workspace/modeling/executable.rs),
[typed mathematical bridge](../../crates/pse-compiler/src/typed_math.rs) and
[mathematical library owner](../../crates/pse-math/src/library.rs) are integration foundations.
Migrate affected selection, derivative and expression-inspection clients to the admitted
product shared by A and this operation. Do not introduce a whole-model flattened CAS path,
a process-global analytical cache or a serializable second instruction language.

### Selected caloric derivative, series and identity

Use native `AtomCore::derivative`, `AtomCore::series` and exact polynomial/rational normalization
on the selected original product. The maintained Symbolica 3.0.1
[public analytical operations](../../vendor/symbolica/src/atom/core.rs),
[derivative implementation](../../vendor/symbolica/src/derivative.rs) and
[series implementation](../../vendor/symbolica/src/poly/series.rs) establish the composition.
`series` returns `Series<AtomField>` with an expansion point and order; `SeriesDepth` distinguishes
absolute from relative depth. Retain `relative_order`, `absolute_order`, ramification/shift
meaning and expansion point in the result's supported projection. Converting only with
`to_atom` loses omitted-order information. An exact finite polynomial may have no finite
truncation order; the projection must distinguish exact termination from an omitted tail
without calling finite-order accessors on an infinite-order sentinel.

Identity checking is exact normalization of the selected difference under admitted physical
and domain conditions, or exact coefficient comparison in the fitting polynomial/rational
field. A nonzero normal form or inconclusive unsupported transformation is not a numerical
falsification of physical validity; report the actual mathematical result and premises. The
[exact conversion/domain account](../library_leverage/libraries/symbolica/exact-algebra-and-solving.md)
explains why exact algebraic identity, numerical agreement and empirical applicability remain
different claims. Do not add a local derivative, series or polynomial-algebra engine.

The first delivered scientific consumer is
[`CaloricReference`](../../packages/reference/methods/models/caloric.pse), through the
[caloric property facades](../../packages/reference/domain/models/properties.pse) and their
selected authored correlation definitions. They already distinguish heat capacity, enthalpy
and entropy increments, applicability intervals and reference values. Retain the consumer
requirements in [caloric numerical policy](../../crates/pse-math/src/numerics/caloric.rs),
including temperature resolution and datum-bearing enthalpy. Analysis never substitutes a
fitted decimal with a convenient rational, changes a datum or broadens a correlation's envelope.

Deliver an exact small polynomial caloric law and a selected ordinary authored caloric
correlation. For the revealing independent polynomial case, let the temperature-interval
coordinate be `θ = T - T_reference` and `cp = a + b*θ`. The expected enthalpy increment is
`a*θ + b*θ²/2`, with the original `h_reference` retained. Its derivative and series coefficients
follow directly from that law, independently of the supplier transformation. Entropy's
expected derivative is `cp/T_absolute` on positive absolute temperature, with the corresponding
physical derivative type and reference entropy retained. A log-containing entropy expression
can support exact symbolic inspection without qualifying for D's arithmetic-only enclosure.
The ordinary correlation checks source selection, callable expansion, parameter binding and
physical attribution beyond a synthetic formula. Unsupported provider derivatives remain
explicit; additional selected-source lowering needed for this actual consumer belongs here
and in A's shared contract, not in a deferred old-plan package.

### Selected native indexed canonicalization

Offer a bounded analytical request for an admitted finite indexed constitutive expression,
not a raw Atom parser. Its typed selection identifies finite index groups, declared mathematical
symmetries, fixed external axes and the resolved component expressions/bindings. Preserve that
structure from the compiler's finite memberships before flattening; do not infer a contraction
from coincident scalar expression text. If a missing indexed view/admission contract is needed,
E01 owns that decision and E03 its implementation through ordinary typed source/operation
selection. This adds no second tensor authoring language or automatic executable tensor kernel.

Select a small constitutive contraction such as `J_i = sum_j L_ij*X_j`. Only the dummy index
`j` may rename within its declared group. The external axis `i`, species/component identities,
ports, quantity dimensions and provider occurrences stay fixed. A mathematical rename must
not become a physical component permutation. The result retains the canonical mathematics and
the native external/dummy index lists plus separately retained compiler source/binder attribution. These lists are not an original-to-canonical dummy permutation; later numerical preparation must not infer such a permutation from them.

The concrete authoring route is the existing DSL's typed finite set/entity declarations,
indexed variable/parameter families (`var x[j in items]: Scalar`), indexed references (`x[j]`),
and `sum(j in items | expression)` with an optional admitted membership filter. The resolved
owner already distinguishes `Node::Reduce { kind, binder, filter, body }`,
`Reference::{Lexical,Indexed}`, `Carrier` and `IndexedGroup` in
[resolved.rs](../../crates/pse-modeling/src/resolved.rs). `Arena::finite_folds` / `FiniteFold`
record actual ordered selected memberships for constructed folds, and `node_sources` retain
source attribution. They are not a general tensor symmetry declaration. In contrast,
[typed_math/direct.rs](../../crates/pse-compiler/src/typed_math/direct.rs) explicitly refuses
unselected `Node::Reduce`, `Node::Fold` and indexed references at scalar construction. E03
must project the selected original reduction/binder/group correspondence before that scalar
boundary; an already expanded sum cannot reliably reconstruct it. Identity symmetry is enough
for the first constitutive case. Additional tensor symmetries require an actual admitted
declaration/operation premise, rather than being inferred from equal component values.

Call `AtomCore::canonize_tensors` on that admitted indexed view. In maintained Symbolica 3.0.1
[the public contract](../../vendor/symbolica/src/atom/core.rs) represents tensors as functions
with index arguments; [the native implementation and tests](../../vendor/symbolica/src/tensors.rs)
build `TensorGraphNode`, invoke Graphica canonicalization and reconstruct mathematics. It
returns `canonical_form`, `external_indices` and ordered `dummy_indices`, respects group
markers and diagnoses inconsistent open indices or invalid repeated contraction. Graphica
3.0.1 is the locked supplier used internally; a second application graph serialization or
dummy-index canonicalizer would duplicate that complete composition.

Canonical equality establishes the admitted mathematical comparison only. It neither equates
physical instances nor proves arbitrary independently authored models interchangeable.
Retain ordinary scalar preparation for the small fixed numerical case; this analytical
capability does not replace faer matrices or require Spenso. Future optional numerical
preparation can consume the canonical result through A only if coordinate and provider
contracts remain supported and exact; unsupported execution is visible.

### Native mathematical views over protected originals

Use `AtomCore::printer` with native `PrintOptions::{latex,typst}` for derived
mathematics, joined to the original product's physical/source labels. The maintained
Symbolica 3.0.1 [printer entry point](../../vendor/symbolica/src/atom/core.rs) and
[native formatting implementation](../../vendor/symbolica/src/printer.rs) are the relied-on
mechanism; the options themselves come from Numerica 3.0.1
[`PrintOptions`](https://docs.rs/numerica/3.0.1/numerica/printer/struct.PrintOptions.html).
Preserve namespace distinctions, unambiguous bound/free symbols and actual
unsupported fields. Text escaping and names/units/layout are projection duties; do not
implement another mathematical pretty-printer.

The DSL renderer continues serving authored declarations and round-trip edits. Mathematical
printer strings are neither identity keys nor lossless symbolic/scientific codecs. An equation
view cannot replace units, reference state, applicability, source mapping, claim strength or
partiality. No formatted string is reparsed into an execution product or accepted as proof.
Use native expression formatting as a view of an admitted protected artifact, never as a way to
reconstruct the result's original science from current source.

Extend the actual [protected result reader](../../crates/pse-runtime/src/workflow/result_inspection.rs),
[result projection](../../crates/pse-runtime/src/workflow/result_projection.rs) and
[physical inspection](../../crates/pse-runtime/src/workflow/physical_inspection.rs) where they
consume these derived products. Retain exact original run, terminal attempt, selected relation,
closure manifest, field selection and permission. Capture the mathematical/source projection
needed for later reading under original result custody at production time: native mathematical
text for the selected formats plus its source/physical correspondence and claim fields. The
reader lays out those protected projections; it does not need live Symbolica callbacks or a
new symbolic import to display an old artifact. Reading/rendering
must work after source evolution and without recompiling, reevaluating, rerunning scientific
admission or acquiring broader payload permission. A page/document limit returns explicit
bounded observation and continuation, not a claim that the original analytical request
succeeded on a smaller scope.

The same native presentation serves D's exact/assurance mathematics. This is the selected
structural overlap with the existing native-view work: E owns implementing the mathematical
projection, original-result reader migration and necessary deletion now. General stream/plot
population beyond the selected analytical/exact consumer remains with its normal owner.

### Conditional analytical and presentation leads

E05 must settle the following bounded investigations. A favorable result includes its selected
implementation, consumers, tests and deletions within this series; an unfavorable result records
the failed criterion and an observable reopen condition. A library's mere existence is neither
adoption nor evidence of absence. These decisions do not block E02–E04 unless they change an
actual consumed contract.

| Lead | Named case and question | Decision consequence and independent check |
| --- | --- | --- |
| Symbolic integration | For E02's `cp=a+b*(T-T_reference)` and the selected authored caloric correlation, can separate `symbolica-integrate` (`Integrate`; overview lead 2.0.1) produce a native antiderivative compatible with maintained Symbolica 3.0.1, with finite/cancellable invocation and an explicit domain/result contract? Exact compatible release/revision and feature closure remain to be established; the core supplier is not assumed to contain integration. | If compatible and bounded, add an explicitly requested antiderivative result, recover the reference constant, differentiate back independently against the authored cp and reject definite intervals crossing poles/branches or applicability limits. Otherwise record which compatibility/resource condition fails and reopen on a compatible API/revision or concrete missing caloric integral. Derivative/series/identity still ship; no handwritten fallback. |
| Padé, interpolation and executable series | For the selected log-containing caloric entropy law near its admitted positive `T_reference`, does a native rational approximant or coefficient interpolation provide a useful *separate approximation artifact* beyond E02's remainder-aware series? Resolve the exact native API/release, zero-pivot/pole and insufficient-data behavior before selection; the overview alone is not an API contract. | Select only an explicit approximation product with point/order/domain and changed-model meaning, checking independent values and poles against the original law. A production surrogate additionally needs an authored scope/error contract and qualified value/derivative backend; without them choose analytical-only or not selected. Formal omitted order cannot fill an interval-error field. Reopen executable use on a named consumer plus that contract. |
| PSLQ/integer relations and lattices | Offer an opt-in research comparison for high-precision values of a known exact coefficient triple such as `(1,sqrt(2),2*sqrt(2))`, then an empirical coefficient triple with no promised identity. Can Numerica 3.0.1 `Integer::solve_integer_relation(x,tolerance,max_iter,max_coeff,gamma)` return a useful bounded candidate with honest stopping semantics? Its documented iteration exhaustion may return the current best vector; `Ok` is not convergence/proof. | If selected, admit nonempty/nondegenerate inputs, retain working precision/tolerance/limits and verify a candidate on the original exact product separately. The known exact case has relation `(0,-2,1)` up to scale, not a required particular PSLQ vector; the empirical case must remain unverified discovery. Do not automatically rationalize data. A lattice backend is researched only if this case needs it; otherwise record no lattice need and reopen on a named candidate-recovery case. |
| Algebraic extensions and special functions | Can E02/E03's actual derivative/series/identity output represent its exact coefficients and heads without loss? E02 entropy already requires logarithmic mathematics; a selected exact irrational coefficient is the discriminator for algebraic embedding/branch retention. Inspect the concrete Symbolica 3.0.1 coefficient/function representation and chosen output backend, rather than admitting every builtin. | Preserve exact output/branch/embedding and report unavailable numerical realization. If the selected cases need no extension, record that result; reopen for the first actual unsupported exact result. Any new executable primitive follows its decision route. A decimal/radical rewrite for appearance is not a replacement. |
| Spenso contraction | On E03's finite `J_i=sum_j L_ij*X_j` with the same declared external axes and finite membership, would Spenso 0.6.0 `Contract`, `DenseTensor`/`SparseTensor` and `TensorNetwork` improve the complete repeated numerical route compared with scalar preparation? Establish exact type compatibility, contraction ordering/intermediate retention and recovery of physical axes. | For this small fixed case scalar preparation is the baseline. Adopt only if the actual structured/sparse case warrants the full route, then implement value/required derivative support and compare with an independently hand-computed 2-by-2 contraction. Otherwise keep native canonicalization for equality and ordinary scalar execution, reopening for a named larger/sparse constitutive contraction. Retain faer's matrix role. |
| Structured Typst modules | Use E04's concrete caloric analysis page containing formula, source mapping, units/reference datum, omitted series order and unsupported-region annotations. Is structure needed that native `PrintOptions::typst` plus ordinary layout cannot express? Inspect `payload`, `math_display`, `typst_ast` and native attachment/callback custody at an exactly pinned revision. Published plugin 0.1.0 and development 0.2.0 are distinct evidence; native Rust compatibility is unestablished. | Adopt only if the page's actual structural need justifies the additional closure and stored readback stays independent of live callbacks/science. Compare protected readback and namespace/escaping output with the simple printer path; delete its replaced adapter only if adoption occurs. Otherwise retain printer plus layout and reopen on a named annotation/layout requirement it cannot represent. |
| Native combinatorial iterators | E02's delivered derivatives/series are univariate, so they do not need a custom multi-index enumerator. Does any selected extension actually introduce one, for example a bivariate caloric parameter/temperature coefficient request? Inspect Numerica 3.0.1 combinations/permutations/multiset-partition API and multiplicity/materialization for that exact need. | With only the delivered univariate cases, record not selected and no local iterator to delete. If a selected extension needs enumeration, use the native fitting iterator and verify a hand-enumerated degree-two index set and multiplicities before deleting its actual local loop. Reopen for a requested mixed-index analytical product, not a generic combinatorics service. |

Source support and limits are in the [analytical source account](../library_leverage/libraries/symbolica/exact-algebra-and-solving.md),
[ecosystem source account](../library_leverage/libraries/symbolica/ecosystem.md),
[Spenso 0.6.0 primary API](https://docs.rs/spenso/0.6.0/spenso/) and
[Typst primary README](https://github.com/symbolica-dev/symbolica-typst-plugin/blob/main/README.md).
The integration companion's refreshed native compatibility was not established by the review;
E05 must obtain it rather than turn the lead into a current guarantee. Numerica's selected
relation API is documented in [3.0.1 Integer](https://docs.rs/numerica/3.0.1/numerica/domains/integer/enum.Integer.html)
and installed `domains/integer.rs`; numerical relation discovery is never promoted directly to
exact identity. E05 records source revisions with the resulting decisions.

WASM and Python remain deployment/reference alternatives, not changes to native product
science in this series. HEPKit, Idenso, Vakint and AMFlow have no selected process-science
adoption from this evidence; a named process-engineering workflow with a complete relevant
contract is their reopen condition. Symbolica-ODE remains an integration reference; it does not
replace current IDAS/Diffsol consistency, event or sensitivity semantics. These dispositions
preserve the review's leads without creating a second solver stack or denying library eligibility.

## Packages

The coordinator owns overall O08/O11/O13 disposition; this table owns these packages' live
progress. All start open. A supplies working admitted products and coordinate correspondence;
B supplies maintained-source/provenance and any transferred-product receiving contract actually
used. Neither dependency waits for old Plan66 packages. Cross-companion shared vendor/registry
files require editing coordination even where package prerequisites are otherwise independent.

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [E01](#e01) | open | Coordinator decision route; A's admitted-product/coordinate contract; D01 claim vocabulary | Selected analytical/indexed request and original-result semantics admitted, including physical/source support and honest refusal. |
| [E02](#e02) | open | E01; A's working selected definitions/derivative product | Real caloric derivative/series/identity operation delivered, including ordinary authored correlation and independent exact case. |
| [E03](#e03) | open | E01; compiler finite-index membership and A binding correspondence | Native indexed canonicalization integrated for one admitted constitutive family with fixed physical/external axes. |
| [E04](#e04) | open | E02–E03 result producers; D01 result envelope, D results as available | Native mathematical printers and protected original reader/export migrated, with source/physical correspondence and replaced presentation paths removed. |
| [E05](#e05) | open | E02–E04 named consumer cases; exact-version companion evidence | Conditional lead decisions closed; selected extensions fully delivered; remaining leads carry specific evidence and reopen conditions. |

<a id="e01"></a>
### E01 — Admit selected analytical and indexed operations

The request selects an original definition/expression or bounded indexed region, operation,
physical derivative coordinates and optional expansion point/order. The result retains actual
scope, source version/declaration/occurrence, physical units/basis/datum, guard/applicability,
provider support and the operation's exact/approximate/unsupported meaning. D01's shared claim
contract supplies the early prerequisite; the other D realizations are unnecessary to begin.
Opaque regions can be refused or explicitly selected as a supported local request; they cannot
be silently omitted from a successful whole-expression response.

**Footprint and readiness.** Extend the existing compiler scientific operation boundary in
`workspace/modeling/executable.rs`, `typed_math` and `pse-math/src/library.rs`, with typed runtime
request/result projections and authored relation declarations regenerated through `just codegen`.
A must issue the selected original product and coordinate/source map; the coordinator routes
new operation and any indexed admission contracts first. Native Symbolica 3.0.1 derivative,
series and tensor APIs described in E02/E03 own mathematics. Do not put raw Atom parsing or
scientific policy in the native host.

**Migration/deletion.** These are new analytical variants: no baseline general inspector or
parallel tensor authoring language is identified. Reuse the existing original-product selection
owner instead of adding another name reconstruction/flattening path. If the current selection
adapter must change for both executable and analytical consumers, migrate those consumers in
the same change and delete its superseded branch; do not promise deletion of unspecified
expression-dump code.

**Independent acceptance.** A selected real definition succeeds with its authored physical IDs;
a wrong derivative axis, changed guard, absent provider support and unsupported whole-region
selection refuse with source attribution. An explicitly selected supported subregion remains a
different request. A source edit cannot relabel a stored old result as current.

<a id="e02"></a>
### E02 — Deliver caloric analytical inspection

Use Symbolica 3.0.1 `AtomCore::{derivative,series}` and exact polynomial/rational normalization
over A's original selected product, through the source contracts above. A series result retains
expansion point, `SeriesDepth` / actual relative and absolute order, ramification/shift and
exact-termination status; `to_atom` alone is only a view. Do not invoke finite-order accessors
on the infinite-order sentinel. Identity checks return exact zero/nonzero/inconclusive under
original premises, not an empirical property-validity certificate.

**Actual consumer and footprint.** Select `CaloricReference`'s `cp`, `enthalpy` and `entropy`
from `packages/reference/methods/models/caloric.pse` through
`properties::caloric_{cp,dh,ds}` in `packages/reference/domain/models/properties.pse` and their
bound `caloric_set` callables. In addition to the exact polynomial fixture, use the authored
NIST/Shomate caloric selection already represented by the reference package data and callable
owners. Resolve the actual bound callable and parameters; a bare synthetic Atom does not
establish this journey. Extend selected function lowering in compiler `typed_math` and the
`pse-math` analytical operation where needed. The concrete ordinary functions are
`shomate_{cp,dh,ds}` in
[correlations.pse](../../packages/reference/methods/models/correlations.pse), selected by
the nitrogen data in `packages/reference/data/nist/models/parameters.pse`; the independent
authored journey is `nitrogen_shomate_values` in
`packages/reference/seed-data/models/caloric.pse`. Preserve `numerics/caloric.rs`'s temperature
resolution and datum-bearing enthalpy interpretation. E01 plus A's working selected definition
and derivative support are the readiness boundary.

**Migration/deletion.** Derivative/series/identity inspection is new, so no predecessor
analytical engine is removed. Shared selected-source preparation replaces any duplicate
extraction introduced by separate views; remove those view-local preparations in the same
cutover. Existing authored correlations, caloric evaluation and independent tests remain their
scientific owners, not obsolete code merely because their derivatives can now be inspected.

**Independent acceptance.** For `theta=T-T_reference`, specify `cp=a+b*theta`,
`h=h_reference+a*theta+b*theta^2/2` independently. Check `dh/dT=cp`, reference value and the
series coefficients directly. For entropy check `ds/dT=cp/T_absolute` on positive absolute
T, original entropy datum and units. Exercise the ordinary authored correlation through its
callable selection, an exactly terminating polynomial, a log-containing series, a domain
boundary, unsupported provider/series, and changed source with retained historical output.
Formal order, exact identity and physical envelope remain separate assertions.

<a id="e03"></a>
### E03 — Deliver bounded indexed comparison and preserve source contractions

Implement the compiler projection from the original typed finite-index structures described
above into a selected indexed mathematical view. For `J_i=sum_j L_ij*X_j`, retain the binder
for j, carrier kind/type/ordered membership, indexed group axes and member lookup, the fixed
external i and the selected coefficient/force source occurrences. A static admitted filter is
part of the actual contraction domain. Missing group members remain missing, never zero;
unsupported dynamic filters/providers or genuinely non-contraction folds are explicit refusals.
Only admitted dummy groups rename. Identity tensor symmetry is sufficient initially; extra
symmetry needs a real declared premise.

**Footprint and mechanism.** `pse-modeling/src/resolved.rs` and its
`resolved/{selection,construction,records}.rs` owners already carry nodes, carriers, groups,
finite memberships and source maps. Preserve or extend their selected projection only where
needed; do not replace that arena with a tensor parser. In
`pse-compiler/src/workspace/modeling/executable.rs` / `typed_math`, take the analytical view
before scalar construction loses the contraction. `typed_math/direct.rs`'s refusal of
unselected `Reduce`/indexed operations remains correct for scalar execution. The new
`pse-math` indexed operation calls maintained Symbolica 3.0.1
`AtomCore::canonize_tensors(indices)`; `tensors.rs` owns `TensorGraphNode` construction,
Graphica 3.0.1 canonicalization and reconstruction, returning `canonical_form`, `external_indices` and `dummy_indices` lists. The dummy list describes canonical labels and assignment priority, not a map from original dummy names.

Before that call, the compiler analytical projection assigns bound indices from each admitted group to a deterministic common index alphabet (group identity plus ordinal, independent of source spelling). Rebind the resolved binder occurrences simultaneously using native `replace_map`, retaining the original-binder→input-alphabet map and original expression/source projection. Supply the same complete group-qualified dummy alphabet to compared expressions; preserve external/physical indices as fixed identities, not dummy candidates. Canonicalization then compares j/k lexical renamings over the same available labels. This is typed lexical lowering into the supplier representation, not an application isomorphism or index canonicalization algorithm.

Retain native output lists and the original-to-input mapping as distinct result fields. Do not invent a unique original-to-canonical permutation when automorphisms make it ambiguous. The initial operation displays canonical mathematics and separately attributed original mathematics; it does not transport source occurrence IDs onto canonical dummy positions. If a later executable contraction consumer requires that exact map, first add a narrow supplier witness projection with its ambiguity contract and tests. No application graph serializer/isomorphism or dummy canonicalizer is introduced.
E01's admitted index contract and A's original coordinate transport make this package ready;
Spenso and topology G are independent optional work.

**Migration/deletion.** This adds indexed analytical capability; no baseline general tensor
canonicalizer exists in the inspected owners. Keep scalar expansion for the small finite
numerical counterpart. If a selected analytical projection currently reaches scalar expansion
first, replace that routing with the pre-expansion view and delete its reconstruction adapter;
never preserve both routes as interchangeable scientific authorities. No graph serialization
helper needs removal unless execution actually finds one serving this replaced route.

**Independent acceptance.** Author two finite contractions differing only by j's lexical name;
canonical mathematics agrees after lowering to the same admitted index alphabet, while fixed external identities and separately retained source/binder attribution stay correct. Include different native supplied index pools as a negative contract control: bare supplier calls with unrelated pools are not required to print identical canonical labels. Changing external
component identity, carrier type/group or declared physical occurrence must prevent scientific
equivalence even if a scalar value agrees. Test invalid repeated contraction, open-index
mismatch, an admitted filtered membership and a missing member. Independently compute a 2-by-2
contraction with `L=[[1,2],[3,4]]`, `X=[5,6]`, expecting `J=[17,39]`, through the preserved scalar
counterpart. Native canonical equality alone is not a physical-equivalence assertion.

<a id="e04"></a>
### E04 — Publish native mathematical views and read protected originals

Use Symbolica 3.0.1 `AtomCore::printer` with Numerica 3.0.1
`PrintOptions::{latex,typst}` while each E02/E03 or D result's admitted mathematics is available.
Persist selected native expression text together with source/physical correspondence and the
operation-specific fields: derivative axes, series order/point, native dummy/external lists and compiler binder attribution, exact-block
conditions or assurance claim. Printer text is a view, never an identity key, executable codec
or proof. Namespace/symbol display and text escaping are projection duties; units/datum and
actual support remain explicit fields rather than implicit formula annotations.

The concrete producer-to-reader route is:

1. E02/E03 and D's scientific operation producers issue their admitted typed result and native
   formatted projection through the existing runtime result services. Add their declared
   relation/schema fields in `pse-schema/src/catalog/native_math.rs` and relation selection
   in `pse-runtime/src/workflow/results.rs` to the operation's result population; a new
   registry name without a producing `RunResult` table is insufficient.
2. `workflow/durable.rs`'s result publication enumerates the operation's `table_ids()` and calls
   `result_projection::store_run_result_table_queued` through `ResultAppendQueue` under the
   attempt fence. Store the projection in that original relation payload, with ordinary sealed
   block/manifest acknowledgments and original physical closure. Do not create an unprotected
   adjacent formula file or a second symbolic serialization store.
3. `RetainedDataServices::inspect_result` / `Runtime::inspect_result` in
   `workflow/result_inspection.rs` select exact run, terminal attempt, relation and range.
   Existing `results`, `inspection_projection`, `inspection_pack_selection`,
   `original_result_physical` and governing-permission lookup bind the observed payload and
   physical context. The mathematical view projects only selected permitted fields, retaining
   `expected_manifest`, schema fingerprint and cursor/basis checks.
4. `result_inspection` and `physical_inspection` lay out the stored formula and retained physical
   descriptions with bounded continuation. They do not call `derivative`, `series`,
   `canonize_tensors`, the compiler or scientific admission while displaying an old result.

**Footprint/readiness.** The producing analytical/exact operation, its authored result relation,
`pse-runtime/src/workflow/{durable,result_projection,result_inspection,physical_inspection}.rs`
and native tooling projection are one integrated cutover. Begin with E01/D01's stored result
contract and one working producer; wire additional D/E products as delivered. E05's richer
Typst decision is not a prerequisite. Supplier printer contracts are in
`vendor/symbolica/src/{atom/core.rs,printer.rs}`; original result custody remains the existing
runtime contract. Regenerate declaration outputs rather than editing generated files.

**Migration/deletion.** This is new mathematical presentation over the existing generic reader;
no current general derived-math pretty-printer, formula reparser or format-time CAS recomputation
was identified in the inspected reader owners. Do not claim their deletion as completed work.
If a producing client currently dumps a formula outside original custody, move that actual
client to the stored relation and remove its dump/reparse path in the same cutover. Preserve DSL
round-trip rendering and generic result publication/inspection; they retain required roles.

**Independent acceptance.** Produce an analytical result, seal its original manifest, change
source, disable scientific construction and read its stored formula, datum, units, source map
and claim. Select exact run/attempt/relation; test stale manifest, changed cursor/basis,
unauthorized fields, bounded nested fields and paging. Assert readable omission/unknown fields
without broadening payload permission or claiming full operation success for a partial page.
Check namespace collisions and literal text requiring escaping against expected labels and
native rendering. Include one D exact/assurance result, so this is a shared consumer rather
than an E-only report demo.

<a id="e05"></a>
### E05 — Settle bounded conditional integrations and finish cutover

Execute each named case/question in the conditional table, using its exact supplied or
candidate API and recording the chosen release/revision/features. The table distinguishes
already consumed 3.0.1 contracts from the integration/Padé/Spenso/Typst compatibility questions
still needing source evidence. A negative lookup does not settle capability absence. Resolve
only the uncertainties that change these actual analytical products, then record adopted and
delivered, analytical-only, or not selected plus reason and observable reopen condition here.
This is a bounded decision package, not a requirement to adopt every ecosystem library.

For PSLQ specifically, installed Numerica 3.0.1 `domains/integer.rs` documents that iteration
exhaustion can return the current best solution and indexes the first input to obtain its
domain. Admission therefore checks valid nonempty/nondegenerate input before invocation;
returned vectors are candidate relations until checked against original mathematics. API success
must not be labeled proof or guaranteed convergence. Other optional routes likewise retain
their actual refusal, singularity and stopping semantics.

**Readiness and footprint.** E02's actual caloric law/output, E03's finite contraction and E04's
report page are the discriminators; inspect each once its relevant contract is available,
without making independent leads wait for the whole series. Selected integration belongs in
the same `pse-math` analytical owner, compiler admission, runtime result producer and protected
reader, plus exactly pinned dependency/lock changes or maintained supplier patches. B owns
shared supplier provenance coordination. New bindings/kernel semantics follow the coordinator's
decision route. A candidate that requires an unresolved different type universe is not
implemented by silently mixing incompatible Symbolica families.

**Completion and deletion.** A favorable decision includes its actual input admission,
producer/consumer, resource/cancellation behavior, stored physical/source projection and
independent checks specified in the table. Remove replaced numerical preparation, report
adapters or local enumeration only when that adopted route actually supersedes them; most
optional products are new and have no predecessor to delete. A not-selected decision is
complete when it answers the case with evidence and a consequential reopen condition; it cannot
serve as an excuse to leave E02–E04's mandatory target incomplete. Verify that approximants,
formal order, relation candidates, graph equality and printed mathematics never populate stronger
scientific claim fields. Hand H the assembled caloric/indexed/protected-reader scope and actual
unsupported regions for series qualification.

## Verification and completion

**Proposed:** use affected-package `scripts/pse check <package> --all-targets` and focused
`scripts/pse test <package> --lib -E 'test(=NAME)'` checks while implementing each new mechanism;
select integration targets with `<package>:<target>`. Determine actual test names from the
implementation. Arrow-consuming correctness routes explicitly retain force-validation. All
failure baselines are zero; no checks or tests have been run for this plan-writing task.

Completion requires actual selected analytical and indexed operations, caloric and indexed
consumer journeys, native mathematical presentation, protected readback without scientific
rerun, conditional decisions with completed selected work, migrated consumers and deleted
replacements. Companion H owns assembled qualification and scope-end hygiene/governance,
not one campaign per package. Benefit expectations are qualitative; no timing benchmark or
precision count substitutes for the revealing scientific and readback checks.

At closure label evidence and name command, mode, scope, zero baseline and actual failures;
record corrected mistakes, deliberate deviations and any explicitly unsupported regions. Move
enduring contracts to their owners through the decision route and retire the completed plan
with the series under ADR-0096. No implementation Outcome is claimed by this draft.
