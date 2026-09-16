# Design review: schema engineering for typed domain values across DataFusion and Delta

## 1. Decision and scope

**Decision: Revise.** The registry already declares nested Arrow values widely; the
remaining leverage is not "more nesting" but declaring correspondences that today live
in conventions, persisting the declaration into the durable table, removing
self-inflicted durable conversions, and carrying row-varying meaning as structure rather
than as field metadata that every native function must be taught to preserve. Three
MUST-level gaps (DM-07, DM-09, DM-42) are open on implemented paths; the relation
reshaping the capability document argues for is a measured hypothesis, not a defect.

**Proposal under review.** Systematically leverage Arrow nested types (`Struct`, `List`,
`FixedSizeList`, extension types), DataFusion's full-field planning surface and Delta's
table capabilities so that coupled facts are typed domain values with one contract, as
argued in `docs/capability-maps/struct_schema_review.md`. The maintainer asked that
existing policies not constrain the solution space; this review therefore judges the
design against the charter only and defers governance consequences to a later decision.

**Status.** Document claims: *Proposed*. Library routes: *Interface-checked* at the pins
below. Codebase paths: *Implemented* where cited by file and line; nothing in this review
is *Tested* or *Measured* unless a named test is cited.

**Reviewer.** Claude (Fable 5.1), for the sole maintainer. **Date:** 2026-09-16.

**Affected revisions.** Blueprint revision 39 (§4, §6.9, §6.11, §6.13, §18.1–18.2, §20,
§21), proposed ADR-0068, Plan 07 UD02/UD03/UD07, the `pse-schema` registry, the
`pse-catalog` session and Delta layers, `pse-mathir`, `pse-numerics`,
`pse-backend-native`, `python/pse`.

**Pins.** DataFusion 55.1.0, Arrow/Parquet 59.3.0, object_store 0.13.2, delta-rs
`58f07cd62bfbce3649a7e1c87c696288068ae184`, delta kernel
`8ba063f8f84fec222000f66d40d70911d7c79675` (both from `Cargo.lock`). Library facts
below cite the skill indexes under `.claude/skills/{datafusion,deltalake}/content/` or
the pinned source in the local cargo checkouts; they say nothing about other releases.

**Observable outcome sought.** Fewer independently maintained statements of what a
coupled value is; invalid combinations rejected structurally or by one generated check
rather than by prose; a durable table that describes itself; fewer conversions and
reassembly passes between declaration, plan, disk, solver and Python.

**Baseline.** One registry (`crates/pse-schema/src/catalog/`, 562 relations across seven
namespaces per `docs/generated/relations/*.md`) declares 189 nested-typed columns and
eleven extension types, several with nested storage (`crates/pse-schema/src/model/extension.rs:79-160`).
The logical type catalog admits `List`, `FixedSizeList`, `Struct` and the extension uses
(`crates/pse-schema/src/model/logical_type.rs:27-62`) and rejects `Map` and `Union`
(`logical_type.rs:397-398`; `crates/pse-ids/src/contract.rs:322`). Nested values cross
DataFusion plans through a family of field-preserving wrapper functions
(`crates/pse-catalog/src/session/scalar/`), reach Delta through a generated durable
projection (`crates/pse-schema/src/delta.rs`, `crates/pse-catalog/src/delta/layout.rs`)
and reach Python as an Arrow C stream with generated nested attrs classes.

**Supported scope and non-goals.** In scope: representation of coupled facts, semantic
transfer through plans, the durable Delta declaration, identity of composite-key rows,
the numerical output contract, and the Python projection. Out of scope: publication
atomicity, retry reconciliation and retention (reviewed on 2026-09-15 as F01/F04/F12 of
`design_review_unified-datafusion-delta_2026-09-15.md`); the unfinished simulator
backends; governance and policy text.

**Constraints and uncertainty.** Delta and Parquet store no `Union`; Delta has no
unsigned integers; the pinned Delta scan does not push expression projections into file
reads; nested-projection I/O savings are unmeasured; seven crates named by the blueprint
are stubs, so their contracts are document-stage only.

### Method and coverage

Read directly: charter, directive, template; blueprint §1, §3.3.3, §4, §5.4, §6.4, §6.7–
6.13, §7.5–7.6, §18.1–18.2, §20, §21; Plan 07 context and UD02/UD03; ADR-0068; the two
prior reviews' findings; the capability document under review; registry model files
(`logical_type.rs`, `extension.rs`, `relation.rs`, `invariant.rs`), catalog declarations
for math, numerical, runtime, cases, property and configuration relations; `delta.rs`;
catalog `delta/{layout,contract,publication_plan,admission}.rs`; session
`{admission,output,query_schema,registry,scalar,scalar/*}.rs`; `pse-relations/src/cells.rs`;
`pse-mathir/src/{lib,payload}.rs`; `pse-numerics/src/expressions.rs`;
`pse-backend-native/src/driver{,/workspace}.rs`; `pse-compiler` sites cited in §7;
`pse-rules/src/{derivations,invariants,strata/native/output}.rs`;
`pse-authoring/src/document/load.rs` and `p1/mod.rs`; `python/pse/codec/__init__.py`,
`python/pse/contracts/{values,compiled}.py`; pinned delta-rs `writer/stats.rs`,
`delta_datafusion/data_validation.rs`, `operations/write/generated_columns.rs`,
`table_provider/next/scan/expr_adapter.rs`; pinned kernel `engine/arrow_conversion/mod.rs`;
Arrow 59.3 `arrow-cast/src/cast/mod.rs` and `arrow-array/src/array/struct_array.rs`;
DataFusion 55.1 `datafusion-datasource-parquet/src/source.rs`.

