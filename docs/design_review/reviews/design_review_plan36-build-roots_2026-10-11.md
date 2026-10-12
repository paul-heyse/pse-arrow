---
title: Plan 36 responsibility-local admission and build roots
review_date: 2026-10-11
tier: change
purpose: target
standard: core-3.4 + process-simulator-1.5
baseline: 2d6bf6ae7d03a8a9f6f109696bfe3d9cc8455466
disposition_owner: docs/plans/36e-build-test-and-development.md
---

# Plan 36 responsibility-local admission and build roots

## 1. Scope and decision

**Accept the proposed target at Proposed / Interface-checked design level.** The independent reviewer assessed ADR-0175, blueprint §3.2 and Plan 36e P-E4 against Core 3.4, Heuristics for Efficient Architecture 1.0, process-simulator 1.5 and the pse-arrow binding. Behavioral adequacy and architectural fitness are satisfied for this bounded target. Migrated compilation, feature closure and scientific qualification remain open.

The functional target is compiling/testing a responsibility with its actual dependencies while preserving production source admission, physical meaning, finite ownership, generated authority and scientific acceptance. Variation includes admission changes, generator capability selection, surrogate integration and package/feature/target selection. Current policies/ADRs were context and potential rule impacts, not judgment criteria. Scope excludes the whole series, R36-01/02/03, new solver algorithms, general compiler redesign, host policy and whole-product qualification. Plan 36e owns acceptance; the coordinator owns integration; P-D1/P-D5/P-C9 retain their prerequisite owners.

## 2–3. Responsibilities and contracts

`pse-admission` owns effectful document intake and physical source admission. Pure document values, grammar, identity assignment and parse vocabulary stay lower; scientific preparation, solvers and canonical publication stay Runtime-owned. Inspected authoring-driver production code uses lower authoring/model/modeling, relation, engine, quantity, parsing/data, allocation and cancellation contracts. Its error preserves classified causes. Physical inventory/plans/preconditions likewise use lower owners; compiler-facing preconditions are quantity types, not a compiler instance.

Tests currently reaching workflow PhysicalContext/WorkflowError must split into lower inventory assertions and retained Runtime composition tests. Moving them unchanged or adding a Runtime dev dependency defeats the boundary. Actual BoundModel/engine validation supplies implementation/settings identity; a registry fingerprint is insufficient. Source bytes, checked batches, allocation and spill retain original lifetimes. Relocation creates no second validator.

| Contract | Preserved owner/meaning |
|---|---|
| Document bytes, parse budgets, identity and edit/source ownership | Admission effectful loading/editing; pure contracts remain lower |
| Local admission | Actual bound native validation; unchanged owned inputs retain applicable admission |
| Physical inventory | Exact rational/affine quantities, units, bases, reference states and typed preconditions; absent optional inputs stay absent |
| Mathematical context | Neutral quantity and Boolean kind from actual reference.math_context; conflicts refuse, no invented neutral declaration |
| Operand requirements | Original operand order, equal-basis and reference-difference distinctions |
| Resource/error lifetime | Finite pool, cancellation and retained leases; classified lower causes may be wrapped by Runtime |
| Surrogate coordinates/observables | Original ordered physical identities, fidelity/correspondence and original-model screening/correction |

Source loading does not establish well-posedness. Structural analysis and numerical class selection remain downstream. No formulation, derivative, tolerance or solver acceptance changes are selected.

## 4–5. Scenarios and execution fit

**S01: admission change/test.** The lower crate cannot depend on Runtime/math/compiler/operations/native backend, including test setup. All Runtime/Python/bench/xtask imports migrate; replaced modules/paths retire. Pure-authoring relocation would widen unrelated DSL consumers; a Runtime facade only hides the dependency burden.

**S02: physical generation.** Sources::load currently constructs SharedRuntime only for pool/factory capability. EngineResources::build and EngineFactory::new already provide it, demonstrated by the standalone pure route. Direct finite construction preserves actual bound validation, package selection, registry agreement, declaration/package resolution, native invariants and documents/session/resources/spill lifetime. It does not substitute a reduced validator or create a store-free general Runtime.

**S03: generator selection/ownership.** Registry bootstrap, physical admission, full schemas and native ABI must be real optional dependency/target capabilities. A subcommand in an always-broad binary is insufficient. Existing package-fixtures combines unrelated reasons for change. Full schemas legitimately derive actual workflow/native/operations DTOs and therefore retain higher dependencies; copying DTOs/handwritten schemas would duplicate authority. Native ABI retains actual selected headers/bindgen. Adapter declarations and roots move before regeneration; pure generated values remain pure. Focused pruning follows declared roots; aggregate comparison covers changed/missing/stale/untracked outputs.

**S04: surrogate integration.** Egobox/argmin/ndarray integration has one Runtime production consumer; moving it beside that consumer narrows math/compiler roots without a new surrogate crate. Keep it distinct from admitted actor orchestration. Non-Send state stays worker-owned; task identity, typed terminal causes, finite work/memory, cancellation and joined drain remain. A surrogate proposal grants no scientific permission. P-C9's compact advancement/charged explicit snapshots must govern relocation; the old copy API cannot be reproduced under a new path.

**S05: feature/target selections.** Retain pinned-nightly workspace unification; remove fallback edges only after actual target/host/dev/native unit controls. Intentionally retire Cargo ignoring that configuration. Lexical trees and feature names do not prove closure. Explicit validation, native absence, one family, locks and per-checkout target remain independent. P-D5 explicitly owns float roundtrip first. Preserve DataFusion SQL/functions/file/compression and consumed external formats; fixture spellings cannot define a reduced product input contract.

