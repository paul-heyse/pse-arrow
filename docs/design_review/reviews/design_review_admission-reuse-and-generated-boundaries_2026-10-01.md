---
title: Admission, owned reuse and generated operation boundaries
date: 2026-10-02
purpose: target
tier: design
standard: core-3.3
profile: process-simulator-1.3
baseline: bbd7e0d85
evidence: Proposed
decision: Accept
adrs: [ADR-0150, ADR-0151]
---

# Admission, owned reuse and generated operation boundaries

**Accept both ADR targets at Proposed evidence.** The design puts interpretation in checked occurrences and explicit immutable admission contexts, separates reusable mathematics from revision attribution and value bindings, and generates transport from the Rust operations that own admission. Those boundaries fit the authorized 25h/25i/25j scope. They preserve actual effect, configuration, allocation and selected-case structural enforcement while deleting obsolete front doors. The historical identity clarification incorporated during review closes the one material ambiguity found in the initial draft.

Behavioral and semantic adequacy of the **proposed contracts** is accepted. Architectural fitness of those targets is accepted independently. This is not acceptance of the concurrent implementation, a claim that the baseline conforms, or product/scientific qualification.

## Scope and evidence

Independent delegated review by the design-reviewer role, completed 2026-10-02. The assigned filename retains the decision drafts' 2026-10-01 date. The selected standard is Core 3.3, process-simulator 1.3 and the pse-arrow binding. This is a design-tier, target-purpose review of [ADR-0150](../../adr/0150-checked-admission-and-owned-reuse.md) and [ADR-0151](../../adr/0151-generated-operation-boundaries.md), including their contracts in [25h](../../plans/25h-authoring-and-admission-ownership.md), [25i](../../plans/25i-identity-reuse-and-resource-ownership.md) and [25j](../../plans/25j-generated-boundaries-and-library-consolidation.md).

Source evidence uses clean `bbd7e0d85`; concurrent H/I edits were excluded from implementation judgment. Inspected neighbors include source occurrence/checking, compiler workspace and selected-flow preparation, runtime preparation/rebinding and package view retention, engine policy and validation, rule invariant lowering, operational attempt/schema owners, Python flow decoding and generator collection validation. Architecture owners examined were blueprint §4.6, §5.3, §7.7, §14.3/§14.4, §18.8 and §21.5, with ADR-0146's preservation contract. Pinned Salsa 0.28.4 capability evidence and the existing native DataFusion cache envelope support library fit, not performance claims.

The workloads considered are immutable edit/re-solve, A/B/A bindings and studies over shared preparation, local admission, durable identity evolution, and language-boundary success/refusal/partial/cancellation. Square simulation, optimization, dynamics and estimation are consumers of these boundaries; their numerical algorithms, physical-model adequacy and full execution journeys are outside this review. No tests, probes, full qualification, formatters or linters were run. Production source was not edited.

## Responsibilities and contracts

| Owner | Decision hidden and contract consumed | Effects and lifecycle |
|---|---|---|
| Source checker and specializer | Occurrence role, declaration/document-relative position, lexical/import context, physical admissions and unresolved specialization obligations; consumers use the occurrence key, not a text match | Checked syntax and attribution are immutable; synthetic nodes record their originating operation |
| Runtime/engine composition root | Registry, actual planner/functions/configuration and local versus runtime capability | Context captures one immutable generation; memo construction releases the map lock before building; typed local and cross-worker cycles refuse |
| Identity owner | Canonical framing, role-specific projections and version changes | New semantic preimages receive new frames; recorded evidence retains its historical provenance |
| Compiler/prepared-product owner | Mathematics, selected structural view, revision provenance and bound values | Local Salsa queries remain pure; owned products and allocation leases escape, database handles do not |
| Shared runtime retention | Admission/body/view retention by complete key | Eviction removes retention, never a live product's allocation owner; mutable evaluators and attempt/native state are excluded |
| Rust operation and generation owners | Decoded transport, contextual admission, effective defaults and typed outcomes | Python performs conversion and shape checks; native admission binds requests to the selected immutable revision |