Two read-only survey agents located candidate sites; every site cited here was then read
by the reviewer at the cited lines. Not inspected: generated trees, the seven stub crates,
`pse-templates`, `pse-material`, `pse-structural`, test bodies except the one cited as an
illustration of a consumer pattern, and any benchmark. No probe, test or benchmark was
executed for this review; every "would" in §7 is a stated consequence, not an observed
failure. Guarantees not attacked: Delta commit atomicity, cancellation, memory
accounting.

## 2. Authority and lifecycle map

| Concept or fact | Semantic type and identity | Authority / owner | Revision or snapshot boundary | Permitted update path | Derived representations |
|---|---|---|---|---|---|
| Relation shape (columns, types, nullability, keys) | `RelationSpec` with fingerprint | `pse-schema` catalog declarations | registry version, per-relation `@n` | edit declaration, `just codegen` | Arrow schema with `pse.*` metadata, typed views, builders, Python classes, docs |
| Extension type meaning and storage | `EXTENSION_TYPES[11]` | `extension.rs:172` | metadata `v` | new generation accepted by factory | Rust `ExtensionType` impls, DataFusion registrations (`session/registry.rs:23-63`), pyarrow classes |
| Column quantity contract | `QuantityContract::{None,Column,PerRow}` | `relation.rs:87-98` | with relation | edit declaration | `pse.semantic.quantity_type` metadata; for `PerRow` a sibling column named by convention (`relation.rs:93-97`, `:216`) |
| Tag/payload consistency of alternatives | prose | blueprint §6.9, §6.13 comments; none in registry | — | **no declared path** (invented cell) | none: writer conventions (`builder.rs:1019-1031`) |
| Which semantic metadata survives which native operator | §4.6 transfer table | blueprint §4.6 | — | edit blueprint | implemented twice: per wrapper (`session/scalar/*`) and in `session/admission.rs:234-362` |
| Durable Delta schema | generated projection | `pse-schema/src/delta.rs:23-92` | with relation | regenerate | Delta `StructType` **without** field metadata (`delta.rs:31-36`); fingerprint only inside the CHECK name (`delta/contract.rs:46-58`) |
| Row identity for composite keys | JSON text `row_key` | `pse-rules/src/derivations.rs:60-75` codec | codec `VERSION` | change codec | in-plan re-encoding (`session/scalar/encode.rs:189-211`) |
| Expression DAG | `math_expr_nodes` + `math_expr_args` + 14 payload relations, ×6 families | registry (`s6_9_math.rs`, `expr_family.rs:286-373`) | derived stage | pass output | in-memory `Payload` union (`pse-mathir/src/payload.rs:57`), Python row classes |
| Evaluation program output | column names `residual_{r}`, `jacobian_{r}_{c}` plus `Vec<(usize,usize)>` | `pse-numerics/src/expressions.rs:78-97` | per compile | recompile | Ipopt triples reconstructed from offsets (`driver/workspace.rs:92-97`) |
| Publication vector | `runtime.publications.members: list<struct{…}>` | `catalog/publication.rs:20-30` | one Delta commit | conditional publish | control row built with `array_agg … order_by` and `named_struct` (`delta/publication_plan.rs:60-140`) |

**Deliberately opaque behavior.** The Delta CHECK body is a registered UDF whose
implementation rebuilds a `RecordBatch` and runs the full recursive validator
(`delta/contract.rs:201-219`); no other engine can evaluate it. Kernel implementations
remain opaque by contract (blueprint §18.5); their contracts are declared as nested
values in `reference.kernel_specs`, which is the right placement.

**Identity behavior.** Reordering struct children: the durable projection's guard uses
`equals_datatype` (`delta/layout.rs:129`), which ignores nested names and metadata, while
Arrow's struct cast matches by name and falls back to position when names differ
(`arrow-cast-59.3.0/src/cast/mod.rs:2276-2298`). Reordering rows: authored source spans
are held in a side vector aligned by ordinal (`pse-authoring/src/document/load.rs:37`,
rejoined at `p1/mod.rs:233-239`), so row order is identity for spans. Renaming a
relation or column: the fingerprint changes; nothing in the durable table records the old
one except the CHECK constraint's function name.

## 3. Semantic contracts and invariants

