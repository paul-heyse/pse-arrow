# Design review — Arrow-native IDAES core architecture blueprint (revision 2)

**Standard:** `docs/design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md` (DM-01–DM-60, gates G1–G7), through `AGENT_DESIGN_DIRECTIVE.md` and `DESIGN_REVIEW_TEMPLATE.md`.
**Subject:** `docs/design_review/Arrow-native-idaes-core-architecture-blueprint.md`, revision 2 (2026-09-13), 2,792 lines. A design document; no code exists.
**Prior review:** `design_review_arrow-native-idaes-core-blueprint_2026-09-13.md` (revision 1; decision Revise; findings F1–F17). This review checks that revision 2 closed F1–F7, F9 and F15, and re-adjudicates the design against the four capability maps compiled after that review.
**Depth:** standard. **Date:** 2026-09-13.

---

## 1. Decision and scope

**Proposal.** Unchanged in architecture from revision 1: typed Arrow relations in seven namespaces, sixteen contracted passes (now P0–P16 with P12 index expansion), DataFusion 55.1 for inference rules, catalog, batch kernels and analytics, native numerics with in-process Ipopt, plus NL and generated-Pyomo backends. Revision 2 changed identity assignment, expression authority, policy placement, undecided-outcome representation, the unit algebra, pass structure, parameter-folding policy, the engine-as-input memo key, and the test plan.

**Status.** Proposed throughout. Where this review says *Interface-checked* it inspected the library surface itself through context7 (Appendix A). Where it says *capability map, measured* the claim rests on a probe program and output committed under `docs/design_review/evidence/` by the four maps — inspected as documents here, not re-run.

**Reviewer.** Claude (Fable 5.1), design reviewer for the project owner.

**Affected revisions.** Blueprint revision 2; the four capability maps (`Arrow-rust-`, `DataFusion-rust-`, `Supporting-rust-library-`, `Python-library-capability-map.md`), all compiled 2026-09-13 against DataFusion 55.1.0, Arrow 59.3.0, and the current PyPI releases (Pyomo 6.10.1, pyarrow 25.0.1, IDAES 2.12.0).

**Observable outcome claimed.** As revision 1, with the nine priority-1 defects removed.

**Supported scope and non-goals.** Unchanged (§0.2 of the blueprint).

**Constraints and uncertainty.** Nothing is measured on the platform side. The maps now carry twenty-one measured probes of the libraries; several of them contradict blueprint sentences, which is the substance of this review.

### Method and coverage

Examined:

- Every section of revision 2 that revision 2 changed, at the grain cited (§3.1, §4.3–§4.4, §5.1, §6.1, §6.6–§6.13, §7.1–§7.7, §8.1–§8.3, §14.1–§14.5, §18.3–§18.4, §19.1, §20.1–§20.2, §22.1–§22.2, §24.1, §26, Appendices B–C), plus the unchanged sections the maps now contradict (§3.2, §3.3, §5.3, §12.5, §15.3–§15.5, §21.2–§21.5, §25).
- The Python map in full. In the three Rust maps: the anchor sections, every cluster the maps mark as new or as measured against revision 2 (DataFusion D2, D3, D9, D10, Pass 3; Arrow Pass 3; Supporting §1 and Pass 3), and each map's register, gate review, leverage matrix, and open items. The evidence README.
- Five context7 queries (Appendix A).

Guarantees attacked this round:

| Guarantee | Result |
|---|---|
| F1–F7, F9, F15 are closed as specified in revision 2 | Closed at the document level. F9's mechanism is specified but, as the DataFusion map measured, not reproducible as written (finding R2-1) |
| §14.2 rule 5's plan fingerprint is reproducible across processes | Fails (capability map, measured, PROBE C): `HashMap` metadata order enters the `datafusion-proto` bytes |
| "`-0.0` is preserved" (§5.3, §26) holds through the engine | Holds for ordering and the content hash; fails under `GROUP BY` and hash joins (capability map, measured, PROBE D) |
| "Nothing in DataFusion acts on these keys" (§4.3) | False for `ARROW:extension:name` at 55.1.0 (capability map, measured, PROBE X1; the `TypePlanner` pattern Interface-checked) |
| §3.3's "no Python-side units library" is consistent with §21.2's units requirement | Contradictory: Pyomo's units machinery is pint (capability map, measured, PROBE 4) |
| §21.5's "no `Any` fields" and strict structuring are enforced by the named libraries | Not by default: cattrs ignores extra keys unless `forbid_extra_keys` is set (Interface-checked) and accepts `Any` (capability map, measured) |
| §3.2's "carries every pin once" | One of twenty-four supporting crates and none of the Python libraries is anchored (blueprint text) |
| §20.1's commit rule names a method that exists at the pinned `object_store` | It does not: `rename` is not on the 0.13.2 trait (capability map, docs-checked) |
| §5.4's `Exact` pushdown is reachable on `FixedSizeBinary(16)` keys | Reachable for equality (capability map, measured, PROBE A); `IN` arrives rewritten to `OR` |

Not examined: physics and algorithms as before; the maps' probe programs were not re-run; the Python API dumps were not opened.

---

## 2. Authority and lifecycle map

