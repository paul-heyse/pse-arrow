---
id: ADR-0073
title: Consolidate native engine resource and boundary ownership
status: proposed
date: 2026-09-18
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-07, DM-15, DM-24, DM-47, DM-54, DM-56, DM-60]
blueprint: [§3.2, §5.3, §14.3, §23.2]
review: docs/design_review/reviews/design_review_native-consolidation-reassessment_2026-09-18.md
evidence: Implemented
supersedes: []
superseded-by: null
revisit: A new engine adapter, shared allocator, persistent operation or Python transfer contract is introduced.
verification: Plan 10 N06-N17 package units and deletion inventory; N18 independent A05-A16 acceptance after the implementation barrier.
---

# ADR-0073: Consolidate native engine resource and boundary ownership

## Context

Plan 10 D04–D12 requires focused engine ownership, native resource accounting and
shared boundary contracts. N00 allocates this decision before N06–N16 implementation.
ADR-0072 separately owns foundation validation, values, identity and diagnostics.

## Scope

ADR-0074 and Plan 11 govern the integrated lifetime, evidence-preservation and
implementation-testing amendments. This proposed record retains its historical
evidence; its prior per-boundary or whole-stage mechanisms do not restrict that target.

Introduce the planned `pse-engine` and development-only `pse-testkit` crates. Move generic
session/provider/function/execution services out of catalog. Reconcile ownership,
publication and Python boundary conventions through the dedicated
`design: consolidated engine and boundary contracts` change in blueprint revision 41.
This proposed record neither accepts itself nor edits accepted historical decisions.

## Drivers

Native capability preservation, explicit ownership, one implementation per meaning,
observable resource lifetime and retained high-level process-model outcomes.

## Options

- Retain generic catalog wrappers and duplicate test factories: competing ownership.
- Create universal erased runtime frameworks: extra abstractions with little benefit.
- Use focused engine/testkit ownership and concrete native operation families: selected.

## Outcome

Engine owns common DataFusion assembly and execution, complete native function adapters,
scoped dependency/configuration classification and native memory reservations. Catalog
composes engine with Delta publication, strict storage mapping, bounded reads and retention.
Testkit uses production factories. Python exposes generated named reports, declared widths
and units, and the shared exact identity/transfer contracts. Preserve Arrow buffer owners
through all asynchronous and FFI boundaries.

Generated Python identity values call the native codec through the existing `_build`
gateway. The generated-contract import restriction therefore allows exactly
`pse.contracts.values -> pse._build`; all other prohibited edges remain enforced.
This replaces independent Python hexadecimal admission instead of retaining it to
satisfy the predecessor isolation rule. Plan 10 N15 authorizes this boundary change.

### Consequences

Delete superseded services, incomplete adapters, recursive numerical expansions, parallel
resource claims, Delta builder policies and obsolete Python conversions as their selected
replacements land. No compatibility API or historical-data migration is retained.

### Compensating controls

Plan 10's exact-source implementation/deletion receipt blocks the final campaign until
N00–N16 and L01–L18 close. Isolated package units establish ownership, delegation, semantics
and failure behavior while integration and measurement remain N18 obligations.

### Confirmation

N06–N16 record command-qualified units and deletions in the single execution inventory.
N18 separately assesses A05–A16 and G1–G7; a green command cannot supply those verdicts.

## Pros and cons

Native primitives reduce duplicated authority. Complete adapter delegation and ownership
checks require precise tests and feature-qualified evidence at library upgrades.

## More information

[Plan 10](../plans/10-native-contract-consolidation.md), D04–D12 and N06–N18. The exact
DataFusion and Delta skill profiles establish callable interfaces; their capability alone
does not establish application publication or scientific validity.

## Status history

- 2026-09-19 — N14/N15 implemented. Native operation/declaration ownership, bounded
  action observations, explicit settlement and effective retention replace duplicated
  catalog policies. Native declarations generate Python settings/reports; identity
  text delegates to Rust and structured stream failures retain original native causes.
  Isolated Rust/Python units and static checks are recorded in
  `docs/plans/10-native-contracts-n14-n15.md`. This ADR remains proposed; N16–N18 and
  final functional acceptance remain open.