| Contract or invariant | Representation | Enforcement boundary | Failure behavior | Verification evidence |
|---|---|---|---|---|
| A rule literal has exactly the payload its `literal_kind` names | `literal_kind` + seven nullable columns (`s6_11_numerical.rs:178-203`) | none declared; single writer (`builder.rs:1019-1031`) | undefined for an inconsistent row | Proposed |
| Kernel outcome: success iff `value` non-null; failure iff `reason_code` present | enum + nullable `value` + nullable `reason_code` (`s6_13_runtime.rs:247-266`) | blueprint §6.13 prose only; no registry invariant (census of `catalog/invariant*.rs`) | none | Proposed |
| `incidence.coefficient` present iff `linear` | bool + nullable f64 (`s6_12_derived.rs:145-160`) | none | none | Proposed |
| Equation `sense` agrees with which of `lower_node_id`/`upper_node_id` exist | enum + two nullable ordinals (blueprint §6.9, `s6_9_math.rs:462-496`) | none in registry | none | Proposed |
| `config_values.value.kind` agrees with its populated child | struct of `kind` + ten nullable children (`s6_15_semantic/configuration.rs:176`, docs) | semantic invariant plan (`invariant_semantic.rs:58`) | commit refused | Implemented |
| `math_expr_args.argument_ordinal` is contiguous from 0 per parent | relation key `(parent, ordinal)` | contiguity nowhere declared; loader remaps (`pse-mathir/src/relations/load.rs`, per survey; not read at line grain) | consumer-defined | Proposed |
| Every node with opcode X has exactly one row in X's payload relation | separate relations | none in registry | consumer-defined | Proposed |
| Unbounded is `pse.bound{kind}`; never NaN/null/±inf | struct extension (`extension.rs:112-127`) | generated validators; backend adapter maps absent to ±`f64::MAX` explicitly (`driver.rs:240-246`) | typed error | Implemented; negative tests not cited |
| A `PerRow` quantity column has its sibling `<name>_quantity_type_id` | naming convention (`relation.rs:93-97`, `:216`) | registry assembly rejects a missing sibling | declaration error | Implemented |
| Declared semantic metadata survives an admitted plan | `pse.semantic.*`, `ARROW:extension:*` | output boundary (`session/output.rs:285-312`), wrappers, cast normalization | plan refused | Implemented; per-operator coverage unknown |
| Non-nullable nested children hold no unmasked nulls after Delta read | Arrow `StructArray::try_new` (`arrow-array-59.3.0/src/array/struct_array.rs:169-175`) reached through `DurableCast` | cast error | error | Interface-checked; Delta scan relaxes nested nullability before the cast (`delta-rs …/next/scan/expr_adapter.rs:31-41`) |
| Durable widths/signs are lossless | `UInt*→Int*/Decimal128(20,0)`, `FixedSizeBinary→Binary` (`delta.rs:41-45`) | `DurableCast` with `safe:false` and null check (`delta/layout.rs:181-214`) | error | Implemented |

**Absence and uncertainty.** The design distinguishes unbounded from missing for bounds
and decided-true from undecided for inference (blueprint §7.6). It does not distinguish,
at the type level, "not yet computed" from "not applicable" for
`math_expr_nodes.quantity_type_id` (nullable before P10, required after; one schema at
several stages, blueprint §4.1), nor "success with null value" from "failure without
reason" in `kernel_evaluation_outcomes`, nor an absent payload from a wrong-family payload
on an expression node. Publication members are complete-or-absent as one list, which is
correct.

**Equivalence requirements.** Registry identity is structural over declared cells;
durable equivalence is value-lossless after cast; struct equivalence at the durable guard
is storage-only (`equals_datatype`), which is weaker than the declared contract.

## 4. Derivation and execution design

| Stage or operation | Input revisions and dependencies | Output contract | Preconditions / assumptions | Effects and mutable ownership | Provenance / invalidation |
|---|---|---|---|---|---|
| Declaration → Arrow schema | registry | `Schema` with `pse.contract.*` and per-field `pse.semantic.*` (`pse-schema/src/arrow.rs:56-160`) | closed logical type catalog | none | fingerprint |
| Plan admission and semantic transfer | plan, registry, session | admitted plan whose output fields carry declared metadata | each operator's transfer known: wrappers or analyzer restore | none | per plan |
| Durable encode/decode | execution schema | storage schema (`delta.rs`), `DurableCast` per column (`layout.rs:165-214`) | `equals_datatype` per field (`layout.rs:129`) | none | none recorded on disk beyond CHECK name |
| Delta write with CHECK | storage batch | committed version | `pse_check_<fingerprint>` registered in session (`contract.rs:88-95`) | table commit | commit metadata `pse.attempt` (`publication_plan.rs:80`) |
| Cold read | table version, registry | execution schema restored from registry | registry identical to the writer's | none | none in table |
| Python export | admitted stream | C stream; `FieldTransfer` report (`python/pse/_transfer.py:14-25`) | consumer registered extension classes | one-shot stream | — |
| Numerical evaluation | program, one-row batch | wide row of named Float64 columns (`expressions.rs:78-97`) | schema equal, no nulls | reservation lease | — |

**Relationship structures.** Expression children are dataflow edges stored as an edge
relation; payload relations are ownership by 1:1 optional row; provenance support is a
list of typed `(relation_id, row_key)` pairs whose `row_key` is text; publication members
are ownership as a nested list. These are kept distinct, which is aligned (DM-34).

**Provider selection and limitations: what the pinned libraries supply for nested values.**

