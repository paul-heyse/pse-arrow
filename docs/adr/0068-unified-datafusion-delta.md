---
id: ADR-0068
title: Unify simulator data and execution through DataFusion and Delta Lake
status: proposed
date: 2026-09-15
deciders: [paul-heyse]
level: decision
principles: [DM-02, DM-07, DM-21, DM-22, DM-24, DM-26, DM-27, DM-31, DM-32, DM-37, DM-41, DM-45, DM-46, DM-53, DM-59]
blueprint: [§D1, §D3, §D4, §D6, §D10, §D11, §D14, §3.3.3, §4, §5, §14, §18, §19, §20, §21, §22, §24, §25]
review: docs/design_review/reviews/design_review_unified-datafusion-delta_2026-09-15.md
evidence: Proposed
supersedes: [ADR-0006, ADR-0007, ADR-0009, ADR-0041, ADR-0042, ADR-0045, ADR-0046, ADR-0061, ADR-0067]
superseded-by: null
revisit: A required simulator outcome cannot be implemented through native DataFusion extensions and Delta transactions, or an exact-pin qualification disproves a selected contract
verification: just adr-lint; just family-check; just simulator-acceptance NEW_OUTPUT_DIRECTORY; docs/plans/07-unified-datafusion-delta-hard-pivot.md Verification
---

# ADR-0068: Unify simulator data and execution through DataFusion and Delta Lake

## Context

The unified DataFusion/Delta review identifies parallel storage, execution and
validation paths, root-only extension planning, and unfinished simulator backends.
The maintainer authorized the entirety of Plan 07 on 2026-09-15. High-level simulator
functionality and the unified target govern this design-phase hard pivot.

## Scope

Amend the cited blueprint sections. DataFusion providers, native plans, functions
and physical extensions govern all product data operations. Delta owns durable
relations and per-table transactions. Model, case, problem and run are typed
relational contracts, not retained predecessor object graphs.

Supersession removes the old graph-object, artifact-hash memo, manifest/CAS and
mandatory replay contracts. Entity identity, physical/mathematical meaning,
owned buffers, explicit complete dependencies, and honest failure reporting remain.
ADR-0065/ADR-0066 full library eligibility remains in force. Historical accepted
records retain their arguments; only their status and supersession links change.

## Drivers

- One semantic declaration generates storage, execution and inspection contracts.
- Native capabilities implement generic behavior; extensions supply domain meaning.
- Actual native and generated backend solves establish a process simulator.
- Exact version publications, resource ownership and failure semantics remain explicit.

## Options

1. Wrap existing stores and pass interpreters: rejected; retains independent authority.
2. Migrate old development objects through dual paths: rejected; no product requirement.
3. Build the target directly and delete each replaced path with its callers: selected.

## Outcome

Implement Plan 07 UD00–UD12 in dependency order. Use one actual DataFusion session
assembly with a shared runtime, composed Delta/PSE planners and bound providers.
All writes use a qualified validating Delta route with actual session and commit
properties. Native provider hooks forward to that route or explicitly reject.

A typed Delta control table atomically selects coherent exact member versions and
revision slices with a qualified expected-parent conflict check. Candidate table
commits are not globally atomic. Readers pin the selected publication and members;
retry reconciles the attempt identity, including commit-then-hook failures.

Symbolic relations and faithful native expressions retain quantity, indexed,
guarded and implicit meaning. Derived structural and numerical layouts exist only
for real algorithms inside native execution. Ipopt, generated Pyomo and NL/SOL
operate on the same problem contracts. Python owns coarse Arrow streams and handles.

### Consequences

Delete custom JSON refs/manifests, publication journals, encoding identities,
sidecars, predecessor snapshots/graphs, stage memo and root-only execution paths.
Development data is disposable. No compatibility reader, transition flag or old/new
qualification campaign is introduced. Retain only code that implements the target.

### Compensating controls

Pin delta-rs and its kernel to the reviewed commits and retain one dependency family.
Test native child execution, validation bypasses, lossless type boundaries, concurrent
publication, uncertain commits, exact reopen, guarded numerics and resource ownership.
Retention protects live publications/runs/readers and their data and log requirements.
Refuse unsafe destructive collection when the backend cannot establish that protection.

### Confirmation

Plan 07 maps S01–S11 and V01–V13 to executable gates. Simulator acceptance requires
fresh authored Slice A sources, native/Pyomo/NL solves, independent numerical oracles,
cold Rust/Python reopen, edits/reuse, extension and failure cases. Incomplete work
cannot produce a successful terminal receipt. Final review assesses G1–G7 separately.

## Pros and cons

Native engine and Delta contracts remove generic maintenance surfaces and support
consistent optimization and inspection. The pivot requires real backend work and
qualification of application-level publication semantics beyond per-table atomicity.

## More information

- [Implementation plan](../plans/07-unified-datafusion-delta-hard-pivot.md)
- [Design review](../design_review/reviews/design_review_unified-datafusion-delta_2026-09-15.md)
- [Provider capability map](../capability-maps/datafusion_provider_contracts.md)

## Status history

- 2026-09-15 — proposed before authorized Plan 07 implementation. Review remains Revise;
  implementation authorization is not a claim of design-gate acceptance.
