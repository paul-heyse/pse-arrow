---
id: ADR-0254
title: Govern original fit trajectory members independently
status: proposed
date: 2026-11-06
deciders: [paul-heyse]
level: decision
principles: [AP-04, DP-01, DP-05, DP-19, PS-11, PS-12]
blueprint: [§19.2, §19.4, §19.8]
review: "not-required: bounded original-member permission within authorized Plan66 A02; existing local scientific qualification and exact native custody remain the owners"
evidence: Interface-checked
supersedes: []
superseded-by: null
revisit: A consumer requires a member point, partial trajectory interval, different mesh/site, or reuse of fitted parameter estimates rather than the original evaluated member.
verification: Challenge independently qualified siblings after aggregate failure, exact declared membership, run/attempt/source/report mismatch, duplicate or missing permission, historical unavailable proof, native endpoint/coverage/check refusal, and original coordinate/input projection without repeating integration.
---

# ADR-0254: Govern original fit trajectory members independently

## Context

Original fit assessments now retain experiment outcomes and partial observations under
ADR-0253. `FitTrajectory` borrows an exact joined producer and native report but exposes
only whole-fit permission. `ProductScope::governing` refuses member points. A native
trajectory carries physical endpoint/coverage and source coordinates, without a semantic
point identity for each sample. A time hash, row ordinal or quantity cannot supply one.

## Scope

The first consuming scope is one complete original transient fit experiment, evaluated at
its actual retained candidate inputs. Its local trajectory qualification is independent
of fit-estimate optimality and other experiments. The next consuming scope is the original
root instance of a standalone `ModelingTrajectory`, using its already retained completion,
report, checks and preparation. Both use the same whole-member observation and private
custody owner. Neither qualifies fitted parameters, arbitrary member points or independent
partial-interval integration. Reading a selected subset of an originally permitted member
is descriptive partial coverage, not a new interval or point capability.

## Drivers

Keep original declared membership, source coordinates and actual native completion
authoritative. Independently healthy trajectories must remain usable after an unrelated
fit failure, while passive storage and decoding cannot reconstruct native permission.

## Options

Copying aggregate fit permission couples trajectories to fit optimality. Caller maps or
synthetic sample identities cannot establish original membership. Select an explicit
whole-member native decision with exact producer/report custody; retain member-point and
later mathematical-product refusal until their actual owners exist.

## Outcome

Add an explicit `ProductScope::Member` keyed by the originally declared `InstanceId`
inside the producer's exact run and attempt. Preserve existing point/member refusal
until a real retained point owner supplies that authority. Keep `ScientificPermission`
as the shared decision shape; a source-owned `MemberScientificPermission` supplies its
original run/attempt/member and source/physical/preparation references. Completion owns
the member population. Missing or duplicate facts never choose an arbitrary record.

Assess the local member once using existing native trajectory endpoint/coverage,
returned original physical checks and the original simulation numerical policy.
Unobserved demanded accuracy remains unavailable and refuses use. Retain failed,
partial and unattempted observations without granting use. One sibling's failure cannot
replace a completed member's independent decision. Whole-fit estimate permission remains
separate.

Retain the actual converted integration parameters already produced for that member's
assessment; do not recreate them from a caller map. Expose source/layout coordinates
through the original simulation's existing coordinate projection with those exact inputs.
The constructor-default projection delegates to the same length/finite-checked helper.
Physical initial-condition and numerical-guess meanings remain at their current owners.

Native member custody binds the actual immutable native report and allocation owner.
The consuming trajectory verifies original run, attempt, member, source, preparation and
report correspondence. Serialization and generated observations omit native proof; decoding
cannot manufacture it. Historical member populations default unavailable, with the stored
completion's enclosing attempt remaining the historical binding authority.
The historical snapshot retains source/preparation references and the recorded decision;
it does not independently restore actual applied parameters/configuration or live proof.
The protected result reader may characterize that original recorded decision only with the
same protected completion/attempt and `scientific_contexts` membership: original run/member,
source/physical/preparation/profile, sample time and output/quantity/unit correspondence.
It shares the existing protected read and decoder owners. Missing, foreign, duplicate or
ambiguous facts remain unknown; native failure remains unusable. Selected subsets cannot
become complete-population claims. Historical interpretation never restores live custody,
providers, numerical execution or derivative permission.
Required and observed counts retain the existing original trajectory sample-population
meaning (`profile.samples.len()` and native coverage), rather than counting assessment objects.

For the standalone sampled-value consumer, select an authored output at an exact physical
time in the original qualified full trajectory. Native snapshot custody, original coverage,
and the prepared output coordinate bind the value to its run, attempt and member. Its stored
row index is a projection of that coordinate. Exact sample selection neither establishes
independent derivative or interval validity nor assigns a `ProductScope::Point` identity.

Pass the already-owned `DurableAttempt` identity into completion capture from the existing
finished/refused transition. Do not introduce a second retained mutable attempt stamp.
Genuinely ephemeral runs retain `None`; no NIL or synthetic attempt is introduced.

### Consequences

The member observation is declared once with output-only generated projection. Existing
scope tags remain stable and the new member alternative is explicit. The native fit
trajectory reader consumes this scope rather than aggregate usability or quantity-only
lookup. Original input storage, completion metadata, custody and projection allocations
remain accounted. Broader A02 steady partial-buffer and member-point work stays open.

### Compensating controls

The native trajectory gate checks exact run/attempt/member/source/preparation and both
report and local assessment custody. For standalone simulation it retains and matches the
actual immutable `ModelingTrajectory` owner and original request instead. Missing, ambiguous or decoded proof refuses use;
the raw report remains readable. Whole-member permission cannot grant point derivatives.

### Confirmation

The existing completion capture order, scope resolver, retained fitting assessments,
trajectory borrow, simulation coordinate projection and local completion helpers were
inspected before this record. Functional evidence belongs to the implementation commit;
this proposed decision does not claim assembled qualification.

## Pros and cons

The existing completion policy and original native trajectory supply a bounded independent
scope without a second simulation or mirrored decision schema. Retaining actual local
inputs and completion adds charged native state; other trajectory and point consumers
still require their own original correspondence.

## More information

- [Plan66a](../plans/66a-scientific-operations-and-product-semantics.md)
- [Completion owner](../../crates/pse-runtime/src/workflow/completion.rs)
- [Original fit trajectory](../../crates/pse-runtime/src/workflow/fitting/trajectory.rs)
- [Fitting assessment](../../crates/pse-runtime/src/workflow/fitting/modeling.rs)
- [Original simulation coordinates](../../crates/pse-runtime/src/workflow/modeling/dynamics.rs)

## Status history

- 2026-11-06 — Proposed before the authorized original-member permission declaration.
- 2026-11-06 — Extended the same proposed contract to the standalone original-root consumer
  and descriptive protected trajectory reader before those consuming paths were integrated.
- 2026-11-08 — Proposed bounded native selection of an original sampled output value.