The baseline already supplies useful domain distinctions. `pse-modeling/src/expression/occurrences.rs` distinguishes declaration, field role and repeated position and retains lexical context and physical obligations. `pse-compiler/src/workspace/modeling/flow.rs` resolves selected nodes and connections against the admitted model, refuses cut connections or unknown policies, and admits the physical graph. J should move adapter-owned duplicate checks and decoding into that Rust-owned operation boundary; it should preserve this contextual admission rather than replace it with generated schemas.

The current `PreparedCase` has the **selected-case** `StructuralAnalysis`, plan, derivation and value facts. That witness remains consumed. Deleting an unused whole-model structural product does not authorize deleting selected-view assessment or substituting topology for equation incidence. Likewise, the baseline engine `EffectivePolicy` composes effect ceilings, settings and origins, and byte limits. Its requirement-planner API has a real lowerer and test surfaces; removing the obsolete public SQL-requirement capability is not a blanket deletion of policy. ADR-0150 explicitly retains the actual effect/settings/bytes behavior and source/physical enforcement.

### Complete reuse and ownership

“Complete admitted dependencies” means all dependencies actually consumed, including failed lookups and membership additions/removals. The role map in I1 distinguishes source revision, admitted closure, semantic body, prepared view, binding, profile, lineage request and operational occurrence. Imports, consumed physical definitions, provider/capability contracts, functions and relevant admission limits cannot be hidden behind unchanged root bytes. Structural roles, coordinate ordering and compiler settings belong to prepared-view identity; execution settings belong to effective profile identity. Value bindings are immutable keyed inputs so A/B/A can coexist. Conservative invalidation is lawful where finer dependency capture is not established.

A body reused across revisions must acquire the new revision's occurrence/source association at use. An old result keeps its original attribution. Reuse must also preserve applicability/domain prerequisites and selected provider obligations; numerical equality alone cannot supply permission. These are semantic boundaries, not names for interchangeable `ContentHash` values.

The I2-before-I3 ordering is necessary. In the baseline, `PreparedCase::rebind` clones the owned occurrence map while runtime ownership distinguishes shared and newly allocated payload. The target must either share that unchanged provenance or charge its new allocation. An aggregate retained-size estimate cannot prove unique live allocation ownership. Mathematics, attribution and binding owners must survive escaping products, eviction, dropping originals and workspace rotation, with final release only after the last owner. Positive cache-entry overhead matters even when payload is mostly shared. The existing conservative stack/opaque-library allowances serve a separate purpose and need not disappear.

### Historical identities and boundary equality

The preserving V6→V7 transition must retain recorded bytes **and their historical interpretation**. Renaming `request_identity` to an operational role cannot prove that every old value was computed with a new operational-job frame: baseline durable requests include several scientific/request projections. The revised ADR-0150 explicitly keeps historical or unknown-frame values as typed recorded evidence and requires frame proof before a current job/cache key can consume them. Old-frame verification is distinct from computing a new identity. “Never recomputed” is read here as forbidding historical replacement under new semantics; verification of the original recorded contract remains permitted under ADR-0146.

Identity float framing and collection equality intentionally differ. I1 canonicalizes NaN payloads while retaining signed-zero/infinity distinctions under its named frame contract. J3 uniqueness uses the **admitted element domain's equality**, so `-0.0` and `+0.0`, including nested values, are duplicates when collection equality says so. Reordered maps cannot become distinct merely because serialization order changes. A role hash is therefore not a generic uniqueness key. The type-directed equality key remains implementation work, as J explicitly states; unsupported values may not be newly rejected to make a set implementation convenient.

## Distinguishing scenarios

