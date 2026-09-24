---
title: Rust computation stage contracts
status: abandoned
date: 2026-09-23
adrs: [ADR-0076]
phase: 1
evidence: Implemented — target stage owners and consumers; isolated repair evidence refreshed, W19/W20 acceptance outstanding
---

# Rust computation stage contracts

**Historical Plan 13 contracts.** [Plan 14](14-library-owned-process-simulator.md)
replaces the execution design. Reuse an individual contract only where its new
consumer and semantics are established; the MathIR stages are not the new target.
The implementation and acceptance statements below describe the earlier checkpoint.

These are the RCA §9 contracts adopted by W03. Model meaning remains registry-owned.
[Plan 13](13-rust-computation-architecture.md) owns scope; the
[execution inventory](13-execution-inventory.md) owns completion. All capacities are
finite; overflow, cancellation and incomplete input are failures, never empty success.
The W07–W17 consumers now use these boundaries. The
[W19 repair checkpoint](13-w19-repair-checkpoint.md) records current implementation,
isolated evidence and outstanding functional/performance qualification. The contracts
below describe the target in the working tree, not a claim of complete acceptance.

## S1 — exact input admission

`pse-compiler::AdmittedModel` owns definitions, generated interfaces/instances,
complete source-local `DefinitionBody` values and separate occurrence locations. Bodies
retain every lowered grammar root, predicate/equation, binding/path and unit reference;
a bare rootless expression graph is not a definition body. Normalization installs these
facets atomically after native obligations pass, using the unchanged authoritative source
identity. Source replacement clears derived facets and cases. `SelectedRelease` retains the full generated
publication member (including selector, table URI/version and native role), store
generation and capability identity. Definitions/instances are keyed sets; symbol
declarations and name-value bindings are canonically ordered keyed sets; source
spans are a set. MathIR operands and row transport sequences preserve order and
repetition. Generated equality uses canonical float bits, including signed zero.

Admission rejects duplicate identities/names/bindings, dangling instance templates,
invalid finite bounds/spans and cyclic/dangling parents. ExprGraph construction owns
operator/operand validation. This is **not** a certificate for unresolved quantity,
material or other relational references: the existing native obligation templates
own those checks over a complete selected relation inventory before dependent stages.
Unknown candidates and missing relations remain distinct from present-empty data.
S1 decoding is O(input values); canonical membership sorting is O(n log n).

`FieldCheckedBatch` certifies exact declared fields and local values only. Slice,
bounds-checked nonnull Arrow take and filter preserve those checks and shared buffer
owners; they do not certify keys, foreign keys or whole-region completeness. Exact
projection checks target fields and executes any new row checks. Explicit Arrow
error-on-failure cast runs full target local admission and preserves no old value
certificate. It follows Arrow numerical conversion semantics, not a losslessness claim.

Pure registry construction installs no native SQL planner. The effectful authoring
document loader installs the standard engine validation owner before SQL-bearing row
admission. Custom engine candidates retain the actual session/function/configuration
owner through the native validation adapter. SQL-dependent generated boundary controls
therefore live in the engine test crate; pure row/codec controls remain in relations.

## S2 — synchronous semantic reuse

Stable semantic IDs map to Salsa 0.28.4 inputs inside one bounded database. Queries
read name/membership, interface, body, source mapping and instance fields separately.
Absent lookup is a tracked result. Compare before every setter. A definition key,
complete structural specialization request and actual instance ID are distinct.
Reachable graph projection and complete consumed `InstantiationEnvironment` values
determine reusable bodies; occurrence ordinals and actual destination bindings remain
separate. The old hash-only specialization ID and independent database are deleted.
Realization reads installed Body/Interface/Sources queries; public instance inspection
reads the actual normalized Instance query from the same source-qualified coordinator.

No IO, clock, randomness or mutable registry access is permitted in tracked queries.
The current queries terminate over finite admitted maps; recursive expansion and
its explicit rejection of infinite specialization use the typed template contracts.
Predicate graph admission uses petgraph cycle checks and iterative postorder. Generated
rows retain ordered/repeated operands independently of deduplicated scheduling edges.
The runtime contributes native reference values and complete support evidence; exact
scalar kernels and four-valued Boolean semantics belong to the pure compiler. Bodies retain
an owned Arc and have bounded LRU; input/query-key/tombstone/revision limits remain
independent of LRU. Salsa's memory report exposes metadata separately and identifies
ingredients without a heap callback; it is not a total-RSS measurement.

Runtime closes query admission, cancels each worker, drains actual blocking exits,
then applies the entire admitted update. One application revision becomes visible.
A failed installation rebuilds the last complete state. No Salsa handle crosses
async IO. Retired generations remain reserved until the last leased result drops.
Cardinality caps bound metadata; generation allowances bound admitted retained
payloads conservatively. W14 implements shared lifetime/admission accounting. Its
integrated enforcement remains a W19 qualification obligation; allocator overhead
and process-RSS measurement remain W20 obligations, not exact-byte claims here.

## S3 — graph projections

`pse-structural::{projection,domains}` owns canonical immutable petgraph 0.8.3
graphs and reversible local ID maps, with u32 sentinel checks before allocation.
Nodes include isolates; typed edge IDs retain parallel edges, loops, direction and
source labels. Whole scope is an assertion by the complete input owner; a partial
scope cannot obtain whole-region order/SCC results. Independent scope can only be
minted from a whole projection after checking every crossing edge.

Package/template/kernel prerequisites point toward dependents; containment points
parent to child; ports and conversions retain their declared directions. Rule
edges preserve positive, negative, nonmonotone and conflict-sensitive meaning.
Only positive cycles may remain within a stratum. Conversions retain kernel and
coefficient choices; reachability never silently chooses a path. Expression
DfsPostOrder traverses the existing MathIR adjacency; occurrence multiplicity
and operand roles remain in the source graph. Incidence explicitly selects active
equalities, free variables, isolates and the zero-elision/lowering contract.

