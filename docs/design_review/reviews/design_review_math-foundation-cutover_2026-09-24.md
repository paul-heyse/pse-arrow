---
title: Library-owned mathematics foundation cutover
date: 2026-09-24
status: accepted-scoped
scope: Plan 14 M00–M05 and ADR-0082–0084
evidence: Implemented and Tested for the named foundation controls; later simulator capabilities remain Proposed
---

# 1. Decision and scope

**Accept the scoped foundation design.** This review covers direct typed-definition
lowering, finite binding, guarded value evaluation, library composition profiles and
the approved removal of old compile/solve routes. It does not accept the full native
simulator or confer accepted status on an ADR. The user-approved Plan 14 target
governs this alignment review; conflicting blueprint text has an explicit amendment
route in §10.

**Method and coverage:** inspected the actual math/compiler/provider/runtime
boundaries, retained graph consumers, registry/generator removals, native contracts
and scope runner. Executed focused Rust and tooling units and workspace compilation;
regenerated contracts/fixtures. Used the Symbolica/faer/FeOS/num-dual and native-solver
skills and exact pinned public interfaces. Symbolica source was not inspected.
No whole-model solve, Python journey, durable math artifact, performance campaign,
cross-platform qualification or full static/governance suite was run. Tests are
counterexamples to specific risks, not formal equivalence proofs.

The former implementation's mathematical identity, derivative engines and Pyomo
routes are removed. The remaining uncertainty concerns later packages, especially
complete provider derivatives, original-model quality and resource closure.

# 2. Authority and lifecycle map

| Meaning | Owner and revision | Derived representation/update |
|---|---|---|
| Definition and physical interpretation | Authored source plus selected quantity registry | Typed AST to Symbolica; physical checks precede normalization. |
| Mathematical body | `BodySpec` semantic definition/structure/physical/provider/policy key | Immutable evaluator artifact; rebuilding does not change model authority. |
| Instance and case | Semantic ports/rows and fixed/free structure; ordinary values separate | Affine canonical slot bindings and free-variable layout. |
| Provider | Executable factory and complete immutable `ProviderSpec` | Worker-local state; registration-only declarations cannot execute. |
| Trial | Supplied input values, cancellation and admitted neighborhood order | Fresh frame and cloned library scratch; complete finite values or typed failure. |
| Persistent result | Outside M00–M05 | No new mathematical publication or solver result is advertised. |

Library optimization and allocation are deliberately opaque mechanisms, not competing
semantic authorities. Semantic hashes exclude printed atoms, local handles and source
spans. Distinct source definitions retain their own diagnostic identity.

# 3. Semantic contracts and invariants

| Contract | Enforcement and refusal | Evidence |
|---|---|---|
| Physical meaning before CAS | `pse-math/src/typed.rs:294`, `binary` calls physical inference before constructing arithmetic; typed result contracts cannot be externally retagged | Point subtraction/addition and datum mismatch controls. |
| Original-domain obligations | `typed.rs` materializes arguments before `Require`, then replaces them with barrier formals | `x/x`, `exp(log x)`, nested inverse/log and invalid constant untaken branch controls. |
| Complete finite shape | Compiler group admission and actual tuple lookup; missing tuple is an error, not zero | Empty, filtered, multiplicity and ragged controls. |
| Canonical instance coordinates | `pse-math/src/binding.rs:111`, `SlotBinding::new` checks full contract and canonical formal representation | Cross-unit connection and alias/fixed-free controls. |
| Executable provider and neighborhood | `pse-kernels/src/lib.rs:233`, `Registration::new` constructs and checks the worker; every subsequent worker must match | Attributed recoverable failure, recovery, phase mismatch and insufficient-order controls. |
| Safe control schedule | `pse-math/src/guarded.rs:477`, dependency validation uses library symbol discovery and branch intersection | Forward reads, overwrite/partial-output refusal and lazy short-circuit tests. |

Empty sum/product have physically checked identities. Empty min/max are refused.
Value validity, smoothness and derivative implementation are separate. Mathematical
equality is real-algebra equality on the admitted domain; numeric fixtures use stated
analytic tolerances. Ordered floating-point identity is intentionally withdrawn.