| Capability | Status at pin | Evidence |
|---|---|---|
| Full-field UDF return (`return_field_from_args(ReturnFieldArgs{arg_fields, scalar_arguments}) -> FieldRef`) | available | `content/traits/ScalarUDFImpl.md`; `content/api/datafusion_expr.udf.md` §ReturnFieldArgs |
| `struct_field_mapping` for ordering propagation through struct constructors | available, unused in workspace | same trait page; zero hits in `crates/` |
| `ExprSchemable::to_field` / `metadata` | available, used | `content/api/datafusion_expr.expr_schema.md:41-45`; `session/admission.rs:341,505-522` |
| Name-based struct coercion and comparison | available | `content/corpus/guides/user-guide/sql/struct_coercion.md` |
| `nested_struct::cast_column`, `validate_struct_compatibility`, `adapt_batch_to_schema` | available | `content/api/datafusion_common.nested_struct.md` |
| `unnest_columns_with_options` with `NullHandling::{Drop,Preserve,PreserveAndExpandEmpty}` and recursion depth | available, used | `content/api/datafusion_common.unnest.md`; `pse-rules/src/plan/lower.rs:416` (per survey) |
| Higher-order lambdas (`array_transform`, `array_filter`, `any_match`, `arrays_zip`) | available | `content/catalogs/sql-functions.md`; `content/api/datafusion_expr.higher_order_function.md` |
| Extension type registry (`MemoryExtensionTypeRegistry`, `DFExtensionType`) | available, installed | `content/api/datafusion_expr.registry.md:125-146`; `session/registry.rs:23-63` |
| Leaf-expression pushdown toward scans (`datafusion.optimizer.enable_leaf_expression_pushdown`, default true) | available | `content/catalogs/config-options.md:154` |
| Parquet source accepts pushed `ProjectionExprs` | available | `datafusion-datasource-parquet-55.1.0/src/source.rs:692-702` |
| Delta scan accepts pushed `ProjectionExprs` | **absent** | no `ProjectionExprs`/`try_pushdown_projection` under `delta-rs@58f07cd/crates/core/src/delta_datafusion/` |
| `Union` in memory (`union_extract`, `union_tag`) | available in DataFusion; not storable in Parquet or Delta | `content/index/symbols.tsv`; kernel `DataType` has no union (`content/api/buoyant_kernel.schema.md:39`) |
| `Map` | DataFusion and Delta support it; registry rejects it | `logical_type.rs:398`; `pse-ids/src/contract.rs:322` |
| Variant | Arrow 59.3 ships `parquet_variant_compute` (`VariantArray`, `shred_variant`); kernel has `DataType::Variant`; delta-rs reads Spark variant tables; no DataFusion variant functions indexed; write path not evidenced | `content/index/symbols.tsv`; `content/corpus/tests/variant.rs` |
| Field metadata preserved Arrow→Delta→Arrow | kernel copies all field metadata both ways | `delta-kernel-rs@8ba063f8/kernel/src/engine/arrow_conversion/mod.rs:40-60,181-186,422-482` |
| Kernel type narrowing | `UInt64→LONG` ("undocumented"), `UInt32→INTEGER`, `FixedSizeBinary→BINARY`, `Dictionary` collapsed | same file `:527,559-568,639` |
| Nested per-leaf statistics; top-level column consumes one indexing slot; binary min/max emitted | available | `delta-rs@58f07cd/crates/core/src/writer/stats.rs:160-200,348-356,1155` |
| CHECK constraints parsed as DataFusion SQL against the table `DFSchema` | available | `…/delta_datafusion/data_validation.rs:433-441` |
| Generated columns computed on write when the feature is enabled | available | `…/operations/write/generated_columns.rs:38-63` |
| Nested nullability relaxed on Delta read | present | `…/table_provider/next/scan/expr_adapter.rs:11-41` |
| Table features incl. `RowTracking`, `DomainMetadata`, `TypeWidening`, `ClusteredTable`, `VariantType` | enumerated; write support per feature not verified here | `content/catalogs/table-features.md` |
| `add_columns`, `update_field_metadata`, `set_tbl_properties`, `add_constraint`, `add_feature` | available | `content/catalogs/operations.md` |
| Struct-to-struct cast: by name, else positional fallback | Arrow 59.3 | `arrow-cast-59.3.0/src/cast/mod.rs:2269-2298` |

**Boundary contracts.** Execution fields carry semantics; durable fields carry none;
Python receives execution fields through the C stream and reports loss per field. The
numerical boundary carries structure in column names and a side vector.

**Coherent publication.** Covered by the 2026-09-15 review; the nested member list is the
part relevant here and is aligned.

## 5. Representative journeys

### Ordinary extension: a new operator payload family

Today: a new `math_<family>` relation is declared six times through `expr_family.rs`
(`:286-373`), a `Payload` variant is added (`payload.rs:57`), sink and source callbacks
gain a method (`pse-mathir/src/relations/mod.rs`, per survey), the generated Python class
appears, and the opcode↔payload existence rule stays unwritten. Under the correction in
F01/F02, the family is one arm in a declared tagged payload; the union variant, the
existence check, the CHECK and the Python class are generated from that one arm. That is
one semantic declaration plus the kernel that evaluates it.

### Meaningful change: a kernel outcome gains a new failure class

Today: add an enum member; the "reason_code present iff not success" rule remains prose.
Under F01: the arm declaration names required and forbidden children; the generated
CHECK, invariant plan and Rust accessor change together; a fixture with `success` and a
null value fails at every write route.

### Boundary: an expression node from compiler to Delta to Python

`node_id: u64` becomes `Decimal128(20,0)` on disk (`delta.rs:44`) and back through two
casts (`layout.rs:181-214`); child edges are read as a second table and rejoined by hand in
Python (`python/pse/tests/test_engineering_inspection.py:60-70` illustrates the consumer
pattern); the Python validator carries the unsigned range
(`python/pse/contracts/compiled.py:429`). With `children: list<i64>` and a folded payload,
one row is one self-contained node value on every side.

### Failure: a struct with wrongly named children reaches the durable projection

`project` accepts the input because `equals_datatype` ignores child names
(`layout.rs:129`); `cast_with_options` finds no name match and casts positionally
(`arrow-cast … :2295-2298`); the table receives relabelled children with no error. The
declared contract was not enforced at the boundary that claims it.

## 6. Acceptance gates

| Gate | Pass / fail / unresolved / not applicable | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | Unresolved | The rule for which semantic metadata survives an operator is decided independently in each wrapper (`session/scalar/*`) and again in `session/admission.rs:234-362`; both derive from blueprint §4.6 but no artifact links them (F03) | One transfer table, one consumer |
| G2 — Semantic fidelity | Fail on the durable guard; unresolved on alternatives | `layout.rs:129` plus positional cast fallback can relabel struct children silently (F09); tagged alternatives admit indistinguishable inconsistent rows (F01) | Full-field guard; declared alternatives |
| G3 — Validity | Unresolved | No enforcement exists for the tag/payload and ordinal-contiguity invariants in §3; the only producers today are internal, so the invalid state is not yet reachable from user data | Generated checks before external writers exist |
| G4 — Hidden behavior | Pass for inspected paths | Validation UDFs are `Immutable` and error rather than mutate; inspection paths read metadata only | — |
| G5 — Consistency and recovery | Not applicable | Reviewed 2026-09-15 (F01, F04, F12 there); nothing in this scope changes it | — |
| G6 — Transformation and reuse | Unresolved | Per-operator metadata transfer is untested as a suite; a UNION keeps only metadata common to all branches (`content/api/datafusion_expr.expr.md:639-667`), which the wrappers must know | Operator-by-operator transfer conformance |
| G7 — Truthful capability claims | Pass with scope | Blueprint §3.3.3 claims "lossless named conversions"; values are lossless, the durable schema carries no semantic metadata (F04); the capability document's nested-projection benefits are hypotheses (F11) | Narrow the claim or persist the declaration |

