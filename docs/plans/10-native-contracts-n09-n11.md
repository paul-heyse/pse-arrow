---
title: Scoped native dependencies and graph capability bindings — N09/N11
status: done
date: 2026-09-19
adrs: [ADR-0073]
phase: 1
evidence: Tested
---

# N09 and N11 implementation contracts

This records the N09 and N11 slice of [Plan 10](10-native-contract-consolidation.md).
N10 and N12–N18 remain separate work. In particular, this slice does not implement
staged numerical DAG execution or certify integrated compiler, storage or solver behavior.

## Scoped native planning

`pse-engine::session::traversal` owns iterative enumeration and postorder reconstruction
over retained `LogicalPlan` owners. Rewrite, evidence and observation use separate
purposes. Keys include actual node ownership, lexical subquery occurrences, correlated
references, recursive worktable declarations and effect ancestry. Temporary native
`Subquery` wrappers never supply memo identities. Observation still renders those wrappers
through DataFusion's own single-node display, with explicit graph edges.

Field derivation/admission, source discovery, freshness, codec function admission,
closedness, cache staging/expansion/rebinding, physical producer discovery and observation share
these mechanics. Definition ports and required-command hoisting retain their distinct
effect semantics in the contract rewrite. Native analyzer and optimizer rules remain
DataFusion implementations. Traversal preflight bounds native recursive reconstruction;
expression nesting is limited to 128 and rewrite regions to 512 plan edges. Diagnostic
enumeration remains iterative, including deeper graphs. Closed cache boundaries bound
the native optimizer's visible regions.

### Dependency transfer

`SemanticDemand` separates field ordinals and source keys from presence, multiplicity,
ordering, requirements and absence. Operation definitions provide input demand with
validated child arity and field bounds. Fixed tuple/unknown operations default to whole
input evidence. An execution contract transfers requested value fields independently of
its complete requirement inputs. Unlowered policy requirements remain conservative.

Native `OptimizeProjections` handles ordinary relational demand. Hidden cache producers
are represented temporarily by native scans and qualified projections in the analysis
only. Synthetic ordinal names avoid duplicate-name collisions; aliases restore the
original qualifiers. Accumulated demand schedules each producer region. These temporary
tables never execute or enter the retained product plan. Empty-column reads still retain
row presence and duplicate counts. Catalog evidence adds declared source keys and keeps
exact Delta selections, versions, inclusive CDF intervals and endpoint comparisons.
Ordering demand propagates through producer regions. Such sources retain exact selected
version evidence because multiset comparisons cannot prove physical ordering.

### Reuse contracts

| Retained item | Authority and invalidation |
|---|---|
| SQL syntax | Exact SQL, pinned parser identity, dialect and complete sorted native parser settings; bind every call against its actual session |
| Scoped preparation | Registry, function inventory, analyzer/optimizer owners, actual table bindings/providers/schema, requirement planner, purpose, semantic settings/policies and implementation generation |
| Structural producer proof | Same semantic witness plus actual source owners/schema; a rewrite or foreign witness clears the proof and temporary prepared/leaf state |
| Closed producer optimization | Actual producer identity and required-work flag inside the owner-bound preparation scope; intrinsic closedness proves independence from outside correlation/worktables |
| Completed execution | Existing invocation/epoch owners; no structural proof grants execution completion, effects, authorization or read leases |
| Delta resident selection | Actual semantic setting values replace hash-only equality; exact store generation, selection, schema, implementation and policy evidence remain in the key |

One versioned setting classifier supplies semantic keys, policy comparison and effective
read-back. Unknown settings remain semantic. Resource limits remain operational and are
admitted again for every invocation. Witness retention and traversal control tables use
the caller's reservation. These are accounting contracts, not whole-process RSS bounds.

`CacheBudget.syntax_bytes` defaults to `min(memory_bytes / 256, 4 MiB)`; zero disables
retention. Parsing and cache-hit cloning re-enter the actual runtime pool. Failed or
cancelled parsing cannot populate an entry. Reports and entry inspection include syntax;
Python `CacheSettings.syntax_bytes` is generated from the compiled native API.

## Graph and operator contracts

