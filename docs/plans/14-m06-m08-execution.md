---
title: M06–M08 thermodynamics, library derivatives and sparse assembly
status: complete
date: 2026-09-24
implements: [ADR-0082, ADR-0083, ADR-0084]
evidence: Tested — 45 targeted units passed with explicit force-validation and zero failures; complete qualification remains M22
---

# M06–M08 execution

This packet executes Plan 14 M06–M08 and the approved improvements to M01–M05.
Build the target directly, remove replaced interfaces and callers, and preserve the
single M22 full-qualification boundary. No legacy math or Pyomo route returns.

**Current-state pointer:** this packet's completion receipts are historical. M10–M14
has since implemented the native solver lifecycle; see the
[current packet](14-m10-m14-execution.md) and [inventory](14-execution-inventory.md).
M15 is the next package; full qualification remains M22.

## Decisions

- Symbolica owns arithmetic, differentiation, coefficient extraction and generated
  provider-derivative composition. Numerica hyperduals carry arithmetic-block jets.
- FeOS owns thermodynamic formulas and states; its compatible num-dual release owns
  provider AD. Library scalar types remain private to the provider.
- Prefer Symbolica/Numerica for equivalent exact/symbolic matrix work. Retain faer
  for duplicate-aware numerical sparse construction and reusable numeric storage.
- The first property package is methane/ethane/propane, PC-SAFT plus DIPPR ideal-gas
  caloric properties, explicit zero binary interactions and recorded source data.
- Differentiable coordinates are temperature, molar density and two independent
  mole fractions. Pressure is an output/equation. FeOS NPT solving is initialization
  and diagnostics: vapor/liquid initialization does not establish a smooth branch.
- Raw provider derivatives and solver derivatives differ from Numerica's Taylor
  coefficients. One checked jet layout owns factorial conversion.
- Interpreted evaluation is the supported profile. JIT/SIMD remain deferred to
  bounded artifact ownership and equivalence qualification. Do not call
  `optimize_stack()` on vectorized evaluators: the pinned probe aborted.

## Execution order

1. Seal unchecked mathematical construction; introduce complete provider keys,
   output/order requests and local evaluation contexts. Coalesce provider calls
   only within their original active region. Preserve occurrence diagnostics.
2. Implement data-identified FeOS factories, worker-local state, value/first/second
   profiles and six coherent outputs: pressure, total molar enthalpy/entropy and
   three logarithmic fugacity coefficients. Reject invalid/nonfinite trials without
   clipping. Keep initialization and phase diagnostics outside callbacks.
3. Separate immutable semantic preparation, compiled artifacts and worker scratch.
   Compile compact arithmetic inputs, derivative layouts and Symbolica-generated
   provider lifts. Retain domain obligations and lazy branches. Bound resources and
   cancellation; failure never publishes prior-trial output.
4. Separate objective/constraint/gradient/Jacobian/Hessian demand. Hessian inputs
   include objective weight and row multipliers; unchanged property derivatives may
   be reused when weights change. Track consumed providers only.
5. Implement `z = Gx + Bp + c`, row contribution maps and mapped Lagrangian weights.
   Use faer to establish canonical sparse patterns once; retain only mechanical
   contribution/refill maps. Sum aliases before triangle projection, preserve all
   branch support and guard-only invalidation dependencies.
6. Derive affine/quadratic views with Symbolica, retaining original obligations.
   Add objective constants/sense, explicit integrality, parameter-sensitive class
   evidence and convexity admission. Validate explicit Clarabel cone views. Supply
   separate NLP/NLE products, assembled NLE Jacobians and faer JVPs. All-fixed
   problems use constant evaluation rather than fabricated solver variables.
7. Update the foundation review, current contract, main plan and inventory with the
   implemented boundary and remaining M09–M22 work. Proposed ADRs stay proposed;
   accepted decisions and the blueprint use their normal formal amendment route.

## Acceptance