## 6. Foundations and gates

| Foundation | Verdict and grounding |
|---|---|
| AP-01 | satisfied: effectful admission, pure values, workflow, generator and surrogate owners are coherent |
| AP-02 | satisfied: existing physical/source/error contracts cross the boundary without a Runtime facade |
| AP-03 | satisfied: actual lower resources/bound validation compose; full schemas retain real higher types |
| AP-04 | satisfied: physical/context/precondition and proposal/acceptance distinctions govern behavior |
| AP-05 | satisfied: forbidden reverse dependencies, capability roots and qualification are explicit |
| AP-06 | satisfied: lower tests consume actual dependencies; workflow assertions remain integration |
| AP-07 | satisfied: unrelated construction/build work is removed while necessary engine/schema work remains |

| Gate | Verdict and scope |
|---|---|
| G1 | pass: one generated/source owner and actual bound validation |
| G2 | pass: source identity, physical distinctions, diagnostics and correspondence preserved |
| G3 | pass at design level: no reduced/unchecked validator; actual controls required |
| G4 | pass: explicit intake/resource effects; pure authoring stays pure |
| G5 | pass at design level: source/allocation/spill lifetime and joined surrogate drain |
| G6 | pass: contract-preserving relocation and complete generator/actual-unit comparison |
| G7 | pass: graph population, compilation, latency and science claims remain distinct |
| G8 | pass: actual Cargo/DataFusion/Arrow/egobox mechanisms retained |
| G9 | pass: applicable foundations satisfied for the bounded target |
| PS-G1 | pass at preservation boundary: exact physical inventory/context/preconditions |
| PS-G2 | not applicable: no formulation/solver entry; source admission claims no well-posedness |
| PS-G3 | pass at preservation boundary: proposal versus original scientific acceptance retained |

## 7. Findings

No additional consequential target defect was established. This depends on explicit no-reverse-dependency, actual-capability, genuine full-schema, P-C9 snapshot and actual-unit qualification contracts. Existing broad dependencies are the diagnosed baseline, not a new target finding. Implementation review must reject renamed facades, still-activated optional dependencies or tests reintroducing higher roots.

## 8–9. Alternatives and library fit

Keeping Runtime admission retains the burden; pure-authoring relocation widens pure consumers; a facade changes appearance only. The cohesive lower crate follows actual consumed dependencies. A separate surrogate crate is unjustified for one consumer; reopen for a genuine independent consumer. Keeping the fallback retains breadth for an unselected support mode; qualified pinned-nightly retirement preserves native Cargo unification and correctness.

Context7 `/rust-lang/cargo` checked workspace feature unification and target selection. Workspace merging, binary required-features and actual unit selection are distinct. Current docs are discovery; pinned config/graphs establish this checkout. Sources: [feature unification](https://github.com/rust-lang/cargo/blob/master/doc/book/src/reference/unstable.md), [build performance](https://github.com/rust-lang/cargo/blob/master/doc/book/src/guide/build-performance.md), [target selection](https://github.com/rust-lang/cargo/blob/master/doc/man/cargo-build.md). Library skills informed exact-release/ownership boundaries; no new numerical API/library is substituted. No stale utilization catalog/capability map was needed.

One concrete crate and explicit wiring trade against narrower roots. Necessary broad generation/science remain. No compile-time or runtime benefit was measured.

## 10. Verification and limits

**Interface-checked:** ADR/plan/architecture, actual admission/physical/errors/resources/bound validation, generator declarations/schemas, surrogate/original-target screening, manifests and Cargo configuration. **Proposed:** extraction, generated migration, capability separation, relocation and qualified fallback removal. Reviewer tests/benchmarks: **not_run**. Read-only review; no edits, topology changes, commits or generated publication.

Investigation graph populations are historical scoped evidence, not migrated compilation, all-feature proof or latency. Acceptance must show lower/physical compile without forbidden dependencies including dev setup; declaration-led adapter regeneration with full aggregate coverage; actual target/host/dev/native family/validation/absence; all Runtime/Python/bench/xtask consumers; preserved physical context/ownership/cancellation/diagnostics; and surrogate proposals/termination/callback/drain with P-C9 snapshot controls. No model added; parity/model conformance/whole-product science retain their owners.

## 11. Rule impacts and disposition

- **build-roots#RC01:** §3.2/§14.3 adds lower admission, generated adapters and direct tests. ADR-0175 rationale; Plan 36e implementation. Keeping ownership retains burden.
- **build-roots#RC02:** ADR-0122 Outcome5 and §3.1/§3.2 retire ignored-unstable fallback only after actual controls. Hack crate/edges, hakari config/generation/checks and generated exception retire together. Keeping fallback retains broad closure; review waives no controls.
- **build-roots#RC03:** Generator capabilities/roots reflect independent registry/physical/schema/ABI dependencies; real DTO/header authority and aggregate comparison remain. Command-only separation cannot deliver the target.

No toolchain date/floor, exact pins/locks, family, per-checkout target, explicit validation, physical or scientific acceptance relaxation. No SHOULD exception. Plan 36e alone owns progress, with P-D1/P-D5/P-C9 prerequisites.

## 12. Decision and handoff

Behavioral adequacy, architectural fitness and overall target: **Accept at design level**. The strongest evidence is inspected lower dependency/ownership composition, genuine higher DTO needs and explicit validation/scientific preservation. Migrated compilation, actual-unit qualification, regeneration and scoped science remain open. Independent review was returned by the design-reviewer; the coordinator published this proportionate synthesis without changing its judgment.
