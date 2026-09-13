# DataFusion (Rust) — capability map

**Companion to** `arrow-rust.md`; together they cover the engine stack that `supporting-rust-libraries.md` deliberately excluded. The register (§13) and gate review (§14) here are now **DataFusion-only**; the Arrow map carries its own. In the previous edition both were combined here, which meant neither document could be updated alone.

**Compiled** 2026-09-13 against **`datafusion` 55.1.0** on **`arrow` 59.3.0**.
**Adjudicates** blueprint **revision 2** (2026-09-13). The previous edition adjudicated revision 1 and never said so; §0.1 lists what changed underneath it.

---

## 0. Purpose, sourcing constraint, and evidence rules

Per cluster: what DataFusion can actually do, which APIs the blueprint's requirements land on, what we deliberately will not use, and where the blueprint assumes a capability that does not exist or is under-specified. A second pass asks what the engine offers that the blueprint has *not* claimed, which would improve alignment with `design_principles/DATA_MODEL_DESIGN_CHARTER.md` (DM-01…DM-60, gates G1–G7).

**Sourcing constraint.** Written **without** the repository's existing Arrow/DataFusion material — no `docs/library_ref/` prose, no `datafusion-pyarrow-*` skills, no pre-existing `build/facts/df55-default` surface (which is at 55.0.0 and would misrepresent the pin), no vendored `datafusion/` checkout. Two of those are now moot rather than merely avoided: `docs/library_ref/` has been **deleted**, and the vendored checkouts survive only as orphaned worktrees whose parent repositories are gone, so their tags can no longer be verified. Five fresh lanes:

| Lane | Marker | What it is |
|---|---|---|
| **locally generated rustdoc JSON** | `[rustdoc:crate@55.1.0]` | the authoritative lane — exact signatures from the pinned crates; receipt in §1 |
| **locally run probe program** | `[probe]` | **new in this edition.** The previous edition had *no* measurements: every claim was API shape. Six probes now measure behaviour the blueprint asserts. |
| context7 | `[c7:/apache/datafusion]` | the upstream `docs/source/library-user-guide/` tree and the per-version `upgrading/` guides — idiom and intent |
| docs.rs | `[docs.rs:crate@version]` | version-addressed pages, feature flags, release dates |
| upstream source at a release tag | `[gh:repo@tag/path]` | workspace manifests, config references and changelogs read at the exact pinned tag — used for the §12 enumerations, where a *complete* list matters and rustdoc gives only what was extracted |

Checked mechanically: no provenance marker is of the form `[ref:…]`, and none cites `/apache/arrow` (the multi-language C++/Python line). A claim carrying no marker is a defect.

**Status vocabulary** and the Pass-2 discipline (charter §G, **DM-58**) are as defined in the Arrow map §0.3. **Pass 3** (§12) is new and answers a third question: taking the *complete* inventory of the family — every crate, every public trait, every config key, every release delta — what is this design not leveraging at all?

### 0.1 What changed underneath this map

The previous edition was compiled at 14:35 on 2026-09-13; the blueprint was rewritten at 15:24.

