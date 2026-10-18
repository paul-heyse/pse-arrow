---
title: Plan45 physical premises and fixed-pattern KLU
date: 2026-10-18
tier: change
purpose: target
decision: Accept
doc_role: review
doc_topics: [design, numerical-execution, performance]
standard:
  core: "3.4"
  efficient-architecture: "1.0"
  process-simulator: "1.5"
---

# Plan45 physical premises and fixed-pattern KLU

## 1. Decision and scope

**Accept the bounded target design.** Resolver-issued consumed premises provide a sound responsibility boundary for finer physical reuse. A private fixed-pattern KLU owner is also a suitable conditional extension when the short stock screen establishes a plausible complete-operation benefit. Acceptance covers these contracts, including the preservation requirements below. It establishes neither an implemented closure nor qualified native reuse, and does not resolve Plan45 F05 or F13.

The baseline inspected is `b0d2683009c147ec7b3d1f7e2b79d0e9ae161d7d`. Six unrelated dirty environment/workflow files were observed and left unchanged. The coordinator's proposed contract, including its clarification of unsupported checker/provider behavior, is the design subject. Proposed ADR-0183/0200 amendments and blueprint changes remain coordinator-owned prerequisites.

This is an independent, read-only change/target review under Core 3.4, the efficient-architecture companion 1.0, Process Simulator 1.5 and the pse-arrow binding. It examines physical admission, compiler reuse and refresh, receiving ownership, and KINSOL/KLU lifecycle. It excludes whole-product qualification, broader solver selection, new physical models, environment attestation and long performance campaigns. Plan45 owns dispositions; 45a/A06 and 45e/E05 own execution.

## 2. Source-grounded assessment

### Physical premises and current receiving ownership

**Interface-checked:** `QuantityRegistry::resolve_key`, `operations_for`, `conversions_between`, `physical_name` and `neutral_dimensionless` expose distinctions that successful selected IDs alone cannot capture. In `pse-quantity::resolved`, an absent exact quantity key can select raw representation; a newly named result changes that decision. Operation and conversion candidate populations can change selection or ambiguity. `ResolvedInference` retains operands, selected authority, conversions and scales, but is not a complete negative/candidate dependency closure.

The current compiler takes conservative dependencies through `Inventory` and the `resolved`, `typed` and `complete` queries in `workspace/modeling/granular.rs`. `publish_modeling_revision` rejects a changed physical context. Runtime compiler lanes compare complete `CompilerContext` values, while frontier completion separately checks physical, environment and provider identities. These are coordinated compatibility boundaries, not interchangeable cache lookups.

**Proposed:** let actual physical resolution/checking issue the sealed, versioned premise closure. It must include positive and transitive declarations, exact absence, complete candidate buckets and ambiguity, visibility, and consumed checker/provider interpretation and configuration. Consumers compare that issued meaning rather than independently reconstructing dependencies. Canonical inventory/source identities retain their existing meanings; changed consumed preimages use new frames without rehashing history.

The proposed scoped recorder and immutable sparse projection are compatible with this ownership. Recording complete buckets before lazy traversal prevents a first-match iterator from concealing later ambiguity. Any capture must remain synchronous under its actual borrowed owners; nested, foreign-owner, cancellation and unwind behavior cannot silently produce a partial eligible closure. Failed checks either carry complete refusal premises or remain conservatively recomputed. Snapshot construction must not recursively contaminate capture with unrelated inventory reads.

`InvariantChecker::immutable()` promises lifetime immutability, not a complete behavior identity. Therefore “full-context fallback” means a complete immutable behavior/interpretation context where representable. An arbitrary checker or provider without that contract is non-reusable and recomputed; an inventory digest alone cannot certify it. The built-in checker has an explicit interpretation identity.

Refresh must stage current `Inventory` and `self.inputs` together under exclusive same-lineage authority, re-admit current source, and publish only a coherent successful candidate. Failure restores the prior usable state or destroys the failed candidate. Tracked queries consume immutable resolver-minted projections; a plain field containing a new context is not a tracked dependency. Equal projection results can backdate.

Equal mathematics does not transfer current authority. `CheckedPackage`, `PreparedModeling` and `PreparedCase` retain physical owners; refreshed shells must carry current registry/checker/provider owners, attribution and receiving checks while sharing eligible immutable mathematics. Source/invocation/schema/session/registration guards remain fresh. Historical products retain their own context, and a historical request cannot rewind the live lane.

### Fixed-pattern KLU

**Interface-checked:** bundled SUNDIALS 7.1.1 `SUNLinSolInitialize_KLU` forces first factorization; the next stock setup replaces symbolic analysis. `SUNLinSol_KLUReInit` frees both symbolic and numeric state. The stock adapter exposes mutable symbolic, numeric and common pointers. `SUNMatZero_Sparse` clears indices as well as values; the Rust Jacobian callback already restores the Session-owned canonical CSC pattern.

Current `Session::solve` clears Rust setup state and requests fresh initial setup, but an accepted initial point may return before native setup. Consequently a symbolic-retaining extension needs an explicit application boundary independent of setup callbacks.

**Proposed:** a dedicated private constructor binds the Session's canonical structure, actual matrix/CSC mode and canonical analysis defaults. Frozen ownership avoids maintaining a copied structural key or scanning CSC before every substitution. Mutable KLU accessors and analysis-setting mutation must be unavailable or rejected in this mode; accounting uses a scalar getter. Generic stock behavior remains unchanged.

`BeginApplication` always frees/invalidates numeric state before possible early return, including an accepted start. Compatible Symbolic may survive. The first setup fully factors current values; subsequent refactorization, condition checks and recovery remain native-owned. Every failed setup leaves numeric state unusable, prevents substitution, and makes the next setup full. Matrix zeroing does not change the logical structure authority: the callback restores the canonical CSC storage before setup.

