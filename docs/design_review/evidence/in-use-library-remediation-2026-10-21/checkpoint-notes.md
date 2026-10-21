# Checkpoint notes for the principal reviewer (coordinator, 2026-10-21)

> **Supporting evidence** for [Design review: in-use-library remediation of the first design-principle audit](../../reviews/design_review_in-use-library-remediation_2026-10-21.md).
> Role: Coordinator notes given to the principal reviewer: maintainer corrections, probe outcomes and census corrections.
> Baseline: source `a791c533e`, audit publication `0530dcaa1`, 2026-10-21. Paths under `/tmp/...scratchpad/` refer to the review's working copies; their published equivalents are in this folder. The principal review owns the judgment; this file is input to it.

## Maintainer guidance received during preparation

These are binding for this review. They are also recorded in the brief.

- **Library scope.** The review is library-targeted: only libraries already in use, especially functionality we don't use or under-use. Massive design changes are allowed.
- **No new categories.** It improves *existing* functionality. Small functionality improvements are fine only where they are clear wins. No major additions that carry tradeoffs. Never a "compromised" approach that reverts or weakens another part of the system. If a finding cannot be fixed without such a regression, record it as remedy-open and state the conflict.
- **Premises P1–P3** (in the brief): SurrealDB stays canonical as the enabler of 16-lane parallelism; durability is daemonless and may outlive the caller; other formats are tactical only.
- **P4.** Vendored or patched solver libraries carry *integration hooks*, not core-solver edits. Examples: SUNDIALS KLU observers and fixed-pattern hooks, IDAS checkpoint and replay budgets, POUNCE observed execution. Do not treat these as forks to eliminate.
  - For the Uno patch (DA-018), classify each hunk as integration or numerics before judging it.
  - **Retracted:** the native inventory's §0 "vendored SUNDIALS sources modified; notes wrong", its W5, and C6 "replace the KLU patch". Do not carry them forward as framed.
- **Premises are a judgment call.** Not every premise needs a source, but never present a current mechanism as a target requirement.

## Probe results (Tested or Measured; full reports in this folder)

### `probe-worktree.md`

**A. Body granularity** (BT_PR flash, 19 instance groups, 118 outputs). Measured.
- **Refuted:** one `evaluator_multiple` plus a Dualizer over a whole dispatch group. Taylor work follows the union of coordinates: 9× today at first order, 280× at second, and about 10 s to build at second order.
- **`merge`** gives counts identical to the group evaluator, so it adds nothing.
- **Winner:** symbolic `derivative` per output, then one joint `evaluator_multiple`. About 41% below today at value order and about 58–59% below at first and second order.
- **Shared members:** zero `let` members are shared across outputs in this fixture. The value-order win is cross-output CSE.
- **Where the cost is:** the non-substitutable outputs (provider or branch; all PR phase instances) dominate today's cost, 13854 / 57250 / 201392 ops, and grouping cannot reach them.
- **Consequence:** prep-inventory C1's mechanism is reversed for derivatives. DA-028's "members recomputed k times" premise did not show in this fixture. The real costs are the per-output Taylor programs and the provider/branch bodies.

**B. Demand and catalog** (ThreeUnits reference flowsheet). Tested and Measured.
- **B1.** Demanding a path that is already a symbol is bookkeeping only. Demanding a new `let` member changes the dispatch group: hash, coordinates 42→44, expressions 20→21. So moving demand out of `Request` is necessary but not sufficient.
- **B2.** Under *production default limits* (`query_values` 64; retention 16384 entries / 256 MiB), editing a declaration the root does not reach rebuilds the whole compiler workspace:
  - generation 1→2;
  - every granular query re-runs;
  - `selected`, `specialized`, `projection` and 143 `semantic_body` re-execute, with 0 bodies reused.
  - The package's retained footprint is about 54.5k entries and about 606 MB, which exceeds the defaults, so the rebuild happens even at first prepare.
- **B2, limits lifted.** Only `selected` and `projection` re-execute; `specialized` backdates; all 143 bodies are reused. This confirms the catalog-firewall mechanism.
- **LRU.** At `query_values=64`, a no-edit re-prepare re-runs 79 `semantic_body` queries.
- **Consequence.** DA-030's dominant cause at default settings is retention and LRU capacity versus footprint (workspace rotation). Coarse dependency granularity is secondary.