## 7. Principle findings

Ranked by correctness and authority, then semantic duplication and extension locality,
then measured cost. Evidence labels: code paths are *Implemented*; consequences are
*Proposed* unless a test is named.

| Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|
| **F01 — Tagged alternatives are conventions, not contracts** | DM-07, DM-08, DM-09, DM-52 — Violated (rule literals, kernel outcomes, incidence, equation bounds); Satisfied for `config_values` | `rule_expr_nodes`: `literal_kind` plus seven nullable payload columns (`s6_11_numerical.rs:178-203`), written by a `match` (`builder.rs:1019-1031`); `kernel_evaluation_outcomes` (`s6_13_runtime.rs:247-266`) whose rule exists only as a blueprint comment; `incidence.linear`/`coefficient` (`s6_12_derived.rs:145-160`); `math_equations.sense` with nullable bound ordinals; `math_implicit_systems` two parallel id lists with squareness unstated (`s6_9_math.rs:514-532`). Registry invariant census names none of these; only `config_values` and `instance_features` have a semantic plan (`invariant_semantic.rs:58`) | A row with `outcome = success` and a null value, or a `range` equation with one bound, is representable and undetected; a consumer aggregating `value` treats the failure as a missing measurement, the exact conflation blueprint §7.6 forbids | A registry `TaggedAlternative` declaration: tag column or child, arms with required and forbidden children, cardinality equalities. Generate from it (a) the execution layout as a struct of optional arm structs (durable in Delta; in-memory `Union` optional later), (b) native CHECK SQL, (c) the invariant plan, (d) the Rust enum accessor and Python class. Surface: ~5 declarations, one generator branch | Negative rows rejected through every write route; a DataFusion session with no PSE UDF evaluates the generated CHECK |
| **F02 — Expression graphs keep structure in ordinal edge tables and per-opcode side relations** | DM-09, DM-18, DM-56 — Violated for the unstated invariants; layout change is a hypothesis | `math_expr_args` `(parent, ordinal) → child` (`s6_9_math.rs:104-118`); fourteen payload relations cloned across six families (`expr_family.rs:286-373`); the in-memory truth is already a union (`payload.rs:57`) and an ordered child vector (`node.rs:7-8`, per survey); the Python side re-derives the join by hand | Ordinal contiguity and opcode↔payload existence are invariants nobody declares; each family adds ~16 relations; every consumer performs the same rejoin; provenance and Python consumers cannot treat a node as one value | Fold `children: list<int64>` onto the node row (order is structural, exactly as `math_affine.terms` already does) and fold payloads into one tagged struct column via F01, giving one relation per family; keep the node table, `subtree_hash` and hash-consing. Reading only `opcode` still reads one Parquet leaf; reading one payload arm reads the whole `payload` column under the pinned Delta scan (no nested projection pushdown) | Relation and invariant counts; plan inspection; `EXPLAIN ANALYZE` bytes read for opcode-only, arm-only and whole-node scans before and after; equal canonical hashes |
| **F03 — Field metadata is the only carrier of semantics, so every native function needs a PSE twin** | DM-02, DM-23, DM-56, DM-57 — Violated (second authority for transfer rules) | Nine wrapper modules plus an aggregate wrapper (`session/scalar.rs:1-80`); `pse_named_struct` re-zips argument fields into the return field (`scalar/structure.rs:52-73`); `pse_preserve_field`, `pse_retain_metadata`, `pse_list_field`, `pse_require_nonnull`; role/FK keys stripped by a UDF at output (`session/output.rs:25-50`); cast metadata normalized in the analyzer (`session/admission.rs:332-362`); "repairs the pinned engine's lost list-child metadata" (`:234`). The registry's extension registration is passive (`session/registry.rs:23-63`); `struct_field_mapping` is never implemented | Using any of the array, map, struct or lambda functions the pin ships requires writing a wrapper first, so extension locality fails on the mechanism meant to enable it; two places decide what survives a rewrite and can drift | (a) Carry row-varying meaning as structure: `PerRow` quantities become `pse.quantity_value` struct columns, tagged alternatives per F01, so the meaning survives any function without help. (b) One analyzer rule that implements the §4.6 transfer table keyed by native function identity, replacing the wrappers whose only job is metadata. (c) Implement `struct_field_mapping` on the struct constructors that remain | Delete a wrapper and admit the same plan with identical output fields; admit a plan using `array_sort` and `array_transform` over a quantity list; operator-by-operator transfer conformance suite including UNION's intersection rule |
| **F04 — The durable Delta declaration drops the semantic contract it was generated from** | DM-42, DM-51, DM-55 — Violated | `storage_field` builds `Field::new(name, type, nullable)` with no metadata (`delta.rs:31-36`); dictionaries collapse to their values (`:48`); the only durable anchor is the fingerprint inside the CHECK function name (`delta/contract.rs:46-58`, verified by string equality at `:105-109`); no table properties are set. The kernel preserves Arrow field metadata in both directions (`arrow_conversion/mod.rs:40-60,181-186,422-482`), so the loss is self-inflicted | A cold table describes itself only to a process holding the identical registry; blueprint §20.5's explicit schema evolution has no in-table version to evolve from; external Arrow readers see `Binary` and `Utf8` with no extension names, and a registry mismatch is detected, if at all, by a constraint-name string compare | Persist `ARROW:extension:*` and `pse.semantic.*` into the Delta `StructField` metadata; write `pse.contract.{id,version,fingerprint}` as table properties; make cold open reconcile table declaration against registry explicitly (match, migrate, refuse). Surface: `delta.rs`, table creation, cold-open path | Open a table with `pyarrow`/`deltalake` and observe extension names; property round trip; a registry-mismatch fixture refuses with a `schema.*` code |
| **F05 — Unsigned ordinals and fixed-width identities force the widest durable conversions on the hottest keys** | DM-36, DM-42, DM-56 — Violated (cost without semantic gain) | `UInt64 → Decimal128(20,0)`, `UInt8/16/32 → Int16/32/64`, `FixedSizeBinary → Binary` (`delta.rs:41-45`); `DurableCast` casts twice per column each way (`layout.rs:181-214`); the kernel maps `UInt64` to `LONG` and calls it undocumented (`arrow_conversion/mod.rs:559`); Python validators carry `integer_range(0, 18446744073709551615)` (`contracts/compiled.py:429`) | Every node, child and ordinal key is a 16-byte decimal on disk and cast on every read and write; no engine outside PSE can join on the declared type; non-negativity is a domain rule expressed as a machine type | Declare artifact-local ordinals and counts as `i64` with generated `>= 0` checks; remove unsigned types from the durable-eligible catalog. Keep `FixedSizeBinary(16)` identities (Delta emits binary min/max: `stats.rs:348-356`); treat a `Struct<hi:i64, lo:i64>` identity layout as a hypothesis to measure, not a change | Durable schema equals execution schema for integer columns; `DurableCast` becomes identity there; join and scan benchmark on `math_expr_args` before and after |
| **F06 — Composite-row identity is a JSON string** | DM-10, DM-11, DM-15 — Violated | `provenance.derivations.row_key` and `supporting[].row_key` are text (`s6_13_runtime.rs:315-324`) encoded as `["v",[[col,literal],…]]` (`pse-rules/src/derivations.rs:60-75`); relations are re-encoded to that text inside plans by a UDF with its own type-tag table (`session/scalar/encode.rs:189-211`); `inferred.undecided.key` is the same text (blueprint §6.7) | Lineage and support joins compare JSON text; every relation joined to provenance pays an in-plan encode; a codec formatting change changes identity; Delta statistics on a text key prune nothing useful | Generate a `row_key: pse.content_hash` (or a row semantic id) on every relation from the canonical key cells the codec already orders; make `derivations`/`undecided` reference `(relation_id, row_key)` with typed FK; keep the JSON as a display formatter | Provenance plans contain no `pse_encode`; hash stability test across batch layouts; FK invariant over `supporting` |
| **F07 — Positional and name-encoded structure at the numerical boundary** | DM-01, DM-09, DM-37 — Violated (structure in names); cost is a hypothesis | Output columns named `residual_{row}` and `jacobian_{row}_{column}` with the sparsity pattern in a side `Vec<(usize,usize)>` (`expressions.rs:78-97`, `:156-162`); `(result_column, iRow, jCol)` reconstructed from an offset convention (`driver/workspace.rs:92-97`); one `Float64Array` allocation per output column and a downcast loop per Ipopt callback (`:129-146`) | The program's output contract is not typed: `m`, `nnz` and the row/column pairing exist only in naming and vector order; two consumers must agree on the offset rule | Type the output: `residuals: FixedSizeList<Float64, m>`, `jacobian: FixedSizeList<Float64, nnz>` per scenario row, with the coordinate pattern as a program-level typed relation (blueprint §18.1 already names `sparsity_patterns`); the Ipopt copy becomes one contiguous slice per callback. The explicit `±f64::MAX` adapter for unbounded (`driver.rs:240-246`) is a declared conversion and stays | Program schema test; callback allocation count and per-evaluation time on a representative problem (DM-39) |
| **F08 — In-memory parallel correspondences that a column would make structural** | DM-09, DM-11 — Violated | `Document.row_spans: BTreeMap<_, Vec<SourceSpan>>` aligned to emitted row order (`document/load.rs:37`) and rejoined by ordinal (`p1/mod.rs:233-239`); a nine-cell diagnostic built positionally (`records.rs:280-305`); per-row coefficient reassembly re-locating five string source keys and a list per row (`coefficients/rows.rs:45-106`); ownership recovered by two anti-joins and an inner join on a constructed ordinal (`native_outputs.rs:488-518`); a tuple re-materialized as one join per dimension (`parameter_indices.rs:107-170`) | Row order is identity for spans; a diagnostic column reorder silently misfiles fields; support recovery is repeated per row in Rust after the plan already had it | Emit `span: pse.source_span` as a column of the authored batch; build diagnostics through the generated builder; emit `support: list<struct<relation_id, row_key>>` (F06 keys) from the plan instead of per-row lookup | No `Vec<SourceSpan>` beside a batch; `records.rs` uses the builder; the ordinal join chain reduces to one |
| **F09 — The durable guard compares storage only while the cast falls back to position** | DM-42, DM-53 — Violated | `equals_datatype` at `delta/layout.rs:129` ignores nested names and metadata; Arrow's struct cast matches by name and otherwise casts positionally (`arrow-cast … cast/mod.rs:2295-2298`); the capability document's "equality trap" (§4) describes exactly this | A struct whose children are named differently but typed compatibly is relabelled silently on the way to disk or back | Compare full fields (names, nullability, metadata) as `declared_output` already does (`layout.rs:20-30`), or use `nested_struct::validate_struct_compatibility`; add the renamed-children negative test | Negative test fails today, passes after |
| **F10 — Durable validity is one opaque UDF instead of declared native checks** | DM-04, DM-42, DM-55, DM-58 — Violated | CHECK `pse_contract` is `pse_check_<fingerprint>(cols…)` (`contract.rs:46-58`); its body rebuilds a batch and runs the recursive validator (`:201-219`); delta-rs parses constraints as DataFusion SQL over the table schema (`data_validation.rs:433-441`) and computes generated columns on write (`generated_columns.rs:38-63`), neither of which is used for the expressible predicates (enum membership, bound kind/value agreement, tag/payload, non-negativity) | No engine without the PSE function can validate, plan or even describe the table's local contract; the fingerprint is smuggled through a function name; Delta-native evaluation and future generated columns are foreclosed | Generate native CHECK SQL for every predicate SQL can express (from F01/F05 declarations); keep the UDF only for recursive value admission; record the fingerprint as a property (F04) | A PSE-free session evaluates the native checks; the same invalid rows are rejected on each public write route |
| **F11 — The capability document's claims need pin labels** | DM-39, DM-59 — Unresolved | Verified at the pin: `return_field_from_args`, `ReturnFieldArgs{arg_fields, scalar_arguments}`, `struct_field_mapping`, `to_field`, `named_struct`/`get_field`, `unnest_columns_with_options`, scan projection as `Option<&[usize]>` (`content/api/datafusion_session.table.md`), `equals_datatype` semantics. Not established: the "August 20, 2026 struct filtering fix with a branch-55 backport" (unverifiable offline); nested Parquet I/O savings; that "nested access paths" are pushed to Delta files (they are not at this pin, see §4) | Read without labels, the essay could be cited as evidence in an ADR | Add a header naming the pins and marking each library claim Interface-checked or Proposed; keep it as a design essay, not a capability map | Header present; each §7 row above that cites it also cites a pinned source |