Use iterative kosaraju_scc, Dfs/Reversed, petgraph condensation with unit edge
weights, rustworkx-core 0.18.1 semantic-key topological order and real cycle witnesses.
Canonicalize component membership and ordering; original edges remain available.
The rust-igraph 0.7.0 adapter uses disjoint typed partitions and validates narrower
indices. SCC/traversal are O(V+E); canonical sorting adds O(V log V + E log E), and
lexicographical ready-order adds logarithmic queue work. Projection is recomputed
per changed complete region; there is no mutable/index-hole or dynamic graph engine.
No transitive closure is materialized without an explicit all-pairs consumer.

## S4 — complete relational phases

Tracked code returns a plain `PhaseRequest`: exact direct-input identity, selected
release, implementation, region, program, settings and complete output inventory.
The runtime adds generation/application revision in `Submission`. A bounded
in-flight map coalesces only identical descriptors; it owns no semantic invalidation
graph. The native executor retains the actual session/function/configuration and
immutable source owners. Checked in-memory candidates use the existing immutable
MemorySourceConfig provider; raw mutable MemTables are refused. Durable selections
are checked against the actual member witnesses, including unused selected inputs.

The native adapter accepts keyed-set outputs, sorts declared primary keys, runs
existing relational obligations and admits every decoded batch locally. Sequence
transport exists for explicitly ordered custom producers; a producer must establish
its ordering contract. Output maps include empty relations. Predecode allowance,
row limits, engine memory reservations and phase slots bound work. Validation and
sorting cost follow the actual relational plans; no constant-time claim is made.
Cancellation/failure installs nothing; late completions fail; identical installed
completion is a no-op. The staging owner lives through atomic completion admission.
The W08 fixed-point consumer derives `compiled.rule_strata` as a Derived snapshot and
retains its schedule provenance alongside head derivations. Settlement dependencies
determine execution order. Forbidden recursive dependencies retain the typed graph
projection error and cycle witness; caller-assigned strata do not override it.

## S5 — MathIR transformations

The existing `ExprGraph` owns opcode/literal/quantity/scope and ordered repeated
operands, with typed declaration references and structural keys. Source occurrences
remain separate from numerical equality. W09 replaced native relation round-trips
with owned typed passes. Each pass declares changed quantities, binding context,
policy/kernel choices and failure/cycle semantics; an unresolved type or reference
cannot be interpreted as a default. Work is bounded by admitted nodes/edges and
pass-specific scratch, with cooperative synchronous cancellation. No publication
or solver state occurs inside these transformations.

## S6 — exact structure

W10 consumes the complete S3 incidence contract, calls qualified maximum bipartite
matching and derives coarse DM and matching-projected block order. Matching is
not necessarily unique; compare cardinality and partition invariants, then apply
the declared deterministic block policy. Missing vertices differ from structural
singularity; SCC blocks are valid cycles. The foundational adapter test checks
matching result shape and isolates; the W10 packet records the independent structural
controls. Full production journeys and performance remain W19/W20 obligations.
Whole-problem cardinality/solver workspace limits apply before conversion.

## S7 — reusable numerical lowering

W11 owns numerical program/ABI, sparse pattern and reusable preparation. Structural
keys include ordered MathIR, quantities, kernel/provider/policy choices and ABI;
runtime values, bounds and warm starts stay in instance/attempt bindings. Built-in
numerical kernels and solver interfaces remain owners of their algorithms. Missing
capabilities and invalid domains fail explicitly. Workspace and conversion costs
are charged independently of retained compiled programs; no solver state enters Salsa.

## S8 — attempts and scenarios

The existing runtime owns attempt identity, mutable workspaces, solver execution,
deadline/cancellation and result ingestion. Attempts execute even when compilation
is reused. A dropped waiter does not release a running blocking task's permits or
reservation. W11/W14/W15 implement aggregate resource accounting and consumer moves;
the complete concurrent edit/solve journey still requires current W19 qualification.
Temporal coexistence and scenario semantics are explicit inputs; no graph snapshot
is presented as a temporal analysis. Attempts never mutate selected input providers.

## S9 — publication and inspection

Existing catalog/Delta owners retain immutable selected views, prepared commit
protocols, attempt completion/recovery and complete publication membership. W12/W15
make product selection and bounded inspection materialization explicit.
Artifact validity includes actual source bytes (including dirty files), locks,
toolchain/profile/flags, registry and caller-supplied actual feature/plugin/kernel
ABI configuration. Version strings, Salsa IDs, graph indices, pointers and assembly
nonces are insufficient validity keys. Hashes detect mismatch, not input validity.
Publication completeness and commit effects remain outside memoized computation;
failure/cancellation cannot produce a successful partial publication. W13 implements
exact updates/CDF as an accelerator with a full-reload path.

Artifact descriptor output passes through the existing checked relation declaration
before durable encoding, restoring the declared relation metadata absent from a plain
query scan. Shared object-store stream wrappers retain admission through consumption
and use fused EOF behavior; an exhausted nested stream stays exhausted. The isolated
descriptor/stream controls pass. Full Delta publication, recovery, CDF and inspection
journeys remain explicitly unqualified after the W19 repair batch.

## Verification

**Tested:** the current checkpoint records 530 isolated controls in
`development-units-07`, plus 16 generator and 16 registry admission controls. Native
tests explicitly enable force-validation in the ci profile; all final isolated
groups have zero failures against baseline zero. Static and pure-generation receipts
are recorded separately. W18 source sealing authorizes qualification; the first W19
campaign failed and was interrupted. W19 remains incomplete and W20 remains unrun.