| # | Change | Consequence |
|---|---|---|
| 1 | Revision 2 absorbed amendment **A** (pin discipline) and **C**/**D** with must-settle **#4** and **#6** — the engine became a *declared input*: `reference.engine_profiles`, ordered rule lists, a semantic-settings hash, and a plan fingerprint all enter the memo key and the pass record (**F9**) | §1.2's erratum is **resolved**. Register row #18 is no longer a recommendation but a binding to verify — and §5 does verify it, with a result that is not good news. |
| 2 | **F9 chose `datafusion-proto` as the fingerprint mechanism** | The previous edition never evaluated `datafusion-proto`: its open item 6 offered only "a formatting surface" or "a platform structural walk", and §10 rejected Substrait without mentioning proto at all. The crate was **absent from the extraction corpus**. It is now extracted, and measured — see **D9**, which is the most consequential finding in this edition. |
| 3 | Revision 2 added pass **P12** (index expansion) and renumbered P12–P15 to P13–P16 | Every pass reference in this map re-checked; §4 and §6 note where the new cardinality-changing pass lands. |
| 4 | The rustdoc extraction behind 60 `[rustdoc:]` citations **no longer exists on disk** | §1.4's new extraction is committed with its lockfile so this cannot recur. |
| 5 | Coverage was **20 of ~40** DataFusion crates | §1.4 now extracts **37**, including `datafusion-proto`, `-ffi`, `-substrait`, the `-datasource-*` family and `-physical-expr-adapter`. §12 states the denominator. |

---

## 1. Version anchors and the extraction receipt

### 1.1 Anchors

| Component | This document | Blueprint §3.1 (rev 2) | Note |
|---|---|---|---|
| `datafusion` and every `datafusion-*` sub-crate | **55.1.0** (2026-09-11) | **55.1.0, each crate `=`-pinned** | **Now agreed.** Still the current release at this compile `[crates.io:datafusion]` — no re-pin. |
| `arrow` family | **59.3.0** | **59.3.0** | DataFusion 55.x *requires* `arrow ^59.2.0` `[docs.rs:datafusion@55.1.0]`, so 59.3.0 resolves. But its workspace **builds and tests against `arrow = "59.2.0"`** `[gh:apache/datafusion@55.1.0/Cargo.toml]`, so the platform runs one minor ahead of the Arrow the engine is exercised against. Not a defect; a fact §3.1 should record. |
| `object_store` | **0.13.2** | 0.13.2 | **Resolved by cargo, not chosen**: DataFusion 55.1.0 requires `object_store ^0.13.2` `[docs.rs:datafusion@55.1.0]`. §3.1's pin is correct and 0.14.1 must **not** be used — this closes open item #3 of the supporting-library map. |
| `tokio` | 1.53.1 | 1.52+ | resolved |
| MSRV / edition | **1.94.0** / 2024 | 1.94.0 / 2024 | confirmed at the tag `[gh:apache/datafusion@55.1.0/Cargo.toml]` |

### 1.2 Why 55.1.0, and a pin-discipline erratum — **resolved in revision 2**

Retained as a record of why §3.1 reads as it now does, not as a live recommendation.

Two corrections to earlier reconnaissance, both worth recording because they are easy to repeat:

1. **55.1.0, not 55.0.0, is current.** A web search reported 55.0.0 as latest; the crate pages show `datafusion` **55.1.0** released 2026-09-11 `[docs.rs:datafusion@55.1.0]`, with `datafusion-expr` 55.1.0 the same day `[docs.rs:datafusion-expr@55.1.0]`. Search results were stale by two days.

2. **Pinning the umbrella does not pin the family.** Resolving `datafusion = "=55.0.0"` produced `datafusion` 55.0.0 with **every sub-crate at 55.1.0**, because the umbrella depends on `datafusion-* ^55.0.0`. That mixed state is worse than either coherent choice, and **`cargo tree -d` does not detect it** — it is not a duplicate. The identical problem exists in the Arrow family (`arrow` 59.2.0 resolving `arrow-schema` 59.3.0).

**Recommended §3.1 amendment (half applied):** pin every crate in both families with `=`, or commit a `Cargo.lock` and enforce it in CI. Revision 2 adopted the `=` pins; it did **not** adopt the lockfile. `=` pins bind direct dependencies only — nothing stops a transitive path from introducing a second version, and only a committed lockfile plus a CI assertion catches that. This extraction does **both**, and its lockfile is committed under `evidence/rust/`.

### 1.3 Features (`datafusion` 55.1.0)

Default: `nested_expressions`, `compression` (xz2/bzip2/flate2/zstd), `crypto_expressions`, `datetime_expressions`, `encoding_expressions`, `parquet`, `sql`, `regex_expressions`, `unicode_expressions`, `unparser`, `recursive_protection`. Optional: `avro`, `backtrace`, `parquet_encryption`, `serde` `[docs.rs:datafusion@55.1.0]`. Dispositions in §10.

### 1.4 Extraction receipt

```text
toolchain       rustc 1.100.0-nightly (809936eac 2026-09-12)
cargo           1.100.0-nightly (7941be6fb 2026-09-11)
flags           -Z unstable-options --output-format json
format_version  61
run             2026-09-13T16:15:24-04:00 .. 16:16:22-04:00
targets         57 requested, 57 OK, 0 FAIL, 60 JSON documents
DataFusion      37 crates, every crate_version 55.1.0
                datafusion, _catalog, _catalog_listing, _common, _common_runtime,
                _datasource, _datasource_arrow, _datasource_csv, _datasource_json,
                _datasource_parquet, _doc, _execution, _expr, _expr_common, _ffi,
                _functions, _functions_aggregate, _functions_aggregate_common,
                _functions_nested, _functions_table, _functions_window,
                _functions_window_common, _macros, _optimizer, _physical_expr,
                _physical_expr_adapter, _physical_expr_common, _physical_optimizer,
                _physical_plan, _proto, _proto_common, _proto_models, _pruning,
                _session, _spark, _sql, _substrait
Arrow           20 crates, every crate_version 59.3.0   (companion map)
lockfile        361 packages; one version per family, asserted
```

**Newly covered relative to the previous edition's 20 crates**, and each for a reason: `datafusion-proto` / `-proto-common` / `-proto-models` (revision 2's **F9** depends on them and they were absent), `datafusion-ffi`, `datafusion-substrait`, the whole `-datasource-*` family, `-physical-expr-adapter`, `-functions-{nested,table,window}`, `-catalog-listing`, `-spark`, `-common-runtime`, `-macros`, `-doc`.

The normalized form is written to `build/facts/df551/` — **76,771 declarations, 169 public traits, 48,968 impl relations** — alongside, and never overwriting, the older `df55-default` profile at 55.0.0. Trait coverage more than doubled: 169 against the previous 148, from nearly twice the crates.

Committed under **`docs/capability-maps/evidence/rust/`**: the extraction manifest, **the resolved `Cargo.lock`**, the generation script and log, the five probe programs, and their captured output (`probe_output.txt`).

### 1.5 §3.1's "55 forms" — all confirmed at 55.1.0

§3.1 states that `scan_with_args`/`ScanArgs`, `PhysicalPlanningContext`, `EnsureRequirements`, `is_strict` and `convert_to_state` "are the 55 forms to code against". Every one resolves:

| Identifier | Crate | Kind |
|---|---|---|
| `scan_with_args`, `ScanArgs` | `datafusion-session` | trait method, struct |
| `PhysicalPlanningContext` | `datafusion-expr` | struct |
| `EnsureRequirements` | `datafusion-physical-optimizer` | struct |
| `is_strict` | `datafusion-expr` (`ScalarUDFImpl`) | trait method |
| `convert_to_state` | `datafusion-expr-common`, `datafusion-functions-aggregate` | function |

`[rustdoc:*@55.1.0]`. §3.1's further note that "planner contracts moved toward `datafusion-session`" is also **confirmed**: `TableProvider`, `CatalogProvider`, `CatalogProviderList`, `SchemaProvider`, `Session` and `ScanArgs` all live there.

### 1.6 Probe receipt — new in this edition

The previous edition made no measurements. These six programs test assertions the blueprint makes, rather than trusting the API shape.

| Probe | Settles | Result in |
|---|---|---|
| **A** | what filter shapes actually reach a provider on a `FixedSizeBinary(16)` key column | §3 (D2) |
| **B** | whether a `UserDefinedLogicalNode` survives the optimizer carrying a `rule_id` | §4 (D3) |
| **C** | whether `datafusion-proto` encoding is usable and stable enough to be a plan fingerprint | §10 (D9) |
| **D** | how null, NaN and `-0.0` behave through sort, GROUP BY and equi-join | §4, §7 |
| **X1** | whether a `pse.*` extension type can be registered and resolved by the engine | §11 (D10) |
| **X2/X3** | which `EXPLAIN` formats exist, and how an invalid config value is reported | §5, §9 |
| **H** | whether a bounded `MemoryPool` yields a typed failure or a process death | §8 (D7) |
---

## 2. Catalog, providers, and `scan_with_args` (D1)

**Role:** §5.4 — "the catalog is a DataFusion `CatalogProviderList` whose catalogs are snapshots …, whose schemas are the seven namespaces, and whose tables are relations"; the provider contract that follows it.

**Where the contracts live.** In 55.1.0 `TableProvider`, `CatalogProvider`, `CatalogProviderList`, `SchemaProvider`, `Session` and `ScanArgs` are all defined in **`datafusion-session`** `[rustdoc:datafusion-session@55.1.0]`. §3.1's note — "planner contracts moved toward `datafusion-session`" — is **confirmed**, and it means `pse-catalog` depends on `datafusion-session`, not on the umbrella crate, which is a smaller and more stable dependency edge.

### Capability inventory

| Trait | Methods | Provenance |
|---|---|---|
| `CatalogProviderList` | `register_catalog(String, Arc<dyn CatalogProvider>)`, `catalog_names() -> Vec<String>`, `catalog(&str) -> Option<Arc<dyn CatalogProvider>>` | `[rustdoc:datafusion-session@55.1.0]` |
| `CatalogProvider` | `schema_names()`, `schema(&str)`, `register_schema`, `deregister_schema(name, cascade)` | `[rustdoc:datafusion-session@55.1.0]` |
| `SchemaProvider` | `owner_name()`, `table_names()`, **`table(&str)` — async**, **`table_type(&str)` — async**, `register_table`, `deregister_table`, `table_exist(&str)` | `[rustdoc:datafusion-session@55.1.0]` |
| `TableProvider` — read path | `schema() -> SchemaRef`, `constraints() -> Option<&Constraints>`, `table_type() -> TableType`, `get_table_definition()`, `get_logical_plan()`, `get_column_default(&str)`, `scan(state, projection, filters, limit)` async, **`scan_with_args(state, ScanArgs)`** async, `supports_filters_pushdown(&[&Expr])`, `statistics() -> Option<Statistics>` | `[rustdoc:datafusion-session@55.1.0]` |
| `TableProvider` — **mutation path** | `insert_into(state, input, InsertOp)`, `delete_from(state, filters)`, `update(state, assignments, filters)`, `truncate(state)`, `merge_into(state, source, merge_schema, on, clauses: Vec<MergeIntoClause>)` — all async with default implementations | `[rustdoc:datafusion-session@55.1.0]` |
| `ScanArgs` | `with_projection(Option<&[usize]>)`, `with_filters(Option<&[Expr]>)`, `with_limit(Option<usize>)`, **`with_statistics_requests(&[StatisticsRequest])`**, and the matching getters | `[rustdoc:datafusion-session@55.1.0]` |
| `TableType` | `Base \| View \| Temporary` | `[rustdoc:datafusion-expr@55.1.0]` |
| Async catalog adapters | `AsyncCatalogProvider`, `AsyncCatalogProviderList`, `AsyncSchemaProvider` | `[rustdoc:datafusion-catalog@55.1.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §5.4 catalogs are snapshots, schemas are the seven namespaces, tables are relations | `CatalogProviderList` → `CatalogProvider` → `SchemaProvider` → `TableProvider` | **confirmed** — the three-level shape is exactly what the trait stack gives |
| §5.4 "mutable aliases such as `head` resolved to a snapshot at session creation" | `catalog(name)` is a plain lookup; resolution happens in platform code before registration | **confirmed** — and correctly placed: resolving at session creation means `catalog()` stays pure, which §5.4's "a session pins a snapshot" requires |
| §5.4 "`scan_with_args` is implemented; `scan` delegates" | both exist as provided methods | **confirmed**, and the direction is right: `ScanArgs` is the extensible carrier, `scan` the legacy narrow form |
| §5.4 "`schema()` is the generated schema; cheap and stable" | `TableProvider::schema()` is synchronous and infallible | **confirmed** |
| §5.4 "Constraints: primary key only" | `constraints() -> Option<&Constraints>` | **confirmed** — see D2 |
| §5.4 statistics from the artifact footer, "answered only from cached metadata" | `statistics() -> Option<Statistics>` plus `ScanArgs::statistics_requests()` | **refined** — see D2 |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Leave all five mutation methods unimplemented, and prove it** | **G1 (authority)**, DM-02, DM-13, DM-20 | **adopt, as a governance test.** `TableProvider` in 55.1.0 carries `insert_into`, `delete_from`, `update`, `truncate` and `merge_into` as *defaulted* methods. The blueprint's artifacts are immutable and content-addressed (§20.1) and authored facts have exactly one authority (D1, D13). A provider that implemented any of these would create a second, mutable authority over the same facts, reachable through SQL — the precise condition G1 rejects. The defaults make this safe *by omission*, which is fragile: a future contributor "completing the trait" would breach the architecture with no test failing. Add `tests/governance/tests/no_mutating_providers.rs`. **The blueprint does not mention this surface at all.** |
| `SchemaProvider::table()` is **async** | DM-28 (declare ambient inputs), DM-26 | **note and constrain.** An async `table()` may do I/O per lookup. §5.4 wants a pinned snapshot with cheap, stable schemas; the provider should resolve the manifest once at session creation and serve `table()` from memory, so the async signature is satisfied without actually performing I/O. State it, or planning latency becomes a function of catalog size. |
| `get_column_default(&str) -> Option<&Expr>` | DM-16 (structure and policy as declarations) | **reject.** Column defaults are an authoring concept the blueprint deliberately does not have — values come from `authored` relations and case overlays (§6.10), not from provider-supplied defaults. Named so it stays unimplemented. |
| `get_logical_plan()` / `TableType::View` | DM-18 (preserve high-level structure until expansion) | **evaluate.** A relation *derived* by a rule (§14.2) could in principle be exposed as a view whose `LogicalPlan` DataFusion inlines and optimises with its consumer, rather than as a materialised artifact. That is genuinely aligned with DM-18. But it conflicts with §5.3 content-addressing — a view has no content hash — so it is only viable for `runtime` analytics (§19.2), never for `compiled` artifacts. Worth a bounded look; **not** a change to the artifact model. |
| `AsyncSchemaProvider` / `AsyncCatalogProviderList` | DM-26 | **reject for now** — these exist for catalogs that must resolve remotely during planning. The snapshot catalog resolves once, up front. Recording the rejection prevents a future "we should be async" drift. |

### Gaps and risks

- `CatalogProviderList::register_catalog` and `SchemaProvider::register_table` are **mutating** methods on the catalog itself. Session setup uses them; nothing after setup should. Since a snapshot session is supposed to be immutable for its lifetime (§5.4), the platform's implementations should reject registration after a `seal()` point rather than relying on nobody calling them (**DM-29**, G5).
- `MergeIntoClause` and `InsertOp` in the trait signature are evidence that DataFusion is growing a full DML surface release over release. This is a **standing upgrade hazard for this design**: each upgrade may add more defaulted mutation methods. The governance test above should assert the *set* of implemented methods, not just the absence of today's five.

---

## 3. Pushdown truthfulness, constraints, and statistics (D2)

The cluster where §5.4 makes its sharpest claim — *"Truthfulness over ambition: a wrongly advertised exact filter is a correctness bug"* — and where **G7 (truthful capability claims)** and **G6 (transformation and reuse)** are decided.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Pushdown verdict | `TableProviderFilterPushDown` = **`Unsupported` \| `Inexact` \| `Exact`**, returned per filter from `supports_filters_pushdown(&[&Expr]) -> Result<Vec<_>>` | `[rustdoc:datafusion-expr@55.1.0]` |
| Constraint kinds | `Constraint` = **`PrimaryKey` \| `Unique`**; `Constraints::new_unverified(Vec<Constraint>)`, `extend`, `project(&[usize]) -> Option<Self>` | `[rustdoc:datafusion-common@55.1.0]` |
| Statistics container | `Statistics` with `new_unknown(&Schema)`, `with_num_rows(Precision<usize>)`, `with_total_byte_size(Precision<usize>)`, `add_column_statistics(ColumnStatistics)`, `calculate_total_byte_size(&Schema)` | `[rustdoc:datafusion-common@55.1.0]` |
| Per-column statistics | `ColumnStatistics { null_count, max_value, min_value, sum_value, distinct_count, byte_size }` | `[rustdoc:datafusion-common@55.1.0]` |
| **Precision wrapper** | `Precision<T>` = **`Exact` \| `Inexact` \| `Absent`** | `[rustdoc:datafusion-common@55.1.0]` |
| **Demand-driven statistics** | `StatisticsRequest` = `Min \| Max \| NullCount \| DistinctCount \| Sum \| ByteSize \| RowCount \| TotalByteSize`, delivered through `ScanArgs::with_statistics_requests` | `[rustdoc:datafusion-expr-common@55.1.0]`, `[rustdoc:datafusion-session@55.1.0]` |
| Pruning | the `datafusion-pruning` crate (186 indexed items) | `[rustdoc:datafusion-pruning@55.1.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §5.4 "filters on key columns and enum columns are handled `Exact`, everything else `Inexact` or unsupported" | the three-variant enum, one verdict per filter | **confirmed exactly** — DataFusion's model is per-filter, so the provider can be precise rather than blanket |
| §5.4 the provider *declares* what it can do rather than being asked to guess | `supports_filters_pushdown`, `constraints()`, `statistics()`, `table_type()` | **confirmed** — this is **DM-19** ("select behavior through declared capabilities and explicit bindings") implemented by the engine: every optimisation DataFusion performs over a provider is gated on a capability the provider declared, never inferred. The design's job is to make those declarations true. |
| §5.4 "a wrongly advertised exact filter is a correctness bug" | — | **confirmed, and it is worth naming the mechanism.** `Exact` tells the optimizer it may **delete** the filter from the plan. If the scan then returns a row the filter would have rejected, that row reaches the consumer with no second check. There is no runtime verification anywhere in DataFusion — the claim is taken on trust. This is the clearest G7 surface in the engine. |
| §5.4 "Constraints: primary key only" | `Constraint::PrimaryKey` | **confirmed**, and note the constructor is named **`new_unverified`** `[rustdoc:datafusion-common@55.1.0]` — DataFusion itself flags that it does not check. §5.4's "declared only where the provider can guarantee them" is the right discipline and the API name is a standing reminder. |
| §5.4 "Statistics: row counts and min/max for key columns from the artifact footer; answered only from cached metadata" | `Statistics` + `ColumnStatistics` + `Precision` | **refined** — see below |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Use `Precision::{Exact, Inexact, Absent}` deliberately, per statistic** | **DM-59** (make design claims falsifiable and label uncertainty), **DM-40**, G7 | **adopt, and write it into §5.4.** The blueprint says statistics come "from the artifact footer" but never says whether they are exact. `Precision` forces the answer, per value: a row count read from a manifest is `Exact`; a min/max over a *filtered* artifact is `Inexact`; anything not stored is `Absent`. Returning `Inexact` where the truth is unknown is free and safe; returning `Exact` wrongly lets the optimizer eliminate work on a false premise. DataFusion built the uncertainty label into the type — the design should use it rather than default everything to one setting. |
| **Answer `StatisticsRequest`s selectively** | DM-26 (separate preparation from repeated execution), DM-58 | **adopt.** `ScanArgs::statistics_requests()` tells the provider exactly which statistics the planner will use. A provider can answer those from cached metadata and return `Absent` for the rest — which is precisely §5.4's "answered only from cached metadata", now expressible as a contract instead of a convention. `Sum` and `DistinctCount` in particular should be `Absent` for snapshot artifacts unless the manifest genuinely carries them. **Unmentioned in the blueprint.** |
| `Constraints::project(&[usize])` | DM-22, DM-24 | **adopt** — when a scan is projected, the primary-key constraint must be projected with it or dropped. Doing this by hand is an easy place to assert a key over a column set that no longer contains it. |
| Declare `Constraint::Unique` where a relation genuinely has one | DM-09 (model relationships and valid domains explicitly), DM-10 | **evaluate.** §5.4 restricts to primary keys "until proven", which is the right default. But §4.2 generates a `unique` invariant kind and P2 *validates* it — so for relations whose unique invariant is enforced at write time, the guarantee is real and declaring it would let the optimizer eliminate redundant aggregations over `runtime` analytics (§19.2). The gate is G7: declare it only where a validator actually runs. |
| Pushdown of `pse.*` extension-typed filters | **G7**, DM-43 | **measured — the spike is done, and the answer is favourable.** See below. |

### Measured (PROBE A) — the §5.4 pushdown spike

The previous edition listed this as must-settle item 1 and blueprint §26 **F17** still says "the pushdown spike precedes any `Exact` claim". It has now been run: a recording `TableProvider` over a `FixedSizeBinary(16)` key column, capturing every expression offered to `supports_filters_pushdown` `[probe]`.

```text
PROBE A  filter shapes offered to a provider

q: WHERE id = arrow_cast(X'…03','FixedSizeBinary(16)')
   offered: ["rel.id = FixedSizeBinary(16, "0,0,…,3")"]        reached scan(): 1

q: WHERE id IN (arrow_cast(X'…01',…), arrow_cast(X'…02',…))
   offered: ["rel.id = FixedSizeBinary(16, "…,1") OR rel.id = FixedSizeBinary(16, "…,2")"]
                                                               reached scan(): 1
q: WHERE kind = 'a'
   offered: ["rel.kind = Utf8("a")"]                            reached scan(): 1

q: WHERE kind = 'a' AND v > 2.0
   offered: ["rel.kind = Utf8("a")", "rel.v > Float64(2)"]      reached scan(): 2

q: WHERE id IS NOT NULL
   offered: []                                                  reached scan(): 0
```

Five facts, and they change §5.4's status from unverifiable to specifiable:

1. **Equality on a `FixedSizeBinary(16)` key arrives as a plain binary expression with a literal on one side** — `rel.id = FixedSizeBinary(16, …)`. That is a shape a provider can match structurally. **`Exact` is achievable on key equality**, which is what §5.4 promises. The blocking item is resolved in the design's favour.
2. **`IN (a, b)` does not arrive as an `InList`.** The optimizer has already rewritten it to a chain of `OR`s. A provider written to match `Expr::InList` — the obvious implementation — would **never fire**, silently falling back to a full scan with no error. This is precisely the kind of thing that cannot be learned from the type signature.
3. **Conjunctions are split.** `A AND B` arrives as two independent filters, so a provider can accept one `Exact` and decline the other. §5.4's per-column policy is expressible exactly as written.
4. **`IS NOT NULL` on a non-nullable column is eliminated before the provider sees it** — zero filters offered. The optimizer used the schema's nullability declaration. This is a small, pleasing confirmation that `Field` nullability is load-bearing, and a reminder that it is the *one* constraint DataFusion actually enforces (see Gaps).
5. **`supports_filters_pushdown` is called more than once per plan** — each filter appeared twice. The method must therefore be **pure and idempotent**; a provider that logged, counted, or mutated state there would double-count. Not documented in the signature.

### Gaps and risks

- **Nothing verifies an `Exact` claim.** PROBE A shows the shapes are matchable; it does not make a match *correct*. Recommend a test-only wrapper provider that re-applies every filter it advertised as `Exact` to the batches it returns and fails loudly on a survivor. Cheap, runs in CI over the golden snapshots (§24), and converts a trust-based claim into a tested one (**DM-53, DM-54**).
- **`supports_filters_pushdown` must be pure** (PROBE A, fact 5). Worth a comment in the generated provider, since the obvious place to put instrumentation is exactly there.
- **DataFusion enforces no relational constraint except `Field` nullability — and does not check that on ingest.** Upstream is explicit: primary, unique and foreign keys and check constraints "are provided for informational purposes", are "not validated or used during query planning", and returning nulls for a non-nullable column "will result in runtime errors during execution" `[c7:/apache/datafusion]`. §5.4 declares "Constraints: primary key only", which is sound **as a declaration** — but it must not be read as enforcement. The platform's P2 invariant validation is the only thing that makes the claim true (**G7**, DM-43).
- `Statistics::new_unknown(&Schema)` is the honest default and should be the starting point for every provider, with statistics added only where the manifest supplies them — rather than constructing `Statistics` optimistically and hoping.
- `ColumnStatistics` has no notion of *which* rows a statistic covers. For a snapshot artifact this is fine (it covers the whole artifact); it becomes wrong the moment a provider serves a subset. Another reason §5.4's "the provider never prunes rows of a coupled mathematical problem" matters beyond its stated rationale.

---

## 4. Logical plans, expressions, and the rule compiler (D3)

**Role:** §14.2 — "a typed `RulePlanSpec` (a small algebra: scan, filter, project, equi-join, anti-join for stratified negation, union, distinct) compiled to a DataFusion `LogicalPlan` through `LogicalPlanBuilder`, optimized, and executed against the snapshot session"; D10's first role; D6's insistence that the math IR is *not* `Expr`.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Plan nodes (25) | `Projection`, `Filter`, `Window`, `Aggregate`, `Sort`, `Join`, `Repartition`, `Union`, `TableScan`, `EmptyRelation`, `Subquery`, `SubqueryAlias`, `Limit`, `Statement`, `Values`, `Explain`, `Analyze`, **`Extension`**, `Distinct`, `Dml`, `Ddl`, `Copy`, `DescribeTable`, `Unnest`, **`RecursiveQuery`** | `[rustdoc:datafusion-expr@55.1.0]` |
| Expressions (37) | `Column`, `Literal`, `BinaryExpr`, `Not`, `Negative`, `Between`, `Case`, `Cast`, `TryCast`, `ScalarFunction`, `AggregateFunction`, `WindowFunction`, `InList`, `Exists`, `InSubquery`, `SetComparison`, `ScalarSubquery`, `GroupingSet`, `Placeholder`, `Unnest`, `Alias`, `Wildcard`, `ScalarVariable`, `OuterReferenceColumn`, `Like`, `SimilarTo`, **`IsNull`/`IsNotNull`/`IsTrue`/`IsFalse`/`IsUnknown`/`IsNotTrue`/`IsNotFalse`/`IsNotUnknown`**, `HigherOrderFunction`, `Lambda`, `LambdaVariable` | `[rustdoc:datafusion-expr@55.1.0]` |
| Builder | `LogicalPlanBuilder::{scan, scan_with_filters, scan_with_filters_fetch, values, values_with_schema, project, project_with_validation, filter, limit, limit_by_expr, sort, sort_by, sort_with_limit, union, union_by_name, to_recursive_query(name, recursive_term, is_distinct), with_options}` and the join/distinct/aggregate family | `[rustdoc:datafusion-expr@55.1.0]` |
| **Extension nodes** | `UserDefinedLogicalNode` — `name`, `inputs`, `schema`, `expressions`, `with_exprs_and_inputs`, **`check_invariants(InvariantLevel)`**, **`prevent_predicate_push_down_columns()`**, `necessary_children_exprs`, `supports_limit_pushdown`, `fmt_for_explain`, `dyn_hash`/`dyn_eq`/`dyn_ord` | `[rustdoc:datafusion-expr@55.1.0]` |
| Invariant levels | `InvariantLevel` = `Always \| Executable` | `[rustdoc:datafusion-expr@55.1.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §14.2 `RulePlanSpec` algebra → `LogicalPlan` via `LogicalPlanBuilder` | every primitive in the algebra has a builder method | **confirmed** — scan, filter, project, join (incl. anti-join via `JoinType`), union, distinct all present |
| §14.2 rule 1 "DataFusion's recursive CTE support may be used for closures whose provenance can be reconstructed from the result" | `LogicalPlan::RecursiveQuery` + `LogicalPlanBuilder::to_recursive_query(name, recursive_term, is_distinct)`; `CteWorkTable` provides the working table | **confirmed** — and the `is_distinct` flag is the fixed-point dedup §14.2 describes |
| §14.2 rule 5 "the plan fingerprint is part of the pass record" | no plan-hash API on `LogicalPlan` itself; `UserDefinedLogicalNode::dyn_hash` exists for extension nodes | **superseded.** Revision 2 chose `datafusion-proto`, which the previous edition did not consider. Measured and adjudicated in **D9** — the mechanism works but is not reproducible as specified. |
| D6 "the math IR is richer than DataFusion `Expr`" | 37 `Expr` variants, none of which carry physical type, unit, or domain | **confirmed** — `Expr` has no place to put a quantity type, so D6's separate IR is necessary, not merely preferred |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **`LogicalPlan::Extension` + `UserDefinedLogicalNode` for rule bodies** | **DM-18** (preserve high-level structure until expansion is required), **DM-21** (explicit IRs and progressive lowering), DM-49 | **upgraded from evaluate to adopt-candidate — the prerequisite is now measured.** §14.2 currently lowers a `RulePlanSpec` straight to relational primitives, so by the time a plan is optimised or explained, the *rule* has disappeared into a pile of joins. An extension node keeps `rule_id` visible through planning and into `EXPLAIN`, serving §14.2 rule 4 ("every head row carries a derivation") from the plan rather than only from the result. The previous edition could not recommend it because whether such a node survives the optimizer was unknown. **PROBE B settles it — it does.** The remaining cost is the 14-method contract and an `ExtensionPlanner` for physical execution; both are known quantities. |
| **`UserDefinedLogicalNode::check_invariants(InvariantLevel)`** | **DM-07**, **G3**, DM-53 | **adopt if extension nodes are adopted.** DataFusion will call this to verify a node's invariants — `Always` (structural) and `Executable` (before execution). That is a free, engine-driven hook for the kind of checking §4.2's invariants describe, at plan level rather than data level. |
| **`prevent_predicate_push_down_columns()`** | **G6** (a rewrite must not change required behavior), DM-24 | **adopt if extension nodes are adopted.** This is how a node tells the optimizer *not* to push a predicate below it. §5.4 already worries about the provider's pushdown truthfulness; this is the same concern one level up, and the only declarative way to stop a semantically invalid pushdown. |
| `Expr::IsUnknown` / `IsNotUnknown` | **DM-08** | **evaluate.** §14.2 rule 3 defines four-valued predicates (`true \| false \| unknown \| conflict`) and writes `unknown`/`conflict` rows with a `status` column. DataFusion's `IsUnknown` covers the SQL three-valued case natively. `conflict` has no engine equivalent and stays platform-level — so the mapping is partial, which is worth stating so nobody assumes the engine models all four. |
| `project_with_validation` | DM-22, G3 | **adopt** over plain `project` where the rule compiler builds projections from a spec — it validates the projection against the input schema rather than trusting the caller. |
| `Expr::Placeholder` | DM-26 (separate preparation from repeated execution) | **evaluate** for §19.3 sweeps: one prepared plan with placeholders, executed per sample, instead of rebuilding a plan per case. Only worthwhile if sweeps prove plan-construction-bound. |
| `HigherOrderFunction` / `Lambda` / `LambdaVariable` | — | **reject.** New in this line and appealing for expressing per-element logic, but D6 is explicit that the math IR is not `Expr`; pushing model semantics into lambdas inside plans would blur exactly the boundary D6 draws. |

### Measured (PROBE B) — does an extension node survive the optimizer?

A `UserDefinedLogicalNodeCore` carrying a `rule_id`, wrapped around an ordinary filter-and-project plan, put through `SessionState::optimize` `[probe]`:

```text
PROBE B  UserDefinedLogicalNode survival

  before optimize:            after optimize:
    PseRule: rule_id=rule-42    PseRule: rule_id=rule-42
      Projection: rel.v            Filter: rel.v > Float64(1)
        Filter: rel.v > Float64(1)   TableScan: rel projection=[v]
          TableScan: rel

  node survived        : true
  rule_id still visible: true
  physical planning    : FAILS without an ExtensionPlanner
                         -> "No installed planner was able to convert the
                             custom node to an execution plan: RuleNode"
```

Three facts. The node **survives intact** and `rule_id` is still readable in the rendered plan, so the DM-18 opportunity is real rather than hypothetical. The optimizer **still optimises through it** — the projection was pushed down into the `TableScan` *beneath* the extension node, so wrapping a rule body does not cost the rewrites. And physical execution requires registering a `datafusion_session::ExtensionPlanner`; the failure is a clean typed planning error, not a silent fallback.

### Measured (PROBE D) — float and null semantics through the engine

§14.2 rule 3 and §5.3 both depend on how the engine treats null, NaN and `-0.0`. The Arrow map measured the *kernel* behaviour; this measures the *engine* behaviour over the same five values `[0.0, -0.0, NaN, -1.0, null]` `[probe]`:

```text
PROBE D
  ORDER BY x            : ["-1", "-0", "0", "NaN", "null"]
  GROUP BY x (distinct) : ["-1", "0", "NaN", "null"]     <- 4 groups from 5 rows
  self equi-join rows   : 6
```

**Sorting and grouping disagree about whether `-0.0` and `+0.0` are the same value.** `ORDER BY` reproduces Arrow's IEEE 754 `totalOrder` exactly — `-0.0` strictly before `+0.0`, NaN after all finite values — which confirms the Arrow map's PROBE 3 survives into the engine. But `GROUP BY` returns **four** groups from five rows: `-0.0` and `+0.0` are merged, and the surviving representative is `0`, not `-0`. The self equi-join returns six rows, which decomposes as four from the `{0.0, -0.0}` pair matching each other, one from `-1.0`, and **one from NaN matching itself** — so the hash join also treats NaN as equal to NaN, which is not IEEE comparison semantics.

None of this is a DataFusion defect; grouping and joining use value-equality semantics by design. It matters because §5.3 states that "`-0.0` is preserved (it is distinguishable and can matter to solvers)" and §26 repeats it. That is true of Arrow ordering and of the content hash — and **false of any aggregation or join keyed on a `Float64` column**. A `GROUP BY` over a float key silently merges the two, and which sign survives is an engine choice. The design should either declare that float columns are never grouping or join keys, or state the collapse explicitly as a selected loss (**DM-42**, **G2**, DM-08).

### Gaps and risks

- **Plan fingerprinting is not a DataFusion capability**, and revision 2's choice of `datafusion-proto` does not change that — it relocates the problem rather than solving it. `LogicalPlan` offers no stable hash; proto offers an encoding whose stability depends on inputs the platform controls. See **D9**, which is where this risk now lives (DM-48, DM-51).
- **A `Float64` column used as a grouping or join key loses the `-0.0` distinction** (PROBE D). Not a defect, but it contradicts a claim §5.3 and §26 both make without qualification.
- `LogicalPlan` carries `Dml`, `Ddl`, `Copy` and `Statement` variants. Nothing in the blueprint should construct them, for the same authority reason as D1's mutation methods — the rule compiler builds read-only plans only.

---

## 5. Optimizer, analyzer, and plan explainability (D4)

**Role:** §14.2 — plans are "optimized" before execution, and rule 5 requires determinism: "the DataFusion session is pinned (config snapshot, function registry hash, catalog snapshot) and the plan fingerprint is part of the pass record."

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| `AnalyzerRule` | `analyze(LogicalPlan, &ConfigOptions) -> Result<LogicalPlan>`, `name()` — **3 built-in impls** | `[rustdoc:datafusion-optimizer@55.1.0]` |
| `OptimizerRule` | `name()`, `apply_order() -> Option<ApplyOrder>`, `supports_rewrite() -> bool`, `rewrite(LogicalPlan, &dyn OptimizerConfig) -> Result<Transformed<LogicalPlan>>` — **25 built-in impls** | `[rustdoc:datafusion-optimizer@55.1.0]` |
| Traversal order | `ApplyOrder` = `TopDown \| BottomUp` | `[rustdoc:datafusion-optimizer@55.1.0]` |
| Rewrite signalling | `Transformed<T>` — carries whether a rewrite fired, so the driver knows when a fixed point is reached | `[rustdoc:datafusion-optimizer@55.1.0]` |
| Explainability | `LogicalPlan::{Explain, Analyze}`; `UserDefinedLogicalNode::fmt_for_explain` | `[rustdoc:datafusion-expr@55.1.0]` |
| **Explicit rule installation** | `Optimizer::with_rules(Vec<Arc<dyn OptimizerRule + Send + Sync>>)`, `Analyzer::with_rules(Vec<Arc<dyn AnalyzerRule + Send + Sync>>)`; `Optimizer::new()` / `Analyzer::new()` take the defaults | `[rustdoc:datafusion-optimizer@55.1.0]` |
| Rule-firing observation | `Optimizer::optimize(plan, config, observer)` and `Analyzer::execute_and_check(plan, config, observer)` — both take an observer closure called per rule | `[rustdoc:datafusion-optimizer@55.1.0]` |
| Function rewrites | `Analyzer::{add_function_rewrite, function_rewrites}` over `FunctionRewrite` | `[rustdoc:datafusion-optimizer@55.1.0]` |
| Physical side | `datafusion-physical-optimizer` (840 items) including **`EnsureRequirements`** | `[rustdoc:datafusion-physical-optimizer@55.1.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §14.2 "compiled … optimized, and executed" | the analyzer → optimizer → physical planner pipeline | **confirmed** |
| §3.1 "`EnsureRequirements` … are the 55 forms to code against" | `EnsureRequirements` present in `datafusion-physical-optimizer` | **confirmed at 55.1.0** |
| §14.2 rule 5 "installed explicitly with `Optimizer::with_rules` rather than taken from the engine defaults" | `Optimizer::with_rules(Vec<Arc<dyn OptimizerRule + Send + Sync>>)`; `Analyzer::with_rules` is its analyzer-side twin | **confirmed, and the blueprint names only half of it.** `Optimizer::with_rules` pins the 25 optimizer rules; the 3 `AnalyzerRule`s are installed separately and are equally capable of changing a plan. §14.2 should name both. There is also a third surface — `Analyzer::add_function_rewrite` — which mutates an analyzer after construction and would escape any list captured at construction time; platform code should not use it. |
| §14.2 rule 5 pinned session (config snapshot, function registry hash, catalog snapshot) | `ConfigOptions` is threaded into every `AnalyzerRule::analyze` and `OptimizerRule::rewrite` | **confirmed as necessary.** Because config reaches every rule, an unpinned config means an unpinned *plan* — and therefore an unpinned result. §14.2 rule 5's pinning is not belt-and-braces; it is what makes optimisation deterministic. |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Record the rule set and its versions in the pass record, not just the config** | **DM-31** (expose every dependency that can affect meaning or output), **G6**, DM-48 | **adopt.** §14.2 rule 5 pins config, function registry and catalog — but *not* the optimizer rule set. There are 25 built-in `OptimizerRule`s plus 3 `AnalyzerRule`s at 55.1.0, and that set changes between DataFusion releases. Two runs with identical inputs, identical config and different DataFusion patch versions can produce different plans and therefore different execution. Since the rules expose `name()`, capturing the ordered list of rule names alongside the DataFusion version is cheap and closes the gap. |
| **Disable or pin optimizer rules that can change numerical semantics** | **G6** ("a rewrite … changes required behavior without a valid contract"), DM-24, DM-40 | **evaluate, with a bias to caution.** Relational rewrites (projection pushdown, join reordering) are semantics-preserving over sets. Expression-level rewrites are where risk lives: constant folding and simplification over `Float64` can reassociate arithmetic and change the last bits of a result. §14.2's rule bodies are set-oriented, so exposure is limited — but §18.5's kernel UDFs *are* invoked inside plans, and `simplify` is a UDF method the design already plans to implement. The rule: any rewrite that touches floating-point arithmetic must be justified or disabled, and §5.3's hashes make a violation loudly visible. |
| `Transformed<T>` in the platform's own rewrites | DM-22 | **adopt** if the platform ever writes an `OptimizerRule`; it makes "did this rule fire" explicit rather than inferred. |
| **The `observer` closure on `Optimizer::optimize` / `Analyzer::execute_and_check`** | **DM-46** (preserve source-to-result lineage), DM-50, DM-49 | **adopt.** Both entry points take a per-rule observer. That turns "the rule set was X" — which revision 2's F9 already records — into "**these** rules actually fired, in this order, on this plan". The first is a declaration of what *could* have happened; the second is evidence of what did, and it is what makes a plan difference explainable at the level of meaning rather than by diffing two renderings. Effectively free: the seam exists and the platform already drives optimisation explicitly. **New.** |
| **`EXPLAIN` output captured into the pass record — and `pgjson`, not `indent`** | **DM-49**, **DM-27** (represent workflows as inspectable plans), DM-47 | **adopt, with a caveat and a correction.** Storing the rendered plan alongside a pass run is excellent for explainability. The caveat is D3's: the rendering is not a stable contract, so store it as *evidence for humans*, never as the fingerprint. The correction is that the previous edition treated `EXPLAIN` as a single text surface; it is four, and one of them is **machine-readable** — see the measurement below. |

### Measured (PROBE X2) — `EXPLAIN` is four surfaces, not one

```text
PROBE X2  EXPLAIN formats accepted by 55.1.0
   indent   -> OK    "TableScan: rel projection=[v]"
   tree     -> OK    box-drawing render of the physical plan
   pgjson   -> OK    structured JSON
   graphviz -> OK    "// Begin DataFusion GraphViz Plan"
   json     -> rejected: "Expected 'indent', 'tree', 'pgjson' or 'graphviz'"
```

`pgjson` turns the plan from a string a human reads into a **structure a pass can store, diff and query** — which is what DM-27 actually asks for. §23.1's observability and §19's analytics both become considerably better if a pass record carries a parsed plan rather than a blob of text. And `datafusion.explain.{analyze_level, analyze_categories, show_statistics, show_schema}` select what it contains, so the verbosity is declared policy rather than a fixed cost (§12 E3).

### Gaps and risks

- **The optimizer is an ambient dependency of every derived relation.** This is the cluster's real finding. §2 D14 says "incrementality follows declared dependencies" and §14.4 keys memoization on input content hashes plus pass version. The DataFusion version and its rule set are inputs that affect output and are *not* in that key. A DataFusion upgrade should therefore invalidate compiled artifacts — which argues for including the engine version in the pass key (**DM-31, DM-32**).
- No physical-optimizer trait was exposed in the extraction's `datafusion-physical-optimizer` index (the struct `EnsureRequirements` is present, but the rule trait did not surface), so the physical rule contract should be read directly before any custom physical rule is written. **Open item.**

---

## 6. Physical planning, streams, and partitioning (D5)

**Role:** §14.3 session and parallelism; §18.8 thread budget; D10's boundary — DataFusion "does not run inside Newton iterations, and a solve is never a scalar function call".

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Execution contract | `ExecutionPlan` trait; `ExecutionPlanProperties`; `PlanProperties` | `[rustdoc:datafusion-physical-plan@55.1.0]` |
| Output shape declarations | `Partitioning`, **`EmissionType`**, **`Boundedness`** | `[rustdoc:datafusion-physical-plan@55.1.0]`, `[c7:/apache/datafusion]` |
| Streaming | `SendableRecordBatchStream` (type alias), `RecordBatchStream` trait | `[rustdoc:datafusion-execution@55.1.0]` |
| Planning context | **`PhysicalPlanningContext`** | `[rustdoc:datafusion-expr@55.1.0]` |
| Requirement enforcement | **`EnsureRequirements`** | `[rustdoc:datafusion-physical-optimizer@55.1.0]` |
| Aggregate state conversion | **`convert_to_state`** | `[rustdoc:datafusion-expr-common@55.1.0]`, `[rustdoc:datafusion-functions-aggregate@55.1.0]` |

**§3.1's "55 forms" claim is confirmed in full at 55.1.0**: `scan_with_args`, `ScanArgs`, `PhysicalPlanningContext`, `EnsureRequirements`, `is_strict` and `convert_to_state` all exist, in the crates named above.

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §14.3 "DataFusion partitions relational work internally" | `Partitioning` on `PlanProperties`; `EnsureRequirements` inserts repartitions to satisfy a plan's requirements | **confirmed** |
| §5.4 provider returns an `ExecutionPlan` from `scan_with_args` | `Arc<dyn ExecutionPlan>` | **confirmed** |
| §14.2 execution to `RecordBatch` | `SendableRecordBatchStream` | **confirmed** — execution is a stream, so a pass consuming a rule result must collect it, which is where §5.3's concatenation step belongs |
| D10 "does not run inside Newton iterations" | — | **confirmed as a sound boundary.** A DataFusion plan is async, streaming and partition-parallel, with a memory pool and spill machinery behind it (D7). Putting that inside a Newton iteration would make each residual evaluation a scheduled, allocating, potentially spilling operation. The blueprint's boundary is not conservatism; it is the only workable choice. |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Declare `EmissionType` and `Boundedness` honestly on any custom `ExecutionPlan`** | **G7**, DM-43, DM-30 | **adopt.** A snapshot-artifact scan is `Boundedness::Bounded` and (per the upstream example) `EmissionType::Incremental`. These are capability claims the engine plans against: declaring `Bounded` for something unbounded causes an operator to wait forever for an end that never comes. |
| **Partitioning as a declared property, not an accident** | DM-35 (concurrency respects dependencies and declared ordering), DM-40 | **adopt.** §5.3 requires canonical ordering, and a partitioned scan emits batches in nondeterministic order across partitions. That is fine — ordering is imposed later by the sort in §5.3 step 1 — but it must be *stated*, because a pass that reads a rule result and assumes arrival order is deterministic will be intermittently wrong. |
| Single-partition providers for `compiled` relations | DM-36, DM-58 | **evaluate.** §5.4 says the provider "never prunes rows of a coupled mathematical problem". A related discipline: relations consumed as a whole by a native pass gain nothing from partitioning and lose ordering determinism. Declaring `Partitioning::UnknownPartitioning(1)` for those is simpler and cheaper. Measure before generalising (DM-58). |
| `convert_to_state` | DM-25 (unify operation contracts) | **note only.** It belongs to the aggregate-UDAF path, which the blueprint does not currently use — §14.2's algebra has no aggregate. Listed because §3.1 names it; if UDAFs are never written, this is a pin-table entry with no consumer, and §3.1 could say so. |

### Gaps and risks

- **Thread budget crosses three owners.** §18.8 gives DataFusion the tokio runtime and rayon the batch kernels. DataFusion's partition parallelism is driven by `target_partitions` in `SessionConfig` (D7) — so the budget is set in *config*, not by the pool. Getting this wrong oversubscribes silently, which §18.8 says must be a configuration error rather than a runtime surprise. The configuration must therefore be validated at session construction.
- The `ExecutionPlan` trait's full method set was not enumerated here (the physical-plan crate indexes 6008 items); a custom physical operator is out of scope for the blueprint (D11 keeps numerics native), so only the provider's returned plan matters. **If a custom operator is ever written, re-derive this cluster.**

---

## 7. Scalar UDFs and the generated kernel adapters (D6)

**Role:** §18.5 — the `ScalarUDFImpl` row of the kernel-adapter table; §14.2 — generated wrappers registered in the session so rule bodies can evaluate kernels in batch; D10's third DataFusion role.

### Capability inventory — `ScalarUDFImpl` has 25 methods in 55.1.0

| Group | Methods | Provenance |
|---|---|---|
| Identity | `name()`, `aliases()`, `display_name(&[Expr])`, `schema_name(&[Expr])`, `documentation()` | `[rustdoc:datafusion-expr@55.1.0]` |
| Typing | `signature() -> &Signature`, `return_type(&[DataType])`, **`return_field_from_args(ReturnFieldArgs) -> Result<FieldRef>`**, `is_nullable(&[Expr], &dyn …)`, `coerce_types(&[DataType])` | `[rustdoc:datafusion-expr@55.1.0]` |
| Evaluation | `invoke_with_args(ScalarFunctionArgs) -> Result<ColumnarValue>`, **`is_strict()`** | `[rustdoc:datafusion-expr@55.1.0]` |
| Rewriting | `simplify(Vec<Expr>, &SimplifyContext) -> Result<ExprSimplifyResult>` (`Simplified \| Original`), **`preimage(&[Expr], &Expr, &SimplifyContext) -> Result<PreimageResult>`** (`None \| Range`) | `[rustdoc:datafusion-expr@55.1.0]` |
| Conditional evaluation | **`short_circuits()`**, **`conditional_arguments(&[Expr]) -> Option<(Vec<&Expr>, Vec<&Expr>)>`** | `[rustdoc:datafusion-expr@55.1.0]` |
| Range analysis | `evaluate_bounds(&[&Interval]) -> Result<Interval>`, `propagate_constraints(&Interval, &[&Interval]) -> Result<Option<Vec<Interval>>>` | `[rustdoc:datafusion-expr@55.1.0]` |
| Ordering | **`output_ordering(&[ExprProperties]) -> Result<SortProperties>`**, **`preserves_lex_ordering`**, **`strictly_order_preserving`** | `[rustdoc:datafusion-expr@55.1.0]` |
| Misc | `with_updated_config(&ConfigOptions)`, `struct_field_mapping`, `placement(&[ExpressionPlacement])` | `[rustdoc:datafusion-expr@55.1.0]` |

Argument carriers: `ReturnFieldArgs { arg_fields, scalar_arguments }` and `ScalarFunctionArgs { args, arg_fields, number_rows, return_field, config_options }` `[rustdoc:datafusion-expr@55.1.0]`.

### What the blueprint binds to

| §18.5 requirement | API | Status |
|---|---|---|
| `name`, `signature` (exact Arrow types, dictionary preservation off) | `name()`, `signature()`, and `coerce_types()` to refuse implicit coercion | **confirmed** |
| `return_field_from_args` "attaches `pse.semantic.*` metadata and the output quantity type" | `ReturnFieldArgs { arg_fields, … } -> Result<FieldRef>` | **confirmed, and better than the blueprint claims.** The argument carrier supplies the input **`Field`s**, not just `DataType`s — so the adapter can *read* each argument's `pse.semantic.quantity_type` from its field metadata and *derive* the output quantity type, rather than hard-coding it. That is unit inference (§8.3) enforced at the plan boundary, and it is only possible because the API passes fields. |
| `is_strict = true` when every input is required | `is_strict()` | **confirmed** |
| `volatility = Immutable` | via `Signature` | **confirmed** |
| §18.5 kernels selected through `KernelSpec` bindings rather than discovered | `signature()`, `is_strict()`, `volatility`, `coerce_types()` — the declared capability surface the planner binds against | **confirmed** — **DM-19** again, one level down: a UDF is chosen and optimised entirely from its declarations. A declaration that overstates the kernel (e.g. `Immutable` for something config-dependent) is a G7 failure, not merely a performance bug. |
| `simplify` for all-constant inputs | `simplify(...) -> ExprSimplifyResult::{Simplified, Original}` | **confirmed** |
| `evaluate_bounds` / `propagate_constraints` "derived from monotonicity and validity where declared" | both present | **confirmed** |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **`conditional_arguments()` + `short_circuits()`** | **G4 (hidden behavior)**, **DM-28**, DM-08 | **adopt — the most consequential unmentioned method in the engine for this design.** §7.2's IR has a `Conditional` operator and §18.2 step 6 defines domain guards: evaluating `log` of a non-positive value raises a recoverable evaluation error. If a kernel's arguments are evaluated *eagerly* inside an `if/then/else`, the guarded branch is computed anyway and the guard fires on a value the model never intended to evaluate — a spurious `solve.evaluation_error` with a misleading culprit. `conditional_arguments` is how a UDF tells DataFusion which arguments are conditional; `short_circuits` declares the behaviour. Generated adapters for conditional kernels **must** implement both. |
| **`preimage()`** | **DM-43** (negotiate capabilities), G6, G7 | **evaluate — potentially large.** Given `f(x) = literal`, `preimage` returns a `Range` on `x`. That is what lets a filter over a *computed* column be pushed down to a scan on the underlying column. For §19 analytics over `runtime` relations ("find states where enthalpy exceeds …"), this is the difference between a pushdown and a full scan. It is also a **correctness surface with the same character as `Exact` pushdown**: a wrong preimage silently drops rows. Adopt only for kernels with a proven monotone inverse, and test it adversarially. |
| **`output_ordering` / `preserves_lex_ordering` / `strictly_order_preserving`** | **DM-24** (preserve semantics across rewrites), DM-26, DM-40 | **adopt for monotone kernels.** §5.3 depends on canonical ordering and §15.3's block triangularisation depends on deterministic order. Declaring that a strictly increasing kernel preserves ordering lets DataFusion keep a sort through the call instead of re-sorting — and, more importantly, makes the *claim* explicit and testable rather than implicit. The blueprint already tracks monotonicity for `evaluate_bounds`; the same declaration feeds these three. |
| `is_nullable(&[Expr], schema)` | **DM-08**, G2 | **adopt.** §18.5 says the batch adapter produces "nulls plus a diagnostic column when requested" for validity violations. If the UDF declares itself non-nullable while the batch path can emit nulls, the plan's schema lies — a G2 failure. The declaration must be derived from the same `KernelSpec` strictness/validity data that drives the batch adapter, not written independently. |
| `with_updated_config(&ConfigOptions)` and `ScalarFunctionArgs.config_options` | **DM-28** (declare ambient inputs), **G4** | **reject, and assert it.** A kernel whose result depends on session config is nondeterministic with respect to everything §5.3 hashes. §14.2 rule 5 already pins the session config and includes a function-registry hash in the pass record — good — but the generated adapters should simply never read `config_options`, and a governance check should say so. |
| `coerce_types()` returning the identity | DM-42, G2 | **adopt explicitly.** §18.5 wants exact Arrow types. Implementing `coerce_types` as "accept exactly the declared types, else error" prevents DataFusion from silently widening an `Int64` argument into a `Float64` kernel and changing the numerics. Relying on the default is a silent-coercion risk. |

### Gaps and risks

- **The trait is large and growing** — 25 methods in 55.1.0, several (`preimage`, `conditional_arguments`, `placement`, `struct_field_mapping`) clearly newer than the blueprint's description. Because `KernelSpec` *generates* the adapters (§18.5), the cost of adopting a new method is one generator change rather than N hand edits — which is DM-52 working as intended, and a good argument for keeping the generation path even though the trait is a moving target.
- `invoke_with_args` receives `number_rows`, which matters for the all-scalar case (no arrays to infer length from). A generated adapter that infers length from its first array argument breaks on all-scalar invocations.
- `Interval`'s public surface did not resolve to a field list in the extraction `[rustdoc:datafusion-expr-common@55.1.0]`; the exact bound representation for `evaluate_bounds` should be read from the crate before the range-analysis adapters are written. **Open item.**

---

## 8. Session, runtime, memory, and ambient inputs (D7)

**Role:** §14.3 — "one DataFusion `SessionContext` per snapshot with the catalog of §5.4, `MemoryPool` limits, and the generated UDF registry"; §18.8 thread budget; §20.4 reproduction.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Session and task | `SessionConfig`, `RuntimeEnv`, `RuntimeEnvBuilder`, `TaskContext`, `TaskContextProvider` | `[rustdoc:datafusion-execution@55.1.0]` |
| Memory accounting | `MemoryPool` trait — `register`, `unregister`, `grow`, `shrink`, **`try_grow -> Result<()>`**, `reserved() -> usize`, **`memory_limit() -> MemoryLimit`**; `MemoryConsumer`, `MemoryReservation`, `MemoryConsumerMetrics` | `[rustdoc:datafusion-execution@55.1.0]` |
| Pool implementations (5) | `GreedyMemoryPool`, `FairSpillPool`, `UnboundedMemoryPool`, `TrackConsumersPool`, `PeakRecordingPool` | `[rustdoc:datafusion-execution@55.1.0]` |
| Spill | `DiskManager`, `DiskManagerBuilder`, `SpillFile`, `SpillWriter`, `FileSpillWriter`, `SpillingProgress`, `TempFileFactory`, `RefCountedTempFile` | `[rustdoc:datafusion-execution@55.1.0]` |
| **Injectable clock** | `TimeProvider` trait — `now() -> Instant`; `SystemTimeProvider` | `[rustdoc:datafusion-execution@55.1.0]` |
| Metadata caching | `CacheManager`, `CacheManagerConfig`, `Cache`/`CacheKey`/`CacheValue` traits, `DefaultCache`, `FileMetadata`, `CachedFileMetadata`, `CachedFileList`, `CacheEntryInfo`, `LruQueue` | `[rustdoc:datafusion-execution@55.1.0]` |
| Object store wiring | `ObjectStoreRegistry` trait, `DefaultObjectStoreRegistry`, `ObjectStoreUrl` | `[rustdoc:datafusion-execution@55.1.0]` |
| Schema fingerprinting | **`SchemaFingerprint::from_schema(&Schema)`** | `[rustdoc:datafusion-execution@55.1.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §14.3 "one `SessionContext` per snapshot … with `MemoryPool` limits" | `RuntimeEnvBuilder` + a chosen `MemoryPool` | **confirmed** |
| §14.3 "the generated UDF registry"; §14.2 rule 5 "function registry hash" | function registration on the session | **confirmed** — and the hash is platform code over the registered names/signatures |
| §18.8 "one configuration owns the thread budget: the DataFusion `tokio` runtime, the `rayon` pool …" | `SessionConfig` (`target_partitions`) and the runtime | **confirmed**, and see D5's note that the budget is expressed in config |
| §20.1 artifact store on `object_store` | `ObjectStoreRegistry` / `ObjectStoreUrl` | **confirmed** — DataFusion consumes the same `object_store` 0.13.2 the artifact store uses, so one store instance can serve both |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Choose the `MemoryPool` deliberately, and never `UnboundedMemoryPool`** | **DM-30** (make partial failure and recovery explicit), **G5**, DM-39 | **adopt.** Five implementations exist with materially different failure behaviour. `UnboundedMemoryPool` converts an out-of-memory condition from a typed `ResourcesExhausted` error into an OOM kill — which destroys the run, any in-flight artifacts, and §23.2's ability to classify the failure at all. `FairSpillPool` or `GreedyMemoryPool` with an explicit limit turns exhaustion into a recoverable, classifiable failure. **The blueprint says "`MemoryPool` limits" without choosing; this is the choice.** |
| **`TrackConsumersPool` / `PeakRecordingPool` for diagnostics** | **DM-50** (observe the model lifecycle), DM-39 | **evaluate.** These wrap a pool to attribute memory to consumers and record peaks. §24.3 benchmarks and §23.1 metrics both want this, and it converts "the pass used too much memory" into "this operator did". |
| **`TimeProvider` — inject a deterministic clock** | **DM-28** (declare ambient inputs and nondeterminism), **DM-48** (reproducibility contract) | **adopt for tests and reproduction.** Wall-clock time is an ambient input. §20.4's `pse reproduce` asserts byte-identical relation hashes for deterministic passes; anything that reaches a timestamp through the engine breaks that. DataFusion already provides the seam — use `SystemTimeProvider` in production and a fixed provider under test. **Unmentioned in the blueprint, and it is exactly the DM-28 mechanism the charter asks for.** |
| **`CacheManager` for artifact metadata** | DM-26 (separate preparation from repeated execution), DM-32 | **evaluate.** §5.4 requires statistics "answered only from cached metadata"; DataFusion ships a pluggable metadata cache with `FileMetadata`/`CachedFileMetadata`. Whether to use it or keep the manifest cache platform-side is a real choice — the engine's cache is keyed to files, while the platform's natural key is the content hash. Probably platform-side wins, but it should be a decision. |
| `SchemaFingerprint::from_schema(&Schema)` | DM-15, DM-53 | **evaluate, do not adopt blind.** §4.2 defines the contract fingerprint as *blake3 of the `RelationSpec` rows* — derived from the registry, which is the authority. `SchemaFingerprint` hashes the Arrow schema instead. These answer different questions and must not be conflated: the registry fingerprint is the contract identity (stable across Arrow versions), while `SchemaFingerprint`'s stability across DataFusion releases is unknown. **Keep §4.2's definition; this is at most a cheap secondary check.** |
| Validate `target_partitions` against the rayon budget at session construction | **G5**, DM-35 | **adopt** — §18.8 says oversubscription "is a configuration error, not a runtime surprise"; that sentence only becomes true if something checks it. |

### Measured (PROBE H) — what does a bounded pool actually produce?

A 64 KiB `GreedyMemoryPool`, two partitions, sorting 200,000 rows `[probe]`:

```text
PROBE H
  error: "Not enough memory to continue external sort. Consider increasing the
          memory limit config: 'datafusion.runtime.memory_limit', or decreasing
          the config: 'datafusion.execution.sort_spill_reservation_bytes'"
```

The recommendation is confirmed, and more strongly than expected: the failure is not merely typed, it is **actionable** — the message names the two configuration keys that would resolve it. That is precisely the shape §23.2 wants from a `runtime.resources` diagnostic, and it arrives for free simply by choosing a bounded pool. The counterfactual is an OOM kill with no message at all.

### Gaps and risks

- **Spill introduces a filesystem dependency the blueprint does not discuss.** `DiskManager` writes temp files when a pool spills. For a reproducible, content-addressed platform that is benign (spill is invisible in results) but it is a real resource and failure surface: a full temp filesystem becomes an execution failure mid-pass. §23.2's `runtime.infrastructure` class covers it; the disk manager should be configured explicitly rather than defaulted. **55.1.0 makes this pluggable** — `datafusion_execution::{SpillFile, SpillWriter, TempFileFactory}` let the platform own spill placement and lifetime, and `datafusion.execution.{spill_compression, max_spill_file_size_bytes}` govern its shape (§12 E2, E3).
- `MemoryLimit` as a return type means a pool can report "unbounded" — worth asserting at startup that the configured pool reports a finite limit.
- **Arrow allocations outside a query are invisible to this pool.** §18.6's result ingestion and §4.2's builders construct arrays directly; DataFusion's `MemoryPool` governs query operators only. Arrow's own `MemoryPool` (the `pool` feature) covers the remainder — Arrow map §11 register row 17.

---

## 9. Errors, diagnostics, and observability (D8)

**Role:** §23.1 observability and §23.2's failure taxonomy — "failures carry the relation rows and source spans involved; they are never flattened into 'run failed'".

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Error enum (20 variants) | `DataFusionError::{ArrowError, ParquetError, ObjectStore, IoError, SQL, NotImplemented, Internal, Plan, Configuration, SchemaError, Execution, ExecutionJoin, ResourcesExhausted, External, Context, Substrait, **Diagnostic**, **Collection**, Shared, Ffi}` | `[rustdoc:datafusion-common@55.1.0]` |
| Structured diagnostic | `Diagnostic::{new_error(message, Option<Span>), new_warning(message, Option<Span>), add_note(message, Option<Span>), add_help(message, Option<Span>), with_note, with_help}` | `[rustdoc:datafusion-common@55.1.0]` |
| Multiple failures | `DataFusionError::Collection` | `[rustdoc:datafusion-common@55.1.0]` |
| Error chaining | `DataFusionError::Context`, `External`, `Shared` | `[rustdoc:datafusion-common@55.1.0]` |
| Memory metrics | `MemoryConsumerMetrics`, `SpillingProgress` (D7) | `[rustdoc:datafusion-execution@55.1.0]` |

### What the blueprint binds to

| Blueprint requirement | API | Status |
|---|---|---|
| §23.2 mapping engine failures into the platform taxonomy | the 20 `DataFusionError` variants | **confirmed as a mapping problem, and the mapping is not 1:1.** `ResourcesExhausted` → `runtime.infrastructure`; `SchemaError`/`Plan` → `validation.invariant` or `internal.invariant` depending on origin; `Execution` → depends entirely on context; `Internal` → `internal.invariant` (§23.2's `BurntToast`). **The blueprint's §23.2 table has no DataFusion column; it should, or the mapping will be invented per call site.** |
| §23.2 "never flattened into 'run failed'" | `Collection` carries multiple errors; `Context` preserves a chain | **leverage** — the engine supports non-flattening; the platform must not undo it by stringifying |

### Pass 2 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Add a DataFusion column to §23.2's failure table** | **DM-47** (represent diagnostics as structured evidence), G3 | **adopt.** Twenty engine variants currently map to platform classes by convention at each call site. A declared mapping — ideally a single generated conversion function — makes the taxonomy data (§23.2's own premise) rather than folklore, and prevents the common failure where every engine error becomes `internal.invariant`. |
| **`DataFusionError::Collection` for multi-error reporting** | DM-47, DM-30 | **adopt.** §4.2's validators return *violating keys* — plural — and §14.2's rule evaluation can produce many diagnostics in one pass. Collapsing those into the first error loses exactly the evidence §23.2 wants preserved. |
| `Diagnostic` with notes and helps | DM-47, DM-49 | **evaluate.** It is a good structured carrier, but its `Span` is a *SQL source* span, and the blueprint's authoring source spans are `pse.source_span` over YAML/TOML/DSL text (§4.4). So `Diagnostic` is useful for engine-originated problems, while platform failures keep their own span type. Do not try to unify them; state the boundary. |
| Distinguish `ResourcesExhausted` from other execution failures in `runtime.solver_events`/pass records | **G5**, DM-30 | **adopt** — it is the one engine error that is expected, recoverable and configuration-driven (D7's pool choice). Treating it as a generic execution failure hides a tuning problem as a bug. |
| `DataFusionError::Ffi` | DM-41, G3 | **note** — relevant if the Python boundary ever surfaces engine errors across FFI; another reason not to unwind across that boundary. |

### Measured (PROBE X3) — how is an invalid configuration reported?

§14.3 constructs a session from a configuration the platform assembles, and §23.2 requires failures to be typed. Measured `[probe]`:

```text
PROBE X3  setting datafusion.explain.format = "json" (not a valid value)
   SessionConfig::set_str  -> PANICS
        thread panicked at datafusion-execution-55.1.0/src/config.rs:186
        called `Result::unwrap()` on an Err value: Configuration("Invalid explain
        format. Expected 'indent', 'tree', 'pgjson' or 'graphviz'. Got 'json'")
   ConfigOptions::set      -> typed Err: "Error setting config datafusion.explain.format"
```

**The convenient API panics; the correct one returns a typed error.** This is the exact shape of the Arrow map's `Field::extension_type()` versus `try_extension_type()` finding, and it deserves the same treatment: a panic during session construction is not a `config.invalid` row in §23.2's taxonomy, it is a process abort with a backtrace. **Ban `SessionConfig::set_str` in platform code; assemble configuration through `ConfigOptions::set` and convert the `Err` into a typed failure.** Governance grep, beside the `Field::extension_type()` one.

This also matters for §14.2 rule 5, which reads semantic settings back from `information_schema.df_settings`: the *write* path is where validation happens, and it is the path that panics.

### Gaps and risks

- **DataFusion emits no `tracing` spans of its own by default.** §23.1 wants spans "per rule evaluation"; the engine does not provide them, so rule-level spans are platform code around plan execution, and operator-level visibility requires an external instrumentation layer. This is a real gap between §23.1's ambition and what the engine supplies out of the box. **Two 55.1.0 mechanisms narrow it** (§12 E2): `datafusion_common_runtime::JoinSetTracer` injects instrumentation into every future and blocking closure the engine spawns, and `CustomMetricValue` lets platform-defined metrics travel with an operator and appear in `EXPLAIN ANALYZE`. Neither is a tracing span, but together they cover most of what §23.1 wants. **Open item, now with named candidates.**
- **`SessionConfig::set_str` panics on an invalid value** (PROBE X3). Ban it.
- `DataFusionError::Shared` (an `Arc`-wrapped error) exists so one error can be reported to several consumers — a hint that concurrent execution can surface the same failure repeatedly. Pass-level failure recording should deduplicate.

---

## 10. Plan serialization, Substrait, SQL, and the unparser (D9)

Promoted from a survey to a full cluster, because blueprint revision 2 moved a load-bearing decision into it. **F9** now specifies that the memo key and the pass record carry "a `datafusion-proto` plan fingerprint", computed as "blake3 of the plan's `datafusion-proto` encoding under the platform's `LogicalExtensionCodec`" (§14.2). The previous edition of this map never evaluated `datafusion-proto` — its open item 6 offered only "a formatting surface" or "a platform structural walk", and the crate was **absent from the extraction corpus**. It is now extracted and measured, and the result is the most consequential finding in this edition.

**Role:** §14.2 rule 5 and the plan fingerprint; §14.3 memoization; §20.2 manifest; §20.4 reproduction; §5.4 the catalog the plan refers to.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Logical plan bytes | `datafusion_proto::bytes::{logical_plan_to_bytes, logical_plan_from_bytes}` and the `_with_extension_codec` variants | `[rustdoc:datafusion-proto@55.1.0]` |
| Physical plan bytes | `physical_plan_to_bytes{,_with_extension_codec,_with_proto_converter}` and the matching readers | `[rustdoc:datafusion-proto@55.1.0]` |
| Expression bytes | `datafusion_proto::bytes::Serializeable` — `to_bytes`, `from_bytes_with_ctx`, provided `from_bytes` | `[rustdoc:datafusion-proto@55.1.0]` |
| **The codec contract** | `LogicalExtensionCodec` — **required**: `try_decode`, `try_encode`, `try_decode_table_provider`, `try_encode_table_provider`. **Provided** (i.e. default, and by default not implemented): `try_{decode,encode}_file_format`, `try_{decode,encode}_udf`, `try_{decode,encode}_higher_order_function`, `try_{decode,encode}_udaf`, `try_{decode,encode}_udwf` | `[rustdoc:datafusion-proto@55.1.0]` |
| Physical counterpart | `PhysicalExtensionCodec`, `PhysicalProtoConverterExtension`, `PhysicalPlanNodeExt`; expression-level `PhysicalExprEncode`/`PhysicalExprDecode` | `[rustdoc:datafusion-proto@55.1.0]`, `[rustdoc:datafusion-physical-expr-common@55.1.0]` |
| Model types | `datafusion-proto-models` — the generated protobuf message types, separately published | `[rustdoc:datafusion-proto-models@55.1.0]` |
| Substrait | `datafusion_substrait::logical_plan::{consumer::SubstraitConsumer, producer::SubstraitProducer}` | `[rustdoc:datafusion-substrait@55.1.0]` |

### Measured (PROBE C) — is a `datafusion-proto` fingerprint sound?

Three questions, in the order they matter: can it encode our plans at all; is the encoding deterministic; is it stable across sessions `[probe]`.

```text
PROBE C  datafusion-proto encoding

   without a codec:   FAILS -> NotImplemented("LogicalExtensionCodec is not provided")
                               ("Error serializing custom table")
   with a codec:      len=220   same plan encoded twice: identical=true
   equivalent plan, fresh session:                       identical=false
   same query, different whitespace:                     identical=false
   different predicate:                                  differs=true
   round-trip (encode -> decode -> re-encode):           identical=false
```

That last line is the alarming one: encoding, decoding and re-encoding **in the same process with the same codec** does not reproduce the bytes. So the instability is not a session artefact. A second probe isolated the cause `[probe]`:

```text
PROBE C2/C3  plan-byte stability vs the number of field-metadata keys
             (6 independent encodings of the same query and schema)

   0 metadata key(s):  len=35   distinct encodings among 6 = 1   STABLE
   1 metadata key(s):  len=49   distinct encodings among 6 = 1   STABLE
   2 metadata key(s):  len=63   distinct encodings among 6 = 2   UNSTABLE
   3 metadata key(s):  len=77   distinct encodings among 6 = 3   UNSTABLE
   5 metadata key(s):  len=105  distinct encodings among 6 = 6   UNSTABLE

   3 keys, one shared Schema object reused:      distinct = 1   STABLE
```

**The cause is `Field::metadata`'s `HashMap` iteration order.** Rust seeds each `HashMap` independently, so two separately constructed but semantically identical schemas iterate their metadata in different orders, and `datafusion-proto` serialises them in iteration order. Reuse one `Schema` object and the instability vanishes; rebuild it and it returns. At five metadata keys, six encodings of the same plan produced six different byte strings.

### What the blueprint binds to

| Blueprint requirement | Status |
|---|---|
| §14.2 rule 5 / **F9**: "blake3 of the plan's `datafusion-proto` encoding under the platform's `LogicalExtensionCodec`" | **erratum — as specified, the fingerprint is not reproducible.** Every `pse.*` relation carries two or more field-metadata keys by §4.3's own conventions (`pse.semantic.logical_type`, `…quantity_type`, `…role`, plus `ARROW:extension:name` on extension columns). The fingerprint therefore differs on essentially every process start. Since it is *in the memo key* (§14.3), memoized artifacts would never be reused; and since it is also recorded in `provenance.pass_records.plan_fingerprints`, two byte-identical runs would claim different provenance. **This is fixable and cheap — see below — but it must be fixed before F9's mechanism is written.** |
| §14.2 "the rendered `EXPLAIN` text … is never an identity" | **confirmed, and revision 2 is right to say so.** D4 and PROBE X2 both support it: the rendering is a display surface with four selectable formats, not a stability contract. |
| §5.4's catalog is entirely custom `TableProvider`s | **confirmed, with an obligation revision 2 does not state.** `logical_plan_to_bytes` **fails outright** on a plan containing a custom provider — `NotImplemented("LogicalExtensionCodec is not provided")`. The platform must implement `try_encode_table_provider` / `try_decode_table_provider`, which are *required* methods. The fingerprint is therefore not "DataFusion's encoding of the plan" but "DataFusion's encoding **plus the platform's own encoding of every table reference**" — and the stability of the second half is entirely the platform's responsibility. §14.2 should say so. |
| §14.2 the fingerprint distinguishes different plans | **confirmed** — a changed predicate changed the bytes. |

### Pass 2 and Pass 3 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Canonicalise field and schema metadata at schema construction** | **DM-48**, DM-15, **G6** | **adopt — this is the fix for the erratum above, and it is one function.** Build every `Schema` through a constructor that inserts metadata from a sorted, deterministic source. It costs nothing, it makes `datafusion-proto` bytes reproducible, and it simultaneously fixes the platform-side `pse.contract.fingerprint` (Arrow map §11 register row 8). Fixing it at fingerprint time instead would fix neither, because the instability is inside DataFusion's encoder. |
| **Implement `try_encode_udf` on the platform codec** | **DM-31**, **G6** | **adopt.** `try_encode_udf` is a *provided* method, so a codec that does not override it encodes a UDF by **name only**. §18.5's kernels are generated UDFs whose *definitions* change when `KernelSpec` changes. A fingerprint that captures only the name would be identical across two plans that compute different things — exactly the "cache hit reuses an artifact produced under different semantics" failure **G6** describes. §14.2 already hashes the kernel digest separately, so the information exists; the codec must include it or §14.2 must state that the fingerprint deliberately excludes UDF bodies and the kernel digest covers them. Either is defensible; silence is not. |
| Verify fingerprint stability in CI | DM-48, DM-53 | **adopt.** Encode the same plan in two fresh processes and assert byte equality. This is a three-line test that would have caught the erratum above, and it is the only thing that keeps the canonicalisation from regressing. |
| `Serializeable` for individual `Expr`s | DM-48 | **evaluate.** §6.6's `pse.expr_dsl` columns are the authored form and `normalized.*_expr_*` the parsed graphs — both platform-owned, so this is not needed for the IR. It is useful for *caching a compiled filter expression* alongside a pass record. Low priority. |
| **`datafusion-substrait`** | DM-48, DM-55, **DM-57**, DM-58 | **reject as a plan format, and the reason is now stronger.** Substrait would give §20.4 a portable, engine-independent record — attractive for reproducibility. Against it: `RulePlanSpec` is already the authoritative typed versioned plan description, and Substrait would be a second one (**DM-57**, **DM-58**). Revision 2's choice of `datafusion-proto` is the better call *provided* the canonicalisation above lands: proto is an encoding of a plan the platform already has, not a new IR to maintain. Reconsider Substrait only if cross-engine execution becomes a requirement. |
| **SQL frontend** (`datafusion-sql`) | DM-10, DM-16 | **used only for human and analytic queries.** §5.4 shows SQL over the snapshot catalog and §19.2 reporting is a natural SQL surface. §14.2 is explicit that rule bodies are typed `RulePlanSpec`s compiled through `LogicalPlanBuilder`, *not* SQL strings — the right call: a SQL string is untyped, unversioned, and not queryable as structure. **Keep SQL at the analytics edge; never in the compiler.** |
| **Unparser** (default feature at 55.1.0 `[docs.rs:datafusion@55.1.0]`) | **DM-49** | **evaluate for explainability only.** Rendering a `LogicalPlan` back to SQL lets a reviewer read what a rule compiled to in a language they know. Same caveat as `EXPLAIN`: evidence for humans, never an identity or contract. `UserDefinedLogicalNodeUnparser` extends it to the platform's own nodes. |
| `LogicalPlan::{Dml, Ddl, Copy, Statement}` | **G1** | **never constructed** — see D1. |
| `datafusion` optional features `avro`, `parquet_encryption`, `serde`, `backtrace` | — | **not enabled.** No Avro source; no encryption requirement stated; `serde` on plans is superseded by `RulePlanSpec` being the serialized form; `backtrace` is a debugging aid enabled ad hoc. |
| `recursive_protection` (default on) | DM-30 | **keep enabled** — guards against stack overflow on deeply nested plans, relevant because §14.2's fixed-point executor builds iterated plans. |

### Gaps and risks

- **The fingerprint's stability is a platform property, not an engine guarantee.** Between the `HashMap` ordering above and the platform-written `try_encode_table_provider`, most of what determines the bytes is ours. DataFusion offers no stable-plan-hash API and does not claim to. §14.2 should state the fingerprint's definition as a platform contract with its own version, exactly as §5.3 does for content hashes.
- **Physical plans are a separate, larger surface.** `physical_plan_to_bytes` exists and takes its own codec. F9 speaks only of a logical-plan fingerprint, which is the right scope — the physical plan depends on `target_partitions` and other execution-only settings that §14.2 deliberately excludes from the semantic-settings hash.
- `datafusion-proto-models` is separately published and separately versioned in principle; at 55.1.0 it moves with the family. Worth noting only because it is the crate whose wire format actually defines the bytes.

---

## 11. Logical types, extension types, and the type registry (D10) — new cluster

**This is the largest single gap between the blueprint and DataFusion 55.1.0**, and it did not exist in the previous edition because the surface did not exist in the extraction corpus.

The blueprint's §4.3 says of `pse.*` metadata keys: *"nothing in DataFusion acts on these keys unless a platform component explicitly reads them."* At 55.1.0 that is an **erratum**. DataFusion has a first-class logical-type layer and a name-keyed extension-type registry, wired into `SessionStateBuilder`, which resolves `ARROW:extension:name` to registered behaviour and validates storage types during planning.

**Role:** §4.4 (the ten `pse.*` extension types), §4.5 (the logical type catalog), §4.3 (metadata conventions), §5.4 (what the provider hands the engine), §18.5 (`return_field_from_args` and UDF typing).

This cluster is where **DM-06** ("type semantic distinctions, not only machine representations") stops being an Arrow-format property and becomes an *engine* property. The Arrow map's A2 establishes that `pse.semantic_id` and `pse.content_hash` are two different semantic types over one storage type. What A2 could not offer was anywhere for the engine to enforce that. This is that place.

### Capability inventory

| Capability | Surface | Provenance |
|---|---|---|
| Logical type layer | `datafusion_common::types::LogicalType` — `native() -> &NativeType`, `signature() -> TypeSignature`, provided `default_cast_for(&DataType)`; `LogicalTypeRef = Arc<dyn LogicalType>` | `[rustdoc:datafusion-common@55.1.0]` |
| Native type | `NativeType` with predicates `is_numeric`, `is_integer`, `is_float`, `is_decimal`, `is_timestamp`, `is_date`, `is_time`, `is_duration`, `is_interval`, `is_binary`, `is_null` | `[rustdoc:datafusion-common@55.1.0]` |
| Type signatures | `TypeSignature::{arity, is_one_of, supports_zero_argument, get_example_types, to_string_repr_with_names}`; `TypeSignatureClass::{matches_native_type, default_casted_type}` | `[rustdoc:datafusion-common@55.1.0]` |
| **Extension type contract** | `datafusion_common::types::DFExtensionType` — required `storage_type() -> DataType`, `serialize_metadata() -> Option<String>`; provided `create_array_formatter(&dyn Array, &FormatOptions) -> Result<Option<ArrayFormatter>>` | `[rustdoc:datafusion-common@55.1.0]` |
| **The registry** | `datafusion_expr::registry::ExtensionTypeRegistry` — `extension_type_registration(name)`, `extension_type_registrations()`, `add_extension_type_registration(..)`, `remove_extension_type_registration(name)`; provided **`create_extension_type_for_field(&Field) -> Result<Option<DFExtensionTypeRef>>`** and `extend(&[..])` | `[rustdoc:datafusion-expr@55.1.0]` |
| Registration | `ExtensionTypeRegistration::new_arc(name: impl Into<String>, factory: impl Fn(&DataType, Option<&str>) -> Result<DFExtensionTypeRef> + Send + Sync + 'static)`; `type_name()`; `create_df_extension_type(&DataType, Option<&str>)` | `[rustdoc:datafusion-expr@55.1.0]` |
| Default implementation | `MemoryExtensionTypeRegistry::{new_empty, new_with_canonical_extension_types, new_with_types}`; `From<HashMap<String, ExtensionTypeRegistrationRef>>` | `[rustdoc:datafusion-expr@55.1.0]` |
| Session wiring | `SessionStateBuilder::with_extension_type_registry(ExtensionTypeRegistryRef)`; `Session::extension_type_registry()`; `SessionStateDefaults::default_extension_types()` | `[rustdoc:datafusion-session@55.1.0]` |
| SQL-level type mapping | `datafusion_expr::planner::TypePlanner` — `plan_type(&ast::DataType)`, `plan_type_field(..) -> Result<Option<FieldRef>>`; installed via `SessionStateBuilder::with_type_planner` | `[rustdoc:datafusion-expr@55.1.0]`, `[c7:/apache/datafusion]` |
| Field/DataType helpers | `datafusion_common::datatype::{FieldExt, DataTypeExt}` — `renamed`, `retyped`, `with_field_metadata`, `into_list`, `into_fixed_size_list`, `into_list_item` | `[rustdoc:datafusion-common@55.1.0]` |

### Measured (PROBE X1)

Registering a `pse.semantic_id` type over `FixedSizeBinary(16)`, exactly as §4.4 declares it `[probe]`:

```text
PROBE X1  ExtensionTypeRegistry
   canonical types preloaded (7): arrow.timestamp_with_offset, arrow.opaque,
      arrow.fixed_shape_tensor, arrow.uuid, arrow.variable_shape_tensor,
      arrow.bool8, arrow.json
   register pse.semantic_id           -> 8 registrations
   resolve from a Field carrying
     ARROW:extension:name             -> storage_type = FixedSizeBinary(16)
   Field with the WRONG storage type  -> REJECTED
        "Error during planning: pse.semantic_id requires FixedSizeBinary(16), got Utf8"
   unregistered pse.* name            -> Err("Logical type not found")
```

And separately, that extension metadata survives the engine end to end `[probe]`:

```text
PROBE E  ARROW:extension:name through planning and execution
   planned output field 'id'   : Some("pse.semantic_id")
   executed batch field 'id'   : Some("pse.semantic_id")
   after `SELECT id AS key_out`: Some("pse.semantic_id")
   as a GROUP BY key           : Some("pse.semantic_id")
```

Four consequences, in order of importance.

1. **Storage-type validation becomes an engine-level property of every plan**, not a check the platform performs only inside generated views. The Arrow map's §11 register row 1 (`try_extension_type` inside `try_from`) validates a batch the platform constructs; this validates *anything* that reaches the session, including whatever a future analytics query or an imported artifact puts in front of it. Same principle (**DM-07**, **G3**), one level lower, and covering paths generated code never sees.

2. **An unregistered `pse.*` name errors rather than degrading.** This is the opposite of the Python boundary's behaviour, where the companion Python map measured that an unregistered extension type silently becomes its bare storage type. The same design decision is therefore *enforced* on the Rust side and *unenforced* on the Python side — which is exactly the asymmetry §21's loss profile must declare (**DM-42**, **G7**).

3. **`ARROW:extension:name` propagates through projection, aliasing and aggregation** (PROBE E). §4.3's claim that metadata is "a carrier" understates it: the carrier is preserved by the engine across operations that rewrite schemas. That is what makes an end-to-end semantic-type discipline feasible at all.

4. **Seven canonical Arrow extension types are preloaded by default.** `MemoryExtensionTypeRegistry::new_with_canonical_extension_types()` is what `SessionStateDefaults` uses, so `arrow.json`, `arrow.uuid`, `arrow.bool8`, `arrow.opaque` and the two tensor types are *already* recognised. The Arrow map's §11 register row 2 (enable `canonical_extension_types`) has an engine-side counterpart that is on by default.

### What the blueprint binds to

| Blueprint requirement | Status |
|---|---|
| §4.3 "nothing in DataFusion acts on these keys unless a platform component explicitly reads them" | **erratum.** True of ordinary `pse.*` keys; **false of `ARROW:extension:name`**, which the engine resolves against its registry and uses to reject mismatched storage types. The sentence should be narrowed to non-`ARROW:` keys. |
| §4.4 the ten `pse.*` extension types and their standard storage types | **confirmed registrable, and enforceable.** Each type's `supports_data_type` logic has a direct home in the registration factory, which receives `(&DataType, Option<&str>)` — the storage type and the serialized metadata — and returns an error the planner surfaces. |
| §4.5 logical type catalog | **refined.** §4.5 is a catalog of *Arrow* storage types. DataFusion has its own `LogicalType`/`NativeType` layer above them, used for function signature resolution and coercion. The two are compatible but distinct, and §4.5 should say which layer it is describing. |
| §18.5 `return_field_from_args` "attaches `pse.semantic.*` metadata and the output quantity type" | **confirmed and strengthened.** Because the field's metadata survives (PROBE E), a UDF that attaches an extension name on output gets that name validated downstream by the registry — so §18.5's claim becomes checkable rather than advisory. |
| §4.4 "Python consumers register matching `pyarrow.ExtensionType` classes" | unchanged — see the Python map; the asymmetry in consequence 2 above is the finding |

### Pass 2 / Pass 3 — alignment opportunities

| Item | DM / gate | Recommendation |
|---|---|---|
| **Register the ten `pse.*` types in an `ExtensionTypeRegistry` at session construction** | **DM-06**, **DM-07**, **G3**, DM-44 | **adopt — the headline recommendation of this edition.** One registration per type, generated from §4.4's table exactly as §4.2 generates everything else. It converts §4.4 from a documentation convention into an engine-enforced constraint, covering every plan rather than only generated views, and it costs one generated function. |
| **`DFExtensionType::create_array_formatter`** | **DM-49**, DM-47 | **adopt alongside the Arrow map's §11 register row 11.** The same formatter implementation satisfies Arrow's `ArrayFormatterFactory` and DataFusion's per-type hook, so `pse.semantic_id` renders meaningfully in `EXPLAIN`, in error messages and in diagnostics instead of as 16 raw bytes. |
| **`TypePlanner` for `pse.*` types in analytic SQL** | DM-06, DM-55 | **evaluate.** §19.2's reporting surface is SQL. A `TypePlanner` lets a query say `CAST(x AS SEMANTIC_ID)` — or any platform type name — and have it resolve to the right storage type *with* its extension metadata. Upstream's own example does exactly this for `UUID` → `FixedSizeBinary(16)` + `ARROW:extension:name` `[c7:/apache/datafusion]`, which is the same shape as `pse.semantic_id`. Only worth it if analytic SQL becomes a supported interface rather than an ad-hoc one. |
| **Extension-type versioning via the registry** | **DM-44**, DM-51 | **evaluate — this is where the Arrow map's open item can finally be answered.** Arrow has no registry, so A2 could only record that `pse.*` metadata has no version-resolution story. A registration factory receives the serialized metadata string and may branch on a version field inside it, so a single registered name can accept several metadata generations and reject the rest. That is a migration mechanism (DM-51) in the place DM-44 wants it. |
| `LogicalType` / `NativeType` for the platform's own quantity types | DM-06, DM-58 | **reject, for now.** It is tempting to model `QuantityType` as a DataFusion `LogicalType`. But §8's dynamic registry is the authority for quantity types and units, and DataFusion's logical-type layer exists to drive *function signature coercion* — a different job. Implementing it would create a second place where a quantity type is defined (**G1**) for no gain the extension registry does not already provide. Recorded because it is the obvious next step and it is the wrong one. |
| `FieldExt` / `DataTypeExt` helpers | DM-58 | **note only** — convenience methods (`renamed`, `retyped`, `with_field_metadata`, `into_list`). Worth knowing they exist so generated code does not re-implement them; not a design decision. |

### Gaps and risks

- **The registry is session-scoped, which is both the point and the risk.** A relation read outside a `SessionContext` — by a native pass borrowing Arrow buffers directly (D11), or by the Python boundary — gets no registry and therefore no validation. The engine-level guarantee covers plans, not the whole platform. The Arrow map's §11 register row 1 is therefore complementary, not redundant: it covers the paths that never touch a session.
- **`create_extension_type_for_field` is a *provided* method.** A custom `ExtensionTypeRegistry` implementation that overrides it could weaken the check silently. Use `MemoryExtensionTypeRegistry` unless there is a reason not to.
- **Registration order and duplicate names.** `add_extension_type_registration` returns the previous registration for that name, so a later registration silently replaces an earlier one. Generated code should assert the return is `None` — a duplicate `pse.*` name is a build error, not a runtime surprise (**G1**).
- The registry validates *storage type and metadata*. It cannot validate cross-field constraints such as `pse.ordinal_ref`'s target relation, which the Arrow map records as an open item. That gap is unchanged.

---

## 12. Pass 3 — the under-leverage sweep

Passes 1 and 2 both start from the blueprint. Neither can find a capability the blueprint never gestured at, because neither enumerates the library independently. This pass does: four complete enumerations, each with a stated denominator, each row adjudicated **bound**, **adopt**, **evaluate** or **reject, with reason**. Charter §G and **DM-58** still apply.

The denominator matters because of the rule this corpus inherits: **absence from the extraction is not evidence that a feature does not exist.** The previous edition covered 20 crates and 148 traits; this one covers 37 and 169.

### 12.1 E1 — the crate inventory

DataFusion 55.1.0 declares **44 workspace members** `[gh:apache/datafusion@55.1.0/Cargo.toml]`; removing binaries, test harnesses, codegen helpers and example crates (`datafusion-cli`, `datafusion-examples` and its three FFI examples, `datafusion-sqllogictest`, `datafusion-wasmtest`, `test-utils`, `benchmarks`, the two `*/gen` crates) leaves **~37 publishable library crates**. All 37 are extracted.

| Crate group | Status | Adjudication |
|---|---|---|
| `datafusion` (umbrella), `-common`, `-expr`, `-expr-common`, `-session`, `-catalog`, `-execution`, `-physical-plan`, `-physical-expr`, `-physical-expr-common`, `-physical-optimizer`, `-optimizer`, `-sql`, `-pruning` | **bound** | the surface D1–D8 already cover |
| **`-proto`, `-proto-common`, `-proto-models`** | **now bound — by revision 2** | **F9's plan fingerprint.** Absent from the previous extraction, which is why D9 could not be written. Measured there. |
| `-datasource`, `-datasource-{arrow,csv,json,parquet}`, `-catalog-listing` | **reject for platform data, with one exception** | §5.4's providers are platform-written over content-addressed artifacts; listing tables and format inference are the opposite of D1's model. **The exception is `SchemaAdapterFactory`/`PhysicalExprAdapterFactory`** (E2), which live here and are about schema evolution, not file discovery. |
| `-functions`, `-functions-aggregate`, `-functions-aggregate-common`, `-functions-nested`, `-functions-window`, `-functions-window-common`, `-functions-table` | **bound (as the default registry)** | The session ships them; §14.2's rule algebra uses comparison and boolean functions. §18.5's kernels are *additional* UDFs, not replacements. `-functions-table` is the table-function surface (E2). |
| **`-ffi`** | **evaluate** | A stable ABI for `TableProvider`, `ScalarUDF`, `ExecutionPlan` and even `FFI_PhysicalOptimizerRule` across a dynamic-library boundary. The blueprint has no plugin boundary — everything is one workspace — so **DM-58** says no today. It becomes interesting only if §22's extension model ever admits third-party kernel packages as compiled artifacts rather than source. Recorded so the option is known. |
| **`-substrait`** | **reject as a plan format** | D9 — `RulePlanSpec` is already the authoritative typed plan description; a second one cuts against **DM-57** and **DM-58**. |
| **`-physical-expr-adapter`** | **evaluate** | Schema-evolution machinery — see E2. |
| **`-spark`** | **reject** | Spark-compatible function semantics (`monthname`, `SparkPow`, ANSI mode). No Spark compatibility requirement exists, and adopting Spark semantics for arithmetic would silently change numerical behaviour (**G2**). |
| `-common-runtime` | **bound (transitively)**, one interesting trait | `JoinSetTracer` — see E2 |
| `-macros`, `-doc` | **bound (transitively)** | the `#[user_doc]` machinery behind function documentation; no decision |

