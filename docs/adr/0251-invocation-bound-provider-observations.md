---
id: ADR-0251
title: Observe selected provider regimes at their original invocation
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, AP-05, DP-01, PS-05, PS-10, PS-12]
blueprint: [§18.2, §19.2]
review: "not-required: bounded provider completion observation within the existing kernel/attempt contract and authorized Plan65i; focused independent source assessment before implementation"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A named provider needs additional scientific observations, a consumer needs transferable native authority, or an observation changes derivative validity rather than describing an invocation.
verification: Native BT liquid/two-phase/vapor observations and component order; current input/context and failed-fill isolation; local derivative completion versus regime crossing; scalar providers emit no chemistry facts; nonselected attempts and absent stability remain explicit; independent generated read grants no permission.
---

# ADR-0251: Observe selected provider regimes at their original invocation

## Context

Plan65i's independent BT reader needs the regime actually selected by the native provider.
Preparation inspection describes possible source branches. The private selected numerical
product already retains the actual winning regime, but provider completion carries numerical
outputs and an optional derivative-context token only. Inferring selection from a phase
fraction or diagnostic text would add a second scientific interpretation.

## Scope

Extend blueprint §18.2/§19.2 with optional typed invocation observations at the existing
provider completion owner. Scientific selection, derivative qualification, physical stability,
native product-use permission and durable custody remain distinct responsibilities. This is
a kernel-contract addition within the existing native provider boundary.

## Drivers

Retain actual producer facts through original-coordinate evaluation and generated readback.
Support selected chemistry without imposing chemistry metadata on scalar-only providers.
Keep conditional active occurrence, exact context, cancellation, memory admission and
failed-trial clearing at the owners that already enforce them.

## Options

Preparation metadata cannot report a later winner. Deriving a phase from output values or
parsing logs would duplicate science and lose invocation correspondence. An independent
observer service adds lifetime and synchronization. Select an optional native provider hook
and checked completion observation alongside the existing linearization seam; the current
evaluator and result owner carry it to the reader.

## Outcome

The provider may expose an optional typed selection fact for its just-completed invocation;
the default is absence. Follow the existing default-none `Provider::completed_linearization`
pattern. `ProviderBinding::fill` issues the observational completion only after worker fill,
context/cancellation, complete finite-output and expected-linearization checks all succeed.
Bind it to the actual provider registration/binding, exact input context, authored selector
and policy interpretation, selected regime and the mathematical qualification actually
established. A public decoded observation never constructs a native completion or permission.

The native `SelectedProvider` supplies facts from its current `SelectionProduct`. Clear
pending observations at invocation start and on every refusal. Cache/replay must satisfy
the same current input/configuration/source correspondence; a prior successful numerical
product cannot leak after a failed call. Distinguish demanded fill products from the cached
selection's established derivative order: action requests may use First selection while
computing their demanded derivatives separately. An array or request label alone proves no
local regularity, neighborhood, uniqueness or stability.

Record the actual active original provider-stage occurrence after successful fill, through
the existing evaluation owner. Keep passive observations separate from `ProviderVisit`'s
derivative-premise equality: diagnostic counts must not change linearization compatibility.
Clear them with trial invalidation/failure and preserve only facts visible in the original
requested evaluation; do not evaluate inactive branches just to fill a report.

### Consequences

Admission must cover observation records, any retained input context and escaped result
copies. The existing provider-visit allowance does not automatically cover a second vector.
Reserve before materialization and retain custody through the actual evaluation/readback
lifetime. Observation-free providers add no invented material/phase meaning.

Runtime reporting snapshots facts from the actual final original-coordinate evaluation,
binding original run/attempt/step/member and coordinate meaning. Earlier or nonselected
attempts remain attributable raw observations. Scientific product selection still consumes
its original native permission. The generated observation is output-only.

### Compensating controls

A selected minimum among eligible authored regimes is not a thermodynamic stability
certificate. Report available physical stability only from its separate actual assessment
owner; absence stays unknown. An intentionally metastable declared purpose remains eligible
under its own contract. Preserve typed recoverable `RegimeCrossing` and the independent
`ProviderLinearizationToken`; neither this observation nor its decoding permits crossing a
branch or replaying effects.

### Confirmation

Focused independent source assessment found the existing completion hook/gate and active
stage occurrence suitable, with three material requirements: current-product clearing and
identity, truthful completed order for action demands, and separate allocation accounting.
Plan65i I01/I02 owns implementation and the named BT/native-reader controls; 65f owns retained
selected-result delivery. This proposal establishes no executed provider-observation behavior.

## Pros and cons

The selected seam reuses actual numerical ownership and supports independent typed inspection.
It adds explicit completion data and bounded retention duties; it cannot supply global
physical stability or scientific permission from a numerical winner.

## More information

- [Plan65i native scientific semantics](../plans/65i-scientific-operations-and-product-semantics.md)
- [Checked provider completion](../../crates/pse-kernels/src/provider.rs), [provider hook](../../crates/pse-kernels/src/lib.rs)
- [Actual selected numerical owner](../../crates/pse-math/src/implicit_regimes.rs)
- [Active stage evaluation](../../crates/pse-math/src/execution.rs), [visible evaluation owner](../../crates/pse-math/src/assembly.rs)
- [Plan65f selected evidence](../plans/65f-diagnostics-and-result-delivery.md)

## Status history

- 2026-11-06 — proposed after the concrete BT consumer exposed the missing invocation observation, before kernel implementation.