# 4. Derivation and execution design

The per-stage RCA §9 contract, profiles, capacities and consumers are recorded in
[the foundation contract](../../plans/14-math-foundation-contract.md). In brief:

| Stage | Backend and dependencies | Ownership, termination and equality |
|---|---|---|
| Physical input | Native Arrow/DataFusion projection of actual selected declarations | Checked batches and allocation lease; absent and empty remain distinct. |
| Typed preparation | Rust physical inference and Symbolica atoms; finite shape, provider, physical and numerical interpretation | Local bounded expansion; no arithmetic relation transport or whole-model graph. |
| Reuse | `BodyStore`, `typed_math.rs:176` and `:263` | Actual physical registry equality, explicit shape capacity and semantic key; hit reuses library compilation. Salsa/persistence is absent until M10. |
| Trial | Scalar interpreted library blocks plus branch/domain/provider control | Fresh output frame and worker scratch. Cancellation and failure cannot return a partial result. |
| Graph work | petgraph/rustworkx package order and rule SCC/strata only | Existing whole-inventory consumers retained; unused incidence/DM/containment/tear paths removed. |

There are no new mathematical persistence effects. Native library construction/linking
is interface evidence; it does not establish solver iteration behavior. Atom-level
normalization and evaluator optimization remain library-owned. Formal symbol count,
local stages/occurrences, integral degree and optimizer work are bounded explicitly.

# 5. Representative journeys

An authored indexed sum resolves its domain and actual tuple membership, performs
physical inference, creates a local body and binds repeated instances to that body.
Adding an instance with the same shape reuses the evaluator. A new shape changes its
semantic key and may consume another bounded specialization slot.

A Celsius-to-Kelvin connection converts each port at the binding boundary before
temperature point subtraction. A datum/basis mismatch is rejected instead of being
treated as a unit scale. A failed provider trial returns its original typed cause
and source identity; the next trial starts with fresh scalar outputs.

# 6. Acceptance gates

| Gate | Scoped verdict | Evidence and limitation |
|---|---|---|
| G1 Authority | Pass | Old arithmetic engines/transport and duplicate executable operator tables removed; typed physical source and actual handler/provider registration own their meanings. Governance amendment is explicit, not silently accepted. |
| G2 Semantic fidelity | Pass | Physical-before-CAS, domain, finite membership and conversion controls pass. Full flowsheet lowering and derivative assembly are excluded. |
| G3 Validity | Pass | Ordinary typed compiler path rejects incompatible physical/profile/binding inputs; control scheduling rejects unassigned reads. Raw library/control APIs require physically admitted construction. |
| G4 Hidden behavior | Pass | Provider creation and library compilation are explicit preparation effects; tracing/thread policy is explicit; no inspection or numerical path writes a model. |
| G5 Consistency and recovery | Pass | Value evaluation returns only complete finite results and preserves failure causes; cloned scratch and fresh frames prevent stale outputs. Durable numerical publication is excluded. |
| G6 Transformation and reuse | Pass | Domain-erasing rewrite and reuse controls pass; body identity excludes library handles/formatting. No cross-revision Salsa or persisted cache claim. |
| G7 Truthful capability claims | Pass | Unsupported functions refuse; function view derives from actual handlers; provider factory required; public solving is withdrawn. Conic/DAE contracts are not advertised executors. |

These verdicts settle the foundation scope only. The original reviews' unresolved
whole-simulator gates remain owned by their subsequent Plan 14 packages and M22.

# 7. Principle findings

