---
title: Native consolidation closure and evidence ownership
status: in-progress
date: 2026-09-19
adrs: [ADR-0071, ADR-0072, ADR-0073]
evidence: Implemented
---

# N16–N18 closure

**Historical closure map:** current source, successor mappings and remaining
qualification belong to [Plan 13](13-rust-computation-architecture.md) and its
[repair checkpoint](13-w19-repair-checkpoint.md). This record preserves earlier
ownership and evidence; it grants no acceptance of the current tree.

This is the source and evidence map for [Plan 10](10-native-contract-consolidation.md).
The [execution inventory](10-execution-inventory.md) remains the status authority.
No entry here awards behavioral acceptance. The machine-readable
[case index](10-acceptance-cases.toml) identifies required tests and artifacts;
A01–A16 and G1–G7 require an independent assessment after execution.

## Shared fixtures and retained differences

- `pse-testkit::NativeFixture` uses `EngineResources` and the actual production factory.
  Its spill directory survives a transferred factory through a native config extension.
  `pse_testkit::factory` accepts an existing pool for allocation-boundary cases; the
  runtime and factory receive that same pool. Caller partitions and settings survive.
- `pse-testkit::execution` binds actual providers and DML targets through production
  preparation, preserving caller functions, rules, planner and runtime ownership.
- `pse-testkit::{fault_store,counting_store}` owns one generic implementation of each
  fixture. Conditions, ranges and consumed-byte observations delegate to object_store.
  FaultStore's injection surface is explicitly single-object `put`/`get`; multipart
  setup delegates without injecting. Tests must prove a requested fault actually fired.
- `tests/support/workflow_runtime.rs` composes the domain registry, production runtime,
  catalog and rule planner once for compiler journeys and xtask inspection. It is a
  shared source module, not a production dependency on testkit or a second runtime.
- `tests/support/catalog_context.rs` composes the explicit Delta extension planner for
  repeated catalog fixtures. Native operation helpers no longer live in catalog tests.
- Intentional variants remain: arbitrary caller planners/UDFs/rules, observer dispatch,
  constrained resource pools, measured cache policy, registered fault stores, and pure
  native predicate reference oracles. These select inputs to the production contracts.
  Conformance's normal test-support library constructs its own native factory without
  introducing a production testkit edge; it shares its actual allocator.

## Specialized mechanism inventory

| Owner / entrypoint | Functional purpose | Why this remains specialized | Reachability / independent oracle |
|---|---|---|---|
| `pse-authoring::p1`, document admission | Parse source-language documents and assign declared identities | DataFusion does not define the process-model source grammar | Compiler document relation; parser/source tests |
| `pse-schema::{resolved_contract,literal,validation}` | Resolve meaning, domain metadata and exact admitted ownership | Native Arrow storage equality does not establish foreign semantic equivalence | Generated/native adapters and registry admission units |
| `pse-ids::{derive,frame,row_token,native_value}` | Semantic identity and lossless scalar preimages | Hash framing and declared float equivalence are domain contracts | Arrow row primitives; independent byte/identity oracles |
| `pse-relations::validate` | Visibility, tagged alternatives and relational obligations | Arrow safety and built-in predicates need domain declarations/masks | One native predicate compiler; local reports, rules, Delta adapters |
| `pse-engine::{operation,dependency,session}` | Effects, required work, dependency ownership and shared producer lifetimes | Generic query optimization cannot invent publication/lease/negative-fact semantics | Native logical/physical extension families, unit rewrite and stream controls |
| `pse-engine::cache_service`, catalog cache components | Actual-owner reuse and bounded retention | Semantic invalidation and exact publication selection are application decisions | Native caches/pools/store instrumentation; cold/warm native-cache measurements |
| `pse-compiler::native`, passes P0–P9 | Finite domain algorithms and native relation correspondence | Process-model interpretation is not a generic relational primitive | Engine operation family; production workflows and independent physical expectations |
| `pse-mathir::{view,walk,topo}`, compiler graph consumers | Shared semantic DAG, scopes, guards and witnesses | Graph algorithms reduce repeated traversal and preserve domain branch semantics | Compiler and numerics; cycle, diamond, guard and derivative units |
| `pse-templates::paths` | Typed coordinate enumeration over declared template paths | Coordinate meaning depends on model declarations | Compiler expansion; bounded-path units |
| `pse-numerics::{expressions,scalar_math,operation}` | Shared value/derivative stages and exact scalar arithmetic | Native scalar evaluation needs a domain differentiation/staging front end | DataFusion evaluation; independent residual/Jacobian and guarded-DAG cases |
| `pse-backend-native::native`, `pse-ipopt-sys` | Native nonlinear solver callback boundary | A query engine is not an NLP solver | Explicit Ipopt feature; independently exercised existing solver cases, no new simulator caller |
| `pse-catalog::delta` | Complete publication, exact retry, version protection and settlement | Delta provides table transactions, not multi-member process-model publication meaning | Native builders/providers/CDF/checkpoints/vacuum; lost-response and post-fence faults |
| `pse-ids::owned_buffer`, `pse-py` and Python transfer | Reserve-before-allocation and foreign owner lifetime | The C-stream ABI does not itself transfer native memory reservations or rich errors | Native pools/Arrow owners; pointer, slice, dictionary and real export/reopen tests |
| `pse-diagnostics` | Preserve native causes with domain-facing codes | Error vocabulary is application semantics | Native source trees and generated Python exception projections |

