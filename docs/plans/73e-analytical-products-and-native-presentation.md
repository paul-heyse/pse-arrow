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

Planning examines baseline `baadc7e32`, with the source review's earlier observations retained
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
external/dummy-index correspondence needed for truthful presentation and any later numerical
preparation.

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

| Lead | Decision boundary and delivered disposition |
| --- | --- |
| Symbolic integration | Investigate a native antiderivative for the selected caloric law. The supplied lead is separate `symbolica-integrate` 2.0.1, not a core Symbolica feature. Establish an exact compatible release/revision and feature closure against maintained 3.0.1, finite invocation/cancellation behavior and result/domain contract. Adopt only a complete native route; differentiate candidates back, retain reference constants and inspect poles/branches/intervals before a definite-integral claim. If compatibility or boundedness fails, retain derivative/series inspection and record a release/API/resource trigger; no handwritten integration fallback. |
| Padé, interpolation and executable series | Investigate a separate approximation artifact for the selected law/region using an actual supported supplier API, establishing exact release/API and failure semantics first. Admit only with explicit approximation/domain/order interpretation and independent original-form checks. A production executable surrogate additionally needs authored scope/error semantics, backend/derivative support and original-domain checks; absent those, close as analytical-only or not selected. Formal omitted order never supplies a uniform interval error bound. |
| PSLQ/integer relations and lattices | Evaluate a small opt-in high-precision coefficient-relation research request using Numerica 3.0.1 `Integer::solve_integer_relation`, with explicit precision, coefficient/iteration limits and nonconvergence meaning. Accept a candidate relation only as discovered; verify exactly when the original mathematical product permits it. Reject automatic rationalization of empirical coefficients. Inspect any lattice API separately if the selected relation case requires it; no general lattice optimizer or mixed-integer replacement. |
| Algebraic extensions and special functions | Admit these in exact analytical output when necessary to represent the selected operation faithfully, retaining branches/embeddings and support. Inspect the exact supplier representation/backend contract used. Unsupported numerical realization remains explicit; no forced radical, float or branch change for prettier output. New executable primitive semantics require their normal decision/admission route. |
| Spenso contraction | Investigate only against the selected finite constitutive contraction or a concrete larger structured counterpart. Spenso 0.6.0 `Contract`, dense/sparse tensors and `TensorNetwork` are candidate operations; establish supplier/type compatibility, contraction planning, external-axis recovery and bounded intermediates. Adopt only if real contraction structure/sparsity makes the full numerical route preferable to scalar expansion. Equality/collection alone stays on `canonize_tensors`; retain faer for its existing matrix responsibility. |
| Structured Typst modules | Compare native printer output with a concrete annotated analytical report needing structure unavailable from that output. Inspect `payload`, `math_display`, `typst_ast`, native feature closure and attachment/callback custody at a pinned revision. Adopt only if that consumer benefits enough to justify the closure and still reads the original protected result. Published Typst 0.1.0 and development 0.2.0 are distinct evidence; a development README is not a released Rust compatibility guarantee. |
| Native combinatorial iterators | Use native combinations/permutations/multiset partitions only for an actual derivative-index/coefficient expansion needed here. Check multiplicity and materialization before deleting a local iterator; do not introduce a generic combinatorial service. |

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
| E01 | open | Coordinator decision route; A's admitted-product/coordinate contract; D01 claim vocabulary | Selected analytical/indexed request and original-result semantics admitted, including physical/source support and honest refusal. |
| E02 | open | E01; A's working selected definitions/derivative product | Real caloric derivative/series/identity operation delivered, including ordinary authored correlation and independent exact case. |
| E03 | open | E01; compiler finite-index membership and A binding correspondence | Native indexed canonicalization integrated for one admitted constitutive family with fixed physical/external axes. |
| E04 | open | E02–E03 result producers; D01 result envelope, D results as available | Native mathematical printers and protected original reader/export migrated, with source/physical correspondence and replaced presentation paths removed. |
| E05 | open | E02–E04 named consumer cases; exact-version companion evidence | Conditional lead decisions closed; selected extensions fully delivered; remaining leads carry specific evidence and reopen conditions. |

### E01 — Establish the analytical operation and original-result envelope

Extend the existing scientific operation owner and authored registry declarations for selected
analysis, including finite indexed selection where necessary. Regenerate through `just codegen`
during implementation. Reuse current result custody and D01's claim distinctions; avoid a CAS
service, raw public Atom interface or a second result store. Make source evolution, unsupported
regions, finite work/cancellation and original physical/domain premises part of admission and
outcome, with typed reasons rather than success-shaped empty formulas.

