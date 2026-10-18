---
title: "Plan45b: Mathematical program preparation and execution"
status: in-progress
date: 2026-10-18
adrs: []
review_sources: [docs/design_review/reviews/design_review_post-plan44-pipeline-optimization_2026-10-18.md]
doc_role: plan
doc_topics: [design, preparation, numerical-execution]
---

# Plan45b: Mathematical program preparation and execution

## Target and current foundation

This companion owns F04/I03. The [coordinator](45-pipeline-execution-optimization.md) owns disposition and accepted RC09; [45a](45a-typed-source-and-incremental-preparation.md) owns source/physical premises and retained ownership. This work consumes their existing admitted products and can start its independent description correction without waiting for a source rewrite.

**Interface-checked baseline:** at `22ae87c16`, `pse-math` preparation and compilation separately form arithmetic regions and inspect/export related instruction descriptions. Plan44's artifact/scalar flights already share immutable compilation. The final compiled constructor already borrows its caller's final export. Exact Symbolica 3.0.1 exposes program composition and JIT; the local skill corpus is 3.0.0, so new adoption must inspect the actual pin. The existing optional compiled-evaluator profile already enables native code generation, with locked SymJIT 2.26.0. API presence is not qualification.

The target is one immutable description per demanded region and required evaluator representation, carrying inspected facts into admission, storage/reset eligibility and execution realization. Libraries own arithmetic/numerical mechanisms. The host retains original domain/order/attribution and effect barriers, current policy admission and private mutable evaluator/provider scratch. All changes and tests below are **Proposed**.

## B01 — Prepare regions and inspect descriptions once per representation

Create an immutable prepared-region product at the existing math owner: ordered expressions/parameters/reads, source/output mapping, original source-check order, demanded derivative/order layout and complete policy-relevant premises. Scalar preparation, compilation and final census consume it rather than rediscovering equivalent regions/read layouts.

Carry the inspected export/census with the actual representation it describes. Exact rational scalars, expanded Taylor forms and final floating evaluators are different representations; “one export” does not erase a legitimate conversion or new derivative/layout construction. Each needed representation is inspected once and its facts follow the owned product. A policy change checks retained facts against current limits without assuming a previous invocation's admission. Do not add another persistent cache or duplicate 45a's physical key.

Migration includes `execution.rs` scalar/region construction and census, `library/admission.rs` bounded conversion/reset eligibility, library product storage, and optional `native_compiled.rs` export consumption. Preserve the already removed inspection clone. Product-held immutable owners survive optional retention clear; request vectors, mutable workers and configuration keep their own bounds.

**Targeted acceptance:** sibling arithmetic and a dependent chain; values, first derivatives, selected second/weighted products; changed expression/order/layout/policy; guard/branch/provider barriers; different mutable workers evaluating distinct points. Count actual region formation, exports and census passes per representation. Verify original domain/nonfinite/source-order outcomes, cancellation/refusal before reusable product publication and final owner release. The correction closes when repeated descriptions are removed from actual consumers, not when a helper is renamed or a compilation cache happens to hit.

## B02 — Qualify and integrate library-owned program composition

**Larger I03 branch; accepted RC09 for altered instruction eligibility/ABI.** Compare current independent-adjacent fusion with exact 3.0.1 `FunctionMap`, aliases and function registration/inlining. Use a short dependent arithmetic chain, independent siblings, and a provider/guard-bearing body with original intermediate failure. Qualify sub-evaluator support through all actually demanded Taylor orders/products, bounded conversion, reset checks, code/export and source attribution. Literal substitution is not an acceptable replacement if it produces uncontrolled expression growth.

A qualified composition may combine dependent straight-line arithmetic while keeping explicit original check points. It must not evaluate beyond a failed original intermediate, remove a domain guard through algebraic cancellation, hide nonfinite values or flatten stateful provider effects. Inlining/function boundaries are library choices only inside that admitted contract. Consumers receive one library-owned program/template for the qualified policy, plus private workers; they do not implement their own instruction engine.

Implement the supported composition and affected consumers if exact semantics, admission and useful removed work are established. Otherwise retain current barriers and implement supported region/module sharing within them, with the exact sub-evaluator/derivative gap recorded. Lack of an eligible larger fusion must not block B01. No unavailable derivative is replaced silently by a finite difference or zero.

## B03 — Conservative JIT versus current interpreted/C++ realization

**Accepted RC09; independent of B02 beyond the actual representation consumed.** Separate preparation/export/compiler time from repeated evaluation. Start with the same demanded small programs under interpretation, the current C++ route and a conservative JIT candidate only where eligible. Existing C++ benefits—generic CPU flags, disabled unsafe contraction/math, bounded artifacts, child cancellation/drain and private loaded scratch—are consumed guarantees that another realization must match or state a deliberately different, explicitly approved support boundary.

Qualify exact Symbolica/SymJIT behavior for ISA/CPU features, FMA/fastmath/SIMD/threads, code/heap/template/scratch ownership, finite admission before opaque construction, cancellation/expiry and actual worker drain, cloning/private mutable state, selected derivative products and original-check attribution. Options to disable fastmath/SIMD/threads exist; they do not alone prove a generic-ISA or interruptible constructor contract. A helper process is eligible only if it removes complete machinery and supplies the needed bounded lifecycle, not as an unmeasured extra daemon.

Choose and implement one qualified native realization per explicit admitted policy/eligibility. Keep interpretation or C++ where the candidate does not qualify. No after-effect compile failure silently launches another backend. Changed code/ABI/export/reset premises use explicit current versioned identities through the decision route; historical bytes are not reinterpreted. Complete original scientific assessment remains independent of the native realization.

**Targeted acceptance:** constants/transcendentals/nonfinite/domain boundaries, generic CPU/math settings, values and all admitted derivative orders, two independent workers, cancelled/expired construction, pressure/refusal and last-owner release. Observe cold preparation and repeated evaluation separately; a faster micro-kernel is not a complete preparation/execution benefit. If no alternative qualifies, document the exact deciding gap and retain the supported route without presenting JIT availability as adoption.

## Work packages

| Packet | Prerequisite | Delivery and acceptance | State |
| --- | --- | --- | --- |
| B01 | Current immutable artifact/representation contracts | Prepared region and once-per-representation inspection across all consumers; original-check/order/accounting controls; redundant descriptions removed | open |
| B02 | B01 consumed description; RC09 before changed eligibility | Exact composition qualification and implemented supported branch; guard/provider/derivative/refusal tests | open |
| B03 | B01 actual export/representation; RC09 before production realization | Conservative JIT/C++ comparison and selected qualified implementation or explicit retained route; numerical/resource/stop/private-worker tests | open |

## Verification and checkpoint

**Planned:** touched math/runtime compile checks, pure targeted arithmetic/admission units and selected force-validated compiled-evaluator units when that optional profile is changed. Use the native-solver and Symbolica/faer/Oximo capability skills only for the contracts actually consumed; verify the exact installed release rather than copying newer Context7 examples. Dependency changes, if warranted, use ordinary exact-pin policy and affected tests; no wholesale re-resolution.

All packages remain open. B01 needs no JIT decision to remove metadata work. The coordinator records I03's integrated outcome; a proposed alternative, a successful compiler invocation or faster warm evaluation alone cannot qualify the complete realization. 45f owns final optional-feature/native/public integration and static qualification.