`pse-mathir::view::GraphView` borrows expression/canonical nodes and kernel binding rows.
Argument ordinals, payload references, guards and binding ports remain distinct edges.
The shared iterative postorder returns deterministic closed cycle witnesses. Guarded
and unconditional occurrences remain distinct; canonicalization retains complete
quantity, representation, expected-type and index-environment request keys.

Topo/walk, relation loading, folding, canonicalization, P4 predicates, P8 provenance and
numerical dependency discovery use this view. P4/P8/numerical callers pass their real
cancellation/resource context. No second persisted graph or petgraph store was added:
the small borrowed view supplies ordered edge roles and cycle witnesses directly.

The schema operator table binds independent evaluation, folding, native lowering,
differentiation and quantity capabilities. P4 and folding use the same checked `i128`
arithmetic kernel. Folding still requires an exact `i64` result and its existing physical
contract; Pow is evaluable and deliberately non-foldable. Payload-dependent operations
retain explicit prerequisites. Numerical lowering and differentiation resolve the same
actual native function implementations, with separate capability selectors. Matching a
display name never supplies a derivative implementation.

Unit conversion retains ordered multiplication then addition. Native CASE keeps excluded
invalid branches unevaluated. Exact extrema, nonintegral division, overflow, signed zero
and conversion bits have independent literal/native oracles. The P8 fixture names five
source rows: the ordinary branches/root, a payload-only guard and a binding-only operand.
The former child-only closure would omit the last two sources.

## Deletions and retained specializations

- Removed scope-blind field-derivation/source/freshness/closedness/physical-inventory and
  observation walkers, plus recursive cache staging and expansion.
- Removed the blanket extension-opacity projection fallback. Unknown families still
  retain whole input evidence; this is a semantic default, not a type-name prohibition.
- Removed direct repeated SQL parsing at native binding and DML construction boundaries.
- Removed hash-only resident setting equality and registry-only proof reuse across sessions.
- Removed both binding-only whole-node-map clones, P8's child-only reachability, P4's
  duplicate postorder stack, fold's duplicate region walk and numerical dependency stack.
- Removed separate arithmetic evaluators and native-function identity rosters replaced by
  the shared executable bindings.
- Retained atomic fold publication, typed canonicalization requests, required-effect
  hoisting and domain/payload algorithms because they implement different contracts.
  N12 still owns recursive native expression/gradient expansion and staged numerical work.

## Verification

**Interface-checked:** local DataFusion/Arrow and Delta skills, the exact resolved APIs
and compile checks. Pins remain DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store
0.13.2 and the existing Delta 1.0 development capture. No dependency upgrade was made.

**Implemented:** the contracts and deletions above.

**Tested:** `just dev-scoped-graphs`, default nextest profile, isolated `--lib` selection
with `pse-relations/force-validate`: 27 passed, 0 failed, baseline zero. Final static
receipts and the generated-file tracking failure are recorded in the
[execution inventory](10-execution-inventory.md).

`just dev-scoped-graphs` selects isolated native operator, scope, syntax, graph, arithmetic
and synthetic CDF units with `pse-relations/force-validate`. It excludes durable storage,
full compiler, solver and performance journeys. `just check-native-contracts` compiles all
targets without running them; `just lint-scoped-graphs` runs Clippy for this slice and its
consumers. The failure baseline is zero. N17/N18 remain open and no integration dispatch
is authorized by completion of this slice.

## Outcome

N09 and N11 and their slice deletions are implemented and unit-tested. The execution
inventory records verification results and the pre-existing generated-file tracking
check failure. N10 and N12–N18 remain open; this slice is not integrated qualification.

### Mistakes found and corrected

The initial analysis scan lost source qualifiers during native projection rebuilding;
explicit qualified aliases and ordinal synthetic fields fix both that case and duplicate
field names. The first shared rewrite also conflated temporary reconstruction with a new
producer owner. Staging now retains the original computation owner. A control-accounting
double charge was removed while preserving the existing 512 KiB observation test limit.

### Deliberate choices

Closed-producer reuse uses intrinsic closedness rather than irrelevant ancestor labels;
open producers remain scope-dependent. Syntax is cached, bound plans are not globally
cached. Native interval/storage contracts and read leases remain authoritative. Broader
qualification is intentionally deferred to N18.
