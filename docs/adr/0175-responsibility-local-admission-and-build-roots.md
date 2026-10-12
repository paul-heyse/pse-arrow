---
id: ADR-0175
title: Responsibility-local admission and build roots
status: proposed
date: 2026-10-11
deciders: [paul-heyse]
level: decision
principles: [AP-01, AP-02, AP-03, AP-05, AP-07, DP-09, DP-13]
blueprint: [§3.1, §3.2, §14.3]
review: docs/design_review/reviews/design_review_plan36-build-roots_2026-10-11.md
evidence: Proposed
supersedes: []
superseded-by: null
revisit: Actual supported pinned-Cargo target/host/dev units fail to preserve required feature-family or explicit validation behavior without the fallback.
verification: Lower admission and physical generator compile without Runtime/math/compiler/operations/native backend; actual target/host/dev/native unit graphs; explicit Arrow validation leaf checks; generated comparison; surrogate consumer controls.
---

# ADR-0175: Responsibility-local admission and build roots

## Context

Plan 36 P-E4 adopts RC17. Document/physical source admission resides in Runtime even though it consumes lower engine and domain services. The physical generator consequently constructs MathService and carries unrelated solver dependencies. A single-consumer surrogate library integration resides in pse-math, widening compiler/math roots. The workspace-hack fallback adds unrelated features even on the supported pinned nightly.

## Scope

Add `pse-admission`, move native authoring-driver and physical inventory responsibilities and generated adapters, separate actual generator capability roots, relocate surrogate integration to Runtime, and retire the workspace-hack fallback after actual closure controls. This changes the crate graph and ADR-0122 Outcome5 fallback; its remaining toolchain, floor, workspace unification, per-checkout target, exact locks/pins and supply-chain contracts stand. ADR status/supersession bookkeeping remains the decision-PR route; this proposed amendment does not rewrite the accepted record.

## Drivers

One responsibility must compile and test with its consumed dependencies. Source admission needs the actual bound engine owner, checked domain rows, finite resources and cancellation, not a scientific workflow or canonical store. Full product schema emission still needs actual product DTOs.

## Options

Moving admission into pure pse-authoring would widen every pure DSL/model consumer. A facade retaining Runtime merely hides broad dependencies. Feature flags alone cannot narrow a normal dependency graph under workspace unification. Select a lower cohesive admission owner and actual capability roots. Retain broad full-schema generation where its actual types require it. No new surrogate crate is justified for one consumer.

## Outcome

`pse-admission` owns document bytes/parsing/loading/editing, checked source adapters and physical quantity/context/precondition inventories. It consumes existing authoring/model/modeling/quantity/schema/relation/columnar/engine/rules owners and actually used parsing/data libraries. It cannot depend on Runtime, math, compiler, operations or the native solver backend. Inputs retain parse budgets, cancellation, finite allocation and the actual BoundModel/EngineSession. Outputs retain original source and allocation ownership. The lower error preserves diagnostic causes; Runtime may wrap it. Scientific preparation/publication remains Runtime-owned.

Migrate Runtime/Python/bench/xtask imports and lower tests together, deleting replaced modules and paths without compatibility exports. Move generator declarations/roots and regenerate adapters. Pure document value types stay pure. Tests requiring scientific workflows remain Runtime integration; direct admission tests assert admitted inventories.

Physical generation builds finite EngineResources and its actual EngineFactory/BoundModel directly, retaining session/documents/spill/allocation through completion. Registry bootstrap, physical admission, full product schemas and native ABI are real optional dependency/target capabilities, not command names in an always-broad binary. Full schema generation derives real DTOs; native ABI comparison uses actual selected headers and bindgen. Aggregate regeneration still includes every root and prunes only declared generated roots.

Move the surrogate library integration and its normal egobox/argmin/ndarray dependencies to Runtime, where its one consumer lives; egobox-doe is a dev dependency. Preserve scientific acceptance separately from proposals, non-Send library ownership, callback terminal cause, task identity, memory, cancellation and joined drain. P-C9 owns compact advance/explicit charged snapshot semantics; relocation cannot duplicate the old copy API.

The pinned nightly with workspace feature unification is the supported Cargo path. Remove the hack crate/member edges and hakari config/generation/check routes only after actual selected target/host/dev/native units show required family/feature behavior and explicit Arrow validation. Stable-Cargo fallback is intentionally retired. Family checks, exact lock resolution and opt-in force-validation/native features remain mandatory. P-D5 explicitly declares JSON float roundtrip first. Preserve DataFusion SQL/file/compression/function roles. Unused BLAKE3 rayon and Python production operations test-support can be removed after complete consumer checks; the public hashing option remains unchanged.

### Consequences

Maintain one source and generated-output owner per meaning. Update crate registration, ceilings, generator route declarations and tooling documentation. The lower dependency graph becomes directly testable. No toolchain/profile move, cross-checkout shared target, reduced SQL contract or scientific bypass is selected.

### Compensating controls

Graph populations from investigation are not compile-time measurements or actual migrated-source proof. Verify real selected units, force-validation leaves, target/host roles and native feature absence, then compile/test the changed consumers. If the fallback is necessary on the supported toolchain, stop its removal and reconcile the concrete failing premise instead of claiming the full remedy.

### Confirmation

Proposed: the bounded change review assesses the target. Implementation, exact graph checks, regenerated comparison and assembled native/scientific acceptance remain owned by Plan 36e/P-G1.

## Pros and cons

The new lower crate has a concrete admission responsibility and removes reverse dependencies. Capability separation adds explicit feature wiring. Fallback retirement narrows closures at the cost of no longer supporting Cargo that ignores the required nightly configuration.

## More information

[Plan 36e](../plans/36e-build-test-and-development.md#p-e4-lower-admission-generators-and-dependency-closure) owns execution. Existing ADR-0122 rationale is retained; its replacement bookkeeping will occur with the decision PR, not an implementation status edit.

## Status history

- 2026-10-11 — proposed during maintainer-authorized execution of the accepted Plan 36 target.
