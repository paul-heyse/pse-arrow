---
title: "Plan73b: Supplier controls and qualified receiving"
status: draft
date: 2026-11-09
doc_role: plan
doc_owner: docs/plans/73b-supplier-controls-and-qualified-receiving.md
doc_topics: [current-work, design, libraries, numerical-execution, validation]
review_sources: [docs/functional_reviews/symbolica-ecosystem-functional-improvements-2026-11-09.md]
---

# Supplier controls and qualified receiving

## Outcome and foundation

**Proposed.** Deliver O06/O07 and supplier compatibility for composition in [Plan73](73-symbolica-ecosystem-improvements.md). Supplier APIs and actual application adoption belong together. Own production cold receiving, original custody and subsequent symbolic use; overlapping Plan66 reconstruction/receiving obligations are delivered here rather than external completion gates.

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
| B01 | open | A01/C01 consumed contracts and existing callers | Checked operation extents, overflow/refusal, temporary overlap, safe points and per-call control settled through applicable decisions. |
| B02 | open | B01 | Checked vectorization/optimization control implemented and consumed; corresponding expansion mirrors/rebinding removed. |
| B03 | open | B01/C01 supported vocabulary | Checked scalar/x4 C++ export and native caller migration cover stable heads/source overlap; replaced exporter predictors removed. |
| B04 | open | A01 module scenario and B01 preservation contract | Composer backport/rebase versus no-adoption decision implemented with A04, preserving maintained source contracts/provenance. |
| B05 | open | A01 bindings; working C02 numerical stable products; C01 identities; B01 control contract | Numerical-only production receiver including selected stable operations qualified cold after producer retirement; replaced consumer encoding/replay retired. |
| B06 | open | A01 and B05 custody conventions | Guarded symbolic production continuation or justified single typed-replay branch delivered with actual later-symbolic consumer and retired replaced path. |
| B07 | open | B02–B06 actual caller deliveries | Adversarial receiving/control integration and source-level retirement complete; evidence handed to H. |

B01 and A01/C01 are joint contract design work; their implementation prerequisites are not a circular requirement to finish all three packages first. B02/B03 use actual scalar, derivative and native callers, checking insufficient allowance before growth, overflow, original cancellation, successful truncation and realized reconciliation. Migrate checked APIs and callers in coherent changes.

B05 uses a real runtime portable mathematical consumer, not only math restore tests, with multiple/reordered outputs and the selected log1p/magnitude Value/First/Second numerical products from working C02. Compiled scalar integration in C03 consumes this completed receiving contract; B05 does not depend on C03 completion. B06 uses a guarded recipe whose cold consumer later differentiates or establishes class/exact evidence. Test foreign/reordered symbols, malformed coefficient/dependency extents, unknown attributes, missing/extra receipts, unsupported layouts and cancellation before registration. Original-coordinate results need independent oracles, not equal bytes or producer/receiver self-agreement.

B07 inspects actual migrated callers and production reader selection. Delete obsolete bounds, callback rebinding, encodings, replay, adapters, configuration and mechanism-specific fixtures where replaced; keep independent corruption/guard/scientific/settlement tests. Future root refinement remains D's bounded scientific decision; this supplier work preserves its current contracts.

## Verification

**Proposed:** scoped supplier/math/runtime compilation and focused tests through supported `pse` routes, explicit Arrow validation where applicable. Construction checks include one-unit-short allowance, overflow, interrupted stages, truncation and no partial retention. Receiving checks include passive inspection, admitted restore, producer retirement, destination compatibility and actual symbolic continuation. H selects assembled native/compiler lifecycle and cold receiver journeys at scope end, including maintained root/conversion controls when supplier code changes. No tests, builds or performance measurements ran in authoring.