### 12.2 E2 — the extension-point inventory

**169 public traits** across the 37 crates `[rustdoc:df551@55.1.0]`, against 148 in the previous 20-crate corpus. Type-level machinery, sealed markers and internal accumulator plumbing are dismissed as a group. What follows is every trait that is a genuine place to insert platform behaviour and is **not already covered** by D1–D10.

| Trait | DM / gate | Adjudication |
|---|---|---|
| **`datafusion_common::config::ConfigExtension` + `ExtensionOptions`** | **DM-16**, **DM-28**, **DM-31** | **adopt.** A typed, namespaced configuration extension (`ConfigExtension::PREFIX`, then `set`/`entries` over typed fields) that lives inside `ConfigOptions` and appears in `information_schema.df_settings` alongside the engine's own. §14.2 rule 5 **already reads semantic settings back from `df_settings`** to build the settings hash — so platform policy registered as a `datafusion.pse.*` extension is captured by the existing mechanism with no new plumbing. Today platform policy travels separately from engine policy and only the engine half is hashed. |
| **`datafusion_common::cse::{Normalizeable, NormalizeEq, CSEController}`** | **DM-15** | **evaluate — closer to §5.3's needs than it looks.** `NormalizeEq::normalize_eq` is DataFusion's answer to "are these two nodes semantically equivalent after normalization", used for common-subexpression elimination. §5.3 defines canonicalization and equivalence for *relations*; §7.4 (P10) defines hash-consing for the *math IR*. This is the engine's version of the same idea for expression trees. Not directly reusable — it is `Expr`-shaped and the math IR is not `Expr` (D6) — but it is prior art worth reading before P10's canonicalizer is written, and `CSEController` is the hook if the rule compiler ever wants CSE over its own plans. |
| **`datafusion_pruning::UnhandledPredicateHook`** | **DM-43**, G7 | **adopt.** `handle(&Arc<dyn PhysicalExpr>) -> Arc<dyn PhysicalExpr>` — called for predicates the pruning rewriter cannot handle. This is capability negotiation made explicit: instead of a predicate silently falling through to a full scan, the platform sees exactly which expressions were not handled. Directly complements D2's `Exact` discipline, and it is the mechanism that turns "we think pushdown fired" into "we know when it did not". |
| **`datafusion_physical_expr_common::metrics::CustomMetricValue`** | **DM-50**, DM-39 | **adopt if any custom `ExecutionPlan` is written.** Application-defined metric values that aggregate and display alongside the engine's own in `EXPLAIN ANALYZE`. With `pgjson` output (D4) this makes platform-specific counters — rows derived per rule, kernels evaluated — machine-readable pass evidence rather than log lines. |
| **`datafusion_common_runtime::JoinSetTracer`** | **DM-50**, DM-28 | **evaluate.** `trace_future` / `trace_block` wrap every future and blocking closure the engine spawns. This is the closest thing DataFusion has to the per-operation instrumentation §23.1 wants, and D8 records that the engine emits no spans of its own. Two methods; the cost is that it sits on a very hot path. |
| **`datafusion_expr::var_provider::VarProvider`** | **DM-28** (declare ambient inputs) | **evaluate.** Supplies values for `@variable` / `@@variable` references in a plan. If §19.3's sweeps or §19.2's reports ever want a parameterised constant, this is the declared, typed way to inject it — as against string interpolation into SQL, which would be an undeclared ambient input. Pairs with `Expr::Placeholder` (D3). |
| **`datafusion_datasource::schema_adapter::{SchemaAdapter, SchemaAdapterFactory, SchemaMapper}`** | DM-51 | **reject — deprecated.** The trait's own documentation at 55.1.0 reads: *"Deprecated: … This trait has been removed. Use `PhysicalExprAdapterFactory` instead"* `[rustdoc:datafusion-datasource@55.1.0]`. Recorded because it is the obvious search hit for "schema evolution" and is the wrong answer. |
| **`datafusion_physical_expr_adapter::{PhysicalExprAdapter, PhysicalExprAdapterFactory}`** | **DM-51**, DM-42 | **evaluate.** The replacement: rewrites a `PhysicalExpr` to match a target schema, for the case where a stored artifact's physical schema differs from the table schema. §22.2's change-set model and §4.2's `pse.contract.version` mean artifacts written under an older `RelationSpec` will exist. Today §20.5 would handle that by conversion at import; this would handle it at scan. A real migration mechanism — but only once schema evolution is a lived problem rather than an anticipated one (**DM-58**). |
| **`datafusion_execution::{SpillFile, SpillWriter, TempFileFactory}`** | DM-28, DM-29, **G5** | **evaluate.** Pluggable spill: the platform chooses where temp files live and how they are named and cleaned up. D7 records that spill is an undiscussed filesystem dependency; this is the seam that makes it an owned one. |
| **`datafusion_execution::cache::{Cache, CacheKey, CacheValue}`** | DM-32 | **evaluate** — as D7's `CacheManager` row already records. Note `CacheKey` requires `table_ref()` and `size()`, which reveals the cache is table-shaped; the platform's natural key is the content hash. Probably platform-side wins. |
| **`datafusion_common::rounding::FloatBits`** | **DM-40** | **note, do not adopt.** `to_bits` / `from_bits` / `float_is_nan` / `infinity` — the bit-level float manipulation behind DataFusion's interval arithmetic. §5.3 step 4 needs exactly this shape (IEEE bits, NaN canonicalisation) and the platform will write its own, because §5.3's canonicalisation rules are the platform's contract, not DataFusion's. Recorded so nobody assumes the engine's helper implements §5.3's semantics — it does not. |
| **`datafusion_expr::async_udf::AsyncScalarUDFImpl`** | DM-38 | **reject.** `invoke_async_with_args` for UDFs that call out to remote services. §18.5's kernels are pure native functions and D10's boundary forbids a solve inside a scalar call; an async UDF would put an await point inside expression evaluation. Recorded as a rejection because "our kernel needs I/O" is a plausible future request that should be answered no. |
| **`datafusion_expr::HigherOrderUDFImpl`** | — | **reject** — as D3 already records for `Lambda`: model semantics belong in the math IR, not in `Expr`. |
| `datafusion_session::{TableFunctionImpl, ExtensionPlanner, PhysicalPlanner, QueryPlanner}` | DM-19 | **`ExtensionPlanner`: adopt if D3's extension nodes are adopted** — PROBE B shows it is the missing piece for physical execution. The others: **reject**, no requirement. |
| `datafusion_catalog::{AsyncCatalogProvider, AsyncSchemaProvider, UrlTableFactory}` | — | **reject** — the snapshot catalog resolves once at session creation (D1). Recorded to prevent a future "we should be async" drift. |
| `datafusion_execution::context::FunctionFactory` | G1 | **reject** — handles `CREATE FUNCTION` from SQL. §18.5's UDFs are generated from `KernelSpec`; admitting SQL-defined functions would create a second authority over what a kernel is. |
| `datafusion_common::tree_node::{TreeNode, TreeNodeRewriter, TreeNodeVisitor, DynTreeNode, …}` | DM-24 | **evaluate as prior art.** A complete, well-tested generic tree-rewriting framework with `Transformed<T>` fixed-point signalling. The math IR needs the same machinery (P10, P11, P12). Implementing `TreeNode` for the platform's own IR nodes is plausible and would give the rewrite drivers for free; the cost is a dependency from `pse-math` onto `datafusion-common`, which §3.2 currently confines to `pse-catalog` and `pse-rules`. **A real architectural question, not a free win.** |
| `datafusion_datasource::morsel::{Morsel, MorselPlanner, Morselizer}` | DM-35 | **reject** — morsel-driven I/O scheduling for file scans. §5.4 reads mmapped IPC artifacts; there is no I/O scheduling problem to solve. |
| `datafusion_physical_plan::operator_statistics::StatisticsProvider` | DM-39 | **evaluate** — customises statistics computation per `ExecutionPlan` node, chained until one answers. Relevant only alongside D2's `Precision` discipline, and only if a custom `ExecutionPlan` is written. |
| `datafusion_expr::registry::SerializerRegistry` | DM-48 | **bound by implication** — D9's codec work. |
| `datafusion_expr::planner::{ExprPlanner, RelationPlanner, ContextProvider}` | — | **reject** — SQL-surface extension points; §14.2 builds plans structurally, not from SQL. `TypePlanner` is the exception and is adjudicated in D10. |
| `datafusion_sql::unparser::{Dialect, UserDefinedLogicalNodeUnparser}` | DM-49 | **evaluate for explainability** — D9. |
| `datafusion_common::heap_size::DFHeapSize`, `utils::proxy::{VecAllocExt, HashTableAllocExt}` | DM-39 | **note** — memory accounting helpers used by the engine's own operators; relevant only if a custom operator is written. |
| ~130 type-level, sealed, accumulator and codec-internal traits | — | **not extension points** — dismissed as a group |

