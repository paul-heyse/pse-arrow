# Salsa

Realizations considered: installed `salsa` 0.28.5, released 2026-09-24. It is the newest release.
Upstream `master` through `8e61e5b47` (2026-10-27) adds nothing relevant. The considered build
has the features `macros`, `inventory` and `salsa_unstable`; the `persistence` feature is
available but not enabled.

Research date 2026-10-27. Sources:
- Installed `~/.cargo/registry/src/*/salsa-0.28.5/src/{lib.rs, key.rs, database.rs, event.rs,
  function.rs}`.
- `tests/persistence*.rs` and `CHANGELOG.md` at the same path.
- `salsa-macros-0.28.5/src/tracked_fn.rs`.
- `gh api repos/salsa-rs/salsa/commits?since=2026-09-20`.

## Types

### salsa/query-identity-and-labels
- **Need it serves:** attributing query executions, validations, waits and discards to a
  meaningful subject without formatting authored data.
- **Recognizable code shapes:** a side map from `DatabaseKeyIndex` to a label, filled by calling
  generated ingredient accessors such as `fn_ingredient_(..).database_key_index(input.as_id())`
  through `salsa::plumbing`.
- **Mechanisms and APIs:**
  - `salsa::Event { kind: EventKind::{WillExecute{database_key}, DidValidateMemoizedValue{..},
    WillBlockOn{..}, DidDiscard{key}, ..} }` delivered to the database's event callback.
  - `DatabaseKeyIndex::{ingredient_index(), key_index() -> salsa::Id}` (public).
  - `Database::ingredient_debug_name(IngredientIndex) -> Cow<str>` (public). Its contents are
    documented as "not semver-guaranteed".
  - `salsa::Id` is public. Its constructor and the `AsId`/`FromId` traits are exported only
    under `salsa::plumbing`.
- **Obligations discharged:** event delivery, and a stable per-ingredient name for each query
  function.
- **Residual caller obligations:** mapping a key's `Id` back to a domain subject.
- **Semantic conditions:**
  - The public route yields *which query* (debug name) and an opaque `Id`.
  - For a single-input tracked function, `key_index()` is the input's `Id`. Turning that `Id`
    into the input value, or into its domain identity, needs `FromId`/`AsId` (plumbing) or
    `Debug` formatting, which may print authored fields.
  - Labels installed before the query runs can be keyed by `(ingredient_index, Id)`. The
    ingredient index can be learned publicly from the first event's `ingredient_debug_name`
    rather than from `fn_ingredient_`. The `Id` of the input still requires `AsId`.
- **Version / build conditions:** 0.28.5. `plumbing` carries no compatibility promise. The
  database revision counter `current_revision` is also exported only under `salsa::plumbing` at
  0.28.5 (`lib.rs` plumbing re-exports), so a caller that snapshots the revision (for example to
  date a retention decision) has a further plumbing dependency with no public replacement (added
  2026-10-27).
- **Limits and negative results:** there is no public "label this query", tag or user-data
  attachment on events or memos at 0.28.5 or on `master`. The search covered the `lib.rs`
  exports, `key.rs`, `event.rs`, `Database` trait methods, and the commit titles since
  2026-09-20.
- **Evidence:** Interface-checked.

### salsa/memo-retention-observation
- **Need it serves:** bounding and observing memoised value retention.
- **Recognizable code shapes:** `#[salsa::tracked(lru = N, heap_size = f)]`, `set_lru_capacity`,
  memory reports that walk ingredients.