`pse-kernels`, `pse-kernels-ext`, `pse-structural`, `pse-plans`, `pse-backend-nl` and
`pse-backend-pyomo` are presently declared crate boundaries without implemented callable
engines. They are not alternative executable paths or accepted simulator capability.
Unused implementation dependencies were removed. The existing native NLP kernel is
retained; creating Pyomo/NL/structural simulator functionality is outside this plan.

## Dependency and SQL decisions

Cargo metadata and import/caller inspection accompany the dependency scanner. Direct
DataFusion subcrate duplicates, predecessor backend/runtime edges and dormant numerics
library edges were removed where unused. Diagnostic-macro `miette` and closed-enum
`serde` dependencies remain where generated macro expansion requires them. The runtime's
optional native backend edge retains the explicit `ipopt` feature route. Scanner output
alone cannot establish these dependencies are dead.

The 130 authored SQL sources under crates are independently owned rule/query programs.
Repeated SQL syntax is not a repeated semantic declaration. They remain readable source;
there is no invented generator or second bound-plan cache. The engine caches syntax and
rebinds against actual scope, providers and implementations (N09).

The native engine assembly disables DataFusion 55.1.0's leaf-expression pushdown
option after a source-span/Delta-decoding unit reproduces its loss of same-named
conversions. This is a bounded upstream correctness mitigation, not an alternative
optimizer or data path. The execution inventory records the source contract and
reproducer needed before enabling that option again.

## Measurement routes

| Plan matrix row | Concrete source and observations |
|---|---|
| Contract admission | `native_consolidation`: registry construction, exact admitted borrow; foreign-contract adversarial units remain independent correctness evidence |
| Validation | `native_consolidation/validation.rs`: scalar/list rows 1/1,024/65,536 and null fractions 0/25/100; cold preparation separated from native evaluation |
| Generated interfaces | Campaign codegen command/log/time and generated source sizes; fresh regeneration plus exact tracked output comparison |
| Function/provider adapters | Native hook conformance tests; engineering JSONL retains actual datafusion-tracing operators/metrics and operation terminals |
| Dependency/reuse | `native_cache`: prepared rounds, cold/warm replay, cyclic strata and maintenance; actual prepare/execute/cache/IO counters |
| Compiler correspondence | Four separate engineering processes: actual source/state sizes, per-algorithm input capture/algorithm/packing durations and captured child counts/rows; P7/P8/P9 remain identifiable by declared name |
| Graph/numerics | `native_consolidation/numerical.rs`: shared residual multiplicity 1/8/64, cold compile versus 1,024-row evaluation; bounded nodes/edges and independent guarded derivative tests |
| Resources | Native pool peaks and process peak RSS separately, cache retention and FFI/shared-buffer lifetime oracles; no claim of global allocator coverage |
| Delta | Native-cache object-store request/consumed-byte counters and phase times; publication/CDF/maintenance cases with protected exact versions |
| Extension locality | `nested_contract_journey.rs` adds one declaration and fixture; no production dispatch branch, alternate validator, write implementation or Python policy is required |

Timings are current-architecture observations, without a predecessor speedup claim.
Native reservation counts are not allocator event counts. Process RSS includes runtime
and library allocations outside the native pool. Trace close does not establish correct
values; independent assertions remain. Missing or incomplete evidence stays unresolved.

## Barrier and continuation protocol

1. Close source, caller, fixture and deletion rows only after compile/static/unit evidence.
2. Record actual development commands, mode, zero-baseline failure count, log and digest in
   ignored `build/plan10/development-checks.json`; `architecture-seal` checks these records.
3. The seal includes revision, all current tracked/untracked source bytes, executable modes,
   symlink targets, complete N/L rows and the verified case index. Preflight refuses drift.
4. `architecture-acceptance` archives source, metadata, exact pins, command logs and native
   JUnit results. Test identities include the actual binary/module, not a function suffix.
   A failed/skipped parameterization cannot be replaced by a passing sibling.
5. A changed-source continuation names affected `--rerun` gates and `--change-reason`.
   Parent receipt digests authenticate the retained chain. Each retained gate keeps its
   original source and artifacts; this is never called one untouched final-source run.
6. Required command coverage, observed cases and independent acceptance verdicts are
   distinct fields. All three are needed for N18 completion.

## Selected bounded choices

N02 uses sealed handles over normalized complete resolved graphs; no digest substitutes
for foreign equivalence. N03 uses one native predicate compiler with explicit visibility
and specialized primitives. N04 uses native arrays/generated typed views and one lossless
literal codec; no serde_arrow row codec remains. Identity retains RowConverter and
shared declared float/framing policy. N11 uses the shared typed adjacency/binding view;
no second petgraph authority was introduced. N12 owns region-aware shared stages.
N13 retains native reservations and buffer owners: Arrow's infallible claim-growth API
does not provide the required fallible pre-admission transfer, so the claim bridge is
not a deferred alternate implementation. N14 is one cohesive catalog module using native
Delta builders and one effective retention policy. Solver and SQL choices are recorded
above; no new workflow or invented rule generator is required. These are selected target
mechanisms whose independent functional oracles execute in N18.