**Strengths, stated as what would break without them.** Without the `pse.bound` struct
(`extension.rs:112-127`) the unbounded case would be a NaN or null sentinel, which §7.6
forbids and which Ipopt would misread. Without `publications.members` as one nested list
(`publication.rs:20-30`) a reader could pair members from different commits. Without
`pse_named_struct`'s field re-zipping (`structure.rs:52-73`) every struct construction
would lose units; the finding is that this should be one rule, not that the rule is wrong.
Without non-nullable list items by declaration (`logical_type.rs:252`) "a list of
maybe-nothing" would be a second undeclared contract. Without the Python `FieldTransfer`
report the extension asymmetry would be invisible.

**Applicability.** Groups 1, 2, 3, 5, 8, 9, 11 and 12 bear on this scope: it is a
canonical-model and physical-layout review across a storage boundary and a language
boundary. Group 4 applies only through DM-18 (F02). Group 6 applies only through DM-28
and DM-29 as inspected for G4; publication and workspace semantics are out of scope for
the reason stated in §1. Group 7 applies through DM-31/DM-32 only where row identity
feeds reuse keys (F06); incremental recomputation itself is not in scope. Group 10
applies through DM-46/DM-47 in F06 and F08. No principle outside those named is
certified.

**Optional maturity assessment.** Not scored; the unresolved gates would be hidden by a
total.

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| Current baseline | Nested values declared once; alternatives, ordinals, transfer rules and row identity re-expressed in writers, wrappers, codecs and consumers | F01, F04, F06, F09 open on implemented paths | Every new native function needs a wrapper; every payload family adds ~16 relations | None | Not acceptable against G2 |
| Proposed full program (F01–F10) | One declaration per alternative, key and transfer rule; node rows self-contained | Reshaping the DAG (F02) changes every math consumer and the canonical hash path; nested-projection I/O under Delta is unknown | Largest: registry generator branches, analyzer rule, durable metadata, DAG reshaping, numerical contract | Hypotheses only (DM-39) | Correct direction; sequence it |
| Simpler viable alternative: declare, persist, unsign — no relation reshaping | Same reduction in independently maintained rules (F01, F04, F05, F06, F09, F10) without touching relation shapes or consumers | None new; each item is a guard, a property, a generated check or a type change | Small: `delta.rs`, `layout.rs:129`, one generator construct, table properties, integer types | Not needed for the correctness items; F05 needs a before/after join benchmark | **Do this first.** It closes G2 and most of G3 and G7. F02, F03(b) and F07 follow only with measurements |

