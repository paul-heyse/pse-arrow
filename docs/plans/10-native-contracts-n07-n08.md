---
title: Native operation and function contracts — N07–N08 implementation
status: done
date: 2026-09-19
adrs: [ADR-0073]
phase: 1
evidence: Tested
---

# N07–N08 implementation contracts

This is the implementation detail for [Plan 10](10-native-contract-consolidation.md),
with status and qualification recorded in the [execution inventory](10-execution-inventory.md).
It does not close N09–N18 or certify integrated process-model behavior.

## Native operation ownership

The engine's `operation::Definition` owns actual implementation identity, output fields,
visible expressions, effect declarations and input consumption. Its typed `Body` owns the
algorithm. `Operation`, one extension planner and `Execution` own reconstruction, caller
state, bounded transport, deferred start and native metrics. Domain errors stay native.
Logical reconstruction retains the same attempt owner and checks expression/child arity
and field meaning; native nonnull refinement is permitted. A distinct operation gets a
distinct attempt even when its display name and data are equal.

| Family / owner | Consumers | Consumption, cancellation and reset | Statistics / observation |
|---|---|---|---|
| Finite operation | Compiler typed algorithms; rule fixed point | Complete bounded inputs; cancellable; never admitted for prepared epoch reset | Unknown output estimates; native starts, output rows/bytes, errors, early drops and poll compute time |
| Command | Delta write, publication, maintenance, DML; private native DML; namespace commands | One attempt; deferred until first poll; retained task settles cooperative foreign work after reader abandonment; no replay/reset | Unknown output estimates; same native metrics; catalog retains conditional-commit and reconciliation algorithms |
| Foreign operation | Native solver | Complete bounded input; one attempt; retained task and callback cancellation; actual solver permits and allocation owners; no reset | Actual physical expressions visible to visitors; same metrics |
| Observation | Scoped metadata resolution | One lazy observation; no reusable epoch promise | Unknown estimates; shared metrics |
| Requirement barrier | `ExecutionContract` | Drain zero-violation relations before values; one shared completion across partitions; last waiter abandonment is terminal | Unknown statistics while requirements exist, preventing statistics-based elision; transparent statistics only without requirements |
| Ownership wrapper | Delta read lease and predicate-cache budget | Transparent native child reconstruction/filter/limit behavior; owner remains in returned stream; reset eligibility still checks child | Modern child-statistics delegation; child metrics counted once |
| Invocation materialization | `NativeCache` / `Cached` | Shared completion, bounded pages, native spill, epoch invalidation; cancellation never restarts partially consumed input | Existing producer/cache/spill metrics retained |
| Selected resident cache | Catalog `SelectedExec` and resident service | Retained because selected-table generation, provider lineage, version checking and lease reacquisition differ from an invocation cache | Existing hit/miss, page and residency instrumentation retained |

`CREATE VIEW` has a definition-only input port. It retains a visible native logical
input without consuming it or imposing the finite-input requirement on that definition.
CTAS consumes its planned input and preserves declared constraints through reconstruction.

### Required work survives value optimization

Before native value optimization, preparation isolates native mutations and places
requirements/effectful descendants into invocation-local shared producers. A root barrier
retains these producers even if a projection, empty relation or zero limit removes all
value demand. Repeated references to the same actual command share one completion; distinct
commands do not merge by SQL name. Native cache and subquery regions retain separate
protection scopes. Correlated requirements are not hoisted into an enclosing caller.

The barrier keeps requirement failures observable and never advertises an exact row count
that could let an aggregate optimizer bypass it. A view definition's inputs are not
promoted into creation-time work. Planning and EXPLAIN create no task or effect.

A shared pure producer stores success or failure for its invocation/epoch. Dropping the
last unfinished reader drops the work and records terminal abandonment, releasing retained
resources. A surviving reader can finish the same producer. Commands and foreign work
instead retain a task through settlement, propagate the current tracing dispatcher/span,
and request cooperative cancellation on reader drop. The shell does not retry or reconcile
a Delta commit; those decisions remain in the typed catalog implementation.

### Native providers and epochs

- Immutable candidate, captured and materialized inputs use one generic provider over native
  `MemorySourceConfig`. Candidates do not advertise unproved keys; captured contracts can
  supply established constraints. Native projection and fetch are retained. The ownership adapter uses native
  `Statistics::with_fetch` for single-partition estimates and conservatively downgrades
  global multi-partition fetch estimates.
- Epoch inputs use native `StreamingTable`, `StreamingTableExec`, `PartitionStream` and
  `RecordBatchStreamAdapter`. The partition acquires an immutable epoch reader when the
  stream is created and retains it through stream drop. Replacement is excluded while any
  reader remains. Only the actual admitted partition owners qualify for round reset.
- Cache statistics use native streaming transport and sample counters only on polling.
- Source wrappers retain leases in streams across rewrites and native fetch. They do not
  copy child metrics onto parents or assert invented exact cardinality.

### Deleted predecessors

Removed bespoke logical/planner/physical triples for compiler domain algorithms,
rule fixed points, Delta writes/publication/maintenance, and native solving. Removed
separate native command/DML, epoch, statistics and metadata-resolution physical shells.
Removed the candidate provider module and catalog-owned lease transport; callers now use
the common engine families/native adapters. Compiler/rule/solver planner registration
APIs were deleted, with all active callers converted. `DeltaWrite`, `DeltaPublish` and
`Solve` remain constructor namespaces for their domain operations, not parallel nodes.

## Function capability and representation contracts

Pinned reference: DataFusion 55.1.0, Arrow 59.3.0 and delta-rs
`58f07cd62bfbce3649a7e1c87c696288068ae184`. Local skill routes and exact resolved source
were the implementation authority. No library family upgrade or Context7 substitution
was used for these contracts.