Revision 1's invented cells are now filled: authored identity is creation-time (§5.1), template expression graphs are `normalized` (§6.6), policies are `authored` (§6.11), undecided outcomes have `inferred.undecided` (§6.7), the engine is a `reference.engine_profiles` row (§6.11). Two authority questions remain, both at boundaries:

| Concept or fact | Semantic type and identity | Authority / owner | Revision boundary | Permitted update path | Derived representations |
|---|---|---|---|---|---|
| Units in a generated Pyomo model | Pyomo/pint unit objects attached per `Var` | **[ambiguous]** — §8's registry by intent; §3.3 forbids a Python units library; §21.2 requires one (finding R2-4) | — | — | pint validates; must never define |
| Python contract classes (§21.5) | attrs classes | **[unstated]** — hand-written or generated from `RelationSpec`? A hand-written column list is what §4.2 calls a governance failure (R2-4) | Registry fingerprint | Regeneration | cattrs structuring |
| Plan fingerprint (§14.2 rule 5) | blake3 of `datafusion-proto` bytes | Platform contract (implicitly) — its stability depends on platform-constructed schema metadata and the platform codec, which §14.2 does not say (R2-1) | Engine profile hash | — | `pass_records.plan_fingerprints` |

**Deliberately opaque behaviour.** Unchanged. The DataFusion extension-type registry is a newly identified place where the engine enforces §4.4's storage types; it is not yet claimed (R2-3).

---

## 3. Semantic contracts and invariants

Revision 2's new contracts, with the verification the maps supply:

| Contract or invariant | Representation | Enforcement boundary | Failure behaviour | Verification evidence |
|---|---|---|---|---|
| Rename changes attributes only | `change_ops.op = rename`; identity-based `case_spec_targets` | P1/P2 | `case.target_text_mismatch` | Proposed; conformance test named in §24.1 |
| No authored FK into `compiled`/`runtime` | Registry query | Governance test | CI failure | Proposed |
| Undecided never consumed as true | Head relations hold decided-true rows only | Rule executor | `inferred.undecided` row + diagnostic | Proposed |
| Mul/Div unrestricted after canonical units | §8.3 algebra | P10 | `math.unit_inconsistent` | Proposed; reference-library test set in §24.1 |
| Every equation scalar before P14 | P12 postcondition | P12 | `expand.unbound_index` | Proposed |
| P10 graph independent of parameter values | §7.4 step 3 | P10/P13 | — | Proposed; two-case hash test |
| Engine version and rule set in the memo key | `engine_profiles.content_hash` | Pass engine | Memo miss | Proposed; the fingerprint component is non-reproducible (R2-1) |
| `pse.*` storage types valid in every plan | **[unclaimed]** — DataFusion `ExtensionTypeRegistry` | Planning | Typed planning error | Capability map, measured (PROBE X1) |
| Python rows structured strictly | **[asserted]** — §21.5 prose | None by default | Unknown keys silently dropped | Interface-checked (cattrs docs) |

**Absence and uncertainty.** Revision 2 added the `undecided` representation; the remaining collapse is at the Python boundary, where a nullable Arrow column converted to `ndarray` turns null into NaN (capability map, measured, PROBE 8) and §21 says nothing about it.

**Equivalence requirements.** §5.3's content hash is now better evidenced than revision 1 could say: the Arrow map measured that canonical IPC bytes are stable regardless of metadata key count (PROBE 6) and that Parquet preserves metadata and is deterministic within a writer version (PROBE 7). The equivalence class the engine applies to float *values* in grouping and joining is different from ordering (R2-2), and §5.3 does not say so.

---

## 4. Derivation and execution design

Only the rows revision 2 changed or the maps affected:

| Stage | Input revisions and dependencies | Output contract | Preconditions / assumptions | Effects and mutable ownership | Provenance / invalidation |
|---|---|---|---|---|---|
| P4–P6, P8 (rule passes) | `normalized.*`, `reference.rule_specs`, **`engine_profiles`** | `inferred.*`, `undecided` | Session built from the profile; `Optimizer::with_rules` (and analyzer rules — R2-1) | None | `plan_fingerprints` — non-reproducible as specified (R2-1) |
| P12 index expansion | indexed equations, tuples, meshes | scalar `math_equations` | Every free index has members | None | IDs per §5.1 |
| P13/P14 | cases, `case_bound_substitutions` | problem, static attributes on the case-bound view | — | None | `parameter_dependence` |
| Python adapter (§21) | bundle streams | Pyomo model | **pint present** (contradicts §3.3); external-function library resolves (only a warning today — capability map, measured) | Pyomo model in process | Loss profile undeclared (parked F16) |

**Provider selection and limitations.** Unchanged, with one correction from the maps: DataFusion enforces no relational constraint except `Field` nullability, so §5.4's "Constraints: primary key only" is a declaration the platform's P2 makes true (capability map, docs-checked).

**Boundary contracts.** The engine errors on an unregistered extension name while Python silently degrades to the storage type — the same design decision enforced on one side and not the other (both capability map, measured). §21 must declare the asymmetry.

---

## 5. Representative journeys

### Meaningful change — a rule-pass memo hit across two processes