**C. Horizon caller cancellation.** Tested.
- **Today:** cancelling the caller is ignored; all 20 of 20 steps complete.
- **After the one-line fix** (`driver.rs:142` → `cancel.child_with_deadline(None)`): the run is cancelled after 1 step, the join takes 0.5 ms, and both existing horizon controls still pass. DA-012's divergence is now *Tested*.

**D. Uno latch.** Tested with synthetic inputs.
- **W2 is not live:** the latch block and `CallbackState::finish` override the native category.
- **W1 is confirmed:** iterate 5 (`INFEASIBLE_SMALL_STEP`) maps to `Infeasible`.

**Environment observations from the probe runs** (not caused by the probes):
- The shared test-resource registry and the shared canonical store profile both refuse. The latter fails with "Missing payload incarnation".
- `pse-backend-native --features uno` alone gives an unloadable test binary (MKL symbol); `uno,ipopt` works.

### `probe-surreal.md` (SurrealDB 3.3.0, memory engine, HTTP and CLI). Tested.

- **Typed lifecycle (U1 shape confirmed).** Enforced on CREATE, INSERT, UPDATE and UPSERT:
  - tagged-union `TYPE` of object literals;
  - literal and `option` unions;
  - `READONLY`;
  - `$before` transition `ASSERT`s.

  An index on `state.phase` is used. Refusals arrive as `kind:"Internal"` prose.
- **UPSERT caveat.** `UPSERT <id> SET/MERGE` on an existing row reports a *wrong field* in the refusal text, though the refusal itself holds. `int` inside an object literal accepts but does not coerce whole floats.
- **THROW (U2 encoding confirmed).** Byte-stable across top level, `IF`, `FOR`, nested `fn`, `BEGIN…COMMIT` and `SET`. `kind:"Thrown"`. Object payloads render as canonical, sorted SurrealQL text and round-trip. In a failed transaction, earlier statements report `NotExecuted`, later ones `Cancelled`, and all writes roll back.
- **REFERENCE ON DELETE REJECT (U3).** Checked only at delete time.
  - A racing reference insert can leave a *dangling* owner, even with the reference side locking the target with `FOR UPDATE`.
  - Only a referrer write that actually changes the target protects in both orders.
  - A dangling owner reads as "not retired".
  - Typed record fields accept non-existent targets.
  - Result: U3 cannot replace the guard-row discipline.
- **Traversal (U4).** Deterministic edge-id order in both directions.
  - Outer `FROM <traversal> LIMIT/START` loads every edge, about 300 ms at 20k edges.
  - In-idiom `->(deps LIMIT n START m)` and keyset `->(deps WHERE id > $c LIMIT n)` take about 0.35–6 ms.
- **Caveat.** The memory engine, not the repository's RocksDB server, so conflict timing may differ.

### `probe-pruning.md` (arrow 59.3, datafusion 55.1). Tested.

- **Correctness.** The canonical predicate lowering is filter-equivalent to `scalar_predicate` on 63/63 grid combinations. Zero soundness violations over 3391 containers and 20107 rows, and exact bits survive `DataFrame::filter`.
- **Premise corrections.**
  - DataFusion 55.1 evaluates `-0.0 = 0.0` as true (`normalize_float_zero` in `apply_cmp`). Only the NaN half of the totalOrder trap remains. inv-data §1 and §4.2 are wrong on ±0.
  - `PruningPredicate::try_new` is deprecated; use `PruningPredicateBuilder`.
- **Effectiveness.**
  - Value pruning on Float64 columns is about 0% for unordered values at 16 rows per batch or more.
  - Key pruning on `step` and `symbol_id` in sort order is effective: 1 container per lookup when `step` is given. A missing id cannot be rejected by min/max.
  - Consequence: the DA-026 remedy's benefit is removing per-row locators plus key pruning, not value pruning. "Every Float64 column prunable" is sound but mostly ineffective.

## Census and inventory corrections

- **Physical optimizer rules.** Two exist (`SemanticFields`, `AggregateFields` in `crates/pse-engine/src/session/physical_fields.rs`).
- **Pool stack.** `ReleaseNotifyingPool(PeakRecordingPool(TrackConsumersPool<FairSpillPool>))`.
- **Symbolica `native_code_generation`** is enabled through `pse-math/compiled-evaluator`.
- **pyo3-arrow** is used for export (`PyRecordBatchReader`/`PySchema` in `inspection/stream.rs`). There is no Arrow *input*.
- **tracing-subscriber** has `env-filter` enabled.
- **statrs default features** are unified on through `egobox-moe`, which pulls in `nalgebra 0.33.3` beside 0.35.0. The pin comment at `Cargo.toml:287-288` is stale.