| Finding | Principles | Evidence/gap | Consequence | Correction/disposition | Verification |
|---|---|---|---|---|---|
| Original obligations can disappear through CAS, including constant invalid untaken branches | DM-07/24; RCA §2 | `BodyBuilder::require` and branch regions; fixed during implementation | A normalized expression could hide a required failure or eagerly fail an untaken branch | Materialize/guard original operands before substituting barrier formals | Guard and compiler units pass. |
| Formal units must agree with the canonical body representation | DM-06/41 | `SlotBinding::new`; fixed during implementation | Valid Celsius inputs could be passed as Kelvin coordinates | Require canonical target formals and check full physical type | Cross-unit connection unit passes. |
| Future derivative/solver capability must not be inferred from a value body or library type | DM-43/59 | `GuardedValueArtifact` returns only values; native traits have no executor | A caller could assume C2 admission supplies a Hessian or a conic struct supplies a solve | Explicitly narrow current scope; complete M06–M18 | M07/M08/M11–M18 and M22 remain required. |
| Whole-attempt resource closure is not established by local capacities | DM-35/40; RCA §8 | Bounded construction and optimizer controls do not intercept all native allocations | Foreign work can consume more memory than a local slot estimate | Keep logical-limit claim; implement shared attempt admission in M10/M21 | M22 resource/cancellation qualification remains open. |

**Applicability:** authority, physical typing, lowering, reuse, capability, boundary and
resource groups apply. Publication recovery and temporal graph analysis are not
exercised by this in-memory math scope. Migrations are excluded by the explicitly
approved hard cut. No performance ranking is assigned without measurement.

# 8. Alternatives and architectural leverage

| Alternative | Tradeoff | Disposition |
|---|---|---|
| Retain legacy engine behind an adapter | Competing arithmetic/derivative/domain meanings and transport costs remain | Rejected by target and hard-cut authorization. |
| Typed library blocks with explicit guards/providers | Small amount of control code preserves domains, lazy branches and typed failure; libraries own arithmetic | Selected and exercised. |
| One direct scalar evaluator for the whole expression | Less control machinery, but no qualified lazy/fallible provider composition for the required domain cases | Simpler alternative remains unsuitable for the admitted semantics; optional monolithic/JIT modes require later evidence. |

No plugin platform, custom AD tape, sparse factorization, root iteration or license
framework was introduced. Ordinary Rust handles physical admission, finite membership
and attributed error transport where those are application semantics.

# 9. Verification and remaining measurement

**Tested:** Linux Rust 1.98.1, nextest default parallel execution, explicit
`pse-relations/force-validate`; locally provisioned Symbolica license. The execution
packet's Verification section lists commands, counts and the zero failure baseline.
It includes compiler and guarded math controls, native contract controls, exact
library composition, retained graph units, Ipopt ABI/linking and scope tooling units.

**Implemented:** workspace compilation and generated Rust/Python/docs/invariant
fixtures reflect the hard cut. The editable extension and its native stub were refreshed
with `just py-sync`. Full repository static gates, installed Python journeys,
provider/solver/component journeys, end-to-end cost and accuracy are **not run** at
this stage; they remain M22. Third-party future-compatibility diagnostics from
`proc-macro-error2` remain visible for final dependency qualification.

# 10. Exceptions and unresolved decisions

The [foundation contract's concrete amendment](../../plans/14-math-foundation-contract.md#concrete-decision-amendment-prepared-for-the-design-pr)
names D6/D9/D10/D11/D12, relevant sections and accepted/pending records to reconcile.
ADRs 0082–0084 remain **proposed**. Formal supersession, blueprint revision markers,
index and governance qualification follow the designated ADR/design PR route.

RCA permits specialized computation and does not mandate Salsa where it has not yet
been adopted. Interpreted guarded blocks are a deliberate bounded baseline; JIT/SIMD,
full physical provider derivatives and broad solver classes are unsupported until
qualified. This narrows scope instead of waiving a MUST-level requirement.

# 11. Decision and implementation changes

| Priority | Decision/change | Evidence and next owner |
|---|---|---|
| Correctness | Accept the M00–M05 typed and guarded value foundation | Named units and compilation; no whole-target acceptance implied. |
| Semantic ownership | Keep the hard deletion and the evidenced package/rule/data consumers | Removal inventory; M09/M10 rebuild new consumers from actual target contracts. |
| Next implementation | Real provider, derivative artifacts and sparse assembly | M06–M08; later native solving and original-model quality remain necessary. |
| Final acceptance | Resolve formal governance and perform complete target qualification once | ADR/design PR and M20–M22; performance benefits remain unmeasured. |