### 12.3 E3 — the declared-policy surface

DataFusion's policy surface is **155 configuration keys** across eight namespaces `[gh:apache/datafusion@55.1.0/docs/source/user-guide/configs.md]`:

| Namespace | Keys | Bearing |
|---|---|---|
| `datafusion.execution` | 71 | mixed — some semantic, most execution-only |
| `datafusion.optimizer` | 39 | **semantic** — rewrite enablement |
| `datafusion.sql_parser` | 10 | **semantic** — dialect, identifier handling |
| `datafusion.format` | 9 | display only |
| `datafusion.explain` | 9 | display only, but see D4's `pgjson` |
| `datafusion.runtime` | 8 | resource limits (D7) |
| `datafusion.catalog` | 8 | mostly disabled by D1's model |
| `datafusion.spark` | 1 | rejected (E1) |

§14.2 rule 5 already gets the core of this right: it captures `datafusion.optimizer.*`, `datafusion.sql_parser.*` and `datafusion.execution.time_zone` into the semantic-settings hash and excludes execution-only keys such as `batch_size` and `target_partitions`. The enumeration confirms that partition is sound, and surfaces four keys that deserve individual attention:

| Key | Default | Finding |
|---|---|---|
| **`datafusion.execution.skip_physical_aggregate_schema_check`** | `false` | **assert it stays false.** When true, the engine skips verifying that the schema produced by planning an aggregate's input matches the input plan's schema. That is a **G3** check being switched off. It is not in §14.2's semantic-settings list because it lives under `execution.*` — an example of the namespace split not aligning perfectly with the semantic/non-semantic split. |
| **`datafusion.execution.enable_ansi_mode`** | `false` | **assert it stays false.** Changes expression semantics for Spark built-ins; combined with E1's rejection of `-spark`, this should be pinned rather than left to a default. |
| **`datafusion.execution.spill_compression`**, **`max_spill_file_size_bytes`** | `uncompressed`, 128 MiB | declare explicitly alongside the `MemoryPool` choice (D7) |
| **`datafusion.explain.{format, analyze_level, analyze_categories}`** | `indent`, `dev`, `all` | set `format = pgjson` where a pass record stores a plan (D4) |