The real implicit Session cache is the relevant retained consumer. Its `take`/`replace`/`give` route removes a Session while borrowed, refreshes current equations, accounts current retained bytes and destroys evicted Sessions outside the cache borrow. The candidate must preserve this thread-affine ownership, current budget, cancellation/drain and destruction behavior. Numeric factors never transfer between applications or into the separate response-factor problem.

## 3. Scenarios, physical meaning and numerical obligations

| Scenario | Required response |
|---|---|
| S01: unrelated physical declaration is added | Re-admit current source; equal consumed products may survive without retaining old receiving shells. |
| S02: consumed conversion/reference state changes, absence becomes present, ambiguity appears, or interpretation changes | Affected products invalidate or refuse; unchanged positive IDs are insufficient. |
| S03: refresh fails, is cancelled, or races another borrower | No mixed old/new compiler context escapes; prior products remain valid under their original owners and one-writer/lease rules hold. |
| S04: compatible KLU application changes coefficients, including an accepted start | Symbolic may survive; prior Numeric is unusable before the application proceeds. |
| S05: setup/refactor fails, structure/settings change, or cache pressure releases the Session | No stale substitution; full next setup or explicit incompatibility, followed by authentic drain/destruction. |

| Physical element | Meaning retained | Authority |
|---|---|---|
| Quantity, unit and conversion | Dimension, scale, offset, shape, basis and exact conversion applicability | Quantity resolution/checking; blueprint §8 |
| Reference state and provider | Typed reference conditions, validity, interpretation, ordered output and configuration | Physical/provider admission; blueprint §8.4 and §9.4 |
| Numerical state | Current coefficients, starts, guards, scaling and original residual obligations | Current case and native application; blueprint §16 and §18 |

No variable-role, equation-structure or formulation change is proposed. Existing degree-of-freedom/structural admission remains before solving; topology is not substituted for incidence or solve order. Derivative sources, domain guards and scaling remain unchanged. KINSOL owns iteration/globalization and native setup policy within each application; original physical residual/outcome checks remain independent. The short comparison uses ordinary `0.001` accuracy, without tightening the engineering contract.

## 4. Foundations and gates

These judgments concern the proposed contract, not executed behavior.

| Foundation | Judgment and basis |
|---|---|
| AP-01 | Satisfied: resolution owns dependency meaning; compiler admission owns refresh; native adapter owns factors. |
| AP-02 | Satisfied: versioned consumed equivalence is separate from interchange and receiving permission. |
| AP-03 | Satisfied: existing lanes, retained mathematics and implicit Session cache compose the change. |
| AP-04 | Satisfied: absence, ambiguity, interpretation and numeric validity have explicit governing meanings. |
| AP-05 | Satisfied: sealed issuance, exclusive refresh and frozen native construction enforce the consequential boundaries. |
| AP-06 | Satisfied: closure/refusal and native lifecycle can be exercised with their actual bounded dependencies. |
| AP-07 | Satisfied for the target: dependency-aligned reuse removes broad invalidation; native adoption is conditional on a useful complete route and removes an unsuccessful candidate. No speedup is claimed. |

G1–G6 pass for the proposed authority, fidelity, validity, hidden-input, recovery and reuse contracts above. G7 passes because eligibility and evidence limits remain explicit. G8 passes because Salsa retains dependency execution and SUNDIALS/KLU retain numerical machinery. G9 follows the foundation judgments.

PS-G1 passes for the proposed complete physical-premise contract. PS-G2 is unaffected by this change and its existing structural admission is preserved, not requalified. PS-G3 passes for the proposed fresh-numeric and independent original-check obligations. Runtime adequacy remains untested by this review.

## 5. Findings, alternatives and follow-up

No additional target-design defect was found. Existing F05 and F13 remain at their Plan45 disposition owner; their current source causes are not new findings under different IDs. The preservation requirements above are part of the reviewed contract.

For F05, whole-context invalidation remains the simplest safe fallback. Positive-ID-only keys are unsound. Resolver-owned projections justify their additional machinery by localizing invalidation without transferring interpretation to each consumer.

For F13, stock KLU remains the simplest supported route; existing eligible SPGMR/preconditioned routes remain complete alternatives. Skipping initialization or carrying previous numeric validity is unacceptable. The private fixed-pattern extension earns adoption only if the short screen and complete comparison justify its lifecycle and maintenance burden.

**Planned verification:** physical controls distinguish unrelated additions from changed transitive units/reference conditions, absence, ambiguity, visibility and checker/provider configuration; compare fresh and retained products while checking current shell ownership, failed refresh and historical isolation. Native controls cover accepted starts, changed coefficients, incompatible structure/settings, failed setup/refactor and recovery, current accounting, drain/destruction and the real implicit-cache caller. Actual native counters and original residual checks distinguish symbolic retention from numerical-factor reuse. These are acceptance questions, not a new assurance platform.

## 6. Rule impacts and final decision

The target follows already accepted Plan45 RC03 and RC08. No additional operator rule decision is proposed. The coordinator must amend the affected current contract text in blueprint §5, §8, §14.4, §18 and §23.1 through the prerequisite ADR/design route. If those amendments are not made, retain current whole-context invalidation and stock KLU.

**Architectural fitness: Accept. Behavioral/semantic contract: Accept at Proposed/Interface-checked evidence. Overall: Accept the bounded target design.** No edits, builds, probes, tests or measurements were performed. Source feasibility is not implementation acceptance; F05/F13 close only on their owning implementation and qualification evidence.