**Abstractions justified by current needs.** A `TaggedAlternative` registry construct is
justified by five present instances and the absence of any enforcement. A single
transfer analyzer is justified by nine wrappers that exist only to restate one table.
Nothing here needs a new crate, service or plan platform.

**What remains ordinary code.** The recursive value validator, hash-consing and
canonicalization of expression graphs, the Ipopt adapter, and every kernel. Declaring
their contracts does not move their algorithms.

## 9. Verification and measurement plan

| Claim or risk | Evidence label | Test / analysis / benchmark | Conditions and expected result | Current result or remaining gap |
|---|---|---|---|---|
| Inconsistent tagged rows are rejected on every route (F01) | Proposed | Negative fixtures per alternative through native INSERT, Delta write and publication | Typed `schema.*` refusal; no row lands | No test exists |
| Struct children cannot be relabelled at the durable boundary (F09) | Proposed | Batch with `{lo, hi}` against declared `{lower, upper}` through `DurableLayout::encode` | Refused | Path reaches `cast_with_options` today |
| Durable metadata and properties round-trip (F04) | Proposed | Write, cold-open with a fresh session and with `pyarrow`, compare per-field `ARROW:extension:name` and `pse.contract.*` properties | Identical | Metadata is stripped today |
| Non-nullable nested children hold no unmasked nulls after Delta read | Interface-checked | Write a struct with a required child, read back, force a null via a mismatched file | `StructArray::try_new` error | Relies on the cast path calling `try_new`; not executed |
| Integer durable identity (F05) | Proposed / Measured after change | `just bench-smoke` on a node/edge join with `Decimal128` versus `Int64` keys, plus scan bytes | Report both; no claim before the run | None |
| Typed row keys replace JSON (F06) | Proposed | `EXPLAIN` of provenance support joins | No `pse_encode` node; FK invariant plan present | Encode UDF present today |
| Transfer rule per operator (F03) | Proposed | Conformance suite: for each native operator family (projection, filter, join, aggregate, window, unnest, UNION, cast, each array/struct/map function used) assert output fields against the transfer table | Exact field equality including UNION's metadata intersection | Only wrapper-level tests exist |
| Nested projection cost (F02) | Proposed | `EXPLAIN ANALYZE` bytes read: opcode-only, one payload arm, whole node, before/after folding, under the pinned Delta scan | Report; folding proceeds only if arm-only reads are not materially worse | None |
| Numerical output contract (F07) | Proposed | Schema assertion plus allocation and time per callback on a representative Slice A problem | Report; adopt only on measured gain or clear contract benefit | None |
| Capability document labels (F11) | Proposed | Review header lists pins and labels | Present | Absent |