- **Mechanisms and APIs:**
  - `lru = N` per tracked function, with `<fn>::set_lru_capacity(db, n)` at runtime.
  - `Database::trigger_lru_eviction(&mut self)`.
  - `heap_size = fn(&Output) -> usize`, a caller-supplied measure.
  - `<dyn Database>::memory_usage(&mut self) -> DatabaseInfo`, gated by `salsa_unstable`, giving
    `IngredientInfo::{debug_name, count, size_of_fields, heap_size_of_fields, size_of_metadata,
    page_info}`. Since 0.28.5 it requires exclusive access (#1328).
  - `Durability` tiers on inputs, and `synthetic_write`.
- **Obligations discharged:**
  - count-bounded memo retention;
  - aggregation of caller-reported heap sizes plus Salsa's own slot and metadata sizes;
  - skipping revalidation for high-durability inputs.
- **Residual caller obligations:**
  - `heap_size` is whatever the caller computes;
  - LRU is by entry count, not bytes;
  - admission before a query runs is not Salsa's concern.
- **Semantic conditions:**
  - `memory_usage` is a *post-hoc* census of existing memos.
  - LRU eviction happens at revision boundaries.
  - Memos are process-local.
- **Version / build conditions:** `memory_usage` requires `salsa_unstable`, which has no
  compatibility guarantee within 0.28.x.
- **Limits and negative results:**
  - no byte-capacity LRU;
  - no pre-execution size estimate;
  - no cross-process memo sharing without persistence.
- **Evidence:** Interface-checked.

### salsa/persistent-caching (prototype)
- **Need it serves:** reloading inputs and memos across process lifetimes.
- **Recognizable code shapes:** `#[salsa::input(persist)]`, `#[salsa::tracked(persist)]`, and
  `db.as_serialize()` / `db.deserialize(..)` around process start and stop.
- **Mechanisms and APIs:**
  - the `persistence` feature;
  - `persist` on inputs, interned structs and tracked functions;
  - `<dyn Database>::as_serialize(&mut self)`, which cancels other workers;
  - `<dyn Database>::deserialize(deserializer)`.
- **Obligations discharged:**
  - Whole-database serialisation of selected ingredients, with dependency edges.
  - Memos are revalidated against inputs after reload.
- **Residual caller obligations:**
  - a serde representation for every persisted value;
  - storage location, atomic publication and bounding;
  - versioning across code changes.
- **Semantic conditions:**
  - Keyed by Salsa `Id`s inside one database image, not by content.
  - A snapshot is all or nothing for the selected ingredients.
  - Serialising cancels concurrent work.
- **Version / build conditions:** 0.28.5 with `persistence`. The CHANGELOG describes it as an
  "Initial persistent caching prototype" (#967) with later fixes (#982).
- **Limits and negative results:**
  - not a content-addressed shared cache;
  - no concurrent multi-process access;
  - opaque native artifacts (shared objects) are not covered.
- **Evidence:** Interface-checked (source, tests, changelog).

## Current applicability

### [CBO-022](../../../library_leverage/codebase/mathematics-and-compilation.md#cbo-022) — Prepared-mathematics reuse in four layers beside Salsa
- **Types:** salsa/query-identity-and-labels, salsa/memo-retention-observation,
  salsa/persistent-caching.
- **Realization:** installed 0.28.5. Persistence would need the `persistence` feature on the same
  pin.
- **Conclusion:** fits with conditions for the labels; ruled out for the cross-process disk layer;
  unsettled for durability-owned evaluator products.
- **What it would discharge / what remains local:**
  - *Labels: query name.* `ingredient_debug_name(key.ingredient_index())` can supply the query
    family publicly, replacing the `fn_ingredient_` lookups.
  - *Labels: subject.* The subject still needs the input `Id`, which `AsId` (plumbing) supplies.
    One small plumbing use would remain, in place of four calls through the private
    `ZalsaDatabase`/`fn_ingredient_` route.
  - *Evaluator retention.* The retention layer (`body_retention`, identity-keyed, outliving
    database rebuilds) serves a lifetime Salsa does not own. Durability only avoids
    revalidation.
  - *Disk layer.* Salsa persistence is a database image keyed by Salsa `Id`s, not a
    content-addressed cross-process cache of compiled bytes.
- **Prerequisites, configuration or extension:** for labels, an event-side lookup keyed by
  `(IngredientIndex, Id)` with the `Id` captured at input creation.
- **Semantic gaps against the CBO's preservation constraints:**
  - The `ingredient_debug_name` text is not semver-guaranteed, so a label family would need a
    local mapping from debug name to family.
  - Persistence cannot preserve "reuse across processes" with identity following the immutable
    dependency closure.
- **Unsettled premises and what would settle them:**
  - Whether stable bodies could live as high-durability Salsa inputs rather than in an
    out-of-Salsa retention map. Reading the runtime retention layer
    (`pse-runtime/src/math/retention.rs`), which this pass did not read in depth, would settle
    this.
- **Evidence:** Interface-checked; Proposed.

**Reviewer correction (2026-10-27).** The premise of four parallel caches is rejected: the four
layers are four lifetimes with natural keys. `ScalarCache` is construction-scoped and frozen into
immutable `ScalarProducts`; `ModelingBodyRetention` is the runtime cross-workspace cache that
§14.4 requires, with an identity and physical check on every hit; the disk layer is ADR-0212's
cross-process cache; and `state.sources`/`requests` are Salsa's input-handle registry. Salsa
persistence is ruled out by §14.4 and ADR-0164, which also answers the unsettled durability
premise. The plumbing footprint is four `fn_ingredient_` and `AsId` uses plus `current_revision`
in the workspace retention snapshot; the public-label route would trade a compile-time break for
a runtime string mapping and still keep two plumbing items, so the review retains the current
route (an upstream request for public `AsId`/`current_revision` is low value). See the
[comparative review](../../reviews/mathematics-preparation.md#cbo-022).

### [CBO-001](../../../library_leverage/codebase/cross-cutting-mechanisms.md#cbo-001) (part) — Salsa `heap_size` beside local `allocation_bound`
- **Types:** salsa/memo-retention-observation.
- **Realization:** installed 0.28.5.
- **Conclusion:** ruled out as a replacement for prepaid admission. It is already a consumer of
  the local measure rather than a second accounting system.
- **What it would discharge / what remains local:**
  - Salsa reports what callers report. The inspected `heap_size` closures (`parsed_heap`,
    `members_heap`, `body_heap`) return the local `retained_bytes()`/`descriptor_bytes()`.
  - Prediction before execution remains local.
- **Prerequisites, configuration or extension:** none.
- **Semantic gaps against the CBO's preservation constraints:** post-hoc measurement, so not
  charged before allocation.
- **Unsettled premises and what would settle them:** none.
- **Evidence:** Interface-checked (repository source at the published baseline; Salsa source).
