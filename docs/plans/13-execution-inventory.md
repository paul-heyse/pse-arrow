---
title: Rust computation execution inventory
status: abandoned
date: 2026-09-23
adrs: [ADR-0076, ADR-0077, ADR-0078, ADR-0079, ADR-0080, ADR-0081]
phase: 1
evidence: Tested — implementation and deletion controls refreshed after W19 repairs; functional and measured acceptance open
---

# Rust computation execution inventory

**Historical inventory:** [Plan 14](14-library-owned-process-simulator.md) supersedes
Plan 13's execution scope. Rows below record the old implementation/qualification
boundary; they do not become Plan 14 requirements without new-target evidence.

Baseline: zero failures. The [W19 repair checkpoint](13-w19-repair-checkpoint.md)
records the former resume boundary. W00–W17 and L01–L16 are implemented; W19 is
incomplete and W20 is unrun. W18 is a source-qualified implementation barrier and
must be refreshed after repairs. The verification entries below are chronological;
their earlier remaining-work statements describe those earlier checkpoints.
Historical Plan 10/11 receipts retain their original outcomes; carried IDs live in the manifest.
Starting source digests are in the local `build/plan13/starting-source.json` receipt.

| ID | Scope | Status |
|---|---|---|
| W00 | Authority, successor scope and acceptance inheritance | complete |
| W01 | Qualified dependency profile and real crate boundary map | complete |
| W02 | Shared generated mechanics and separated boundary types | complete |
| W03 | Typed admission, identities, equality and stage contracts | complete |
| W04 | Salsa database, input transaction and lifetime controls | complete |
| W05 | Synchronous derivation and asynchronous relational handoff | complete |
| W06 | Complete typed graph projections and library adapters | complete |
| W07 | Authoring/resolution/topology compiler replacement | complete |
| W08 | Derived rule strata and bounded relational inference | complete |
| W09 | Shared semantic MathIR and specialization compilation | complete |
| W10 | Exact incidence, matching, DM and block analysis | complete |
| W11 | Direct numerical lowering and reusable preparation | complete |
| W12 | Selected durable artifacts and preserved publication | complete |
| W13 | Exact release updates and optional CDF acceleration | complete |
| W14 | Aggregate budgets, cancellation and last-reader ownership | complete |
| W15 | Public consumers, inspection and end-to-end wiring | complete |
| W16 | Final dependency/features and generated-code reduction | complete |
| W17 | Complete executable case and evidence reconciliation | complete |
| W18 | Full implementation, deletion and authority barrier | complete |
| W19 | Complete local functional acceptance and repairs | open |
| W20 | Measurements, independent gates and final outcome | open |
| L01 | Universal DataFusion execution placement and stale compiler authority text | complete |
| L02 | DataFusion error and reservation dependencies in vocabulary/ID cores | complete |
| L03 | Repeated generated batch mechanics, duplicate payload shapes and runtime generator-only dependencies | complete |
| L04 | Compile memo invalidation keyed by pointer/assembly UUID and duplicated dependency predicates | complete |
| L05 | Per-entity/per-step DataFusion lookups and partial `Expr` interpreter | complete |
| L06 | Hand-written topology/frontier/order loops and unsupported all-pairs closure | complete |
| L07 | Manually authoritative rule strata and incomplete dependency graphs | complete |
| L08 | Full MathIR relation round-trip at each semantic pass and divergent semantic consing | complete |
| L09 | Structural stub and unconditional bespoke matching mandate | complete |
| L10 | Residual one-row callback planning/encoding or solve-state capture in reusable programs | complete |
| L11 | Unconditional durable publication/capture of every intermediate | complete |
| L12 | Unconditional CDF dependency or “missing history means unchanged” behavior | complete |
| L13 | Cache-entry-only leases and value-LRU-only resource claims | complete |
| L14 | Semantic algorithms hidden inside now-unneeded native stage wrappers | complete |
| L15 | Unused enabled features and obsolete direct/transitive dependency edges | complete |
| L16 | Stale tests, callers, active pointers or acceptance seals for replaced mechanisms | complete |

## Verification