**Cost accounting.** Material categories: durable encode/decode casts per column (F05),
in-plan key encoding (F06), per-callback allocation (F07), per-row support relocation
(F08), and wrapper maintenance per native function (F03). Storage width of decimal keys
and wide sparse payload structs is the only category where a change could cost more than
it saves and must be measured.

## 10. Exceptions and unresolved decisions

No SHOULD-level exception is recorded. Two capabilities were examined and are not
recommended now, with reasons: Arrow `Map` for option lists gives no structural key
uniqueness beyond what `list<struct{key,value}>` gives and would widen the registry's
type catalog for a presentational gain; Variant for diagnostic evidence is readable but
its write path and DataFusion functions are not established at the pin, and DM-47 is
better served by typed evidence (`list<struct<role, subject_id?, quantity?, text?>>`)
than by a semi-structured column. In-memory `Union` for tagged alternatives is available
in DataFusion but cannot be stored; it is an optional execution layout after F01, not a
prerequisite.

**Unresolved decisions the maintainer must make.** Whether artifact-local ordinals become
`i64` (F05) — this touches `pse.ordinal_ref` storage and the identity framing constants
(ADR-0050) and therefore an accepted decision. Whether the expression DAG is reshaped
(F02) — after the F02 measurement, since it changes the relation index and every math
consumer.

## 11. Decision and implementation changes

**Decision: Revise.** The registry's nested types are already the right primitives; the
design does not yet use them to make correspondence structural, does not persist its own
declaration into the durable table, and guards the durable boundary with a comparison
weaker than its contract. The simpler alternative in §8 closes the correctness gates
without reshaping any relation; the reshaping and the numerical contract are worth doing
only with the measurements named in §9.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 | Full-field guard at the durable projection (F09) | DM-42, DM-53 | Renamed-children negative test | The test |
| 1 | Persist field metadata and `pse.contract.*` properties in Delta; explicit cold-open reconciliation (F04) | DM-42, DM-51, DM-55 | Round-trip and mismatch-refusal tests | Cold-open fixture in CI |
| 1 | `TaggedAlternative` declaration generating layout, CHECK, invariant plan, accessors (F01) | DM-07, DM-08, DM-52 | Negative rows refused on every route; PSE-free session evaluates CHECK | Generated conformance per alternative |
| 2 | Native CHECK SQL for expressible predicates; UDF only for recursive admission; fingerprint as property (F10) | DM-04, DM-55 | Same rejections through each route without the UDF registered where not needed | Write-route matrix test |
| 2 | Typed row keys and typed provenance references (F06) | DM-11, DM-15, DM-46 | No in-plan encode; FK invariant over `supporting` | Hash stability test |
| 2 | Spans as a column; diagnostics via builder; support lists from plans (F08) | DM-09, DM-11 | Parallel vectors absent; ordinal join chain reduced | Structural lint on `Vec<SourceSpan>` |
| 2 | `PerRow` quantities as `pse.quantity_value` structs; one transfer analyzer replacing metadata-only wrappers; `struct_field_mapping` (F03) | DM-02, DM-23, DM-56 | Wrapper deletion with identical admitted fields; `array_sort`/`array_transform` plan admits | Operator transfer conformance suite |
| 3 | Signed artifact-local ordinals; unsigned types out of the durable catalog (F05) | DM-36, DM-42 | Durable equals execution for integers; join benchmark | Family/codegen checks |
| 3 | Fold `children` and payload onto the node row after the bytes-read measurement (F02) | DM-09, DM-18, DM-56 | Relation and invariant counts; equal canonical hashes; measured scan cost | Math conformance and hash tests |
| 3 | Typed numerical output contract after the callback measurement (F07) | DM-01, DM-37, DM-39 | Program schema test; allocation and timing report | Numerical oracles already planned in UD07 |
| 3 | Pin labels on the capability document (F11) | DM-59 | Header present | Doc lint |

**Final check.** Claims are labeled at the strength of what was read; the supported scope
excludes publication semantics and stub crates by name; the extension path after the
changes is one declaration plus the genuinely new kernel, which is the charter's
extension-locality test.