### Complete native hook disposition

| Adapter / hook group | Disposition |
|---|---|
| `Element`, `FieldStruct`: name, signature, return type, aliases, display/schema names | Original native implementation; adapters preserve its ordinary SQL identity |
| Scalar nullability/strictness, conditional arguments, short circuit, coercion, bounds/constraints, order/lexicographic/strict ordering, struct-field mapping, placement, docs | Shared pin-specific forwarding; actual return-field derivation remains the representation authority |
| Scalar simplify and preimage | Native forwarding; these pinned implementations have no field-changing custom simplification; native expression admission/materialization continues to handle resulting fields |
| Scalar updated configuration | Rewrap the returned native implementation; retain the admitted original implementation owner |
| Scalar return-field hooks | `Element` carries the actual child field (including IndexTuple identity semantics); `FieldStruct` carries actual argument fields and native output names, with nonnull outer struct |
| Scalar invocation | Native scalar/array dispatch; only all-scalar nested `array_element` uses a one-row native array to avoid the pinned scalar-conversion field-loss gap |
| `Selection`, `Collection`: names/signatures/return types, aliases, SQL/window names, display, nullability, coercion, null/within-group clauses, docs | Native forwarding, with explicit field-aware return values |
| Aggregate order sensitivity, descending direction, monotonicity, statistics value, defaults, simplify, expression-vs-literal simplify | Native forwarding; no duplicate statistics or optimizer algorithm |
| Aggregate beneficial ordering / reversal | Rewrap the actual returned native UDAF; preserve original owner. Reversal no longer reuses the pre-reversal implementation |
| Selection state/single/groups/sliding | Native factories and state protocols; selected output retains actual input field meaning |
| Collection state/single/groups/sliding | Native factories and algorithms; restore established List child fields in scalar/array values and first value state, retaining additional native ordering state |
| Accumulator update/evaluate/state/merge/retract/supports-retract/size | All native hooks forwarded; field restoration applies only to returned collection values/state |
| Groups update/evaluate/state/merge/convert-to-state/size | All native hooks forwarded; field restoration applies only to returned collection values/state |
| Window and higher-order UDF implementations | No custom wrappers exist. Native implementations and retained inventory continue directly; aggregate window use inherits the complete UDAF contract |

Adapter equality/hashing uses the actual current native implementation and admitted origin
owner, plus its concrete adapter type. Neither SQL names nor caller assertions establish
identity. Configuration/reversal/ordering can change the current implementation while
preserving the original admitted owner. No generic function DSL or new generator was added.

`pse-schema::literal::scalar_from_array` is the shared one-value Arrow-to-ScalarValue boundary.
NativeLiteral, array element and collection adapters use it without JSON roundtrips. It
retains exact nested child fields for List/LargeList/fixed-list/list-view/struct/map values.

### Statistics substitution and field preservation

The factory replaces the actual built-in `AggregateStatistics` rule at its existing position
with a small field-preserving adapter retaining that exact native rule object. Native
eligibility, exact-statistics selection, values and nullability refinement remain native.
After a substitution, native `ProjectionExec::try_new_with_schema_metadata` restores the
original aggregate's field/schema metadata. Restoration occurs during the rule's bottom-up
walk so parent expressions see the preserved meaning. Existing nested Cartesian-field
protection remains a separate, justified physical rule.

### Deliberately specialized semantic UDFs

These are PSE operations with stronger semantics, not incomplete wrappers around native UDFs:

- `TypedConcat` keeps equal-layout List and IndexTuple admission. Unrestricted native
  `array_concat` remains available. Native execution and Arrow checked casts are reused;
  scalar inputs return a scalar, with the actual buffer reservation retained. Native
  coercion/signature/strictness hooks cannot be imported when they would broaden admission.
- `ListField` and `PreserveField` share checked Arrow field realization. PreserveField's
  array materialization is intentional: the pinned projection scalar-broadcast path would
  otherwise discard nested metadata. It cannot be simplified away as an unchecked identity.
- `RetainMetadata`, `RequireNonNull`, `CheckedValue` and `IndexTuple` retain explicit
  field/nullability/admission postconditions. Defaults for unsupported ordering, preimage,
  simplification and coercion are conservative because a stronger proof is not established.
- Diagnostic/ID codecs use existing Rust kernels. Capture `Values`/`KeyCheck` retain their
  caller-bound observation/validation semantics; importing pure native simplifications
  would erase those obligations. No aggregate/window state contract applies to them.

## Verification boundary

**Implemented:** common families, native transport, cancellation/completion, metrics,
complete native adapters, statistics field restoration and predecessor deletions above.

**Tested:** `just dev-native-contracts` passed 34 isolated units with zero failures
against a zero baseline (default nextest profile, Arrow force-validate). Static/feature
checks and their conditions are recorded in the execution inventory. The unit fixtures exercise native Arrow/DataFusion
operators with fake effects and small values. They do not compile a process model, run a
solver, publish a Delta table or run an integration/performance campaign.

**Proposed / deferred to N18:** durable cancellation fault journeys, end-to-end compiler and
solver behavior, broad adapter permutations and integrated A05/A06 acceptance. N09–N17
remaining implementation/deletions still precede that campaign.

## Outcome

The initial shared-cache reconstruction allocated a new owner even for an unchanged input.
The cloned-command unit exposed the resulting duplicate attempt; reconstruction now keeps
identity only for the unchanged native computation. Changed inputs still invalidate it.

Deliberate deviations: specialized Delta resident-cache policy remains catalog-owned, and
semantic PSE UDFs keep justified conservative hooks. Foreign solver settlement has its own
family, separate from durable commands. No compatibility wrapper or transition period was
introduced.