**Tested:** [W00–W06 execution packet](13-w00-w06-execution.md) records 82 isolated
foundation units, 8 Rust governance units (both force-validation), 7 Python guard units,
locked compilation, Clippy, dependency ceilings and pure regeneration. Baseline zero;
all named final checks exit 0. Strict regeneration uses a temporary Git index for new
generated files; the real staging area is unchanged. See the packet for that condition
and the retained upstream/tooling notices.

W00–W06 completion is the selected foundation endpoint, not final consumer wiring.
L01 retains formal authority reconciliation; L03 retains W16 generated/dependency closure;
L04 and L11–L16 retain their named replacement owners. ADR-0076 remains proposed. The native
driver wrappers have completed W07–W11 replacements and retain W15 public-consumer
reconciliation. The inventory still refuses the W18 seal, starting at W15.

W19 functional acceptance and W20 measurement are not_run; no old seal authorizes execution.


**Tested:** [W07–W11 execution packet](13-w07-w11-execution.md) records 32 isolated
computation units (force-validation), one linked callback barrier unit (Ipopt plus
force-validation; no solver invocation), seven Python phase-guard units, locked
all-target compilation, affected Clippy, exact generated Rust/Python/docs checks,
family ceilings and environment refresh. Baseline zero; final named checks exit 0.
Local source/log digests live in `build/plan13/w07-w11/final-verification.json`.

L05 closes typed configuration/port-path lookup; P6 required finite coordinate products
remain relational. L06 closes topology ordering/frontier replacements; bounded all-pairs
products remain explicit inspection outputs. L07 closes manual strata, including
benchmark/test callers. L08 closes mandatory inter-pass MathIR reloads and divergent
scalar lowering; the remaining native `Expr` arena serves genuine batch evaluation.
L09 closes the structural stub with qualified exact library matching/DM/blocks.
L10 closes one-row callback fallback and solve-state capture in reusable programs.

L04/L14 still require W15 cross-invocation/public orchestration. W12/W13 own durability
and release updates; W14 aggregate resources; W16 dependency cleanup; W17 full-case
reconciliation. ADR-0077/0078/0079/0080 remain proposed pending formal decision/design PRs.


**Implemented and Tested:** [W12–W14 execution packet](13-w12-w14-execution.md)
closes selected product descriptors, exact endpoint updates with qualified CDF fallback,
whole-release checkpoints, and shared resource/cancellation ownership. It records 30
isolated durability/resource units, 87 foundation units, 32 computation units, 10 selected
Delta boundary units, one linked callback unit and seven Python guard units. Rust unit
modes explicitly retain force-validation; the counts overlap across selected recipes.
Baseline zero; commands, exclusions, corrections and static/generation conditions are in
the packet. No real Delta publication/history/maintenance or solver journey ran.

L11 remains open for W15 public caller orchestration; its W12 unconditional intermediate
destination generation is removed. L12 closes CDF-only release admission and broad failure
fallback. L13 closes detached typed solver/descriptor/checkpoint ownership and value-LRU-only
lifetime claims. Metadata allowances are conservative accounting, not a process-RSS proof.
W15–W20, the remaining deletion rows and formal decision/design acceptance remain open.
ADR-0081 is proposed, and this checkpoint grants no W18 seal.

**Implemented and Tested in part:** [W15–W20 execution packet](13-w15-w20-execution.md)
records the hard deletion of native compiler operation/plan wrappers, the standalone
specialization database and the obsolete acceptance runner. It adds explicit compiler
session ownership, immutable products, bounded inspection, atomic document edits and
exact stored-source reconstruction and checkpointed CDF updates. Selected outputs
now use the relational phase driver, and canonicalization shares the application
Salsa coordinator. Source-expression lowering/unit normalization and complete P5
containment now also have real pure query consumers, with their previous runtime
implementations removed. Ordinary containment no longer generates ancestor-pair tables;
the separate requested inspection contract preflights complete capacity. Scalar
configuration and binding syntax now live in the pure compiler. The duplicated layout
query and unused hash-only specialization API are deleted. All containment callers and
complete tear requests now use the shared compiler coordinator. Pure petgraph tear
analysis replaces the runtime implementation and duplicate graph; runtime's direct
petgraph edge is removed. Full source-edit and typed compiler/edit/solver fixtures,
Q01–Q11 mappings, graph/incremental Criterion cases and isolated build characterization
are authored/registered, not terminally qualified. Remaining wiring is
explicit in that packet. W15–W17 and L01/L03/L04/L11/L14/L15/L16 remain open;
passing isolated units and static checks do not close those exits. W18–W20 remain
unrun and unauthorized by any implementation seal.