**The general finding:** §14.2 rule 5's semantic-settings hash is defined by *namespace*, but at least two semantically significant keys live under `execution.*`. The rule should be restated as an explicit allow-list of keys rather than a namespace prefix match, and the list versioned — otherwise a future DataFusion release adding a semantic key under `execution.*` silently escapes the hash (**DM-31**, **G6**).

Beyond config, the crate's **feature flags** are the compile-time half (§1.3): defaults include `unparser`, `recursive_protection`, `parquet`, `sql` and the expression families; `serde`, `avro`, `parquet_encryption` and `backtrace` are off and stay off (D9).

### 12.4 E4 — the release delta, 54.0 → 55.1

Selected from the 55.0.0 changelog `[gh:apache/datafusion@55.1.0/dev/changelog/55.0.0.md]`. 55.1.0 itself is a bug-fix release with no new features.

| Change | Adjudication |
|---|---|
| **`feat: Support IEEE 754 negative zero semantics`** | **directly relevant, and measured.** PROBE D shows `ORDER BY` distinguishes `-0.0` from `+0.0` while `GROUP BY` and hash joins merge them. The release note explains why ordering behaves correctly; it does not extend to grouping. This is the evidence behind D3's finding that §5.3's "`-0.0` is preserved" needs qualification. |
| **`feat: add strictness metadata for scalar UDF null propagation and use it in outer join elimination`** | **adopt — this is `is_strict` earning its keep.** §3.1 already names `is_strict` and §18.5 binds it. The 55.0 change makes the declaration *load-bearing*: the optimizer uses it to eliminate outer joins. A UDF that declares `is_strict = true` while emitting non-null output for null input would therefore change query results, not merely miss an optimisation. Reinforces register row on `is_nullable`/strictness derivation from `KernelSpec`. |
| **`Add EnsureRequirements: merged EnforceDistribution + EnforceSorting with idempotent pushdown_sorts`** | **confirms §3.1's "55 forms"** — already verified in §1.5 |
| **`feat(sql): Postgres-style EXPLAIN (...) option list`; `pgjson` format for `EXPLAIN ANALYZE`** | **adopt** — D4 |
| **`Add minimal APIs / hooks for granular statistics collection in TableProvider`; `StatisticsContext`; `partition_statistics`** | **evaluate** — extends D2's `Precision` discipline to per-partition granularity. Only useful once §5.4's statistics are richer than row counts and key min/max. |
| **`Add Physical/logical Partitioning::Range` + proto representation** | **evaluate.** Range partitioning preserves ordering across partitions, where hash partitioning does not. D5 records that partitioned output arrives nondeterministically and §5.3 re-sorts. If a large `compiled.*` relation ever makes that sort expensive, range partitioning is the mechanism that avoids it (**DM-35**, **DM-40**). |
| **`feat: introduce pluggable SpillFile trait and TempFileFactory`** | **evaluate** — E2, D7 |
| **`feat: Plumb Parquet virtual columns (row_number)`; `input_file_name` UDF** | **reject.** Provenance columns derived from physical storage position. §5.1 is explicit that row positions are never identity; adopting these would create a second, position-based identity (**G1**, DM-11). |
| **`Add SQL planner, physical planner, and TableProvider hook for MERGE INTO`** | **reject** — a sixth mutation method on `TableProvider`. Strengthens D1's governance-test recommendation: the trait keeps growing, so the test must assert the *set* of implemented methods rather than checking a fixed list. |
| **`feat(catalog): expose InformationSchemataBuilder as public API`** | **note** — §14.2 rule 5 already reads `information_schema.df_settings`; this is the builder behind it. |
| **`feat(physical-expr): DynamicFilterTracker`; dynamic filter pushdown config keys** | **reject for now** — runtime-adaptive filters are a scan-performance mechanism for large external datasets; §5.4's artifacts are small and keyed. |
| **`refactor: wrap HigherOrderUDFImpl in a concrete HigherOrderUDF struct`; lambda proto + Substrait support** | **reject** — D3 |
| **`perf(logical-plan): box CreateExternalTable / CreateFunction in DdlStatement (-45% LogicalPlan size)`** | **note** — no action; illustrates that `LogicalPlan`'s representation changes between releases, which is the argument for not treating its rendering as an identity (D9) |
| **`Allow datafusion-ffi to opt out of proto parquet`; `FFI_PhysicalOptimizerRule::optimize_with_context`** | **note** — E1's `-ffi` evaluation |