- 2026-09-19 — N14/N15 implementation authorized: consolidate native Delta builder
  context, storage/local checks, bounded commit observations and effective retention;
  generate thin Python settings and named reports, share identity/duration admission,
  and retain structured errors and native owners through Arrow streams. No compatibility
  layer or new crate. Isolated units precede the existing N17/N18 qualification barrier.

- 2026-09-19 — N10/N12/N13 implementation authorized. Replace application reservation
  traits with native DataFusion pools and reservations, retaining immutable allocation
  owners through Arrow and FFI boundaries. Native split transfers pre-admitted capacity;
  Arrow's infallible, replacing buffer claims remain disabled. Compiler correspondence
  uses native qualified columns and stable keyed provenance. Numerical preparation
  retains a region-aware value/derivative DAG and shallow prepared native expressions;
  callbacks perform no planning. Delete predecessor resource, positional-support and
  recursive-expression paths. Isolated units and static checks qualify this slice;
  complete compiler/storage/solver journeys and measurements remain behind N17/N18.

- 2026-09-19 — N09/N11 implementation authorized. Share bounded native traversal
  with distinct rewrite/evidence/observation contexts; retain actual implementation
  and scope ownership in reuse admission. Cache parsed native SQL independently
  from binding. Catalog retains exact Delta selection, CDF, lease and maintenance
  semantics. Math algorithms use one borrowed typed-edge view and binding overlays,
  preserving ordered cycle witnesses and guarded regions. Operator capabilities bind
  executable implementations separately for evaluation, folding, lowering, quantity
  inference and differentiation. No second persisted graph or global bound-plan cache.
  Integration and measurement remain deferred to N18.

- 2026-09-18 — N07–N08 implementation authorized. Operation definitions retain actual
  implementation identity, complete input contracts and typed algorithm bodies; engine
  owns deferred physical shells and native metrics. Required work survives value pruning.
  Commands retain settlement ownership with durable interpretation in catalog. Function
  adapters delegate native hooks and retain admitted fields through optimizer rewrites.
  Isolated units qualify this slice; integration and performance remain N18 obligations.

- 2026-09-18 — proposed during N00, before the engine/resource/boundary implementation.
- 2026-09-18 — N06 implementation authorized: extract the generic engine and dev-only
  testkit, retain publication meaning in catalog, and centralize execution assurance.
  Native plans and metrics are evidence; stream termination distinguishes completion,
  failure, cancellation and abandonment. Missing capture is inconclusive. Observation
  settings do not change semantic identity or disable validation/freshness enforcement.
  The scoped review is `design_review_engine-extraction-assurance_2026-09-18.md`.
- 2026-09-18 — N06 dependency classification: independently released instrumentation
  crates are explicit family-membership exclusions in workspace metadata. Apache
  DataFusion packages and their transitive versions remain fully checked.

- 2026-09-19 — N07–N08 implemented. Engine-owned operation families, required-work
  protection, native transport, terminal shared completion, retained command settlement,
  complete function hooks and field-preserving native statistics substitution replace
  their predecessors. `just dev-native-contracts`: 34 isolated force-validate units passed,
  zero failures against a zero baseline; workspace/solver-feature compilation and scoped
  Clippy passed. The [contract matrix](../plans/10-native-contracts-n07-n08.md) records
  specialization and limits. N09–N18 and decision acceptance remain open.

- 2026-09-19 — **Implemented / Tested:** N10/N12/N13 replace custom reservation traits
  with native pools/reservations, carry stable compiler support through qualified native
  captures, and prepare shared region-aware numerical stages. `just dev-native-data`
  passed 41 isolated units with `force-validate`, default nextest profile, zero failures
  against zero baseline. Workspace and solver-feature strict Clippy passed; no solver,
  compiler publication or Delta journey executed. The detailed [slice record](../plans/10-native-data-execution.md)
  names retained algorithm boundaries and the pre-existing generated-file tracking failure.
  This record remains proposed; N14–N18 and final functional acceptance remain open.

- 2026-09-19: N16 consolidates generic fixtures and their actual native allocator;
  revision 41 reconciles the implemented contracts. ADR status remains proposed;
  final A05–A16/G1–G7 acceptance and the decision-PR lifecycle are not inferred from code.
