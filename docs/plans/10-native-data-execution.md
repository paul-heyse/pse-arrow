---
title: Native correspondence, numerical stages and allocation ownership
status: done
date: 2026-09-19
adrs: [ADR-0073]
phase: 1
evidence: Tested
---

# Plan 10 N10, N12 and N13 execution

This records the authorized slice of [Plan 10](10-native-contract-consolidation.md).
N14–N18 remain separate work. No compiler/publication/Delta/solver journey or performance
campaign is part of this checkpoint. Final A08–A10/A15 acceptance remains N18 work.

## Dependency order and implementation

### N13: one native resource owner

**Implemented:** application consumers register `MemoryConsumer` with the actual native
`MemoryPool` and use `MemoryReservation`. `pse-ids` reexports these upstream types; it does
not define replacement traits. `EngineFactory` and `SharedRuntime` use the same pool as
native execution. A scoped native pool enforces invocation limits and rolls back parent
admission on failure. Finite deployment limits remain explicit.

`AllocationLease` freezes a native reservation. The Arrow adapter inventories backing
allocation identity, retains the original storage, and uses weak owner entries to share
that reservation across columns, slices, concurrent exports and Arrow C-data owners.
Capacity moves through `MemoryReservation::split`, with no release/reacquire interval.
An allocation already owned by one pool is not reassigned to an importing invocation.
New copies acquire independent capacity; final buffer ownership releases it.

**Interface-checked:** Arrow 59.3's claim/owner replacement and infallible claim growth do
not supply this fallible pre-admission/transfer contract. Arrow claim features remain
**disabled**. The selected implementation is the native reservation owner, not an optional
bridge or deferred claim implementation. Native accounting represents admitted buffers
and explicit working allowances; it is not a process-RSS bound or interception of every
foreign allocation. A foreign owner that hides backing capacity cannot be inferred.

Resident cache batches and exports share allocation owners. Cache metadata has its own
native reservation; cache-attributed resident bytes are not another global buffer charge.
Spill releases resident owners. Reading a spill admits its newly decoded buffers and can
fail under a pool too small to hold them. Streams/exported arrays retain the cache pin and
allocation ownership through eviction and final release.

Columnar admission uses separately named schema/predicate/occurrence working allowances.
`algorithm_decode_extent` remains only for explicit generated-row algorithm boundaries;
it covers decoded value storage rather than a removed `Cell` representation. Transient
batch admission resizes its allowance instead of accumulating completed batch work.

### N10: native correspondence and stable support

**Implemented:** compiler correspondence uses native joins, projections, aggregation and
union captures. Keys are calculated from the same Arrow rows as values. Shared helpers
build qualified `Column`s and ordinary native plans. Named generated `columns::*`
references derive from the registry and supply the P7/P8 correspondence declarations.
Capture flattens qualified outputs and witnesses exactly once, including conditional
witness predicates. Ambiguous unqualified columns are rejected by native resolution.

P7 stores actual source keys with its bounded algorithm views. Native context capture
selects instance/template declarations, domains/products/tuples and source-specific facts;
method/parameter, symbol/body/contract, port/member/target and connection matches use
native correspondence. P8 uses native law/context/axis/decision/descriptor and connection
matches; stable-key lookup preserves multiplicity. P9 selects parameter, declaration,
instance and coefficient values natively before decoding. P6 retains its necessary native
capture barriers and uses the shared qualified construction primitives.

Ordinary join equality follows SQL null semantics. An absent positive match cannot become
positive support. Optional absence has an explicit complete source-read witness. Unique
contracts require exactly one matching key; a join never arbitrarily chooses a row.
Ordered axes/member ordinals are sorted and checked for gaps and duplicates before graph
construction. Identical output values may have several supports; conflicting primary-key
values still fail admission. Input order is not evidence identity.

Template paths use one borrowed `PreparedPaths` view, not repeated whole inventories.
P4 prepares its required instance/tuple projections natively. P5/P7 borrow captured rows.
Enumeration carries validated owner/coordinate/domain state forward; it does not resolve
the complete path again for every result. Existing parent, tuple, duplicate and numeric
coordinate checks remain active.

### Retained finite algorithm boundaries

These are ephemeral views, never parallel persisted model authorities:

| Boundary | Why the typed view/index remains |
|---|---|
| P7/P8 stable-key row lookup | Native matches identify exact captured rows; graph construction needs their typed payloads and positive support. No position search recovers a source key. |
| Ordered actual domains, tuples, predicate masks and free indices | Validate coordinate order/completeness and specialize bound graph regions; arbitrary caller-bound tuples are algorithm inputs. |
| Realized symbols/groups, aliases and graph support | Newly constructed graph objects need direct identity/coordinate lookup, cycle checks and support propagation while construction is in progress. |
| Physical quantity/type registry and coefficient tensor checks | Exact unit, shape, subject and domain arithmetic remains a specialized typed algorithm. Native selection precedes its decoding. |
| Prepared template path indexes | State-dependent parent/child traversal and coordinate validation reuse borrowed rows without graph cloning or repeated relational scans. |
| P6 shared producer captures | Bound native planning growth and repeated source execution; their removal would duplicate work. |