Process A compiles snapshot S: P4 runs its plans, records `plan_fingerprints`, memoizes on (inputs, pass version, engine profile hash). Process B, same binary, same S: the memo key's engine profile and input hashes match, but the plan fingerprint — if it is part of the key or merely of the record — differs, because every `pse.*` relation carries two or more field-metadata keys and `datafusion-proto` serializes them in `HashMap` iteration order (capability map, measured, PROBE C: six encodings, six distinct byte strings at five keys). If the fingerprint is in the key, process B recomputes everything; if it is only in the record, `pse reproduce` finds two byte-different pass records for identical runs and §20.4's assertion fails. Either way F9's mechanism does not deliver what §14.2 promises until schema metadata is canonicalized at construction (R2-1).

### Boundary — a parity run through the Python adapter

The bundle crosses as capsule streams (confirmed by the Python map). The adapter attaches units — which requires pint, which §3.3 excludes (R2-4). Rows are structured into attrs classes with a default cattrs converter: an extra key from a newer bundle version is silently dropped (Interface-checked), so a schema drift between Rust and Python is invisible. A `KernelCall` lowered to `ExternalFunction` with a library that does not resolve constructs successfully with a warning and fails from the solver later (capability map, measured, PROBE 3). Solution values pulled into numpy for comparison with IDAES convert null to NaN if any nullable column is touched (PROBE 8). None of these is architectural; all of them are contract statements §21 lacks.

### Interruption — unchanged from revision 1 and now specified

§18.3's no-unwind rule, §18.4's subprocess rule, §20.1's hash-verify-before-put and the lifecycle test layer are in the text. The Supporting map still lists its amendment F as parked; it is applied.

---

## 6. Acceptance gates

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Pass** | Revision 1's three failures are closed: creation-time IDs with a `rename` op (§5.1, §22.2); expression graphs derived into `normalized` (§6.6, §7.7); policies authored (§6.11) with a governance test. No second authority remains in the core. The units question at the Python boundary is a consistency defect, not a second definition, as long as pint validates and never defines (R2-4). | Reword §3.3 to authority; state that Python contract classes are generated. |
| G2 — Semantic fidelity | **Unresolved** | Revision 1's failures (undecided outcomes, unit algebra) are closed. Two claims are now measured false or unqualified: "`-0.0` is preserved" fails under grouping and joining (R2-2); the Python boundary collapses null into NaN and degrades extension types silently (R2-4). Both are boundary paths with no declared loss policy. | Qualify §5.3; forbid `Float64` distinct/join keys in `RulePlanSpec`; declare the Python loss profile and the `null_count == 0` rule. |
| G3 — Validity | **Unresolved** | Index expansion and the FK direction are fixed. The engine-level storage-type validation DataFusion offers is unclaimed (R2-3); §21.5's strictness is asserted but the default converter drops unknown keys and accepts `Any` (R2-4); `SessionConfig::set_str` panics on invalid input (capability map, measured). | Register the `pse.*` types; `forbid_extra_keys` and an `Any` lint; ban `set_str`. |
| G4 — Hidden behaviour | **Unresolved** | F10 (probe timing) parked. New: `supports_filters_pushdown` is called more than once per plan and must be pure (capability map, measured); a missing external-function library only warns (R2-7, R2-4). | Purity note; library resolution check; F10. |
| G5 — Consistency and recovery | **Pass** (with notes) | Lifecycle tests, FFI and subprocess rules, hash-verify-before-put are specified. Notes: `MemoryPool` choice still unmade (the map measured that a bounded pool yields an actionable typed error — PROBE H); Python extension registration is process-global and non-idempotent. | Choose the pool; idempotent register-once helper. |
| G6 — Transformation and reuse | **Unresolved** | F7 and the engine-profile key are applied. F9's fingerprint is non-reproducible as specified, its table-provider codec obligation is unstated, the analyzer rule list is captured but its installation is unnamed, and the semantic-settings selection by namespace misses two `execution.*` keys (R2-1). F12, F13 parked. | R2-1 corrections, or the simpler alternative in §8. |
| G7 — Truthful capability claims | **Unresolved** | F17 half-resolved (`Exact` reachable; SVD and condition-number routes now known). New: §3.2 and §3.1 disagree on pinning; §3.3 role text overshoots use sites for three crates; two library methods named in the text do not exist at the pinned versions; §4.3 erratum; Python pins and IDAES version wrong; a cited discipline file is deleted (R2-5, R2-6). | Adopt the maps' pin tables; correct the errata. |

No gate fails. Five are unresolved on claim accuracy and boundary contracts; none requires an architectural change.

---

## 7. Principle findings

Ranked by severity. Revision 1 findings are referenced as F*n*; this review's findings are R2-*n*.

| # | Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| R2-1 | F9's plan fingerprint is not reproducible as specified, and three of its dependencies are unstated | DM-48, DM-31, DM-15, DM-59 | §14.2 rule 5 (line 2071): "blake3 of the plan's `datafusion-proto` encoding under the platform's `LogicalExtensionCodec`". DataFusion map D9, PROBE C (capability map, measured): six independent encodings of one plan over a schema with five field-metadata keys gave six distinct byte strings; one shared `Schema` object gave one — the cause is `Field::metadata` `HashMap` order. Same cluster: `logical_plan_to_bytes` fails without a codec on a custom provider, so `try_encode_table_provider`/`try_decode_table_provider` (required methods) are platform obligations §14.2 never names; rule 5 names `Optimizer::with_rules` but not the analyzer's own explicit list (Interface-checked: the analyzer rules are a separate ordered list and "the default pipeline may change between releases"); and the settings hash selects by namespace while `datafusion.execution.skip_physical_aggregate_schema_check` and `enable_ansi_mode` are semantic keys under `execution.*` (DataFusion map E3). | Identical runs record different `plan_fingerprints`, so §20.4's byte-identical pass-record assertion fails on the first reproduction; if the fingerprint is ever folded into the memo key, no rule-pass memo hits. A DataFusion release adding a semantic key under `execution.*` escapes the hash. | Canonicalize schema and field metadata at construction (a generated constructor inserting keys in sorted order — the same fix serves `pse.contract.fingerprint`); define the fingerprint as a platform contract with its own version, stating that it covers the platform's table-reference encoding and the kernel digests (rule 5 already says UDFs are encoded by digest); install analyzer rules explicitly too; replace the namespace rule with a versioned allow-list and assert the two `execution.*` keys stay false. Surface area: §14.2 rule 5, §6.11 `engine_profiles`, one constructor. See §8 for the simpler alternative. | CI test encoding the same plan in two fresh processes and asserting byte equality; a test that adding a rule to the engine profile misses every rule-pass memo. |
| R2-2 | "`-0.0` is preserved" is stated without qualification and fails under grouping and joining | DM-40, DM-42, DM-08 | §5.3 step 4 (line 535) and the §26 risk row. DataFusion map D3, PROBE D (capability map, measured): `ORDER BY` keeps `-0.0` before `+0.0` and NaN last; `GROUP BY` returns four groups from five rows, merging the zeros; a self equi-join matches NaN to itself. §14.2's rule algebra includes `distinct` and equi-join with no restriction on key columns. | A rule whose head relation is produced through `distinct` over rows containing a `Float64` column, or an analytics query grouping on one, merges `-0.0` with `+0.0` and picks a sign the engine chooses; the content hash of that relation then differs from what §5.3's "preserved" promises, silently. | State in §5.3 that preservation holds for ordering and hashing only; add a `RulePlanSpec` invariant that distinct and join keys are key-role columns (semantic IDs, ordinals, enums), never `Float64`; record the grouping collapse in §19.2 as a selected loss for analytics. | A rule-compiler test rejecting a `distinct` over a `Float64` column; an analytics test documenting the collapse. |
| R2-3 | §4.3's "nothing in DataFusion acts on these keys" is false, and the engine's extension-type registry is an unclaimed validity mechanism | DM-06, DM-07, DM-44 | §4.3 (line 437). DataFusion map D10, PROBE X1 (capability map, measured): a `pse.semantic_id` registration over `FixedSizeBinary(16)` resolves from `ARROW:extension:name`; a field with the wrong storage type is rejected during planning; an unregistered `pse.*` name errors; seven `arrow.*` types are preloaded by default. The `FixedSizeBinary(16)` + `ARROW:extension:name` pattern is upstream's own `TypePlanner` example (Interface-checked). | As written, an implementer expects extension metadata to be inert in plans; at 55.1.0 a mismatched storage type produces a planning error the design never anticipated, and an unregistered name errors rather than degrades — behaviour that differs from the Python side and is documented nowhere. Conversely, the cheapest G3 closure available (validation of every plan, not only generated views) is unused. | Narrow §4.3 to non-`ARROW:` keys. Add to §4.4/§14.3: the ten `pse.*` types are registered in the session's `ExtensionTypeRegistry` from §4.4's table (generated), duplicates asserted absent; note that the registry validates storage type and metadata only, and that paths outside a session (D11 borrowing, Python) still need the generated `try_from` check. Record that DataFusion validates no constraint but field nullability (§5.4). | Test: a batch with `pse.semantic_id` over `Utf8` is rejected at planning; a test that the registered set equals §4.4's table. |
| R2-4 | The Python boundary is specified by prose the named libraries do not enforce, and two of its sentences contradict each other | DM-42, DM-06, DM-08, DM-02, DM-52, DM-59 | (a) §3.3 line 333 "Explicitly not added: … a Python-side units library" vs §21.2 step 1 (line 2512) "units attached from the unit set so that `assert_units_consistent` passes" — Pyomo's units container defers to pint and raises without it (capability map, measured, PROBE 4). (b) §21.5 (line 2529): "typed attrs classes … cattrs for structuring, no `Any` fields" and cites `docs/library_ref/contract_substrate_discipline.md`, which no longer exists; cattrs by default ignores extra keys (Interface-checked: "By default, cattrs ignores extra keys in unstructured input dictionaries") and accepts `Any` fields (capability map, measured, PROBE 6). (c) Arrow → numpy with nulls converts null to NaN unless `zero_copy_only=True` refuses (PROBE 8); an unregistered `pse.*` type in Python becomes its storage type silently while `ARROW:extension:name` survives in field metadata (PROBE 1). (d) `ExternalFunction` with an unresolvable library only warns (PROBE 3). (e) §3.1 line 268 floors pyarrow at 15 (current 25.0.1) and names IDAES 2.10 (current 2.12.0, interpreter 3.10–3.13 only); attrs, cattrs, msgspec, numpy, scipy unpinned; msgspec's role unstated. | Under §3.3 as written the adapter cannot be built; under §21.2 pint is present and nothing says it may not define a quantity. A bundle from a newer platform version structures without error, dropping the new columns. A parity comparison through numpy cannot distinguish "not computed" from NaN. A kernel bound to a missing library fails from Ipopt with no kernel id. Parity "against IDAES" is undefined without an exact version. | Reword §3.3: no Python-side units library is an *authority*; pint validates what the adapter emits and never defines a quantity. Generate the contract classes from `RelationSpec` (they join §4.2's artifact list); `forbid_extra_keys` via the hook factory, `detailed_validation` kept on, `transform_error` into §23.2 findings, an `Any`/bare-`dict` lint over `attrs.fields()`. Assign msgspec to the manifest and all JSON/TOML the platform reads, attrs+cattrs to Arrow rows, no type in both. Ship and register the ten `pyarrow.ExtensionType` classes from the registry with an idempotent helper; declare the per-column loss profile (parked F16); assert `null_count == 0` or carry the mask, `zero_copy_only=True` by default. Verify the external-function library resolves before building the model; `SolverFactory(name).available(False)` pre-flight; record `pyomo.contrib.*`, numpy and scipy versions in `runs.environment`. Pin every Python library `==` and state two interpreter ranges (platform ≥3.11; parity 3.10–3.13); pin IDAES 2.12.0 for parity. Replace the dangling citation with the discipline itself. | Python tests: extra key rejected; `Any` field fails the lint; extension names survive Rust→Python→Rust; a nullable column refuses `ndarray` conversion; a missing library raises `capability.backend` before construction. |
| R2-5 | Dependency declarations are incomplete or inaccurate at the pinned versions | DM-31, DM-48, DM-51, DM-59, DM-43 | §3.2 (line 277): "`[workspace.dependencies]` carries every pin once"; §3.1 anchors `tokio` and none of the other 23 supporting crates the Supporting map enumerates (its §1.2 table resolves all of them into a committed lockfile). §3.1 still pins `arrow-flight`, which nothing consumes (Arrow map §10.1). `uom` is named in §3.3 (line 323) and appears nowhere in §8 (Supporting map Q13). §3.3 role text overshoots the body: `faer` "Sparse LU/QR … solver workspaces" (line 322) with no QR or workspace use site; `petgraph` "dependency ordering, SCC, DAG checks" (line 318) while matching and DM are carved out; `feos` "Native Helmholtz/SAFT/cubic" (line 324) while §9.8 delivers Helmholtz by expression template. `serde_yaml` still named; the `num-dual` split is source-level (`DualNum<f64>` vs `DualNum<Primitive = f64>` — capability map, rustdoc). `blake3` is assigned to no crate and its 128-bit derivation is unnamed (`derive_key` vs truncation give different bytes — capability map, measured, PROBE 1). No lockfile requirement; `cargo audit` cannot see `serde_yaml`'s deprecation (capability map, RustSec-checked). | A build resolves whatever is current for 23 crates and every Python library, so §20.4's reproduction claim is false for the toolchain; a transitive second Arrow or `num-dual` version is undetected; a reader justifies `faer` QR or `uom` from §3.3 for work that does not exist; permanent §5.1 IDs are frozen with an unstated derivation. | Adopt the Supporting map §1.2 and Python map §1.1 pin tables into §3.1; require a committed, CI-enforced lockfile with a one-version-per-family assertion; add `cargo deny` maintenance and bans checks to §24; drop `arrow-flight`; either name the rule for `uom`-typed kernels or remove `uom`; narrow the three §3.3 rows; choose `serde-saphyr` (the map's evidence is one-sided); pin `num-dual` 0.15 and mark `feos` blocked on its upstream; assign `blake3` to `pse-ids` and name `derive_key` as the 128-bit route in §5.1. | CI: one version per family; `cargo deny` green; a governance test that every crate in `Cargo.toml` appears in §3.1's table. |
| R2-6 | Four statements about library surfaces are false or stale at the pinned versions | DM-59, DM-43 | §20.1 (line 2461, added in revision 2): "`rename` is never used as a commit primitive" — `object_store` 0.13.2 has no `rename`; 0.13.0 merged the plain methods into `*_opts` (capability map, docs-checked at the pinned version). §15.3 (line 2145): "Hopcroft–Karp (own implementation; `petgraph` lacks it)" — petgraph ships `maximum_matching` (Gabow, general graph, O(\|V\|³), input treated as undirected; capability map, rustdoc). §15.4 (line 2157): "pseudo-inverse for non-square" via sparse LU — the route is QR least-squares or `pseudoinverse_from_svd_with_tolerance` (capability map, rustdoc). §15.5 (line 2172): "iterative for large" — faer's `matrix_free::partial_svd` targets the *largest* singular values; the smallest need shift-invert through the sparse LU (capability map, rustdoc). | An implementer reads a method that does not exist, or implements Hopcroft–Karp believing no alternative exists (the reasoning is right, the stated reason wrong), or reaches for `partial_svd` and gets the wrong end of the spectrum. | §20.1: "the final path is written once with `PutMode::Create`/`Update`; no copy-then-delete sequence is a commit". §15.3: "petgraph ships only general-graph matching (Gabow, O(\|V\|³)); the incidence graph is bipartite, so Hopcroft–Karp's O(\|E\|√\|V\|) is required". §15.4/§15.5: name the QR/SVD-with-tolerance route, shift-invert plus `matrix_free` for the smallest singular values, or restrict the toolbox to dense-tractable sizes. | Doc-lint: every backticked library identifier in §15, §18, §20 resolves in `build/facts/*`. |
| R2-7 | `Exact` pushdown is reachable, but two facts change how §5.4 must be implemented | DM-43, DM-20, DM-53 | DataFusion map D2, PROBE A (capability map, measured): key equality arrives as `rel.id = FixedSizeBinary(16, …)` (matchable); `IN (a, b)` arrives already rewritten to `OR`; conjunctions are split; `IS NOT NULL` on a non-nullable column is eliminated; `supports_filters_pushdown` is called more than once per plan. §5.4 promises `Exact` on key columns and says nothing about `IN` or purity. | A provider matching `Expr::InList` never fires and silently falls back to a full scan; a provider that counts or logs in `supports_filters_pushdown` double-counts; nothing verifies an `Exact` claim (the optimizer deletes the filter — Interface-checked in the prior review). | §5.4: match binary equality and `OR` chains of equalities, not `InList`; declare `supports_filters_pushdown` pure; add the test-only wrapper provider that re-applies every `Exact` filter; use `Precision` per statistic and answer `StatisticsRequest`s selectively (parked from revision 1). §26 F17's pushdown precondition is discharged for key equality. | Wrapper-provider CI test over golden snapshots; a test that an `IN` on a key column reaches the scan as `Exact`. |
| R2-8 | Library mechanisms the design re-implements or renders poorly (leverage) | DM-38, DM-49, DM-47, DM-16, DM-46 | §12.5 (line 1936) and §15.3 list tear selection as hand-written; petgraph `greedy_feedback_arc_set` returns exactly the recycle edge on a flowsheet-shaped graph (capability map, measured, PROBE 3c). §23.2 wants failures that carry rows; a `pse.semantic_id` renders as sixteen raw bytes without an `ArrayFormatterFactory` / `DFExtensionType::create_array_formatter` (both maps). §24.1 does not use Arrow's `force_validate` feature in the test profile. §14.2 rule 5 hashes engine settings but platform policy travels separately — a `ConfigExtension` (`datafusion.pse.*`) would enter the same `df_settings` hash. Pass records store plan text; `EXPLAIN` has a `pgjson` format. The optimizer `observer` closure records which rules fired. `toml::Spanned` supplies `pse.source_span` for free; `num-dual::implicit_derivative` matches a hand-derived implicit function theorem (PROBE 4); the Ipopt C exports are `GetIpoptCurrentIterate`/`GetIpoptCurrentViolations`. | Hand-written search where a library primitive exists; diagnostics that satisfy §23.2's letter and not its purpose; a validity gate available for one profile line left unused; platform policy outside the reproducibility hash. | Adopt: `feedback_arc_set` as the tear search (policy stays ours); one formatter factory driven from §4.4's table; `force_validate` in test/CI profiles; a `datafusion.pse.*` config extension for platform policy; `pgjson` for stored plans; the observer for fired-rule evidence; `toml::Spanned`; `implicit_derivative` as the implicit-kernel binding; correct the Ipopt names when §23.1 is amended. | Each adoption's own test; the formatter has a snapshot test per `pse.*` type. |

**Status of revision 1's parked items.** F8 (JSON rule bodies), F10 (probe timing), F11 (now subsumed by R2-5), F13 (salsa; the Supporting map measured backdating and accumulators work as described — PROBE 6 — which does not change the DM-58 question), F14 (derivation cost), F16 (loss profile; now sharpened by the measured Rust/Python asymmetry), F17 (half-resolved by PROBE A and the faer findings). F12 gained precision: canonical IPC bytes are stable regardless of metadata key order (PROBE 6), so the exposure is platform-side fingerprints and `datafusion-proto`, which is R2-1; the alignment and `MetadataVersion` constants are still unnamed.

**Observations (not findings).** The Supporting map's amendment F (no unwind through Ipopt) is listed as parked; it is applied in §18.3. `datafusion-tracing` and `instrumented-object-store` top out at 55.0.0 against a 55.1.0 engine and should not be added yet. `LogicalPlan::Extension` for rule bodies survived the optimizer in the map's PROBE B and is now an adopt-candidate on cost grounds alone; it remains a DM-58 decision, not a defect.

### Applicability

| Group | Applied | Reason |
|---|---|---|
| 1 Semantic authority | Yes | G1 re-checked after F1–F3; R2-4 units authority |
| 2 Types, schemas, invariants | Yes | R2-2, R2-3, R2-4 |
| 3 Identity, versions | Yes | R2-1 fingerprint, R2-5 `blake3` derivation |
| 4 Declarative composition | Lightly | Unchanged since revision 1; no map finding bears on it |
| 5 Compilation | Yes | P12, parameter folding verified as specified |
| 6 Execution, effects | Yes | R2-7 purity; G5 notes |
| 7 Dependencies, incrementality | Yes | R2-1, R2-5 |
| 8 Execution representations | Lightly | R2-6 faer routes |
| 9 Boundaries, providers | Yes | R2-3, R2-4, R2-7 |
| 10 Provenance | Yes | R2-1 pass records, R2-8 observer |
| 11 Evolution, verification | Yes | R2-5 lockfile and `cargo deny`; R2-8 `force_validate` |
| 12 Leverage | Yes | R2-8; §8 |

**Maturity assessment.** Not scored; nothing is implemented. The library side is now at *Interface-checked* or *capability map, measured* for most of the surfaces the design depends on, which is a materially stronger evidentiary position than revision 1 had.

---

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| Current baseline (revision 1) | As reviewed | Nine priority-1 defects | — | None | Superseded |
| Proposed (revision 2) | Same locality; identity and policy placement now sound | R2-1 through R2-7 — all boundary or claim-accuracy defects | As revision 1 plus one pass | None | Selected, subject to §11 |
| **Simpler alternative for F9:** drop the plan fingerprint from the memo key; key rule passes on (input hashes, `rule_specs` hash, pass version, engine profile hash) only; record a plan rendering in the pass record as evidence, never as an identity | No change to extension locality | Sound, because the plan is a pure function of the rule spec, the catalog snapshot and the engine profile — all already in the key — so the fingerprint adds no information to reuse validity. Removes a dependency on platform-controlled metadata order and on a required codec from the correctness path. The pass record still needs a reproducible rendering, which R2-1's canonicalization supplies for `datafusion-proto` bytes and which `pgjson` supplies for humans | Lower: no fingerprint in the key, no codec on the reuse path | None needed | **Recommended.** Keep the `datafusion-proto` bytes (canonicalized) in `pass_records` for explainability; take them out of the reuse argument. |

**Abstractions justified by current needs.** Unchanged from revision 1's review. The `ExtensionTypeRegistry` registration (R2-3) is one generated function, not new machinery. The Python contract generation (R2-4) extends an existing generator rather than adding one.

**What remains ordinary code.** Unchanged, with one transfer: tear-set *search* moves to petgraph's `feedback_arc_set`; the tear *policy* stays platform code.

---

## 9. Verification and measurement plan

| Claim or risk | Evidence label | Test / analysis / benchmark | Conditions and expected result | Current result or remaining gap |
|---|---|---|---|---|
| Plan fingerprint reproducible (R2-1) | Capability map, measured (negative) | Two-process byte equality | Equal after metadata canonicalization | Six distinct encodings today |
| `-0.0` under distinct/join (R2-2) | Capability map, measured | Rule-compiler rejection of `Float64` keys | Rejected | Unrestricted today |
| `pse.*` storage validated by the engine (R2-3) | Capability map, measured | Wrong-storage batch rejected at planning | Typed error | Mechanism unclaimed |
| Python strictness (R2-4) | Interface-checked / measured | Extra-key rejection; `Any` lint; null→NaN refusal; library-resolution check | All reject | Defaults permit all four |
| Pins and lockfile (R2-5) | Blueprint text | `cargo tree` one version per family; `cargo deny` | Green | 23 crates and all Python libraries unpinned |
| `Exact` on key equality (R2-7) | Capability map, measured | Wrapper-provider CI test | Zero survivors | Not yet written |
| F1–F7, F15 closures | Proposed | The tests revision 2 named in §24.1 | As named | Not implemented |
| End-to-end performance | Hypothesis (DM-39) | §24.3 | No claim | Nothing measured |

**Cost accounting.** Unchanged. One addition: the `ExtensionTypeRegistry` is consulted during planning and its overhead is unmeasured (DataFusion map, unverified) — trivial to measure once a session exists.

---

## 10. Exceptions and unresolved decisions

**Exception E1 (carried).** Commit contract is P2-level validity — still not stated in the blueprint; record it in §22.2.

**Exception E2 (carried).** Rule passes are relational data; other passes are Rust.

**Unresolved MUST-level items:** R2-1 (DM-48), R2-2 (DM-40/DM-42), R2-4 items (a)–(c) (DM-42, DM-06, DM-08), R2-5 pins (DM-48), R2-6 (DM-59). R2-3 is a SHOULD-level leverage item with a MUST-level documentation erratum in §4.3.

**Decisions the author must make (recorded, not resolved):** whether the plan fingerprint stays in the memo key (§8 recommends not); YAML crate (evidence favours `serde-saphyr`); `num-dual` 0.15 with `feos` blocked; `blake3` `derive_key`; `MemoryPool` and its limit; `arrow-pyarrow` vs `pyo3-arrow`; whether to adopt `LogicalPlan::Extension` for rule bodies; F13 salsa; derivation granularity (F14); the SVD toolbox scope (F17).

---

## 11. Decision and implementation changes

**Decision: Revise** (document revision 3), narrower than revision 1's. The architecture is settled and every revision-1 priority-1 finding is closed in the text. What remains is one applied fix whose mechanism the maps measured as non-reproducible (R2-1), two claims measured false or unqualified (R2-2, R2-3), a Python boundary specified by prose its libraries do not enforce (R2-4), and dependency and library-surface statements that are wrong at the pinned versions (R2-5, R2-6). Foundational code that does not touch these — the schema registry, identities, quantities, the math IR, canonicalization — can begin against revision 2; the rule-pass memo key, the Python package, `pse-backend-native`'s FFI surface and §5.3's hashing constants must wait for revision 3.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 | Canonicalize schema/field metadata at construction; take the fingerprint out of the memo key (or make it reproducible); state the codec obligations, analyzer-rule installation and a versioned settings allow-list (R2-1) | DM-48, DM-31, DM-15 | Two-process byte equality | CI test |
| 1 | Qualify `-0.0` preservation; forbid `Float64` distinct/join keys in `RulePlanSpec`; declare the analytics collapse (R2-2) | DM-40, DM-42 | Rule-compiler rejection test | Test |
| 1 | Reword §3.3 units exclusion to authority; generate Python contract classes; strict cattrs with `Any` lint; msgspec/attrs division; ship and register `pse.*` pyarrow types; loss profile; null→NaN rule; library and solver pre-flight checks; Python pins and interpreter ranges; IDAES 2.12.0 (R2-4, F16) | DM-42, DM-06, DM-08, DM-52 | Python boundary tests | Tests |
| 1 | Adopt the maps' pin tables; committed enforced lockfile; `cargo deny`; drop `arrow-flight`; resolve `uom`; narrow §3.3 rows; `serde-saphyr`; `num-dual` 0.15; `blake3` to `pse-ids` with `derive_key` (R2-5) | DM-31, DM-48, DM-51 | One version per family; every crate in §3.1 | CI |
| 1 | Correct §20.1 `rename`, §15.3 petgraph, §15.4/§15.5 faer routes (R2-6) | DM-59 | Identifiers resolve in `build/facts/*` | Doc-lint |
| 2 | Narrow §4.3; register the `pse.*` types in the engine registry; record that DataFusion validates only field nullability (R2-3) | DM-06, DM-07 | Wrong-storage rejection test | Test |
| 2 | §5.4: `OR`-chain matching, purity, wrapper-provider test, `Precision`, selective `StatisticsRequest`s (R2-7) | DM-43, DM-20 | Wrapper test | CI |
| 2 | Parked revision-1 items F8, F10, F12 (name the IPC constants), F13, F14, F17 (choose the SVD route) | as listed in §26 | — | — |
| 3 | Leverage adoptions (R2-8): `feedback_arc_set`, formatter factory, `force_validate`, `ConfigExtension`, `pgjson`, observer, `toml::Spanned`, `implicit_derivative`, Ipopt names | DM-38, DM-49, DM-47, DM-16 | Per-adoption tests | Tests |
| 3 | Decide `LogicalPlan::Extension` for rule bodies on cost (its survival is measured) | DM-18, DM-58 | Decision record | — |

**Final check.** Revision 2's claims match its evidence except where the four maps measured otherwise, and each such place is named above with the line it occurs on; the supported scope is unchanged; the extension path is unchanged and still clear.

---

## Appendix A — Library verification ledger for this review

| # | Fact | Source seen | Label | Used by |
|---|---|---|---|---|
| A1 | Upstream's `TypePlanner` example maps a SQL type to `FixedSizeBinary(16)` with `ARROW:extension:name` metadata and is installed via `SessionStateBuilder::with_type_planner`; only one `TypePlanner` is active at a time | `docs/source/library-user-guide/extending-sql.md` via context7 | Interface-checked | R2-3 (the shape the engine expects for `pse.semantic_id`) |
| A2 | The analyzer rules are a separate ordered list from the optimizer rules; "the default pipeline may change between releases" | `datafusion/core/src/optimizer_rule_reference.md`, `query-optimizer.md` via context7 | Interface-checked | R2-1 (install analyzer rules explicitly) |
| A3 | cattrs: "By default, cattrs ignores extra keys in unstructured input dictionaries"; `forbid_extra_keys` via `make_dict_structure_fn` and a hook factory raises `ForbiddenExtraKeysError`; `transform_error` renders `ClassValidationError` groups | `docs/customizing.md`, `docs/validation.md` via context7 (`/python-attrs/cattrs`) | Interface-checked | R2-4 |
| A4 | `Optimizer::with_rules(vec![…])` and `information_schema.df_settings` (prior review) | as before | Interface-checked | R2-1 |
| A5 | `ExtensionTypeRegistry`, `ExtensionTypeRegistration::new_arc`, `SessionStateBuilder::with_extension_type_registry`; PROBE X1 rejection of a mismatched storage type | DataFusion map D10 (rustdoc extraction and probe under `docs/design_review/evidence/rust/`) | Capability map, measured — not re-run here | R2-3 |
| A6 | `datafusion-proto` byte instability with ≥2 field-metadata keys; codec required methods | DataFusion map D9, PROBE C | Capability map, measured | R2-1 |
| A7 | `GROUP BY`/join merge of `-0.0` and NaN self-equality | DataFusion map D3, PROBE D | Capability map, measured | R2-2 |
| A8 | Pyomo units raise without pint; `ExternalFunction` warns on a missing library; cattrs accepts `Any`; null→NaN on `ndarray` conversion; unregistered extension types degrade with metadata retained | Python map PROBES 1, 3, 4, 6, 8 (`docs/design_review/evidence/python/`) | Capability map, measured | R2-4 |
| A9 | `object_store` 0.13.2 trait surface (no `rename`); petgraph `maximum_matching` docs; faer `partial_svd` targets the largest singular values; `blake3` `derive_key` vs truncation; `feedback_arc_set` result | Supporting map §1, §13, ledger rows 57–61 | Capability map, docs- and probe-checked | R2-5, R2-6, R2-8 |

Additional capabilities surfaced by this review beyond the maps: none. The maps' pass-3 sweeps enumerated the library surfaces with stated denominators; this review adjudicated their findings against the charter rather than adding to the inventory.
