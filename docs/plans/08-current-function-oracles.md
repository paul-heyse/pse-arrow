---
title: Plan 08 current-function oracles and replacement boundaries
status: draft
date: 2026-09-16
adrs: [ADR-0068, ADR-0069]
phase: 1
---

# Plan 08 current-function oracles and replacement boundaries

**Implemented, inspected:** these existing tests contain independent functional
assertions. Their legacy setup is replaced with their callers; none is an old/new
object or byte-equivalence requirement. No fresh passing result is claimed here.

| Outcome | Existing executable oracle | Required replacement |
|---|---|---|
| Authoring changes and exact source correspondence | `tests/engine/tests/native_change_batches.rs`, `source_projection_admission.rs`; authoring crate tests | Typed spans/keys and exact Delta selections replace sidecars and snapshots |
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

**Proposed:** `just architecture-acceptance <new-output-directory>` becomes the
terminal Plan 08 command during SP13. It uses ordinary xtask/test entry points and
records commands, features, counts and conditions. It runs current-function tests,
linked solver qualification, cold Rust/Python inspection, generation and governance,
plus Q01–Q14, the extension proof and independent G1–G7 review receipts. Missing
receipts or surviving forbidden production paths prevent a success receipt. It has
no dependency on the nonexistent `unified_simulator` test or future Pyomo/NL cases.

`just simulator-acceptance` remains historical Plan 07 tooling until the terminal
command replacement; it is not a Plan 08 acceptance gate. SP13 must delete obsolete
entry points with their callers rather than add a compatibility alias.

## Deletion closure

The authoritative [Plan 08 ledger](08-schema-first-native-data-pivot.md#replacement-and-deletion-ledger)
tracks each replacement with producer, consumer, generator, fixture and test closure.
SP08–SP11 remain one caller replacement. A green boundary test does not certify
removal of `Catalog`/`Snapshot`, `Driver`/`StageDag`, RulePlan/RuleExpr, Cell round trips,
JSON refs/manifests, sidecars or old Python handles.

## Measurement inputs

**Proposed:** retain reproducible synthetic nested values with null parents, empty
lists, required children, width and unsigned extremes; expression fixtures from the
two-template and heater/mixer tests; and the existing numerical fixtures above.
SP04/SP13 record sizes and pinned plans before claiming costs or I/O improvements.
No benchmark result or flat-versus-nested speedup has been established by this inventory.