| Scenario | Stimulus and expected boundary | Settling evidence required in implementation |
|---|---|---|
| S01 — source/context edit | Identical text at different locations, inheritance/import changes, identity policy changes and binary wrappers retain correct occurrence and document attribution; final incremental admission agrees with clean loading | Repeated malformed occurrences, inherited/indexed bindings, clean/incremental final-source comparison; synthetic AST origin |
| S02 — context composition | Two incompatible native contexts cannot share prepared validation; local contexts refuse absent runtime semantics; different-slot nesting is allowed while A→B→A construction refuses | Order independence, independent concurrent waiters, same-thread and cross-worker cycles, interrupted-build/wait cleanup |
| S03 — value/revision reuse | A/B/A and span-only edits reuse substitutable mathematics while refreshing attribution; changed consumed dependencies invalidate affected products | Admission/preparation counters, negative/membership dependencies, provider/physical changes, old-result attribution and selected-view witness |
| S04 — eviction/lifecycle | Escaped products outlive cache eviction and database rotation; native work outlives an abandoned waiter through required cleanup | Nonempty provenance, changed/unchanged rebindings, denied retention/allocation, final reservation release; preserve completed I4 controls |
| S05 — operational evolution | V6 rows and lineage survive V7 without modern-role inference; unsupported provenance refuses as typed evidence | Old bytes/history preservation, known old-frame verification, unknown-frame refusal, current-frame role-swap controls |
| S06 — operation representation | Add an enum/field/default or invoke a flow selection through Rust/Python; semantic admission remains in one Rust owner | Regenerate builders/getters/stubs/readers/errors together; omitted/explicit defaults, duplicates/context-invalid selection, pre-result refusal/partial/cancellation; scalar/structured uniqueness |

These cover binding, policy and mechanism variation. Adding new physics and qualifying solver replacement are outside this subsystem target; it preserves their consumed contracts instead of introducing a new universal authority.

## Library fit and alternatives

Local Salsa admission and service retention own different scopes. Salsa's tracked inputs/equality and memo lifecycle fit incremental local computation; untracked configuration reads and reused interned handles across generations do not. Escaping immutable products avoids turning local database IDs into durable identity. The existing DataFusion `DefaultCache`/`Cache` envelope supplies byte-based retention and TTL/LRU with key overhead, but its size callback is not an allocation lease. Reusing that mechanism avoids another hand-written package LRU while leaving immutable product ownership explicit.

For J, strum supplies closed-vocabulary mechanics, petgraph supplies dependency ordering/cycle machinery, and attrs supplies cardinality checks. Semantic projections, deterministic emitted order and equality-consistent structured keys remain their existing owners' obligations. These libraries do not establish Rust contextual admission or invent scientific defaults. Their adoption removes bounded glue, not the domain contracts.

The smallest viable correction is context-complete checked products plus explicit owners, then broader retention. Keeping text-first reparsing, ambient first-wins validation and a fixed-count package LRU leaves the specific defects intact. An all-purpose validation/cache/scheduler framework adds unrelated decisions without serving a demonstrated variation axis. Generating a second schema instead of using the actual owned Rust type also leaves duplicated authority. The selected target is preferable to all three; no speed-up or RSS guarantee follows from that judgment.

## Foundations and gates

All verdicts below assess the target at **Proposed** evidence.

| Foundation | Verdict and reason |
|---|---|
| AP-01 | Satisfied: checking, semantic admission, identity, allocation, retention and transport have distinct reasons for change |
| AP-02 | Satisfied: immutable context, owned products and Rust operation contracts expose required meaning and refusals; historical provenance is explicit |
| AP-03 | Satisfied: runtime roots compose admitted operations and attempt owners without a universal framework or copied Python policy |
| AP-04 | Satisfied: occurrence, context, identity role, physical binding and selected structural witness are adequate concepts for this scope, and behavior consumes their owners |
| AP-05 | Satisfied: dependencies, local/runtime capabilities, cycles, allocations and old/new identity interpretation are explicit |
| AP-06 | Satisfied: checking, context cycles, equality, reuse and lifetimes admit local controls without unrelated stores/solvers |

| Gate | Target judgment and evidence limit |
|---|---|
| G1–G3 | Pass: one owner per admission/identity meaning; typed context and historical refusals; source/physical enforcement retained. Runtime realization remains to be verified |
| G4–G5 | Pass: pure local queries exclude attempt effects; explicit contexts and cycle refusal; immutable revision publication and completion-owned native guards |
| G6 | Pass: complete role keys, source/mathematics separation, immutable bindings, explicit versioned preimages and equality-consistent uniqueness |
| G7 | Pass: ADRs label target benefits Proposed; counters do not claim performance, RSS bounds or scientific qualification |
| G8 | Pass: selected Salsa/native cache/generator libraries fit their bounded mechanism roles; bespoke semantic adapters remain justified |
| G9 | Pass: the six foundation judgments support the representative scenarios without averaging or using functional tests as architectural proof |
| PS-G1 | Pass for preservation: physical scope/definitions, contextual flow admission and source obligations remain authoritative; physical-model qualification excluded |
| PS-G2 | Pass for preservation: selected `PreparedCase` structural witness remains; stream topology is not a substitute |
| PS-G3 | Pass for preservation: mutable evaluators, solver state, cancellation, outcome verification and I4 completion remain attempt-owned; no new numerical capability claimed |

