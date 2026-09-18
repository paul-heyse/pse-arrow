---
title: Plan 08 current-function oracles and replacement boundaries
status: in-progress
date: 2026-09-16
adrs: [ADR-0068, ADR-0069]
phase: 1
---

# Plan 08 current-function oracles and replacement boundaries

**Implemented, inspected:** these existing tests contain independent functional
assertions. Their legacy setup has been replaced with their callers; none is an old/new
object or byte-equivalence requirement. No fresh passing result is claimed here.

| Outcome | Existing executable oracle | Required replacement |
|---|---|---|
| Authoring changes and exact source correspondence | `tests/engine/tests/source_projection_admission.rs`, `incremental_equals_clean_p0_p3.rs`; authoring native edit/rename tests | Typed spans/keys and exact Delta selections replace sidecars and snapshots |
| Normalized mathematical meaning | `tests/engine/tests/native_normalization.rs` | Keep conditional senses/branches, source ownership and meaningful unsupported cases; replace stage setup |
| Template expansion | `tests/engine/tests/native_template_graph.rs` | Two-template independent symbol/equation assertions over coherent node rows |
| Current heater/mixer expansion | `tests/engine/tests/native_engineering_workflows.rs` | Four FTPx/FcTP journeys through P10; no new source-to-solve promise |
| Cold source read and inference/support | `tests/engine/tests/unified_sources.rs` | Exact source text reopens from Delta; native support query assertions |
| Scalar math | `crates/pse-numerics/tests/scalar_math.rs` | Weighted results after graph release; explicit missing binding/inexact integer failures |
| Numerical derivatives | `crates/pse-numerics/tests/native_expressions.rs` | Independent residual/Jacobian oracles, ordering, guards, cancellation and reservation behavior |
| Native capability without linking | `crates/pse-backend-native/tests/planning.rs` | Invalid bounds/scales and explicit unavailable capability |
| Real existing Ipopt functions | `crates/pse-backend-native/tests/native_solve.rs` | Independent feasibility, optimum/duals and mixer/heater assertions; panic/cancel/drop/budget cases |
| Provider and durable contracts | `crates/pse-catalog/tests/unified_delta_contracts.rs`, `unified_delta_dml.rs` | Actual configured child execution, validation, conflicts and cold version binding |
| Recursive boundary | `crates/pse-schema/tests/field_contract.rs`, `native_field_declarations.rs` | Synthetic list/struct mutations reject names/order/metadata/nullability differences; ordinary native types remain eligible |

## Acceptance command contract

**Implemented; not run:** `just architecture-acceptance <new-output-directory>` is the
terminal Plan 09 command carrying all Plan 08 obligations. It uses ordinary xtask/test entry points and
records commands, features, counts and conditions. It runs current-function tests,
linked solver qualification, cold Rust/Python inspection, generation and governance,
plus feature/configuration checks. Q01–Q14, extension proof and G1–G7 are assessed
independently against the command logs and measurements. Surviving forbidden
production paths prevent the campaign. It has
no dependency on the nonexistent `unified_simulator` test or future Pyomo/NL cases.

The obsolete `simulator-acceptance` recipe and xtask entry point are deleted.

## Deletion closure

The authoritative [Plan 08 ledger](08-schema-first-native-data-pivot.md#replacement-and-deletion-ledger)
tracks each replacement with producer, consumer, generator, fixture and test closure.
The current source/caller/deletion status and native caching oracles live in the [Plan 09 execution inventory](09-execution-inventory.md). A green boundary test does not certify
removal of `Catalog`/`Snapshot`, `Driver`/`StageDag`, RulePlan/RuleExpr, Cell round trips,
JSON refs/manifests, sidecars or old Python handles.

## Measurement inputs

**Proposed:** retain reproducible synthetic nested values with null parents, empty
lists, required children, width and unsigned extremes; expression fixtures from the
two-template and heater/mixer tests; and the existing numerical fixtures above.
SP04/SP13 record sizes and pinned plans before claiming costs or I/O improvements.
No benchmark result or flat-versus-nested speedup has been established by this inventory.

## Native caching extension

Plan 09 adds cache-on/off controls, actual projected reuse, epoch reset, cross-process
maintenance, exported Arrow pinning, bounded inspection, four-image CDF, CRC/load-class
and cost-matrix fixtures. See the [current inventory](09-execution-inventory.md).
These fixtures are implemented and remain not_run until the C11 barrier opens.
