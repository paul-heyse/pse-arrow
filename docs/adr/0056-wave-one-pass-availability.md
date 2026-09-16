---
id: ADR-0056
title: Register only complete executable pass contracts
status: proposed
date: 2026-09-14
deciders: [paul-heyse]
level: decision
principles: [DM-04, DM-07, DM-30, DM-31, DM-43, DM-46, DM-47, DM-58, DM-59]
blueprint: [§6.13, §14.1, §14.3, §20.1, §23.2, §24.1, §25]
review: docs/design_review/reviews/design_review_wave-1-contract-corrections_2026-09-14.md#6-acceptance-gates
evidence: Proposed
supersedes: []
superseded-by: null
revisit: The missing later-pass schemas are specified and their complete producer/consumer graph can be validated
verification: `just governance`; stage_bundle_graph and separate P0-P3/P10 golden fixtures; plan 03 terminal gates

---

# ADR-0056: Register only complete executable pass contracts

## Context

The original Wave 1 plan requires all seventeen pass registrations but deliberately defers some later-pass relation schemas. It also claims an authored P0-through-P10 golden despite excluding P4-P9.

## Scope

**Current construction target:** ADR-0067 and blueprint revision 37 replace this
record's former local replay, row-copy and phase-limited execution mechanisms.
Plan 05 owns implementation; the domain/identity/lifetime requirements retained
below are implemented through its single native preparation/completion route.

Amends the cited blueprint sections within the approved Wave 1 boundary. It supplements existing accepted decisions; their arguments remain immutable. Implementation is authorized by the maintainer's approved execution plan; formal ADR acceptance remains the decision-PR lifecycle.

## Drivers

Explicit semantics, enforceable validity, complete dependencies and honest capability claims before reuse or performance work.

## Options

Retain the inconsistent scaffold: rejected because its consumers cannot preserve the declared meaning. Build a general replacement platform: rejected without a demonstrated need. Complete the existing typed contracts: selected.

## Outcome

Keep P0-P16 as the blueprint design inventory. Production registration requires fully specified relation contracts and a closed input/output graph. Plan 05 implements the real P0–P10 product path. The former Wave 1 P10 predecessor fixture remains only if it independently tests a leaf contract; remove fixture-only production registration. A fabricated predecessor graph never satisfies the required source-to-P10 workflow. Deferred later-pass registrations and unspecified relation contracts have explicit register entries. B-evidence and the Ipopt bindgen arm remain optional deferrals; bindings retain hygiene-only checking. Mandatory in-scope validators, publication, query, ownership and typed-math guarantees are not deferred.

### Consequences

P1's output inventory includes the reference-package relations explicitly exposed by DocumentSpec and authored provenance assertions as well as authored namespace relations. These are primitive facts, not compiler-derived writes. P2 consumes those staged facts and pinned registry/reference context without trusted planner constraints. P3 starts from the committed P2-valid primitive bundle; its pinned inputs express that boundary rather than pretending P1 candidate outputs were published stage snapshots.

Consumers must use checked contracts and explicit unsupported outcomes. Fixtures and generated output change with their authoritative declarations; this record does not claim runtime acceptance.

### Compensating controls

The scoped design review, negative fixtures and plan 03 terminal gates guard the correction. Existing canonical framing and dependency-family pins remain unchanged except the explicitly named direct promotion.

The Appendix B coverage test expands every listed relation and family, excluding explanatory inline types. Every fully specified relation is registered even when its producer is deferred. The exact unresolved contracts and their missing authority are enumerated in `tests/governance/appendix-b-deferred.toml`; the test rejects obsolete or unexplained exemptions. The implementation audit identified ten omissions from the original eight-entry list: measurement models, tags, phase-equilibrium species, tear candidates, alternatives and alternative sets, static analysis, scaling plans, solver events, and the closure report. These are explicit incomplete-contract deferrals under R-30. In particular, the closure-report declaration was named in original packet A4 and remains a deliberate scope deviation; solver events have a row shape but no declared event vocabulary; phase-equilibrium species require both an explicit composite pair key and a consistent membership/role rule. None is replaced by guessed columns, dictionary members or a hashed surrogate identity. The four missing but sufficiently specified declarations (state-flash facts, connection equations, initialization order and solve plans) are implemented in this wave.

### Confirmation

`just governance`; stage_bundle_graph and separate P0-P3/P10 golden fixtures; plan 03 terminal gates. Each result records mode and failure count against baseline zero. The execution ledger distinguishes source inspection, focused tests and complete-wave acceptance.