Migrate affected model/tool entry points and selection helpers with the new producer. Delete
superseded expression-selection/name-reconstruction adapters after meaningful target tests.
Acceptance: a selected admitted definition, a wrong derivative coordinate, absent provider
support, a changed guard/source premise and a refused whole-expression request with a supported
explicit local alternative. Verify source/occurrence identity rather than matching formula text.

### E02 — Deliver caloric analytical inspection

Implement exact derivative, series and identity requests over selected caloric definitions and
parameter bindings. Use the native operations described above and preserve full series order
meaning, physical derivative units, reference enthalpy/entropy and applicability. Share original
selected preparation with A; do not derive independent flatten/differentiate/reconstruct paths
for every requested view. Integrate through the actual developer/modeling operation and result
producer, not a standalone library example.

Test the independently specified polynomial law and one ordinary authored correlation. Check
`dh/dT=cp`, the physical entropy derivative, reference values, explicit series coefficients and
omitted order/expansion point. Include an exactly terminating polynomial, a domain boundary,
opaque provider, unsupported series and source edit whose old result remains historical.
An algebraically true identity must not assert empirical validity outside the property envelope.
Delete replaced ad hoc analytical extraction/dumping and its mechanism-only fixtures where
present; preserve independent caloric/property behavior checks against the new operation.

### E03 — Deliver bounded indexed comparison

Implement the compiler-owned finite-index view and declared group/symmetry admission needed
for the selected constitutive family. Pass that view directly to `canonize_tensors` and return
canonical mathematics plus external/dummy maps tied to original source. Keep distinct physical
instances and supported provider effects distinct. Avoid recreating Graphica conversion,
isomorphism or dummy renaming in application code.

Test two legitimate dummy-renamed contractions, an external-axis/component change that prevents
equivalence, a different index group, invalid multiple contraction, open-index mismatch and
source attribution after canonicalization. Preserve the selected scalar numerical counterpart
and compare its independently computed small contraction where execution is exposed. Remove
any superseded local renaming or graph-serialization helpers introduced by a replaced route;
there is no baseline claim that a general bespoke tensor canonicalizer already exists.

### E04 — Migrate mathematical presentation and protected readers

Add native LaTeX and Typst expression views to the existing reader/physical projection. Carry
source labels, namespaces, units/basis/datum, operation/claim meaning, actual support and
partiality alongside formulas. Mathematical projection needed for retained reading is captured
under original custody; formatting observes it without compiling current source or executing
scientific work. Apply the same projection to D's exact-block/enclosure results as those
producers become available; E's early reader contract can be integrated before all D work ends.

Acceptance selects an exact original run/attempt/manifest, reads with bounded permission,
pages a larger view and refuses stale manifest or unauthorized fields. Demonstrate correct
namespace/source correspondence, enthalpy datum and entropy units, unsupported fields, and a
retained artifact after source edits with scientific construction unavailable. Keep the authoring
renderer for DSL round-trip. Delete replaced derived-math pretty-printers, reparse adapters and
format-time scientific recomputation, plus their mechanism-only tests; preserve behavior tests
for truthful physical/source presentation and independent original readback.

### E05 — Settle bounded conditional integrations and finish cutover

Carry out the named investigations in the lead table, recording exact supplier evidence,
selected consumer, complete integration burden, decision and reopen condition. Supplier changes
are allowed when they improve the full selected operation; coordinate them with B rather than
using a wrapper to preserve obsolete supplier limitations. Added dependencies are exactly
pinned, lock movement is targeted and feature closure is reviewed. New bindings/kernel contracts
follow the coordinator's decision route before implementation. A compatibility investigation
cannot quietly broaden the type universe or silently enable unsupported primitive semantics.

Any selected integration/approximation/relation/Spenso/Typst extension finishes its actual producer,
consumer, resource/cancellation, physical/source projection, targeted acceptance and replacement
deletion before E05 closes. A not-selected outcome is a concrete bounded decision, not an
unfinished mandatory implementation. Verify especially that approximate candidates, formal order,
printed mathematics, graph equality and high precision never populate stronger scientific claim
fields. Hand the assembled caloric/indexed/protected-reader journey to companion H with its
actual scope and remaining unsupported regions.

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