Targeted units cover physical/data identity, provider state reuse and failures,
first/second derivatives, Taylor/raw conversion, nested guarded composition,
callback demand, multiplier changes at fixed inputs, isolated workers, resource
refusal/cancellation, aliases (`a*b`, `a=b=x`, Hessian 2), repeated rows, scaling,
permutation, numeric zeros, branch support, native index overflow and class rejection.

Use `just check-package`, `just unit-package` with explicit unit filters and the
recipes' force-validation feature. Required baseline is zero. Generate contracts
only when their declarations change. Integration, component, native-solve, Python,
performance and complete static/documentation qualification remain M22.

## Verification

**Tested:** the following combined targeted-unit gate passed on the pinned Linux
Rust toolchain, Nextest default profile, with the local multi-core Symbolica license
loaded from the untracked environment. Baseline: zero failures. Result: **45 passed,
0 failed**, 32 unrelated tests excluded by the explicit filter. The recipe supplies
`--features pse-relations/force-validate`; no integration/native-solve journey ran.

```bash
source .envrc.local
just unit-package pse-math 'package(pse-math) | package(pse-kernels) | package(pse-backend-native) | test(typed_math::tests::)' -p pse-kernels -p pse-backend-native -p pse-compiler
```

**Interface-checked:** `just check` compiled the complete workspace/all targets with
zero project compile errors. Cargo reports a transitive `proc-macro-error2` future
Rust compatibility notice; final static/dependency qualification remains M22.
No registry declarations changed in this packet. `just py-sync` refreshed the editable
extension and regenerated Python stubs through their owner; it is environment repair,
not Python workflow qualification. No measured performance or
independent thermodynamic/solver acceptance is claimed.

The targeted controls cover all entries in Acceptance above. Gram verification is
exact represented-rational algebra; this does not label the whole numerical pipeline
Formally established. FeOS finite differences are test oracles only. Independent
physical references, phase-envelope/near-singularity campaigns and native convergence
remain final qualification work.

## Outcome

**Implemented:** production PC-SAFT/DIPPR mixture factory and worker, explicit
output/order provider requests and full keys, Symbolica/Numerica derivatives and
provider lifts, sealed physical construction, output-specific domain dependencies,
semantic/artifact/worker separation, preparation-local program sharing, explicit
case maps, stable faer sparse refill, coefficient/Gram/conic admission and split
NLP/NLE products. Repeated rows, aliases, scaling, numerical zeros and all-fixed
cases are preserved. Instance failures retain both the bound instance and original
authored occurrence/provider cause. The unchecked public guarded-value artifact and its dependent
composition probe are deleted; no compatibility path remains.

**Mistakes corrected:** diagonal jet coefficients are Taylor coefficients rather than
raw second derivatives; the layout supplies the factorial. An initial callback
pruning rule kept every provider call, even for unrelated outputs; explicit output
effects now preserve canceled obligations and isolate unrelated demands. A public
Float path was resolved through the skill's facade aliases instead of using its
private implementation module.

**Deliberate deviations:** explicit density plus pressure equations replaces an
implicit phase-selected derivative callback. NPT remains initialization/diagnostics.
Numerica and FeOS num-dual remain on their respective library sides. Symbolica owns
exact coefficient/Gram work; faer owns numerical sparse storage and JVPs. JIT/SIMD,
general bound proofs and arbitrary cone inference remain outside the admitted profile.

**Boundary at this checkpoint:** M00–M08 implementation and paired replacements are complete.
M09 starts from complete selected inventories/support. M10 owns cross-revision
incremental reuse and whole-attempt resource retention; M11–M18 own actual native
workflows. M19–M22 and formal ADR/blueprint reconciliation remain open. The
[foundation contract](14-math-foundation-contract.md),
[inventory](14-execution-inventory.md), [main plan](14-library-owned-process-simulator.md)
and [enhancement review](../design_review/reviews/design_review_math-foundation-enhancements_2026-09-24.md)
record this boundary.

Subsequent [M09–M10 implementation](14-m09-m10-execution.md) closes structural analysis
and incremental/native ownership; [M10–M14](14-m10-m14-execution.md) supplies the native
solver lifecycle. M15 is next; the receipts above remain M06–M08 evidence.