### N12: shared value and derivative stages

**Implemented:** canonical graph nodes lower once into a native-expression arena. Child
edges are slots. Derivative rules consume shallow nodes and intern their results rather
than clone expanded derivative trees. Affine and weighted-mean terms intern incrementally.
Sparse Jacobian ordering and the existing generated numerical output contract remain.

Native physical expressions are prepared against their exact small stage schemas and
actual caller `SessionState`. Each stage records source slots, output field and optional
Boolean region mask. CASE guards partition remaining rows; branch expressions use native
`evaluate_selection`, and inactive lanes are normalized to null. Sharing is keyed by
expression and region. User volatile functions and variable-dependent guards are refused;
trusted generated output constructors retain their existing separate behavior.

`EvaluationProgram::stage_views` exposes logical expressions, physical expressions, exact
schemas, input/stage bindings and masks. Liveness releases intermediate arrays after their
last use. The native numerical operation and solver callback consumers use the same
prepared program. No callback plans a query. The former single logical projection path
was deleted because it allowed shared expression trees to be expanded again by lowering.
No new simulator workflow was added.

## Library evidence

**Interface-checked:** the local DataFusion/Arrow and Delta skills, exact pinned source
and Cargo metadata informed these boundaries: DataFusion 55.1.0, Arrow/Parquet 59.3.0,
object_store 0.13.2; Delta `58f07cd62bfbce3649a7e1c87c696288068ae184` and buoyant kernel
`8ba063f8f84fec222000f66d40d70911d7c79675`. Pins and Delta overlay are unchanged. Native
Delta operation-context/retention changes belong to N14. No Context7 evidence was used
for these libraries. No performance improvement is claimed from structural units.

## Deletions

- Custom `MemoryReserver`/reservation traits, `FixedBudget`, `PoolReserver` and engine
  `reserve.rs`; callers use actual native pools and reservations.
- Old `validation_extent.rs` and its undifferentiated public forecast.
- P7 `capture_keys`, source-position recovery, parallel positional source-key inventories
  and decoded method support scans; unused decoded method/requirement and P4 tuple fields.
- `PathInventory`, per-occurrence inventory copies and full-path re-resolution during
  enumeration.
- Recursive numerical value/gradient expansion forecasts and `logical_projection`.
- Duplicate native alias grammars and construction helpers replaced by qualified Columns.

## Verification

All execution below is development assurance, with zero failures as the required baseline.
The source-qualified receipts are recorded in the execution inventory; tracking-state and final functional limits are explicit there.

- `just dev-native-data`: explicitly selected in-memory operator, graph/numerical, buffer,
  native-cache/spill and path units, with Arrow `force-validate`; no whole-pass workflow.
- `just check-native-contracts` and `just check-native-contracts-solver`: compile consumers
  and their tests, including the pinned solver interface; execute no solver.
- `just lint-native-data`, `just fmt-check`, `just family-check`,
  `just codegen-contracts-check`, `just adr-lint`, `just docs`: static/generation evidence.
- `just codegen-contracts`: regenerates named field references and all schema projections
  through the generator; never patches generated output by hand.

**Tested:** the final `just dev-native-data` run passed **41 selected units**, zero
failed, 195 excluded, nextest default profile with `force-validate`, against zero baseline.
Test execution took 1.776 seconds, excluding compilation; this is not a performance claim.
The selected units cover stable/multiplicity/null source matches, qualified witness
capture, actual path coordinates, diamond/stage growth, independent derivatives, branch
exclusion, native budget rollback, concurrent/shared/FFI owners and cache spill/readback.
The two static error-taxonomy units also passed with zero failures against zero baseline.

**Interface-checked / Tested (static):** workspace all-target compilation, solver-feature
compilation, strict workspace and solver-feature Clippy, formatting, family checks, ADR
lint and the documentation build completed successfully. Python/docs generated-tree
checks passed. Rust generated bytes matched fresh output, then
`just codegen-contracts-check` failed its tracking check on the pre-existing untracked
`crates/pse-relations/src/generated/runtime/validation_findings.rs` (one failure against
zero baseline). That N03 artifact and the existing index were preserved. This is an open
tracking gate, not a passing codegen recipe. N17 cannot seal while it remains unresolved.

Final compiler/Delta/Python/solver functional qualification, performance measurements,
N17 sealing and N18 acceptance have **not** been performed.

## Outcome

The target directly replaces its predecessors. A mistake caught by the isolated compiler
units was using Arrow `with_schema` to remove schema metadata: that API requires a
superset. Query and owned-batch metadata projection now retain exact field contracts and
buffers through explicit batch reconstruction. Another corrected assumption was that a
spilled reader could remain uncharged under a one-byte pool; decoded buffers now require
native admission. Deliberate retained algorithms and accounting limits are listed above.