Physical semantics remain owned by the admitted quantity/physical registry: quantity dimensions and canonical units, basis, datum/reference conventions and validity/applicability obligations participate in interpretation and reuse where consumed. This change introduces no quantity, basis, validity envelope, derivative scheme, scaling rule, solver class, tolerance or status-to-outcome rule. Consequently no new numerical-stage or unit/property conformance claim is made. Source checking captures obligations before lowering; selected-variable roles and structural analysis remain before solving. That is the well-posedness preservation contract, not an executed proof.

## Findings, authority and decision

No blocking finding remains in the reviewed draft. The initial ambiguity about historical digest role/frame provenance was corrected in ADR-0150's Outcome during this review; the correction is **Proposed**, not implemented migration evidence. No MUST exception or SHOULD deviation is requested.

Blueprint §4.6's global SQL-enforcement description, §5.3's serializer owner and affected frame contracts, §14.3/§14.4's reuse/allocation descriptions, and §21's exported boundaries must be amended through the decision/design route identified by the plans. Accepted historical ADRs remain unchanged. The [series coordinator](../../plans/25-design-remediation.md) is the single disposition owner; H/I/J own implementation status and evidence. This review does not create a second backlog or close those plans.

**Behavioral/semantic target adequacy: Accept. Architectural target fitness: Accept. Overall: Accept ADR-0150 and ADR-0151 at Proposed evidence.** Implementation acceptance depends on the scenario controls above and direct producer/consumer/deletion completion. I2 ownership must precede I3 service retention; H's context-complete admission feeds those keys; J migrates each operation's success and failure consumers together. Database/storage, cross-language journeys and performance qualification remain in 25k. Acceptance of these decisions does not establish any of those results.

## Bounded cache-witness clarification — 2026-10-02

**Accept scoped at Interface-checked/Implemented evidence.** This additional static assessment
examines the current compiler semantic-body retention seam and its runtime allocation owner;
it leaves the original `bbd7e0d85` target review and Proposed verdict unchanged. It does not
review the whole concurrent implementation or establish a Tested claim.

The relevant purity is mathematical stability: the same complete immutable admitted key
selects the same admitted mathematics. Cache lookup counters, retention and allocation
accounting are infrastructure effects, so calling this seam effect-free would be inaccurate.
The attachment is selected once before revision publication and cannot change behind Salsa's
published inputs. Attempt effects, mutable evaluators and native worker state remain excluded.

The current `AdmittedBody` scientific fields are compiler-private. Its complete semantic-key
proof is minted only by grouped admission; a raw mathematical request has no such proof.
Callbacks receive immutable products and cannot copy an expected public hash onto different
mathematics. A cache hit checks the sealed key and physical identity; retention checks that its
returned body preserves the original admitted meaning. Owner attachment changes accounting
without replacing that proof. This corrects the weaker check of publicly mutable specification
hash fields, which alone could not establish callback trust.

Transient reservation or retention refusal aborts tracked evaluation through the cancellation
unwind and is recovered as its typed front-door cause. It is not memoized as a semantic result,
so retry with the same inputs remains possible. Confirmation belongs to the targeted controls
`effect_refusal_does_not_poison_incremental_body_admission` and
`foreign_retention_cannot_supply_a_body_for_another_dependency_closure`, together with the
complete-key reuse and owner-release controls in [25i](../../plans/25i-identity-reuse-and-resource-ownership.md).
Their execution evidence and any remaining failures stay with that plan; this static assessment
does not claim performance, integrated qualification or full-plan closure.