**Implemented and Tested in scope:** W15/W16 close the production consumer pivot and
normal dependency/generated-mechanics work. The W15–W20 packet records source-derived
definition/interface/instance installation, actual P7 consumers, pure predicate kernels
and library dependency traversal, exact public instance inspection, selected publication,
and removal of the displaced APIs. Blueprint revision 49 reconciles active execution
placement and sequencing under the authorized design-edit scope. Formal proposed-ADR
acceptance remains independent.

Current receipts include default/no-default and Ipopt all-target Clippy checks,
`family-check-10.log`, all three strict pure-generation checks (`codegen-checks-04.json`,
real Git index unchanged), five engine-owned codec units (`typed-boundary-units-03.log`,
ci profile and explicit force-validation, zero failures), six pure predicate units and
one atomic definition-owner unit. `py-sync-04.log` and `doctor-03.log` establish the current
editable environment. Baseline is zero; none is full product acceptance.

The fresh default and solver listings resolve all 290 native functional case mappings
across 2,888 distinct identities (`native-reconciliation-03.json`). Criterion performance
cases are separately compiled/registered and are not nextest identities. W17 retains the
complete current development-receipt and performance-fixture reconciliation. L16, W18,
W19 and W20 remain open; no barrier is issued by this checkpoint.

**Tested, W17/L16 implementation closure:** the current six-command isolated receipt
(`build/plan13/w15-w20/development-units-04/`) records 347 library units, two tool
units, five typed-boundary units, one linked callback control, 63 setup tests and
107 Python units, each with zero failures against baseline zero. Native correctness
commands explicitly enable force-validation and use the ci nextest profile. The
executed records cover all 149 declared implementation-unit cases through 460 unique
mapped binary/test/mode identities; other successful selected controls remain in the
original reports. The source snapshot is unchanged. The exported development receipt
passes the existing strict validator; its predecessor is preserved beside this run.

Fresh default and solver listings resolve 290 native functional mappings across
2,889 identities, with zero missing mappings (`native-reconciliation-04.json`).
The W15-W20 packet records all 77 supported performance gates and their fixtures.
Pure generation checks pass with the real Git index unchanged (`codegen-checks-05.json`).
Formatting, typing, family ceilings and the repaired actual-function-owner unit pass.
Historical failures, the interrupted second sweep, and all unrun terminal cases remain
explicit. W18 has not yet been issued by this checkpoint; W19/W20 remain unrun.

**Tested, W18:** `just architecture-seal --plan 13` succeeds against baseline zero
(`architecture-seal-01.log`). It verifies every implementation/deletion row, every
current source route, all required development identities and their immutable logs
and source digests. The seal is refreshed after this status update and preflighted
before W19. This authorizes functional execution; it is not functional acceptance.
The formal ADR/design PR status remains separate from local implementation closure.


**Tested, W19 open:** the first functional attempt retains its failed and interrupted
results in `build/plan13/w19-functional-01/`. The native ci/force-validation gate
reports 2,757 passed and 119 failed against baseline zero. Shared boundary and stale
contract repairs are described in the W15–W20 packet. Fresh development evidence
and a refreshed source seal are required before continuation; W20 remains unrun.

**Tested, current repair checkpoint:** `development-units-07` supersedes receipt 04
for development evidence, with unchanged captured source and 530 passing isolated
controls against baseline zero. Native groups use ci mode and explicit force-validation.
The refreshed export covers 149 required case IDs through 461 distinct mapped
binary/test/mode identities. Separate generator and registry-admission suites each
pass 16 controls. Compile, Clippy, family, strict generation and fixture checks also
pass; exact commands and limitations are in the
[W19 repair checkpoint](13-w19-repair-checkpoint.md).

The documentation checkpoint preserves the unsuccessful first W19 receipt and its
unfinished gates. W19 remains incomplete, W20 remains unrun, and proposed ADR status
is unchanged. W18 must identify the final documented source before continuation.