### 12.5 Third-party extenders — bounded survey

| Crate | Latest | Adjudication |
|---|---|---|
| `datafusion-table-providers` (+ the `-postgres`/`-mysql`/`-sqlite`/`-duckdb`/`-flightsql`/`-odbc` family) | 0.13.1 | **reject.** Providers for external databases. D1's catalog is snapshots of content-addressed artifacts; a live external database inside the catalog would be a second, mutable authority (**G1**) and would break §20.4's reproduction contract outright. |
| `datafusion-tracing` | 55.0.0 | **evaluate, with a caveat.** Instruments execution plans with tracing spans — exactly D8's gap. The caveat is that it is at **55.0.0 while the engine is at 55.1.0**, so it is an independently-versioned compatibility edge of the kind that produced the `num-dual`/`feos-core` split in the supporting-library map. `JoinSetTracer` (E2) is the first-party alternative and is one trait. |
| `datafusion-distributed` | 4.0.0 | **reject** — D10's boundary keeps DataFusion out of the numerical inner loop; a distributed tier is machinery for a scale this design does not claim (**DM-58**). |
| `datafusion-postgres` / `-pg-catalog` | 0.18.x | **reject** — exposes the engine over the Postgres wire protocol. That is a service boundary §3.3 explicitly does not want. |
| `datafusion-proto-models` | 55.1.0 | **in-family** — E1 |
| `datafusion-federation`, `clickhouse-datafusion`, `zarr-datafusion`, `qdrant-datafusion`, `hudi-datafusion`, `datafusion-ducklake` | — | **reject** — external-source integrations; same reason as `datafusion-table-providers` |

---

## 13. Principle-alignment register (DataFusion)

Capabilities the blueprint does **not** claim, which would improve alignment with `DATA_MODEL_DESIGN_CHARTER.md`. Every row names a principle and the hand-written work or risk it removes. Rejections are included, because a recorded rejection is worth as much as an adoption.

In the previous edition this table combined Arrow and DataFusion rows. The Arrow rows now live in that map's §11; these are DataFusion's. Numbering is local to this map, and rows marked **new** come from §12's Pass-3 enumeration or from this edition's probes.

### Adopt

| # | Capability | Cluster | DM / gate | What it removes or prevents |
|---|---|---|---|---|
| 1 | **Register the ten `pse.*` types in an `ExtensionTypeRegistry` at session construction** | **D10** | **DM-06**, **DM-07**, **G3**, DM-44 | **The headline of this edition.** Converts §4.4 from a documentation convention into an engine-enforced constraint: a field whose storage type does not match its declared extension name is **rejected during planning** (PROBE X1), across every plan rather than only inside generated views. One generated function, driven from §4.4's existing table. |
| 2 | **Canonicalise field and schema metadata before a schema reaches the engine** | D9 | **DM-48**, DM-15, **G6** | **Blocking for revision 2's F9.** `datafusion-proto` serialises field metadata in `HashMap` iteration order, so plan bytes differ on every process start once a field carries ≥2 metadata keys — which every `pse.*` relation does (PROBE C). As specified, F9's fingerprint would never produce a memo hit and would record different provenance for identical runs. |
| 3 | **Implement `try_encode_udf` on the platform's `LogicalExtensionCodec`** | D9 | **DM-31**, **G6** | It is a *provided* method, so the default encodes a UDF **by name only**. Two plans calling the same-named kernel with different `KernelSpec` bodies would fingerprint identically — a cache hit reusing an artifact produced under different semantics. Either encode the kernel digest, or state in §14.2 that the fingerprint excludes UDF bodies and the kernel digest covers them. Silence is the one unacceptable option. |
| 4 | **CI test: encode the same plan in two fresh processes, assert byte equality** | D9 | DM-48, DM-53 | Three lines. It would have caught row 2, and it is the only thing that stops the canonicalisation from silently regressing. |
| 5 | Governance test: no `TableProvider` mutation method is implemented | D1 | **G1**, DM-02 | The mutation methods are *defaulted*, so the architecture is protected only by omission. 55.0 added a sixth (`merge_into`), which is the argument for asserting the *set* of implemented methods rather than checking a fixed list. |
| 6 | `Precision::{Exact, Inexact, Absent}` used deliberately per statistic | D2 | **DM-59**, G7 | An undifferentiated statistics answer. The uncertainty label is built into the type; using it is free. |
| 7 | Answer `StatisticsRequest`s selectively, `Absent` otherwise | D2 | DM-26 | Makes §5.4's "answered only from cached metadata" a contract rather than a convention. |
| 8 | `Constraints::project()` when a scan is projected | D2 | DM-22, DM-24 | Hand-projecting a primary key over a changed column set. |
| 9 | **Record that DataFusion enforces no constraint but `Field` nullability** | D2 | **G7**, DM-43 | §5.4 declares "Constraints: primary key only". Upstream is explicit that primary, unique and foreign keys "are not validated or used during query planning" `[c7:/apache/datafusion]`. The declaration is sound; reading it as enforcement is not. Platform P2 validation is what makes it true. **New.** |
| 10 | `conditional_arguments()` + `short_circuits()` on conditional kernels | D6 | **G4**, DM-28 | Eager evaluation inside `if/then/else` fires §18.2's domain guards on values the model never intended to evaluate — a spurious `solve.evaluation_error` with a misleading culprit. |
| 11 | `output_ordering` / `preserves_lex_ordering` / `strictly_order_preserving` for monotone kernels | D6 | **DM-24**, DM-40 | Makes a monotonicity claim explicit and testable instead of implicit. |
| 12 | `is_nullable` and `is_strict` derived from `KernelSpec`, not written independently | D6 | **DM-08**, G2 | A UDF declaring non-nullable while its batch path emits nulls makes the plan schema lie. **Raised in stakes by 55.0:** strictness metadata is now used for *outer join elimination*, so a wrong `is_strict` changes results rather than merely missing an optimisation (§12 E4). |
| 13 | `coerce_types` = accept declared types, else error | D6 | DM-42, G2 | Silent widening of an `Int64` argument into a `Float64` kernel. |
| 14 | Record the optimizer/analyzer rule set and engine version in the pass key | D4 | **DM-31**, **DM-32**, G6 | **Applied in revision 2** as part of F9 (`reference.engine_profiles`, ordered rule lists, `engine_profile_hash`). Retained as the closed record of why. |
| 15 | **Restate §14.2 rule 5's semantic-settings hash as an explicit key allow-list** | §12 E3 | **DM-31**, **G6** | Rule 5 selects settings by *namespace* (`datafusion.optimizer.*`, `sql_parser.*`, `execution.time_zone`). At least two semantically significant keys live under `execution.*` — `skip_physical_aggregate_schema_check` and `enable_ansi_mode` — so a namespace match misses them, and a future release adding a semantic key under `execution.*` would escape the hash silently. **New.** |
| 16 | **Assert `skip_physical_aggregate_schema_check = false` and `enable_ansi_mode = false`** | §12 E3 | **G3**, G2 | The first switches off a schema-consistency check; the second changes expression semantics. Both default correctly today; neither is pinned. **New.** |
| 17 | Choose a bounded `MemoryPool`; never `UnboundedMemoryPool` | D7 | **DM-30**, **G5** | Converts an OOM kill into a typed failure. **Measured (PROBE H):** the resulting error is not merely typed but *actionable* — it names the two config keys that would resolve it, which is exactly what §23.2 wants from a resource diagnostic. |
| 18 | Inject `TimeProvider` for tests and reproduction | D7 | **DM-28**, **DM-48** | Wall-clock time is an ambient input that breaks §20.4's byte-identical reproduction claim. The engine already provides the seam. |
| 19 | Validate `target_partitions` against the rayon budget at session construction | D7, D5 | G5, DM-35 | §18.8 says oversubscription "is a configuration error, not a runtime surprise" — only true if something checks. |
| 20 | Add a DataFusion column to §23.2's failure table | D8 | **DM-47**, G3 | Twenty engine variants currently map to platform classes by convention at each call site. |
| 21 | `DataFusionError::Collection` for multi-error reporting | D8 | DM-47, DM-30 | Validators return violating keys *plural*; collapsing to the first error discards the evidence §23.2 wants. |
| 22 | **Ban `SessionConfig::set_str`; assemble configuration through `ConfigOptions::set`** | D8 | **G3**, DM-47 | Measured (PROBE X3): the convenient setter **panics** on an invalid value while `ConfigOptions::set` returns a typed error. A panic during session construction is a process abort, not a `config.invalid` row. Same shape as the Arrow map's `Field::extension_type()` ban, and the same remedy — a governance grep. **New.** |
| 23 | Test-only wrapper that re-checks every `Exact` pushdown claim | D2 | **G7**, DM-53, DM-54 | Nothing in DataFusion verifies an `Exact` claim; the optimizer *deletes* the filter on trust. PROBE A shows the shapes are matchable, which makes the claim *reachable* — it does not make a given match correct. |
| 24 | **A provider must treat `supports_filters_pushdown` as pure** | D2 | DM-20, G4 | Measured (PROBE A): the engine calls it more than once per plan. Instrumentation or counters placed there — the obvious location — would double-count. Undocumented in the signature. **New.** |
| 25 | **`ConfigExtension` / `ExtensionOptions` for a typed `datafusion.pse.*` namespace** | §12 E2 | **DM-16**, **DM-28**, DM-31 | Platform policy registered as a config extension appears in `information_schema.df_settings`, which §14.2 rule 5 **already reads** — so it enters the settings hash with no new plumbing. Today platform policy travels separately and only the engine half is hashed. **New.** |
| 26 | **`UnhandledPredicateHook`** | §12 E2 | **DM-43**, G7 | Turns "we think pushdown fired" into "we know exactly which predicates were not handled". Capability negotiation made explicit, complementing D2's `Exact` discipline. **New.** |
| 27 | **`DFExtensionType::create_array_formatter`** | D10 | **DM-49**, DM-47 | Shares one implementation with the Arrow map's `ArrayFormatterFactory` (its §11 register row 11) so `pse.*` values render meaningfully in `EXPLAIN`, error messages and diagnostics instead of as raw storage. **New.** |
| 28 | **`EXPLAIN` format `pgjson` where a pass record stores a plan** | D4 | **DM-27**, DM-49 | The previous edition treated `EXPLAIN` as one text surface; it is four, and one is structured JSON (PROBE X2). A stored plan becomes queryable evidence rather than a blob. **New.** |
| 29 | **`CustomMetricValue`** if any custom `ExecutionPlan` is written | §12 E2 | **DM-50**, DM-39 | Platform counters aggregate and display alongside engine metrics; with `pgjson` they become machine-readable pass evidence rather than log lines. **New.** |
| 30 | **`ExtensionPlanner`** if D3's extension nodes are adopted | §12 E2 | DM-19 | PROBE B shows it is the one missing piece: the node survives the optimizer intact, and physical planning then fails with a clean typed error until a planner is registered. **New.** |
| 31 | Family-wide `=` pins, **and a committed, CI-enforced lockfile** | §1.2 | DM-48, DM-51 | Revision 2 adopted the pins; it did not adopt the lockfile. `=` pins bind direct dependencies only — a transitive path can still introduce a second version. |