## Pros and cons

The correction removes an ambiguity or false guarantee with bounded implementation work. Conservative rejection/copying/recomputation may cost more until a separately measured refinement is justified.

## More information

[Wave 1 execution plan](../plans/03-wave-1-foundations.md); blueprint §14.1, §24.1, §25; charter DM-59 and G1-G7. The authorized blueprint amendment uses PSE_DESIGN_EDIT=1 and must carry a revision row in the decision PR.

## Status history

- 2026-09-14 — proposed before implementation; maintainer approved the correction plan and its validation-before-hashing clarification.

### Explicit P10 fixture configuration

Proposed: the complete predecessor fixture registry declares a single-row `reference.p10_fixture_context` relation and a required `context` input port on its P10 specification. It carries the optional explicit neutral quantity-type and Boolean quantity-kind selections plus ordered root node IDs. Its declaration and reader are part of the fixture adapter, and the ordinary schema/key/reference checks apply before use. The fixture context participates in the same input/dependency comparison as all other ports. No constructor-only semantic configuration may change P10 output outside those bindings, and no production P4–P9 contract or quantity default is introduced. A standalone uncached canonicalization facade may accept the same typed context directly for leaf tests.

### Fault-injection fixture dependency

Proposed before manifest change: promote the already resolved `futures-core==0.3.34` to a workspace pin used only by the lifecycle and engine test crates. The pinned `object_store::ObjectStore` streaming method signatures name this trait; the shared fault wrapper needs it to delegate the real storage protocol rather than approximate a different API. No runtime dependency edge or resolved family version changes. `just family-check`, `just governance`, the actual publication fault matrix and normal workspace policy gates qualify the promotion.

### Stored P10 predecessor fixture

Proposed clarification before fixture implementation: `FixtureP10Inputs@1` exists only in
the explicit P10 fixture registry. Its pinned input ports name every Model-class relation
required by that fixture P10; its output ports name every required derived relation,
including explicit empty relations. It imports fully specified fixture rows and does not
claim to implement P4–P9. The actual primitive rows are admitted and published in a complete
Model snapshot. The importer Stage binds that Model by its declared input ports and admits
its complete actual derived outputs. The ordinary Driver then executes P10 with fixture
mode explicitly enabled over that pinned predecessor inventory. The context row remains
an actual required input. Golden checks reopen Model, importer and P10 snapshots with their
exact parents, validate complete values, and compare decoded rows and SQL results against
fresh execution. Direct uncached canonicalization remains a leaf test; it does not replace
the required stored producer and Driver qualification.

### Revision 31: complete terminal attempts

**Proposed before implementation.** Blueprint §6.13 and §14.3 now persist complete typed findings inside each existing pass-record sidecar, using the same single diagnostic-field declaration as runtime findings. This completes the already-required failure/cancellation contract: a count alone cannot establish what failed, and a separate unlinked finding artifact would not establish a complete attempt. The nested representation uses one atomic immutable sidecar and no additional publication/index framework. Its status is closed, its failure class is explicit, its count is checked against actual findings, and its pass-run identity is minted before execution. No check identity is invented for an execution error without an invariant origin.

Cancellation has a bounded cleanup recording operation distinct from the cancelled work. Inability to persist the terminal record retains both execution and recording errors; no success output/hint is fabricated. Required controls cover failed preconditions, pass-returned findings, cancellation, storage/budget refusal while recording, complete finding roundtrip and malformed status/class/count combinations. The existing Wave1 implementation review identifies this gap and recommends this bounded completion; formal decision acceptance and implementation qualification remain pending.

Aggregate failures retain every typed leaf diagnostic. The record is cancelled only if all leaves are cancellation; a mixed failure uses the first non-cancellation leaf class in stable traversal order. A hint/index failure after complete output and a successful terminal record is an auxiliary error carrying the existing successful attempt and output, never a second contradictory terminal record. These are explicit outcome-selection rules, not diagnostic-string or hash heuristics.

The successful P2 terminal record and complete immutable change/revision receipt precede the final commit-ref CAS (blueprint §20.1). Recording failure preserves the old visible ref. A subsequent CAS failure is a distinct typed commit-publication outcome carrying the successful P2 attempt/output and original publication error; it cannot retroactively falsify that completed pass record. Required controls cover both failure boundaries.

- 2026-09-14 — reconciled with ADR-0067 and Plan 05; prior receipts describe their original code and do not certify the hard-pivot implementation.
