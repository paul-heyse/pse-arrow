---
title: "Plan73b: Supplier controls and qualified receiving"
status: in-progress
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73b-supplier-controls-and-qualified-receiving.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Supplier controls and qualified receiving

## Outcome and foundation

**Proposed.** Deliver O06/O07 and supplier compatibility for composition in [Plan73](73-symbolica-ecosystem-improvements.md). Supplier APIs and actual application adoption belong together. Own production cold receiving, original custody and subsequent symbolic use; overlapping Plan66 reconstruction/receiving obligations are delivered here rather than external completion gates.

The implementer's scope is:

- **Required mechanism replacement:** put construction-size knowledge and cooperative stop control inside the supplier operations that grow mathematical programs, then move the application callers to them. [B01](#B01) defines the checked API, [B02](#B02) delivers checked construction/vectorization/optimization, and [B03](#B03) delivers checked scalar/x4 source export. This removes applicable private expansion/export predictors and mutable callback rebinding; it is cross-cutting supplier work, not just a new consumer wrapper.
- **Required new capability:** a production cold receiver executes selected Value/First/Second stable products after the producer retires, under original scientific custody ([B05](#B05)). A receiver needing later derivatives or exact/class evidence gets a separately qualified symbolic product, or the single justified typed-replay realization selected by [B06](#B06).
- **Bounded integration decision:** [B04](#B04) settles composer backport/rebase versus joint/host composition for A's named independently reused module. A no-adoption result does not remove A's required module-consumer outcome.
- **Required preservation and retirement:** [B07](#B07) closes adversarial control/receiving acceptance and removes replaced production machinery. Existing bounded roots, proof conversion, original guards and compiler/module settlement remain preservation obligations throughout.

All package states below are open. This document specifies the target and its permitted
bounded choices; the packages do not begin by rediscovering whether the required capabilities
should exist. No old-plan package is a readiness gate.

**Interface-checked.** Baseline `baadc7e32`, maintained Symbolica3.0.1. [bounded_evaluator](../../crates/pse-math/src/library/admission.rs) predicts Taylor expansion/storage then reconciles realized facts; [native_compiled](../../crates/pse-math/src/native_compiled.rs) predicts supported scalar C++ export. [AbortBinding](../../crates/pse-math/src/library.rs) handles producer callbacks, per-call rebinding and successful optimizer truncation after stop. These controls fill real gaps; remove them only with equivalent consumed replacements.

[Supplier F64ProgramView](../../vendor/symbolica/src/evaluate/f64_transfer.rs) supports borrowed allocation-free inspection and checked restoration, consumed by [execution transfer](../../crates/pse-math/src/execution/transfer.rs). Existing numerical restore and sealed-copy/compiler tests do not establish production cold symbolic continuation. [Receiving/source research](../library_leverage/libraries/symbolica/build-memory-and-transfer.md#transfer-the-right-product) distinguishes those products.

## Checked operations and application cutover

Implement maintained supplier operations for vectorization/optimization and scalar/x4 C++ export actually used by A/C. They accept the current operation's cancellation/control and report operation-specific work and backing extents. Before charged expansion or allocation, supplier-owned algorithm code obtains admission for required work, retained output and transient overlap. Checked size arithmetic refuses overflow. Failure or interrupted/truncated optimization publishes no successful partial product.

The application supplies its existing original operation clock, admitted work/backing and scientific policy. The supplier owns its algorithm-dependent expansion/source knowledge. Distinguish generated source, transient construction, retained evaluator, subsequent compiler work and executable/module backing. Public size observations inspect built products, not unbuilt admission; retain realized reconciliation as an independent check where useful. RSS/time monitoring is only a backstop, not a deterministic prepaid bound.

Retained immutable products must not retain a producer's mutable cancellation authority. Each invocation supplies its control without a thread-local callback-rebinding ladder. Document actual safe points; no claim of interruption inside every arbitrary-precision scalar operation is required. Preserve native compiler drain, output-file/module custody and original settlement. No new general budgeting framework, mandatory process isolation or workstation cap is selected.

Migrate current math admission/native compilation consumers with the APIs. Remove corresponding private expansion/export formulas and callback rebinding only when all their consumed channels are covered. New stable heads and x4 source cannot introduce a second application predictor alongside the retired scalar one. Preserve useful scientific/failure controls against the target; no legacy path retained “as evidence.”

## Qualified receiving products

### Numerical execution

Transfer a qualified numerical program with original ordered layout and enclosing scientific custody to a receiver needing execution only. Passive framing/extent/profile inspection precedes admitted restoration and destination/dataflow checks. Publish only the qualified restored product. Numerical instructions do not reconstruct symbolic bodies, class evidence or derivative eligibility.

Extend the narrow builtin profile for C's deterministic operations using fixed known identities/semantics and qualified domain versions. Receiving cannot install arbitrary callbacks or source snippets. Unknown heads, incompatible flags/layouts or missing original receipts refuse before registration/materialization. Stable execution is not portable merely because it ran at the producer.

### Guarded symbolic continuation

Deliver producer-sealed normalized guarded mathematical stages plus original recipe/proof/source custody to the receiver needing later derivatives, exact/class evidence or other symbolic work. Passive qualification covers exact coefficient and dependency extents, destination symbol identity, normalized order, supported attributes, ordered guards/stages and receipt correspondence. Budget new symbolic work separately after restoration. Producer retirement must preserve all required retained backing and custody.

Existing packing/stage foundations are reused; implement missing preconstruction checks and production consumer integration. A general Atom import can register/remap/normalize, and [tagged3.0.2 instruction import](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/instruction/import.rs) consumes already allocated structures and constructs a new layout. Neither alone supplies passive admission or original proof custody.

B06 performs the finite sealed-versus-replay comparison before its production cutover. Select sealing when the complete receiver contract can be established and removes meaningful replay. Otherwise identify the unmet contract and deliver one coherent typed-replay route for that consumer, with its admission/guard/receipt obligations and reasoned no-sealing outcome. This permitted branch cannot become indefinite “complete except for Plan66” status or permanent dual readers. Retire replaced encodings/replay once the selected production replacement qualifies.

## Supplier compatibility and source custody

A's module-alone/chain scenario determines composer value. [Tagged3.0.2 EvaluatorComposer](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/compose.rs) wires outputs and optimizes instructions without repeating symbolic Horner. Compare a narrow backport to maintained3.0.1 with a scoped rebase preserving consumed local contracts. Maintain exact declarations/lockfiles and reproducible supplier provenance through the existing source-maintenance route.

Preserve [maintained bounded roots](../../vendor/symbolica-bounded-roots.md), packing/f64 transfer, original cancellation, conservative numerical profiles and proof conversion. New release number or source convenience does not qualify a replacement. When A selects joint/host modules, close this inquiry with that implemented outcome and no unused production composer adapter.

## Packages

| ID | Status | Prerequisites | Completion |
| --- | --- | --- | --- |
| [B01](#B01) | in-progress | A01/C01 design slices and existing callers | Checked operation extents, overflow/refusal, temporary overlap, safe points and per-call control settled through applicable decisions. |
| [B02](#B02) | in-progress | B01 implemented control interface | Checked construction/vectorization/optimization consumed; corresponding expansion mirrors/rebinding removed. |
| [B03](#B03) | in-progress | B01 interface; C01 supported-head descriptors | Checked scalar/x4 C++ export and native caller migration cover stable heads/source overlap; replaced exporter predictors removed. |
| [B04](#B04) | open | A01 module scenario; B01 preservation contract | Composer backport/rebase versus no-adoption decision implemented with A04, preserving maintained source contracts/provenance. |
| [B05](#B05) | in-progress | A01 binding/custody slice; working C02 interpreted products; C01 identities | Numerical-only production receiver includes stable Value/First/Second cold after producer retirement; replaced consumer encoding/replay retired. |
| [B06](#B06) | open | A01 source/guard contract; B05 early custody conventions | Guarded symbolic production continuation or justified single typed-replay branch delivered with actual later-symbolic consumer and retired replaced path. |
| [B07](#B07) | open | B02–B06 actual caller deliveries | Adversarial receiving/control integration and source-level retirement complete; evidence handed to H. |

<a id="B01"></a>

### B01 — Define the checked supplier operation boundary

Deliver an implemented, maintained supplier control interface and its operation-specific
input/output/error definitions, plus the narrow application adapter to existing construction
admission and `ExecutionScope`. Proposed API spellings such as `try_vectorize_with`,
`try_optimize_with` and `try_export_cpp_with` describe new supplier entry points; they are not
claims that upstream 3.0.1 already supplies them. Settle these semantics before consumers use
them, through the coordinator's decision route where a new operation/kernel contract requires it.

Each call receives the immutable input program, the native operation settings (Dualizer shape
and structural zeros, optimization settings, or export profile), and a borrowed per-call control.
That control checks the original cancellation/deadline and admits a supplier-reported stage
extent before growth. Extents distinguish incremental work, input backing still live,
temporary/overlap peak and returned retained backing. They use checked arithmetic and actual
operation units; wire values and a producer's past allowance never mint current admission.
The supplier may perform a passive shape/count scan first. Before a later optimizer pass or
generator grows beyond its previously admitted extent it must request additional admission,
not allocate and report afterward. The application adapter maps those requests to the existing
construction owner; it does not duplicate supplier expansion formulas or add a second resource
framework. Return either a complete product with realized facts or a typed unsupported,
overflow, admission, allocation, cancellation or interrupted/truncated outcome. Preserve the
caller's original admission error rather than flattening it into a supplier string.

Readiness is the agreed A01 demand/layout and C01 stable-head descriptors, not completed A02
or C02. B01/A01/C01 may design these shared inputs together; B02/B03 supply the implementation
needed by later consumers. The concrete application boundaries are
[construction admission](../../crates/pse-math/src/construction.rs),
[evaluator admission](../../crates/pse-math/src/library/admission.rs),
[exact construction/AbortBinding](../../crates/pse-math/src/library.rs) and
[native export/compilation](../../crates/pse-math/src/native_compiled.rs).
Supplier algorithm owners are maintained 3.0.1
[optimization](../../vendor/symbolica/src/evaluate/optimize.rs),
[vectorization](../../vendor/symbolica/src/evaluate/dual.rs) and
[export](../../vendor/symbolica/src/evaluate/export.rs). They retain implementation knowledge;
the application retains scientific eligibility and original operation ownership.

Acceptance exercises the boundary with exact allowance, one-unit-short admission, integer
overflow and a stop between stages. Prove the growth callback precedes the corresponding
allocation/expansion and that no partial result is exposed. A successful optimizer truncation
after an observed stop must be returned as interruption, not cached as a finished product.
The returned evaluator contains no producer stop flag or control closure. B01 may introduce
the interface before all callers migrate; it cannot remove existing protection by itself.

<a id="B02"></a>

### B02 — Replace mirrored Taylor growth and callback rebinding

Implement B01's checked operation inside the supplier and migrate the actual exact builder,
scalar cache misses, selected derivative expansion and optimization paths. The input is the
admitted exact evaluator plus A's ordered outputs/parameters/definitions and a selected
`JetLayout`/`Dualizer` shape. The output is a fully constructed exact evaluator with actual
instruction, coefficient, stack and retained-backing facts; numerical mapping remains a
subsequent admitted operation. Use the existing native direct translator and
`FunctionMap::add_aliases` rather than rebuilding an application expression graph.

The pivotal existing operation is `ExpressionEvaluator::vectorize(Dualizer)`, including
`Dualizer::vectorize_function` for C's known mathematical heads. Its maintained
[dual source](../../vendor/symbolica/src/evaluate/dual.rs) owns Taylor expansion and factorial
conventions. Wire checked control into expansion of components, generated external derivative
bodies, optimization passes and their temporary overlap; a public final `count_operations`
does not admit the unbuilt computation. If a supplier pass cannot stop inside a scalar
arbitrary-precision operation, identify that safe-point granularity honestly while preserving
checks around the pass. Retained immutable products receive new call control when transformed,
and independent/nested calls cannot overwrite one another's authority.

Migrate `exact_evaluator`, `ScalarCache::prepare` and `bounded_evaluator` in
[library.rs](../../crates/pse-math/src/library.rs) and
[admission.rs](../../crates/pse-math/src/library/admission.rs), including the non-Value path
that currently computes `expansion`, vector counts and `construction_shape` before `vectorize`.
Keep source-domain admission and public realized `EvaluatorFacts` reconciliation. Delete the
replaced supplier-private expansion arithmetic and `CURRENT_ABORT`/`scoped_abort`/
`AbortBinding`/detach machinery once all their construction channels use the checked API.
Retain unrelated source limits and source-owned payload bounds; a supplier instruction bound
does not replace admission of authored syntax or coefficient input bytes.

Readiness requires working B01 controls and A's demand/layout contract; derivative consumers
can move incrementally without waiting for all of A02 or C03. Acceptance combines a selected
First/Second layout, structural zero components, a known-head derivative, an unsupported
retained sub-evaluator, nested and concurrent controls, stop during optimization/vectorization
and unwind. After refusal/cancellation there is no `ScalarCache` entry, native export or retained
partial product. A later invocation must remain usable after the producer's scope is gone.
Preserve independent derivative/product checks; delete tests specific only to TLS rebinding.

<a id="B03"></a>

### B03 — Export scalar and x4 source through checked supplier control

Deliver the checked export producer and migrate `native_compiled::Batch::push` to it. Inputs
are a qualified pure evaluator, fixed registered head descriptors, deterministic function name,
scalar or x4 profile and the B01 control. Outputs are complete generated source and actual
source/temporary/retained extents, or a refusal before unadmitted growth. Compilation is a
separate subsequent effect: exporter admission covers generated text and its construction,
not compiler RSS, executable size or process settlement.

Extend maintained 3.0.1
[`export_cpp_str`/`export_simd_str`](../../vendor/symbolica/src/evaluate/export.rs) and the
[compiled-number/backend selection](../../vendor/symbolica/src/evaluate/backend.rs).
The supplier knows header emission, index spelling, constants, argument joins, generated
external derivative definitions and live text overlap. Use a checked counting/emission path
or equivalent supplier-owned staged growth admission so temporary argument strings are covered
before allocation. Known C01 stable scalar/x4 implementations and generated local-partial
definitions supply closed source descriptors with finite extents; arbitrary user callbacks,
source snippets and unqualified external constants remain unsupported. Do not accept a hidden
external definition merely because it is absent from `export_instructions`.

Replace `export_bound` and its eight-source-population mirror in
[native_compiled.rs](../../crates/pse-math/src/native_compiled.rs) only once the checked API
covers every currently admitted scalar channel and the selected stable/x4 channels. Retain
the existing named module batching, pending-publication boundary, compiler process group,
bounded diagnostics/file output, live shared-library directory and private workers. Keep
`InlineASM::None`, no fast math/FP contraction and no host-native ISA, with an explicit qualified
x4 instruction profile; supplier example defaults cannot override those settings.

Readiness is B01's implemented control plus C01's fixed semantics/source descriptors. B03 can
qualify ordinary scalar export before C02 provides the full stable consumer, then complete
stable/x4 coverage with it. It does not require a completed C03/C05. Acceptance includes wide
argument lists, repeated outputs, generated stable partials, large constants, unknown external
definitions, arithmetic overflow and one-unit-short source/overlap admission. Stop during
export or compiler failure/drain must leave no finalized module or stale function. Preserve
the existing native compiler deadline, child-drain and module-lifetime behavior tests against
the replacement; remove predictor-specific fixtures rather than retaining a second exporter.

<a id="B04"></a>

### B04 — Settle composer support against a complete maintained supplier

Take A01's independently consumed pure constitutive module and response chain, ordered
port/coordinate maps, selected derivative demand and the B01 preservation contract. Deliver
one implemented joint, host-called or composed route selected with A04. This is a bounded
supplier-compatibility decision; the required independently useful module outcome remains in A
even when no composer is adopted. B04 needs the scenario specification and existing reusable
module producer, not completion of A04, which consumes this result.

For native composition compare tagged 3.0.2
[`EvaluatorComposer`](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/compose.rs)
with a narrow backport into maintained 3.0.1 and a scoped maintained-source rebase. The composer
wires program outputs into later inputs and finishes with pruning/CSE/CPE/stack optimization;
it avoids repeating symbolic Horner for a real independently prepared chain. `merge` is only
output concatenation with parameter-count checks, not a semantic wiring operation. Apply A's
meaningful ordered correspondence before native wiring, select demanded derivatives before
inlining and rerun real specialization after rebuilding. Preserve lazy guards/provider effects
at their explicit host boundaries; pure pruning cannot delete an effect or an original guard.

Inventory the consumed maintained contracts before a release move: bounded roots and proof
conversion, passive packing, f64 transfer, operation control, direct construction and existing
supported numerical profiles. Reuse the source-maintenance owner
[bounded-root provenance](../../vendor/symbolica-bounded-roots.md) and exact manifests/lockfiles;
record the selected source revision/patch and targeted movement. A new upstream version alone
is insufficient. If composer support requires disproportionate new machinery or defeats
independent invalidation, implement the selected joint/host route and remove any trial composer
adapter rather than retain unused backend estate.

Acceptance connects the same module alone and through the response chain, checks independently
expected values/actions and reordered ports, and rejects incompatible parameter meanings even
when arity matches. Change one module, cancel composition, and exercise a guarded/provider
countercase. Source inspection establishes deletion of repeated reconstruction where actually
replaced; there is no required benchmark. A selected backport/rebase receives the affected
maintained-root/transfer/control checks in H's assembled supplier qualification.

<a id="B05"></a>

### B05 — Deliver cold numerical Value/First/Second receiving

Implement a production numerical-only publication **and** acquisition path in
[runtime portable math](../../crates/pse-runtime/src/math/portable.rs),
[artifact acquisition](../../crates/pse-runtime/src/math/artifacts.rs) and
[retention](../../crates/pse-runtime/src/math/retention.rs), using
[execution transfer](../../crates/pse-math/src/execution/transfer.rs). The selected receiver
needs an already prepared numerical product and layout; it must not replay symbolic
construction merely to execute that product. Preserve current controlled producer/deployment
qualification, canonical storage selection, original source/guard receipts and exact body
identity. A numerical program cannot grant later symbolic eligibility.

The baseline types make the necessary split concrete: `artifacts::Artifact.program` is an
`Arc<CompiledBody>` with source/preparation support; `portable::reuse_body_current` calls
`reconstruct_portable` and returns `Option<AdmittedBody>`. Neither is an execution-template
acquisition API. Source search for `reuse_body`/`publish_body` outside `portable.rs` found no
production callers at this baseline; the reconnect tests in that module are reusable evidence
foundations, not a production numerical publication path already wired. Do not rename their
return value and claim the new route exists.

**Producer and durable key.** After successful selected-demand preparation in
`MathService::artifact_until`—after `request.build_reusing`, finalized compilation where selected,
realized backing reconciliation and the final stop check—capture the actual numerical program
with `transfer::capture(&CompiledBody, scope, admit)`. Extend capture to selected Second as below.
The publication caller carries the actual `SelectedRead` and compiler-issued source/physical
custody; a generic cache completion without those premises may retain a local artifact but
cannot publish a scientific product. Keep the compiled body, capture owner and canonical
selection alive through encoding/publication/drain; publish no incomplete or stopped product.
Add a numerical description/publication function alongside `describe_body`/`publish_description`,
using `ProductDescription::prepare_with_shared_payload` and the canonical scientific-description
publication owner. This is a separately interpreted numerical product, not bytes attached to
the existing admitted-recipe interpretation.

Its request key binds compiler-sealed semantic body identity, deployment physical identity,
ordered demanded outputs, axes and symmetric pairs/product kind, layout version, original
guard/source receipt correspondence, numerical operation/head semantics version and the selected
numerical profile including coefficient realization and relevant supplier provenance. Existing
`ArtifactRequest::key()` is documented as a process-local numerical accelerator signature; do
not promote that hash alone to durable receiving authority. Extend the compiler-issued request
descriptor to expose/mint this passive execution request meaning without carrying a mutable
worker or requiring a receiver to reconstruct an `AdmittedBody`. The canonical product key
hashes the distinct interpretation/request, controlled producer key/class and exact encoded
payload, under the existing selected revision/dependency protection. Use the ordinary decision
route for a new durable interpretation/frame; no competing hand-written identity authority.
The portable profile identifies these binary64 instructions and their arithmetic/head semantics;
receiver-local interpreted versus native execution is a separate requested realization. C03
adds native compiler/ABI/ISA/FP premises to its local module key rather than requiring an
already native producer product to acquire these portable numerical stages.

**Typed acquisition and actual caller cutover.** Add a distinct numerical acquisition function,
conceptually `portable::reuse_execution_current`, returning
`Result<Option<AcquiredExecution>, PortableError>`. The proposed `AcquiredExecution` contains a
restored checked numerical template, validated original execution custody and its allocation
owner; it never contains or fabricates `AdmittedBody`, `PreparedBody` or `ScalarProducts`.
`transfer::restore` currently returns a scope-bound `CheckedExecution` holding private
`NumericFrame`/`ExecutionTemplate`. Add the narrow math-owned conversion to an immutable,
cloneable numerical-template handle with fresh `worker_scoped` frames; keep raw
`ExecutionTemplate` private and carry current scope/control only on the new worker. This new
handle and runtime `NumericalArtifact` wrapper are an explicit implementation obligation,
not existing public types inferred from the restore result.

Wire `MathService::numerical_artifact_for`/its batch acquisition counterpart into the actual
artifact service, with a distinct typed request/cache entry and the existing flight, lease and
retirement epoch owners. The selected production consumer is pure observation/property
execution reached by
[`prepare_modeling_observations`](../../crates/pse-runtime/src/math/modeling.rs) through
[`assemble_functions`](../../crates/pse-runtime/src/math/functions.rs). Split its numerical
execution binding from source mathematical preparation: the execution-only branch consumes
the numerical handle plus qualified original input/output/guard/attribution maps, rather than
`artifact.program: Arc<CompiledBody>`. Add the corresponding execution-only assembly entry in
[`CasePlan`/assembly](../../crates/pse-math/src/assembly.rs) for that admitted projection. Preserve
the source-bearing case owner wherever class evidence, new demand preparation or provider work
actually needs it; do not create a counterfeit source-bearing wrapper around restored numbers.
The cold execution-only descriptor/custody is acquired from the protected numerical description,
so this branch does not first call `reuse_body_current` merely to form an `ArtifactRequest`.
The normal cold-miss producer may build from currently admitted source and publish as above;
a present corrupt/incompatible product is a typed refusal, never hidden by recipe replay.

Retire the selected pure-observation path's mandatory `prepare_artifacts` →
`Arc<CompiledBody>` → source-bearing `plan.assemble` requirement and duplicate preparation when
its target binding passes. All callers needing new derivatives/exact/class evidence continue
through the distinct B06 body acquisition; no universal `reuse_body` replacement is claimed.

Start with `F64ProgramView::inspect` then `restore_with` from the maintained
[3.0.1 passive transfer implementation](../../vendor/symbolica/src/evaluate/f64_transfer.rs).
Extend its closed instruction profile for C01's deterministic stable operations and their
selected local-partial numerical products: fixed operation tags/version, arity and semantics,
not serialized closures or C++ snippets. Passive inspection must count every byte, operand,
constant, result and definition destination before registration/materialization. Unknown tags,
bad arity, unsupported flags, duplicate/protected writes and forward reads refuse; destination
known-head availability is checked without installing untrusted callbacks.

The application snapshot currently guarantees selected Value+First and explicitly omits Second.
Extend it to the C02 selected Value/First/Second layouts and pair maps, including original
coordinate offsets, ordered/repeated outputs, first-axis and symmetric second-pair meaning.
Inspect the new shape before allocating validation scratch or final backing. Restore with the
receiver's current operation scope/construction owner, retain input-plus-restored overlap until
publication, and reconcile actual backing. Publish the qualified execution template only after
scientific-envelope attribution and numerical dataflow checks both pass. Serialized limits
remain mathematical profile data, never a receiver resource grant.

Working C02 interpreted stable products and the implemented producer/publication/caller split
above are the readiness input. C03 consumes this completed receiving contract; B05 must not
wait for C03 or a compiled artifact. Choose numerical-only acquisition from the caller's
required capability before construction, rather than after a failed symbolic attempt. Delete
the migrated caller's redundant source reconstruction and obsolete numerical encoding/selection
branch where actually present; retain the distinct symbolic product owned by B06.

Acceptance restarts after producer retirement, acquires a multi-output stable Value/First/Second
product through the real `MathService` acquisition and pure-observation execution binding,
and evaluates original coordinates against independent analytic/high-precision controls. Assert
that a producer-published demand/profile key hits cold while changed output/axis/pair or operation
version refuses; no body reconstruction or scalar-family rebuild is invoked on the hit.
Include reordered axes/pairs, tiny log increments, magnitude extreme scales, joint-zero
derivative refusal, constant/zero-parameter programs, corrupted extents/tags, missing/extra
custody records and stop before materialization. The old producer's backing/control cannot be
required to remain live. Equal producer/receiver bytes or numerical self-agreement alone do
not establish scientific receiving.

<a id="B06"></a>

### B06 — Deliver one qualified symbolic continuation route

Select a real runtime portable admitted body whose receiver subsequently requests a derivative
and class/exact evidence, rather than only executing restored numbers. Use the guarded
polynomial/domain case with selected source occurrences as the first ordinary mathematical
consumer and a larger scientific recipe to exercise useful construction avoidance. Inputs are
the producer-sealed normalized mathematical stages, original typed recipe/physical and proof
receipt streams, symbol/layout interpretation and controlled producer qualification. The output
is an admitted symbolic body eligible for newly requested work, under its original authority;
new derivative/exact construction is charged to the receiver's new operation.

Reuse maintained [packing inspection](../../vendor/symbolica/src/atom/packing.rs) and existing
[sealed-copy controls](../../crates/pse-math/src/execution/sealed_restore_tests.rs) as foundations,
not as proof of a general production decoder. Complete passive coefficient/dependency/population
inspection, normalized form/order and attribute checks, destination formal/known-head identity,
guard-stage order and original recipe/proof/source correspondence before materialization.
Preserve the exact receipt-drain and source-role guarantees currently enforced by
[`reconstruct_portable`](../../crates/pse-compiler/src/typed_math_portable.rs); packed bytes do
not establish them. General Atom import, which can register/remap/normalize, and tagged 3.0.2
[instruction import](https://raw.githubusercontent.com/symbolica-dev/symbolica/v3.0.2/src/evaluate/instruction/import.rs)
do not provide this full preconstruction/custody contract by themselves.

Make the finite sealed-versus-typed-replay decision against this consumer before cutover.
Sealing is preferred when its full passive qualification and retained custody are establishable
and it removes meaningful replay. If not, deliver one coherent typed-replay route using the
existing compiler recipe and original receipt owner, record the failed sealing criterion and
observable supplier/consumer trigger, and remove trial sealed adapters. A permitted replay
outcome is not a second fallback behind a partially trusted sealed import. Readiness requires
A01's guard/source/occurrence contract and B05's agreed envelope/owner conventions; it does not
require B05's entire numerical demonstration to finish first.

Migrate the actual acquisition path in runtime `math/portable.rs` and compiler
`typed_math_portable.rs` together, plus affected `AdmittedBody` retention. Retire the replaced
production encoding/replay selection and mechanism-only fixtures after the selected route
passes; preserve independent corruption, physical receipt, proof and domain tests. Acceptance
uses a fresh process after producer retirement, then derives an independently expected
derivative/class result, and rejects foreign/reordered symbols, unsupported attributes,
denominator/domain cancellation, wrong stage order and missing/extra receipts before global
registration or partial publication. Never reconstruct symbolic evidence from B05 instructions.

<a id="B07"></a>

### B07 — Close control/receiving integration and retire replacements

Integrate the actual B02/B03 callers and B05/B06 production reader selections under one
coherent maintained supplier profile. Deliver a source-level cutover with compact evidence
and remaining supported-profile limits for H. This package does not invent another result
format, control abstraction, watchdog or global supplier registry; it closes the mechanisms
introduced by the preceding packages and their affected owners.

Inspect actual call sites in `library.rs`, `library/admission.rs`, `native_compiled.rs`,
`execution/transfer.rs`, compiler `typed_math_portable.rs`, runtime `math/portable.rs` and
`math/retention.rs`. Identify which old formulas, constructors and reader branches each
replacement absorbed. Delete remaining private expansion/export predictors, callback rebinding,
obsolete production encodings, duplicate replay/restore adapters and stale profile/configuration
selection only where replaced. Keep required scientific guards, source-input bounds,
allocation ownership and useful independent behavior verification. There is no blanket deletion
of all reconstruction or all bounds: B06 may deliberately select typed replay and unrelated
authored-source admission still owns its own limits.

Run the targeted integration cases that expose cross-boundary defects: current cancellation
after a product has outlived its producer, one-short admission before supplier growth, failed
compiler with drained children, qualified cold numerical Second execution, cold later-symbolic
continuation, corrupted known-head/profile identity and refused old encoding. Ensure canceled
or truncated construction never publishes success through a different retention or export
caller. Source inspection supplies retirement evidence; original-coordinate independent
mathematical controls supply scientific evidence. H schedules the assembled native/runtime,
maintained-root, conversion and receiving campaign once functional scope is ready. B07 records
the selected B04/B06 decisions and their actual delivered scopes, not an outstanding dependency
on an old plan or a benchmark promised solely to quantify an expected simplification.

## Verification

**Proposed:** scoped supplier/math/runtime compilation and focused tests through supported `pse` routes, explicit Arrow validation where applicable. Construction checks include one-unit-short allowance, overflow, interrupted stages, truncation and no partial retention. Receiving checks include passive inspection, admitted restore, producer retirement, destination compatibility and actual symbolic continuation. H selects assembled native/compiler lifecycle and cold receiver journeys at scope end, including maintained root/conversion controls when supplier code changes. No tests, builds or performance measurements ran in authoring.

## Execution checkpoint — 2026-11-10

Execution of the integrated A/B/C scope began at `d2302f32a` with a clean checkout. The selected first assignments are shared transport consolidation, checked supplier construction and stable primitives. A01/B01/C01 are coordinated prerequisite contracts; A03 proceeds within existing binding semantics. Required new kernel/primitive decisions precede dependent implementation. All other scope remains open until its actual production cutover and targeted evidence; broader Plan73/Plan66 completion is not claimed.

The checked supplier Taylor boundary is committed in `6dbd74418`, including borrowed typed
control, overflow/overlap facts and closed known-head local-partial composition. The application
non-Value construction route now consumes it. The focused default-profile math selection
passed 8/8, zero failures against baseline zero, including independent mixed-axis Second
factorials, typed admission refusal, overflow, nested/staged stop and producer retirement.
The shared all-targets math compile check passed. These results do not close B02: exact
builder/Horner/constant-folding construction still needs before-growth control, so its existing
source guards and builder abort protection remain. Checked scalar/x4 export is in progress.

The numerical prerequisite now separates a restored immutable template from fresh independently
admitted workers. Selected Second layout/pair transfer has been added; its scoped transfer
suite passed 19/19 with zero failures against baseline zero after correcting one new test's
invalid repeated demand ordinal. Closed stable-head transfer is being integrated next.
These are math-owned prerequisites only. B05 remains open for its durable interpretation,
compiler-sealed request/custody, actual publication/acquisition and observation caller cutover;
no source-independent production receiving claim follows from the transfer tests alone.