| 32 | **The per-rule `observer` closure on `Optimizer::optimize` / `Analyzer::execute_and_check`** | D4 | **DM-46**, DM-50, DM-49 | Revision 2's F9 records the rule *set*; the observer records which rules actually **fired, in what order, on this plan**. The first is what could have happened, the second is what did — and it is what makes a plan difference explainable at the level of meaning instead of by diffing two renderings. The seam exists and the platform already drives optimisation explicitly, so this is effectively free. **New.** |

### Evaluate

| # | Capability | Cluster | DM | Why it is not yet an adopt |
|---|---|---|---|---|
| 33 | `LogicalPlan::Extension` + `UserDefinedLogicalNode` for rule bodies, with `check_invariants` and `prevent_predicate_push_down_columns` | D3 | **DM-18**, **DM-21**, DM-07 | Still the largest structural opportunity, and **its blocking uncertainty is now resolved**: PROBE B shows the node survives the optimizer with `rule_id` intact, and that the optimizer still pushes projections down *through* it. What remains is cost, not risk — a 14-method contract plus row 30's planner. This is an adopt-candidate awaiting a decision, not an open question. |
| 34 | `ScalarUDFImpl::preimage()` | D6 | DM-43, G6 | Enables pushdown of filters over computed columns. Same correctness character as `Exact` pushdown: a wrong preimage silently drops rows. Adopt only for proven monotone inverses, tested adversarially. |
| 35 | `TrackConsumersPool` / `PeakRecordingPool` | D7 | DM-50, DM-39 | Turns "the pass used too much memory" into "this operator did". |
| 36 | `CacheManager` for artifact metadata | D7 | DM-26, DM-32 | The engine cache is table/file-keyed (`CacheKey` requires `table_ref()`); the platform's natural key is the content hash. Probably platform-side wins — but decide it. |
| 37 | `Expr::Placeholder` for §19.3 sweeps | D3 | DM-26 | One prepared plan executed per sample. Only if sweeps prove plan-construction-bound. Pairs with row 39. |
| 38 | `EXPLAIN` / unparser output stored as pass evidence | D4, D9 | **DM-49** | Excellent for explainability; must never become an identity. Row 28 settles the *format*; whether to store it at all is still a decision. |
| 39 | **`VarProvider`** for declared parameter values | §12 E2 | **DM-28** | The typed, declared way to inject a constant into a plan, as against string interpolation into SQL — which would be an undeclared ambient input. **New.** |
| 40 | **`JoinSetTracer`** | §12 E2 | **DM-50**, DM-28 | The closest first-party answer to D8's "no engine spans" gap: two methods wrapping every future and blocking closure. Cost: it sits on a very hot path. **New.** |
| 41 | **`PhysicalExprAdapterFactory`** for schema evolution | §12 E2 | **DM-51**, DM-42 | Artifacts written under an older `RelationSpec` will exist (§22.2, §4.2's `pse.contract.version`). This adapts at scan; §20.5 would convert at import. Real migration machinery — once evolution is a lived problem, not an anticipated one (**DM-58**). Note the deprecated `SchemaAdapter` is the wrong trait (row 52). **New.** |
| 42 | **`SpillFile` / `SpillWriter` / `TempFileFactory`** | §12 E2 | DM-28, DM-29, G5 | D7 records spill as an undiscussed filesystem dependency; this is the seam that makes it an owned one. **New.** |
| 43 | **`Partitioning::Range`** | §12 E4 | **DM-35**, DM-40 | Preserves ordering across partitions where hash partitioning does not. Relevant only if §5.3's re-sort ever becomes expensive on a large `compiled.*` relation. **New.** |
| 44 | **`StatisticsContext` / `partition_statistics` / `StatisticsProvider`** | §12 E2, E4 | DM-39, DM-59 | Extends row 6's `Precision` discipline to per-partition granularity. Only once §5.4's statistics are richer than row counts and key min/max. **New.** |
| 45 | **`TreeNode` implemented for the math IR nodes** | §12 E2 | **DM-24**, DM-22 | A complete, tested generic rewriting framework with fixed-point signalling — exactly what P10, P11 and P12 need. The cost is a dependency from `pse-math` onto `datafusion-common`, which §3.2 currently confines to two crates. **A genuine architectural question, not a free win.** **New.** |
| 46 | **`NormalizeEq` / `Normalizeable` / `CSEController`** as prior art | §12 E2 | **DM-15** | The engine's answer to "are these two expression trees semantically equivalent after normalization". Not directly reusable (it is `Expr`-shaped), but worth reading before P10's canonicalizer is written. **New.** |
| 47 | **Extension-type versioning via the registration factory** | D10 | **DM-44**, DM-51 | The factory receives the serialized metadata string, so one registered name can accept several metadata generations and reject the rest. This is where the Arrow map's long-standing open item can finally be answered. **New.** |
| 48 | **`datafusion-ffi`** | §12 E1 | DM-44, DM-58 | A stable ABI for providers, UDFs and even physical optimizer rules across a dynamic-library boundary. No plugin boundary exists today, so **DM-58** says no — it matters only if §22's extension model ever admits compiled third-party kernel packages. **New.** |
| 49 | **`datafusion-tracing`** (third party) | §12 E2, §12.5 | DM-50 | Instruments plans with tracing spans — D8's gap, solved. But it is at **55.0.0 against an engine at 55.1.0**: an independently-versioned compatibility edge of exactly the kind that produced the `num-dual`/`feos-core` split. `JoinSetTracer` (row 40) is the first-party alternative. **New.** |

### Reject, with reason

| # | Capability | Cluster | Why rejected |
|---|---|---|---|
| 50 | `ScalarUDFImpl::with_updated_config` / reading `config_options` in a kernel | D6 | A kernel whose result depends on session config is nondeterministic with respect to everything §5.3 hashes. **G4**, DM-28. |
| 51 | `TableProvider::get_column_default` | D1 | Column defaults are an authoring concept the blueprint deliberately does not have — values come from `authored` relations and case overlays. |
| 52 | **`SchemaAdapter` / `SchemaAdapterFactory` / `SchemaMapper`** | §12 E2 | **Deprecated at 55.1.0** — the trait's own docs say "This trait has been removed. Use `PhysicalExprAdapterFactory` instead". Recorded because it is the obvious search hit for "schema evolution" and is the wrong answer. **New.** |
| 53 | Substrait as a plan format | D9 | `RulePlanSpec` is already the authoritative typed versioned plan description. A second one cuts against **DM-57** and **DM-58**. Revision 2's choice of `datafusion-proto` is better: an encoding of a plan the platform already has, not a new IR. Reconsider only for cross-engine execution. |
| 54 | SQL strings in the compiler | D9 | Untyped, unversioned, not queryable as structure — against DM-10 and DM-16. SQL stays at the analytics edge. |
| 55 | `HigherOrderFunction` / `Lambda` / `HigherOrderUDFImpl` in plans | D3, §12 E2 | Would push model semantics into `Expr`, blurring exactly the boundary **D6** draws. |
| 56 | `AsyncSchemaProvider` / async catalog resolution | D1 | The snapshot catalog resolves once at session creation. Recorded to prevent a future "we should be async" drift. |
| 57 | **`AsyncScalarUDFImpl`** | §12 E2 | An await point inside expression evaluation. §18.5's kernels are pure native functions and D10's boundary forbids I/O there. Recorded because "our kernel needs to call a service" is a plausible future request that should be answered no. **New.** |
| 58 | **`FunctionFactory`** (`CREATE FUNCTION` from SQL) | §12 E2 | §18.5's UDFs are generated from `KernelSpec`. Admitting SQL-defined functions creates a second authority over what a kernel is — **G1**. **New.** |
| 59 | **Parquet virtual columns (`row_number`) and the `input_file_name` UDF** | §12 E4 | Provenance derived from physical storage position. §5.1 is explicit that row positions are never identity; these would create a second, position-based identity — **G1**, DM-11. **New.** |
| 60 | **`MERGE INTO`** (planner, physical planner and `TableProvider` hook) | §12 E4 | A sixth mutation method. Strengthens row 5 rather than weakening it. **New.** |
| 61 | **`datafusion-spark` and `enable_ansi_mode`** | §12 E1, E3 | Spark-compatible arithmetic and function semantics. No compatibility requirement exists, and adopting them would silently change numerical behaviour — **G2**. **New.** |
| 62 | **`Morsel` / `MorselPlanner` / `Morselizer`** | §12 E2 | Morsel-driven I/O scheduling for file scans. §5.4 reads mmapped IPC artifacts; there is no I/O scheduling problem. **New.** |
| 63 | **`LogicalType` for the platform's quantity types** | D10 | Tempting, and wrong. §8's dynamic registry is the authority for quantity types; DataFusion's logical-type layer exists to drive function-signature coercion. Implementing it would create a second definition of a quantity type (**G1**) for no gain the extension registry does not already give. Recorded because it is the obvious next step after row 1. **New.** |
| 64 | **`datafusion-table-providers` family, `-distributed`, `-postgres`, and the external-source integrations** | §12.5 | A live external database inside the catalog is a second, mutable authority (**G1**) and breaks §20.4's reproduction contract. A distributed tier is machinery for a scale this design does not claim (**DM-58**). A Postgres wire protocol is a service boundary §3.3 excludes. **New.** |
| 65 | Dynamic filter pushdown (`DynamicFilterTracker` and its config keys) | §12 E4 | Runtime-adaptive filters are a scan-performance mechanism for large external datasets; §5.4's artifacts are small and keyed. **New.** |

---

## 14. Acceptance-gate review (DataFusion)

Each gate with the DataFusion-specific way this design could fail it, and what closes it. The Arrow map carries the Arrow-specific review of the same seven gates; a design passes only if both do. **Bold** status changes are relative to the previous edition.

| Gate | The specific risk here | Status | Closing action |
|---|---|---|---|
| **G1 — Authority** | `TableProvider` carries *defaulted* mutation methods — five in 55.0.0, **six now that `merge_into` has landed**. Implementing any would make the immutable snapshot mutable through SQL. `LogicalPlan::{Dml, Ddl, Copy}` are the same risk at plan level; `FunctionFactory` and the external-source providers are the same risk at the catalog level. | **open — protected only by omission, and the surface is still growing** | Row 5: a governance test asserting the *set* of implemented methods, so trait growth is caught. Rows 57, 58, 62, 63 record the temptations. |
| **G2 — Semantic fidelity** | Three engine paths. (a) A UDF declaring `is_nullable = false` while emitting nulls — **now higher-stakes**, since 55.0 uses strictness metadata for outer-join elimination, so a wrong declaration changes *results*. (b) Spark/ANSI semantics silently altering arithmetic. (c) **New, and measured:** `GROUP BY` and hash joins merge `-0.0` with `+0.0` and treat NaN as equal to itself, while `ORDER BY` preserves `totalOrder` (PROBE D). §5.3 and §26 both assert "`-0.0` is preserved" without qualification. | **open — one measured contradiction** | Rows 12, 16, 60; and either declare that `Float64` columns are never grouping or join keys, or record the collapse as a selected loss (**DM-42**). |
| **G3 — Validity** | An invalid state reaching an operation that assumes validity. Three paths: extension metadata never checked; `skip_physical_aggregate_schema_check` switching off a schema check; and `SessionConfig::set_str` **panicking** on an invalid value instead of erroring (PROBE X3). | **substantially improved, not closed** | **Row 1 is the big change** — registering the `pse.*` types makes storage-type validation an engine property of every plan, verified by PROBE X1. Rows 16 and 22 close the other two. |
| **G4 — Hidden behavior** | (a) A conditional kernel whose arguments are evaluated eagerly fires a domain guard the model intended to avoid. (b) A UDF reading `config_options` makes its result depend on ambient session state. (c) **New:** `supports_filters_pushdown` is called more than once per plan (PROBE A), so a provider with side effects there behaves unpredictably. | **open** | Rows 10, 49, 24. |
| **G5 — Consistency and recovery** | `UnboundedMemoryPool` converts resource exhaustion into an OOM kill. Oversubscribed threads degrade unpredictably. Spill writes to an unmanaged filesystem. | **improved** | **Measured (PROBE H): a bounded pool yields a typed error that names the two config keys to change** — a better outcome than the previous edition could claim. Rows 17, 19, 41. |
| **G6 — Transformation and reuse** | The optimizer is an ambient dependency of every derived relation — 25 `OptimizerRule`s + 3 `AnalyzerRule`s, changing between releases. **Revision 2 fixed this** via `reference.engine_profiles` and the rule lists (F9). But F9's own fingerprint mechanism is **measurably non-reproducible** (PROBE C), and its codec encodes UDFs by name only. | **the old hole closed; a new one opened inside the fix** | Rows 2, 3, 4 — all cheap, all blocking. Row 15 closes the namespace-vs-allow-list gap. |
| **G7 — Truthful capability claims** | `TableProviderFilterPushDown::Exact` lets the optimizer **delete** the filter and nothing verifies the claim. DataFusion enforces **no** relational constraint except `Field` nullability, and does not check that on ingest. `Statistics` without `Precision` claims more than is known. | **improved — the blocking unknown is resolved** | **PROBE A retires the previous edition's must-settle item 1 and blueprint §26 F17's precondition:** equality on a `FixedSizeBinary(16)` key arrives as a plain binary expression with a literal, which a provider can match, so **`Exact` is achievable**. Two cautions came with it — `IN` arrives pre-rewritten to `OR` (a provider matching `InList` never fires), and nothing validates the claim. Rows 6, 9, 23, 26. |

**Summary.** Two gates moved materially. **G7's blocking unknown is resolved in the design's favour** — the pushdown spike §26 F17 demanded has been run and `Exact` is reachable. **G3 is substantially closed** by an engine facility the previous edition did not know existed: registering the `pse.*` extension types makes storage-type validation a property of every plan. Against that, **G6 acquired a new hole inside revision 2's own fix** — the `datafusion-proto` fingerprint F9 specifies is not reproducible as written — and **G2 acquired a measured contradiction**, the `-0.0` collapse under grouping and joining. Both are cheap to close and neither requires an architectural change. G1, G4 and G5 are unchanged in character, with G5 better evidenced than before.

---

## 15. Leverage matrix (DataFusion)

| Blueprint requirement | API | Status |
|---|---|---|
| §5.4 catalog of snapshots → namespaces → relations | `CatalogProviderList` / `CatalogProvider` / `SchemaProvider` / `TableProvider` (all in `datafusion-session`) | confirmed |
| §5.4 `scan_with_args` implemented, `scan` delegates | both provided methods; `ScanArgs` with projection/filters/limit/statistics-requests | confirmed |
| §5.4 exact vs inexact filters | `TableProviderFilterPushDown::{Exact, Inexact, Unsupported}` | confirmed; **`Exact` is now shown reachable** `[probe]` — key equality arrives as a matchable binary expression (PROBE A). Still unverified *by the engine*, so row 23's test wrapper stands — G7 |
| §5.4 `IN` on key columns | — | **caution** `[probe]` — `IN (a,b)` is rewritten to `OR` before the provider sees it; a provider matching `Expr::InList` never fires |
| §5.4 "Constraints: primary key only" | `Constraints` | **refined** — DataFusion validates **no** constraint but `Field` nullability, and not on ingest `[c7:/apache/datafusion]`; the declaration is informational |
| §5.4 primary key only | `Constraint::{PrimaryKey, Unique}`, `Constraints::new_unverified` | confirmed; constructor name is the standing warning |
| §5.4 statistics from cached metadata | `Statistics`, `ColumnStatistics`, `Precision`, `StatisticsRequest` | **refined** — use `Precision` and answer requests selectively |
| §5.4 read-only provider over immutable artifacts | five defaulted mutation methods exist | **erratum** — must be provably unimplemented (G1) |
| §14.2 `RulePlanSpec` → `LogicalPlan` via `LogicalPlanBuilder` | full builder surface | confirmed |
| §14.2 recursive closures via recursive CTE | `LogicalPlan::RecursiveQuery`, `to_recursive_query(name, term, is_distinct)`, `CteWorkTable` | confirmed |
| §14.2 rule 4 every head row carries a derivation | — | **evaluate** `LogicalPlan::Extension` to keep `rule_id` in the plan (DM-18) |
| §14.2 rule 5 pinned session | `ConfigOptions` threaded into every analyzer/optimizer rule | confirmed as necessary |
| §14.2 rule 5 plan fingerprint (rev 2 **F9**: `datafusion-proto` + `LogicalExtensionCodec`) | `logical_plan_to_bytes_with_extension_codec` | **erratum** `[probe]` — requires a platform codec to encode at all, and the bytes are **not reproducible** once a field carries ≥2 metadata keys (PROBE C). D9, register rows 2–4 |
| §4.3 "nothing in DataFusion acts on these keys" | `ExtensionTypeRegistry`, `create_extension_type_for_field` | **erratum** `[probe]` — `ARROW:extension:name` *is* acted on: registered types resolve, mismatched storage types are rejected during planning (PROBE X1). D10 |
| §4.4 `pse.*` extension types enforceable by the engine | `ExtensionTypeRegistration::new_arc`, `SessionStateBuilder::with_extension_type_registry` | **adopt, unmentioned** — register row 1, the headline of this edition |
| §5.3 "`-0.0` is preserved" | engine `ORDER BY` vs `GROUP BY` / hash join | **refined** `[probe]` — true of ordering, **false of grouping and joining**, which merge `-0.0` with `+0.0` and treat NaN as self-equal (PROBE D) |
| §14.2 rule 5 semantic-settings hash by namespace | 155 config keys in 8 namespaces | **refined** — two semantic keys live under `execution.*`; use an explicit allow-list (register row 15) |
| §14.2 rule 3 four-valued predicates | `Expr::IsUnknown`/`IsNotUnknown` cover three; `conflict` stays platform-level | partial, by design |
| §18.5 `ScalarUDFImpl` adapter row | 25 methods; all six named by §18.5 confirmed | confirmed |
| §18.5 `return_field_from_args` attaches `pse.semantic.*` | `ReturnFieldArgs { arg_fields, scalar_arguments }` | **confirmed and stronger** — input **Fields** are supplied, so output quantity type can be *derived* from input metadata |
| §18.2 domain guards vs conditional evaluation | `conditional_arguments`, `short_circuits` | **adopt** — unmentioned; G4 |
| §14.3 one `SessionContext` per snapshot with `MemoryPool` limits | `SessionConfig`, `RuntimeEnvBuilder`, 5 pool implementations | confirmed; **pool choice is unmade** |
| §14.3 / §18.8 thread budget | `target_partitions` in `SessionConfig` | confirmed; budget lives in config, must be validated |
| §20.1 artifact store | `ObjectStoreRegistry`, `ObjectStoreUrl`, `object_store` 0.13.2 shared with the platform | confirmed |
| §20.4 reproduction | `TimeProvider` seam exists | **adopt** — unmentioned; DM-28/DM-48 |
| §23.2 failure taxonomy | 20 `DataFusionError` variants, `Diagnostic`, `Collection` | **refined** — §23.2 needs a DataFusion column |
| §23.1 spans per rule evaluation | engine emits no `tracing` spans by default | **gap, with named candidates** — `JoinSetTracer` and `CustomMetricValue` (register rows 39, 29) |
| §23.2 typed failures at session construction | `SessionConfig::set_str` vs `ConfigOptions::set` | **erratum** `[probe]` — the convenient setter **panics** on an invalid value (PROBE X3); register row 22 |
| §23.1 / §19 machine-readable plans | `datafusion.explain.format = pgjson` | **adopt, unmentioned** `[probe]` — four formats exist, one is structured JSON (PROBE X2); register row 28 |
| §3.1 the "55 forms" | all five confirmed | confirmed (§1.5) |
| §3.1 one release family | umbrella-only pinning mixes versions | **erratum** (§1.2) |

---

## 16. Evidence ledger

| # | Cluster | Instrument | Source | Target | Outcome |
|---|---|---|---|---|---|
| 1 | setup | c7 resolve | `DataFusion` | Rust query engine, providers, plans, UDFs | `/apache/datafusion` (5032, High); no docs.rs mirror exists |
| 2 | D1 | c7 query-docs | `/apache/datafusion` | custom `TableProvider`, pushdown, statistics | upstream `library-user-guide/custom-table-providers.md` + `upgrading/48.0.0.md`; `PlanProperties`, `EmissionType`, `Boundedness` |
| 3 | setup | WebSearch | crates.io/docs.rs/github | latest DataFusion release | reported 55.0.0 — **stale**, corrected by rows 4–5 |
| 4 | setup | WebFetch | `docs.rs/crate/datafusion-expr/latest` | latest sub-crate version | **55.1.0 (2026-09-11)** |
| 5 | setup | WebFetch | `docs.rs/crate/datafusion/latest` | latest umbrella version, deps, features | **55.1.0**; `arrow ^59.2.0`, `object_store ^0.13.2`; full feature list |
| 6 | setup | cargo | `generate-lockfile` / `fetch` | coherent `=`-pinned resolution | DataFusion family all 55.1.0; Arrow all 59.3.0; `object_store` 0.13.2; `tokio` 1.53.1 |
| 7 | D1 | rustdoc | `datafusion_session@55.1.0` | `TableProvider`, `CatalogProvider(List)`, `SchemaProvider`, `ScanArgs` | 15 `TableProvider` methods incl. five mutation methods; `ScanArgs::with_statistics_requests` |
| 8 | D1 | rustdoc | `datafusion_catalog@55.1.0` | catalog implementations | `MemoryCatalogProvider*`, `MemTable`, `ViewTable`, `CteWorkTable`, `Async*Provider` |
| 9 | D2 | rustdoc | `datafusion_expr@55.1.0`, `datafusion_common@55.1.0`, `datafusion_expr_common@55.1.0` | pushdown, constraints, statistics | `TableProviderFilterPushDown`, `Constraint`, `Precision`, `ColumnStatistics`, `StatisticsRequest` |
| 10 | D3 | rustdoc | `datafusion_expr@55.1.0` | `LogicalPlan`, `Expr`, `LogicalPlanBuilder`, `UserDefinedLogicalNode` | 25 plan variants, 37 expr variants, `to_recursive_query`, `check_invariants`, `InvariantLevel` |
| 11 | D4 | rustdoc | `datafusion_optimizer@55.1.0` | `OptimizerRule`, `AnalyzerRule`, `ApplyOrder` | 25 + 3 built-in rule impls; `Transformed<T>` |
| 12 | D5 | rustdoc | `datafusion_physical_plan@55.1.0`, `datafusion_physical_optimizer@55.1.0`, `datafusion_expr@55.1.0`, `datafusion_expr_common@55.1.0` | the §3.1 "55 forms" | all five confirmed — see §1.5 |
| 13 | D6 | rustdoc | `datafusion_expr@55.1.0` | `ScalarUDFImpl` and its argument carriers | 25 methods; `ReturnFieldArgs{arg_fields,…}`, `ScalarFunctionArgs{…,config_options}`, `PreimageResult`, `ExprSimplifyResult` |
| 14 | D7 | rustdoc | `datafusion_execution@55.1.0` | runtime, memory, spill, cache, time | `MemoryPool` (+5 impls), `DiskManager`, `CacheManager`, **`TimeProvider`**, `SchemaFingerprint`, `ObjectStoreRegistry` |
| 15 | D8 | rustdoc | `datafusion_common@55.1.0` | error and diagnostic surface | 20 `DataFusionError` variants incl. `Diagnostic`/`Collection`; `Diagnostic::{new_error,new_warning,add_note,add_help}` |
| 16 | §1.1 | crates.io API | `crates.io/api/v1/crates/datafusion` | is the pin stale? | **no** — 55.1.0 (2026-09-11) is still the current release at this compile `[crates.io:datafusion]` |
| 17 | §1.1, §12 E1 | GitHub raw | `apache/datafusion@55.1.0/Cargo.toml` | the E1 denominator; which Arrow does the engine build against? | 44 workspace members (~37 publishable libraries); `arrow = "59.2.0"` — the platform pin runs one minor ahead `[gh:apache/datafusion@55.1.0/Cargo.toml]` |
| 18 | all | **rustdoc** | locally generated, full family, `=`-pinned, lockfile committed | complete API surface | **57 targets, 57 OK, 0 FAIL, format_version 61**; DataFusion family 37 crates all 55.1.0. Supersedes the previous edition's 20-crate extraction, whose output was not preserved. Receipt §1.4 |
| 19 | §12 E2 | facts corpus | `build/facts/df551` (normalized from row 18) | the E2 denominator | **169 public traits**, 76,771 declarations, 48,968 impl relations `[rustdoc:df551@55.1.0]` |
| 20 | D2 | **probe** | `df_probe.rs` — a recording `TableProvider` | what filter shapes reach a provider on a `FixedSizeBinary(16)` key? | **PROBE A** — key equality arrives as a matchable binary expression (**resolves the previous edition's must-settle item 1 and §26 F17's precondition**); `IN` pre-rewritten to `OR`; conjunctions split; `IS NOT NULL` eliminated on a non-nullable column; the method is called more than once |
| 21 | D9 | **probe** | `df_probe.rs`, `df_probe_c3.rs` | is `datafusion-proto` usable and stable as a plan fingerprint? | **PROBE C** — fails without a codec on custom providers; **unstable at ≥2 field-metadata keys** (6 distinct encodings in 6 runs at 5 keys); stable when one `Schema` object is reused, isolating `HashMap` iteration order as the cause |
| 22 | D3 | **probe** | `df_probe_b.rs` | does a `UserDefinedLogicalNode` survive the optimizer carrying `rule_id`? | **PROBE B** — yes, intact and still rendered; the optimizer pushes projections down *through* it; physical planning needs an `ExtensionPlanner` and says so with a typed error |
| 23 | D3, §14 | **probe** | `df_probe.rs` | null / NaN / `-0.0` through sort, GROUP BY and equi-join | **PROBE D** — `ORDER BY` preserves `totalOrder`; `GROUP BY` merges `-0.0` with `+0.0` (4 groups from 5 rows); hash join treats NaN as self-equal |
| 24 | D10 | **probe** | `df_probe_x.rs` | can a `pse.*` type be registered and resolved? what happens to an unregistered one? | **PROBE X1** — 7 canonical types preloaded; registration and resolution work; wrong storage type **rejected** with a typed planning error; unregistered name **errors** |
| 25 | D10 | **probe** | `df_probe.rs` | does `ARROW:extension:name` survive planning and execution? | **PROBE E** — survives projection, aliasing and use as a GROUP BY key |
| 26 | D4, D8 | **probe** | `df_probe_x.rs` | which `EXPLAIN` formats exist; how is an invalid config value reported? | **PROBE X2** — `indent`, `tree`, `pgjson`, `graphviz` all accepted. **PROBE X3** — `SessionConfig::set_str` **panics**; `ConfigOptions::set` returns a typed `Err` |
| 27 | D7 | **probe** | `df_probe.rs` | bounded `MemoryPool` — typed failure or process death? | **PROBE H** — a typed, *actionable* error naming `datafusion.runtime.memory_limit` and `datafusion.execution.sort_spill_reservation_bytes` |
| 28 | §12 E3 | GitHub raw | `apache/datafusion@55.1.0/docs/source/user-guide/configs.md` | the E3 denominator | **155 keys in 8 namespaces**; `datafusion.execution` 71, `.optimizer` 39, `.sql_parser` 10, `.format` 9, `.explain` 9, `.runtime` 8, `.catalog` 8, `.spark` 1 `[gh:apache/datafusion@55.1.0/docs/source/user-guide/configs.md]` |
| 29 | §12 E4 | GitHub raw | `apache/datafusion@55.1.0/dev/changelog/55.0.0.md` | the E4 delta | IEEE 754 negative-zero semantics; UDF null-propagation strictness used for outer-join elimination; `EnsureRequirements`; `pgjson` EXPLAIN; `Partitioning::Range`; pluggable `SpillFile`; `StatisticsContext`; MERGE INTO; Parquet virtual columns `[gh:apache/datafusion@55.1.0/dev/changelog/55.0.0.md]` |
| 30 | D2, D10 | c7 query-docs | `/apache/datafusion` | table-constraint enforcement; `TypePlanner` idiom | constraints are "provided for informational purposes … not validated or used during query planning"; only `Field` nullability is enforced. `TypePlanner` upstream example maps SQL `UUID` → `FixedSizeBinary(16)` + `ARROW:extension:name` — the same shape as `pse.semantic_id` |
| 31 | §12.5 | crates.io API | crates.io search, `q=datafusion` | third-party extenders worth adjudicating | `datafusion-table-providers` 0.13.1, `datafusion-tracing` **55.0.0** (one minor behind the engine), `datafusion-distributed` 4.0.0, `datafusion-postgres` 0.18.0, `datafusion-ffi` 55.1.0 |

---

## 17. Open items and recommended blueprint amendments

Every item carries its status against blueprint **revision 2**: **applied**, **parked** (acknowledged under a §26 review finding but unresolved), **resolved** (settled by this edition's measurements), or **new**.

### Must be settled before the affected code is written

| # | Item | Cluster | Status | Why it cannot wait |
|---|---|---|---|---|
| 1 | **Spike: do DataFusion's filter shapes over `FixedSizeBinary(16)` key columns match what a provider can recognise?** | D2 | **RESOLVED — the spike has been run** | PROBE A: key equality arrives as `rel.id = FixedSizeBinary(16, …)`, a matchable binary expression, so **`Exact` is achievable** and §5.4's promise stands. This also discharges §26 **F17**'s precondition ("the pushdown spike precedes any `Exact` claim") for the key-equality case. Two cautions carry forward: `IN` is pre-rewritten to `OR`, so a provider matching `Expr::InList` never fires; and nothing in the engine verifies an `Exact` claim, so register row 23's test wrapper is still required. |
| 2 | **Canonicalise field/schema metadata before any schema reaches the engine** | D9 | **new — and blocking for F9** | Revision 2 put a `datafusion-proto` plan fingerprint into the memo key and the pass record. PROBE C shows those bytes differ on essentially every process start once a field carries ≥2 metadata keys, which every `pse.*` relation does. Until this lands, F9's memo key never hits and `plan_fingerprints` records different values for identical runs. One constructor fixes it, and it fixes the Arrow-side `pse.contract.fingerprint` at the same time. |
| 3 | **Decide what the plan fingerprint covers, and say so in §14.2** | D9 | **new** | `LogicalExtensionCodec::try_encode_udf` is a *provided* method: by default a UDF is encoded **by name only**, so two plans invoking the same-named kernel with different `KernelSpec` bodies fingerprint identically. Either encode the kernel digest in the codec, or state explicitly that the fingerprint excludes UDF bodies and that §14.2's separate kernel digest covers them. Both are defensible; leaving it unsaid is not (**G6**). |
| 4 | **Decide the `MemoryPool` implementation and its limit** | D7 | **open — unchanged** | PROBE H confirms the payoff is larger than claimed: the resulting error names the exact configuration keys to change. The counterfactual is still an OOM kill that destroys the run. |
| 5 | **Governance test: no `TableProvider` mutation method implemented** | D1 | **open — and the surface grew** | Five defaulted mutation methods became six with `merge_into` in 55.0.0. Assert the *set*, not a fixed list. |
| 6 | **Decide whether to register the `pse.*` types in an `ExtensionTypeRegistry`** | D10 | **new** | Not blocking in the sense that nothing breaks without it — but it is the cheapest **G3** closure available (one generated function) and it changes what §4.4 means from a convention into an enforced constraint. Deciding it late means generated code written twice. |
| 7 | **Pin every crate with `=` *and* commit an enforced lockfile** | §1.2 | **half applied** | Revision 2 adopted the `=` pins, not the lockfile. `=` pins bind direct dependencies only. |

### Recommended blueprint amendments

| # | Section | Change | Status |
|---|---|---|---|
| A | §3.1 | Family-wide `=` pins; `object_store 0.13.2` is **required by** DataFusion, not chosen; note `convert_to_state` has no consumer unless UDAFs are written | **applied in revision 2**, except the `convert_to_state` note |
| A′ | §3.1 | *Additions:* require a committed, CI-enforced `Cargo.lock`; record that DataFusion 55.1.0 builds against `arrow 59.2.0`, so the platform pin runs one minor ahead | **new** |
| B | §5.4 | The provider implements no mutation method; statistics carry explicit `Precision`; `StatisticsRequest`s answered selectively; `Constraints::project` applied under projection | **parked** |
| B′ | §5.4 | *Additions:* record that DataFusion validates **no** constraint but `Field` nullability and not on ingest, so "Constraints: primary key only" is a declaration the platform's P2 makes true; record that `IN` reaches a provider as `OR`, not `InList`; record that `supports_filters_pushdown` must be pure | **new** |
| C | §14.2 | Rule 6: the pass record captures the DataFusion version and ordered rule names; the memo key includes them; state how the plan fingerprint is computed | **applied in revision 2** (F9) |
| C′ | §14.2 | *Corrections to F9:* (i) the fingerprint requires a platform `LogicalExtensionCodec` — `logical_plan_to_bytes` fails outright on a custom provider; (ii) it is reproducible **only** if field metadata is canonicalised first; (iii) state whether UDF bodies are in scope; (iv) restate rule 5's settings hash as an explicit key allow-list, since two semantic keys live under `execution.*` | **new — corrects an applied amendment** |
| D | §14.2 rule 3 | `Expr::IsUnknown`/`IsNotUnknown` cover three of four values; `conflict` is platform-level | **applied in revision 2** |
| E | §18.5 | Add `conditional_arguments`/`short_circuits`; `output_ordering`/`preserves_lex_ordering`/`strictly_order_preserving`; `coerce_types` as strict accept-or-error; `is_nullable` derived from `KernelSpec`; prohibit reading `config_options`; note `return_field_from_args` receives input **Fields** | **partially applied** — six of thirteen members named in revision 2 |
| E′ | §18.5 | *Addition:* note that 55.0 uses UDF **strictness metadata for outer-join elimination**, so a wrong `is_strict` changes results rather than merely missing an optimisation | **new** |
| F | §14.3 | Name the `MemoryPool` implementation and limit; validate `target_partitions` against the rayon budget at session construction | **parked** |
| G | §20.4 | Inject a `TimeProvider` so reproduction runs are not exposed to wall-clock time | **parked** |
| H | §23.2 | Add a DataFusion column mapping the 20 `DataFusionError` variants to platform failure classes; use `DataFusionError::Collection` | **parked** |
| H′ | §23.2 | *Addition:* ban `SessionConfig::set_str` (it panics on an invalid value) in favour of `ConfigOptions::set`; add it to the governance greps beside `Field::extension_type()` | **new** |
| I | §23.1 | Record that DataFusion emits no `tracing` spans by default — rule-level spans are platform code; operator-level needs an instrumentation layer | **parked**, now with named candidates (`JoinSetTracer`, `CustomMetricValue`) |
| J | §5.3 / §14.2 | Note that plan execution yields a `SendableRecordBatchStream` whose cross-partition arrival order is nondeterministic; canonical order is imposed by §5.3 step 1, never assumed | **parked** |
| **K** | §4.3 | **Erratum:** "nothing in DataFusion acts on these keys" is false for `ARROW:extension:name` at 55.1.0 — the engine resolves it against a registry and rejects mismatched storage types. Narrow the sentence to non-`ARROW:` keys | **new** |
| **L** | §4.4 / §14.3 | Register the ten `pse.*` types in an `ExtensionTypeRegistry` at session construction, generated from §4.4's table; assert no duplicate name; note that seven canonical `arrow.*` types are preloaded by default | **new** |
| **M** | §5.3 / §26 | **Erratum:** "`-0.0` is preserved" holds for Arrow ordering and for the content hash, and **fails under `GROUP BY` and hash joins**, which merge it with `+0.0` and treat NaN as self-equal. Either declare that `Float64` columns are never grouping or join keys, or record the collapse as a selected loss | **new** |
| **N** | §4.5 | Clarify that §4.5 catalogs *Arrow storage* types, and that DataFusion has a separate `LogicalType`/`NativeType` layer above them used for signature coercion — the two are compatible but must not be conflated | **new** |
| **O** | §23.1 / §19 | Set `datafusion.explain.format = pgjson` where a pass record stores a plan, so the plan is queryable structure rather than text; `analyze_level` and `analyze_categories` make the verbosity declared policy | **new** |
| **O′** | §14.2 rule 5 / §6.11 | §14.2 names `Optimizer::with_rules` but not `Analyzer::with_rules` — the 3 analyzer rules are installed separately and change plans too, so `reference.engine_profiles` must capture **both** ordered lists. Also record the **per-rule `observer`** on `Optimizer::optimize` / `Analyzer::execute_and_check`, which turns the declared rule set into evidence of which rules actually fired — DM-46, DM-50. And note that `Analyzer::add_function_rewrite` mutates an analyzer after construction and must not be used, since it escapes any list captured at construction | **new** |
| **P** | §3.3 | Record the rejections that look like wins: `datafusion-table-providers` and the external-source integrations (a second mutable authority, **G1**), `datafusion-spark`/`enable_ansi_mode` (silently different arithmetic, **G2**), Parquet virtual columns and `input_file_name` (position-based identity, **G1**), `FunctionFactory` (SQL-defined kernels, **G1**), `AsyncScalarUDFImpl` (I/O inside expression evaluation) | **new** |

### Unresolved / unverified

| Item | Status |
|---|---|
| Filter match shapes on extension-typed key columns | **closed** — PROBE A. `Exact` is reachable for key equality; `IN` arrives as `OR` |
| Whether `LogicalPlan::Extension` nodes survive the optimizer intact enough to carry `rule_id` | **closed** — PROBE B. They do, and the optimizer still rewrites through them; physical execution needs an `ExtensionPlanner` |
| Stability of `LogicalPlan`'s `Display`/`EXPLAIN` rendering across releases | **superseded** — revision 2 chose `datafusion-proto` instead, which PROBE C shows has its own stability problem with a known cause and a cheap fix |
| The physical-optimizer rule trait | `[UNVERIFIED]` — `EnsureRequirements` resolves as a struct; the rule trait still did not surface in the extraction. Only needed if a custom physical rule is written |
| `Interval`'s public representation for `evaluate_bounds`/`propagate_constraints` | `[UNVERIFIED]` — did not resolve to a field list; read before writing range-analysis adapters |
| Whether `datafusion-proto` bytes are stable across DataFusion **releases**, not merely across processes | `[UNVERIFIED]` — this edition measured within-version stability only. A release upgrade changing the protobuf representation would invalidate every memo key, which may be the desired behaviour (F9 wants engine version in the key anyway) but should be stated rather than discovered |
| Whether registering `pse.*` types affects plan or execution cost | `[UNVERIFIED]` — the registry is consulted during planning; the overhead was not measured |
