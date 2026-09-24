---
title: Rust computation foundation library qualification
status: abandoned
date: 2026-09-23
adrs: [ADR-0076]
phase: 1
evidence: Interface-checked — resolved locked profile and compiled API surfaces
---

# Foundation library profile

**Historical qualification snapshot.** [Plan 14](14-library-owned-process-simulator.md)
selects its library composition from the new target and current evidence. This
profile neither requires retaining these dependencies nor qualifies the new math stack.

Pins remain declared in Cargo.toml; this is a qualification observation, not a second pin authority.
Source/feature observations come from `just metadata-resolve` and the committed Cargo.lock.

| Package | Resolved | MSRV | Licence | Unified features |
|---|---|---|---|---|
| `buoyant_kernel` | 0.25.1 | 1.88 | Apache-2.0 | `arrow-59`, `arrow-conversion`, `arrow-expression`, `default`, `default-engine-base`, `internal-api`, `nanosecond-timestamps`, `need-arrow`, `reqwest` |
| `deltalake-core` | 1.0.0 | 1.94.1 | Apache-2.0 | `datafusion`, `datafusion-datasource`, `datafusion-physical-expr-adapter`, `datafusion-proto`, `nanosecond-timestamps`, `rustls` |
| `petgraph` | 0.8.3 | 1.64 | MIT OR Apache-2.0 | `default`, `graphmap`, `matrix_graph`, `stable_graph`, `std` |
| `rust-igraph` | 0.7.0 | 1.85 | GPL-2.0-or-later |  |
| `rustworkx-core` | 0.18.1 | 1.85 | Apache-2.0 |  |
| `salsa` | 0.28.4 | 1.88 | Apache-2.0 OR MIT | `inventory`, `macros`, `salsa_unstable` |
| `tokio` | 1.53.1 | 1.71 | MIT | `bytes`, `default`, `fs`, `io-util`, `libc`, `macros`, `mio`, `net`, `parking_lot`, `process`, `rt`, `rt-multi-thread`, `signal`, `signal-hook-registry`, `socket2`, `sync`, `time`, `tokio-macros`, `windows-sys` |

The local Delta source is the existing `vendor/delta-rs` patch. The kernel source is
`https://github.com/buoyant-data/delta-kernel-rs` at `8ba063f8f84fec222000f66d40d70911d7c79675`.
No Arrow, DataFusion, object_store, PyO3, Delta or Tokio family upgrade was used to admit the graph/Salsa profile.

`rustworkx-core` unconditionally brings Rayon and enables petgraph defaults. These are
actual unified features, including graph containers not used by the projection implementation.
Feature removal is W16 work if qualification shows it is useful; direct `default-features = false`
does not promise to remove a transitive enabled feature. GPL eligibility follows ADR-0066.

## Used contracts

- Salsa: input/db/tracked macros, owned cloned returns, compare-before-set, per-handle cancellation, LRU eviction and the unstable memory report; no persistence or unsafe lifetime options.
- petgraph: immutable Graph with u32 indices, Dfs, DfsPostOrder, Reversed, iterative kosaraju_scc and unit-weight condensation.
- rustworkx: lexicographical_topological_sort and find_cycle on the same petgraph type universe. **The topological API returns partial output for cycles; the wrapper checks full node coverage.**
- rust-igraph: create plus maximum_bipartite_matching in an isolated adapter unit. Both typed partitions and all isolates are retained; production DM remains W10.
- DataFusion/Arrow: leaf typed predicates, ExprSimplifier and physical expression lowering below the engine; actual SessionState SQL/function binding in pse-engine; shared RowBuilder/FieldCheckedBatch, checked take/filter and strict cast/readmission.
- Delta/native providers: retain selected publication members and source owners; phase execution uses the existing engine preparation, streams, cache and relational obligation templates.

## Dependency edges and moves

`family-check` traverses resolved **normal** dependencies, including transitive edges.
Dev/build dependencies and proc-macro implementation closures are excluded from product ceilings.
Semantic roots, columnar roots and the generator root are declared in workspace metadata.

| Previous edge/owner | Foundation replacement |
|---|---|
| pse-ids → Arrow/DataFusion allocation/canonicalization | pse-columnar; IDs retain only semantic framing/scalars/hash ownership |
| pse-diagnostics → DataFusion native errors | pse-columnar native classification; pure vocabulary remains diagnostic authority |
| pse-mathir/templates → schema/native relation vocabulary | handwritten operator contracts in MathIR; generated values in pse-model |
| pse-schema → syn/quote/prettyplease | pse-codegen, with no generated DTO bootstrap dependency |
| pse-relations → full DataFusion SQL/session | leaf expression/kernel APIs plus ValidationPlanner; NativeValidation in pse-engine |
| pse-compiler → engine/catalog/rules/Arrow/Tokio | native compiler_driver and direct callers moved to pse-runtime |
| compiler stage wrappers | deleted; explicit CompilerSession, finite native drivers and typed semantic query owners |

## Verification

Commands and final results are in [the execution packet](13-w00-w06-execution.md).
Locked compilation qualifies the combined API surface; isolated units qualify the named controls.
Neither qualifies integrated compilation/storage/solver behavior or measured improvement.

## W15–W16 consumer refresh

**Interface-checked:** `just metadata-resolve` (`build/plan13/w15-w20/metadata-resolve-04.json`)
and `just family-check` (`family-check-10.log`) retain the same pins and unified family
features. The pure compiler now directly consumes the existing petgraph pin for
predicate cycle checks and iterative postorder; repeated Boolean operands remain in
generated rows, not in scheduling edges. Runtime's direct petgraph dependency is deleted.
The pinned Delta consumer enables DataFusion defaults; the kernel enables Arrow/Parquet
and cloud object-store features; rustworkx enables petgraph defaults. Supported APIs and
actual SQL consumers remain enabled. Kernel/TLS changes retain the qualified future
pin/feature trigger; no vendor alteration is used to suppress those features.

The complete generated source-size comparison and current pure/native responsibility
map are in the [W15–W20 execution packet](13-w15-w20-execution.md). They make no measured
build/runtime improvement claim.
