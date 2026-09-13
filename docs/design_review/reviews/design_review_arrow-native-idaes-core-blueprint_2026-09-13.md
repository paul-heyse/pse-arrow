# Design review — Arrow-native IDAES core architecture blueprint (revision 1)

**Standard:** `docs/design_review/design_principles/DATA_MODEL_DESIGN_CHARTER.md` (DM-01–DM-60, gates G1–G7), applied through `AGENT_DESIGN_DIRECTIVE.md` and `DESIGN_REVIEW_TEMPLATE.md`.
**Subject:** `docs/design_review/Arrow-native-idaes-core-architecture-blueprint.md`, revision 1 (2026-09-13), 2,716 lines. A design document; no code exists yet.
**Depth:** standard, with the adversarial checks recorded in §1 (Method).
**Date:** 2026-09-13.

---

## 1. Decision and scope

**Proposal.** Re-create the IDAES-PSE core modelling framework as a versioned snapshot of typed Arrow relations across seven catalog namespaces, compiled by sixteen contracted passes (P0–P15) into a backend-neutral `CanonicalMathProblem`, with DataFusion 55 as the relational engine for inference rules, catalog, batch kernels and analytics, and native Rust numerics (evaluation program, AD, faer, in-process Ipopt) plus AMPL NL and a generated Pyomo adapter as backends.

**Status.** Proposed throughout. Five engine facts are **Interface-checked** in this review (Appendix A). Claims the three capability maps established by rustdoc extraction or probe are labelled *Interface-checked (capability map)* — inspected by those documents, not re-derived here.

**Reviewer.** Claude (Fable 5.1), acting as design reviewer for the project owner.

**Affected revisions.** Blueprint revision 1; the doctrine documents it cites; the three capability maps compiled 2026-09-13; the decided dependency anchors DataFusion **55.1.0** and Arrow **59.3.0** (project owner's instruction during this review; the blueprint still says 55.0.0 / 59.2.0).

**Observable outcome claimed.** A process model whose structure, physics, mathematics and execution artifacts are queryable relations with one authority each, deterministic identities, content-addressed snapshots, contracted passes, and declared backend capabilities — reproducing IDAES's core capabilities without its imperative `build()` methods and lazy property construction.

**Baseline.** IDAES 2.10 on Pyomo: Python class hierarchies with `CONFIG` blocks and `build()`, `build_on_demand` property construction, ad hoc initialization scripts, scaling and diagnostics as utilities over a live Pyomo model.

**Supported scope and non-goals.** In scope: the modelling framework (§0.2 of the blueprint). Out of scope: example libraries, surrogates training, DMF, UI. Electrolytes, Helmholtz, MSContactor, GDP-native lowering are phase 4.

**Constraints and uncertainty.** Nothing is measured (§24.3 says so explicitly). Library surfaces are pre-1.0 for salsa, faer, petgraph, num-dual, egglog, diffsol. Parity depends on Pyomo and IDAES being installed, which they are not in this repository.

### Method and coverage

What was examined:

- The blueprint in full (all sections and appendices), the charter, directive and template in full.
- `DataFusion-rust-capability-map.md`, `Arrow-rust-capability-map.md`, `Supporting-rust-library-capability-map.md` in full, and `docs/design_review/evidence/README.md`.
- Five context7 queries against `/apache/datafusion` (Appendix A) to verify the engine facts on which gate verdicts depend and to look for capabilities the maps did not cover.

Guarantees attacked directly (each produced a finding or was cleared):

| Guarantee attacked | Result |
|---|---|
| "Names are never identity" (D4) against the authored-ID formula in §5.1 | Finding F1 |
| "The graph is the truth" for template expressions (§7.7) against the two authored representations | Finding F2 |
| Authored relations reference only authored/reference relations (implied by "no pass writes backward") | Finding F3 |
| "Unknown is never silent exclusion" (§7.6, §14.2) against the `inferred.*` schemas | Finding F4 |
| The §8.3 unit-inference rule applied to the §9.4 reference expression `P/(R·T)` | Finding F5 |
| Every transformation between P7's indexed equations and P13's scalar incidence has a pass contract | Finding F6 |
| Static families and incidence linearity are a function of the snapshot alone | Finding F7 |
| The memo key contains every input that can change a pass output | Finding F9 |
| The snapshot publication protocol (§20.1) under interruption | Cleared — write artifacts, then manifest, then CAS the ref; readers pin a manifest |
| Overlay precedence (§19.1) | Cleared at document level; tie behaviour is a named diagnostic |
| Null / NaN / bound sentinels (§4.4, §7.6) | Cleared; the Arrow map's probe independently confirms null, NaN and −0.0 are distinct through IPC |

Not examined, so silence is not assurance:

- The physical and mathematical correctness of the IDAES-derived equations in §9–§11, §13, §15–§17 (state definitions, EOS, balances, discretization stencils, diagnostics formulas, initialization sequences). This review judges meaning and authority, not thermodynamics; parity tests are the right instrument.
- Supporting-crate API claims (salsa, faer, num-dual, winnow, toml, miette, blake3, object_store, Ipopt). Taken from the supporting-library map as leads and labelled accordingly.
- The Dulmage–Mendelsohn, Hopcroft–Karp, block-triangularization and NL-writer algorithms as algorithms.
- Hash-consing collision behaviour across scopes; the `scope_instance_id` column on shared nodes (an observation only).

---

## 2. Authority and lifecycle map

Reconstructed from the blueprint. Cells the document does not settle are marked **[invented]**; each is evidence for a finding.

| Concept or fact | Semantic type and identity | Authority / owner | Revision or snapshot boundary | Permitted update path | Derived representations |
|---|---|---|---|---|---|
| Relation schema (columns, types, invariants) | `reference.schema_*` rows; `relation_id`; `pse.contract.fingerprint` | The registry (§4.1) | Registry fingerprint in every manifest | Registry edit + regeneration + migration row | Arrow schemas, typed views, builders, validators, providers, docs, JSON Schema (§4.2) |
| Operator contract | `reference.operator_specs` | The Rust operator table in `pse-mathir` (§7.3) — the relation is generated from code | Compiler version | Code change | The relation, nominal-value visitor, backend lowerings |
| Authored entity (unit, species, template, case) | `pse.semantic_id` = `blake3_128("pse:authored:v1" ‖ package_id ‖ qualified_name)` (§5.1) | Package document via change set | Model revision | Change set only (§22.2) | Everything in `normalized`/`inferred`/`compiled` |
| **Identity under rename** | **[invented]** — the ID formula makes a rename a new entity; no `rename` op exists; conformance tests (§24.1) require rename stability | — | — | — | — |
| Template equation | `authored.template_equations.expression : pse.expr_dsl` **and** `authored.template_expr_*` graph (§6.6 line 831) | **[invented]** — §7.7 says "the graph is the truth"; both are in `authored` and both are reachable by `change_ops` | Model revision | Change set | `compiled.math_*` |
| Solver profile, discretization policy, plan template | `compiled.solver_profiles`, `compiled.discretization_policies` (§6.11) | **[invented]** — written by P14/P11 yet referenced by `authored.case_policies` and `authored.continuous_domains` | **[invented]** | **[invented]** — a case cannot reference a row a later pass creates | Bound plans per problem |
| Method selection | `authored.method_selections` | Property package document | Model revision | Change set | `inferred.method_resolutions` (P6) with preference order persisted |
| Inference outcome (true/false/unknown/conflict) | prose only (§7.6, §14.2 rule 3) | **[invented]** — no `inferred.*` schema except `method_resolutions` carries a `status` column | — | — | — |
| Case specification | `authored.case_specs` (target by DSL path, priority, overlay chain) | Case document / runtime operation producing an overlay | Case revision | Change set; `propagate`, `copy_at_time` produce new overlay cases | `compiled.variable_order`, `equation_order` |
| Compiled mathematics | `compiled.math_*`, `symbols`, `math_equations` (IDs derived from instance × decl × index) | Passes P7–P11 | Snapshot content hash | Never edited; regenerated | Evaluation program, NL text, Pyomo bundle |
| Run and results | `runtime.runs` (run_id), `solutions`, `duals`, `residuals`, `iterations` | The run controller | Run | Append only | Stream tables, reports, sweeps |
| Derivation | `provenance.derivations` per derived row | The pass or rule that produced the row | Snapshot | Append only | Closure report, diff report |
| Ref (`head`) | `refs/<name>.json` (§20.1) **and** `provenance.refs` (§6.13) | **[invented]** — two representations; the CAS object is presumably authoritative and the relation a projection | Ref version (e-tag) | Conditional put | — |

**Deliberately opaque behaviour.** Kernels (`KernelSpec`, §18.5): identity, signature, validity, derivatives, failure classes and bindings are declared; the body is Rust. External solver binaries reached through NL. The Pyomo adapter. Each has a registered contract; this satisfies DM-04.

**Identity behaviour.** Reorder, re-batch, reserialize: handled by §5.3 canonicalization. Regenerate: derived IDs are pure functions of the snapshot (§5.1). Rename: **undefined and contradictory** (F1). Duplicate / split / merge of units: not addressed (an ordinary extension would need "clone unit with new name", which under F1 is a new entity with no link to its origin).

---

## 3. Semantic contracts and invariants

| Contract or invariant | Representation | Enforcement boundary | Failure behaviour | Verification evidence |
|---|---|---|---|---|
| Every authored PK is registered in `authored.entities` | `closure:entity_registered` invariant row | P2 | `validation.invariant` | Proposed; negative fixture required by §24.1 |
| Unit consistency of every equation | `residual_quantity_type_id` after P10 | P10 | `math.unit_inconsistent` | Proposed — but the §8.3 rule rejects valid physics (F5) |
| Species valid in at least one phase | `closure:species_has_phase` | P2/P4 | `validation.invariant` | Proposed |
| Balance-type enum member has a law binding | P8 lookup | P8 | `law.unsupported_binding` (typed Unsupported — aligned with DM-43) | Proposed |
| Backend can lower every opcode | `compiled.backend_bindings.status` | P15 | `backend.unsupported_opcode` | Proposed |
| Connection member sets and quantity types match | `conn.member_mismatch` | P5 | `validation.invariant` | Proposed |
| Reference-state equality across packages requires explicit conversion | P10 refusal | P10 | `math.unit_inconsistent` | Proposed |
| Pushdown truthfulness (`Exact` only where the scan guarantees it) | `supports_filters_pushdown` verdicts | Provider | None — DataFusion deletes the filter on trust (Interface-checked, Appendix A) | **Unverified for `FixedSizeBinary(16)` key shapes** (F17) |
| Snapshot coherence | Manifest lists every relation hash; ref CAS | Store protocol | `runtime.infrastructure`; readers never see a half snapshot | Proposed; interrupted-publication test absent from §24.1 (F15) |
| Memo validity | Key = input content hashes + pass version | Pass engine | Stale hit is silent | Incomplete key (F9) |
| Change sets are the only write path into `authored` | `change_ops` | P1/P2 | Rejection of writes to derived relations | Proposed; governance test named |

**Absence and uncertainty.** The design distinguishes: free unknown (a symbol, never null), no initial guess (`case_specs.initial` null with a recorded substitution source), unbounded (`pse.bound{unbounded}`), missing measurement (null value with a provenance row), unresolved and ambiguous method resolution (typed status). It **does not** give "inference could not decide" a representation in any `inferred.*` head relation other than `method_resolutions` (F4). Overlay "untouched" is null in `case_specs`; there is no "reset to template default" value, which is acceptable because reset is expressed by discarding overlays (§17.1) — worth stating.

**Equivalence requirements.** Byte equality: canonical IPC bytes (§5.3) — under-specified on alignment, metadata version and metadata key order (F12). Structural equality: schema fingerprint. Semantic equivalence: hash-consed canonical form after P10, with commutative reordering by child hash declared as not ulp-preserving (§7.3) — an honest DM-40 declaration. Approximate numerical agreement: parity tolerances per slice (§24.2). These four are kept distinct, which satisfies DM-15 in principle.

---

## 4. Derivation and execution design

| Stage | Input revisions and dependencies | Output contract | Preconditions / assumptions | Effects and mutable ownership | Provenance / invalidation |
|---|---|---|---|---|---|
| P1 parse / change set | Package documents | `authored.*` rows with source spans | Documents parse | Writes a new model or case revision | Document content hash on rows |
| P2 validate | `authored.*`, `reference.*` | Findings; pass record | — | None | — |
| P3 normalize | `authored.*` | `normalized.*` ("canonical copies") | Paths resolve | None | Derivations |
| P4–P6 closures (rule compiler) | `normalized.*`, `reference.method_specs` | `inferred.*` with derivations | Stratified rules; **DataFusion session pinned** | None declared; **engine version and optimizer rule set are undeclared inputs** (F9) | Per-row derivations |
| P7 instantiate | `inferred.*` | `compiled.symbols`, indexed `math_*` | Guards decidable | None | Derivations |
| P8 law expansion | Contributions, connections | Balance and connection equations; negative-completeness rows | Every contribution matched once | None | Derivations |
| P9 method realization | `method_resolutions` | Property equations, kernel bindings, `UnitConvert` | Kernels bound | None | — |
| P10 canonicalize | `math_*` | Hash-consed, typed DAG | Acyclic | None | Rewrite conditions recorded |
| P11 discretize | Policies, DAE links | Meshes, expanded symbols, stencils | Policies attached | None | IDs from parent × node |
| **Index expansion** | **[invented]** — no pass produces one scalar `math_equations` row per valid index tuple, yet P13's incidence requires it | — | — | — | — (F6) |
| P12 case binding | Cases, overlays | `problems`, `variable_order`, `equation_order` | Overlays resolve | None | Ordinals by semantic ID |
| P13 structural analysis | Problem | Incidence, DM partition, blocks, static attributes | Scalar equations | None | — |
| P14 plans | Problem, static attributes, package defaults | Scaling, initialization, solve plans, profiles | Selection from static attributes only | None | — |
| P15 lowering | Problem, plans | Evaluation program, NL, bundle, `backend_bindings` | Backend chosen | None | Artifact hashes |
| Solve (runtime) | Problem, profile, environment | `runtime.*` rows | Backend supports the problem | **Subprocess and file effects for NL solvers; Ipopt callbacks** — not declared as effects in a contract | Run manifest; **resolved solver options after capability probe not recorded** (F10) |
| Publish | Artifacts | Manifest, ref | All artifacts written | Conditional put | Manifest lists everything |

**Relationship structures.** Containment (`instance_tree`), connectivity (`topology_edges`, `connections`), dependency (pass DAG, `requested_by` chains), scheduling (`blocks.order`, `initialization_order`, `tear_candidates`), provenance (`derivations`), equivalence (hash-consing) are separate relations with declared meaning. DM-34 satisfied at document level.

**Provider selection and limitations.** Methods: P6 with a persisted preference order (DM-19 aligned). Backends: `solver_profiles.backend` plus `backend_bindings.status` (DM-43 aligned). Linear solver: capability probe — timing and persistence unspecified (F10).

**Boundary contracts.** Python: Arrow C stream, one bundle (DM-37 aligned); per-column loss of `pse.*` extension types into storage types undeclared, and `variables` in the bundle carry no quantity type (F16). Native: buffer borrowing declared as a preference; offset and nullability preconditions absent (noted by the Arrow map; folded into F12).

**Coherent publication.** Snapshot manifest + ref CAS. Sound. Committed authored revisions are guaranteed valid only to the P2 level (structural and referential); a revision that fails P3 path resolution is publishable. This is acceptable but should be stated as the commit contract (§10 exception record).

---

## 5. Representative journeys

### Ordinary extension — a new unit model

An author writes `templates/absorber.yaml` composing `cv.distributed_1d@1`, declares symbols, contributions, ports and a plan. New semantic decisions: one template document. Specialized implementation: none. Generated: nothing new (the registry is unchanged). Tests: a golden snapshot and a parity fixture. **Extension locality is excellent** — the charter §E pattern exactly. The same holds for a new pure-component method and a new costing method (§22.3). A new kernel adds one `KernelSpec` and one Rust body; adapters are generated. A new opcode legitimately touches the IR and every backend.

### Meaningful change — switching a property package's EOS

`method_selections` row changes → new model revision → P6 onward recomputed for states bound to that package (§14.4). Derived IDs of untouched instances are stable because they derive from instance × declaration × index, not from row position. The diff report (§22.4) lists added and removed equations and changed sparsity. **This journey works as designed.** The one weakness: if the author also *renames* the package while changing it, F1 turns the change into a delete-plus-create and the diff report cannot pair old and new.

### Boundary — the Pyomo parity bundle

Ordinals, bounds, initial values, expression nodes and a source map cross as Arrow streams. Pyomo units are attached from the unit set, so dimension survives; basis, reference state and affine kind do not, and `pse.semantic_id` arrives as `FixedSizeBinary(16)` unless a `pyarrow.ExtensionType` is registered. For a solve this loss is harmless; for the parity harness comparing enthalpy residuals against IDAES it is exactly the information needed to explain a mismatch. The loss is real and undeclared (F16, DM-42).

### Interruption — cancellation during a sequential-modular initialization plan

Stage k is a `runtime.runs` row with `parent_run_id`; cancellation arrives as a token. Overlays are immutable, so the model is untouched (DM-29 satisfied by construction — this is the design's strongest recovery property). What is observable: runs up to k−1 complete, run k `cancelled`, the plan run `cancelled`. What is unspecified: whether a native solve can be interrupted between Ipopt callbacks without unwinding through the C boundary (the supporting map records this as the most important rule in its Ipopt section, and the blueprint does not state it); whether an NL solver subprocess is killed and its partial SOL file discarded; and whether a retried stage (`retry_with_profile`) is distinguishable from its first attempt other than by timestamp. Tests for none of these appear in §24.1 (F15).

---

## 6. Acceptance gates

| Gate | Verdict | Evidence or scope rationale | Required action |
|---|---|---|---|
| G1 — Authority | **Fail** | Two independently mutable representations of a template equation, both in `authored` (§6.6 line 831 vs §7.7). Policies referenced by authored rows live in `compiled` (§6.10 line 1000, §6.11 lines 1044, 1062). Authored identity derives from the qualified name (§5.1 line 478) while D4 (line 183) says names are never identity. Refs exist as a JSON object and a relation. | F1, F2, F3; one-line reconciliation for refs |
| G2 — Semantic fidelity | **Fail** | The four-valued inference outcome has no column in any `inferred.*` head relation except `method_resolutions` (§7.6 line 1249, §14.2 line 2024 vs §6.4–§6.7 schemas). The §8.3 Mul/Div rule (line 1324) rejects the reference expression `P/(R·T)` (line 1463). | F4, F5 |
| G3 — Validity | **Unresolved** | No pass owns index expansion between P7 and P13 (F6); P2 cannot validate a foreign key from `authored` into `compiled` (F3); the commit contract is P2-level validity but this is not stated. | F3, F6; state the commit contract |
| G4 — Hidden behaviour | **Unresolved** | Linear-solver capability probe timing and persistence (F10). Rule bodies are opaque JSON, so "which relations does this rule read" is not a declared, queryable dependency (F8). Everything else — inspection non-mutating, passes pure, overlays immutable — is explicitly designed. | F8, F10 |
| G5 — Consistency and recovery | **Pass** (with notes) | Manifest-then-CAS publication; immutable overlays; per-stage run rows; typed failure taxonomy. Notes: retry attempts unnumbered; FFI unwind rule unstated; adversarial tests absent (F15). | F15 |
| G6 — Transformation and reuse | **Unresolved** | Memo key omits the DataFusion version, optimizer rule set and session config (F9 — mechanisms to close it verified in Appendix A). Parameter folding is ambiguous and changes static families and incidence linearity (F7). Canonical reordering's equivalence class is honestly declared. Hash contract under-specified on IPC parameters (F12). | F7, F9, F12 |
| G7 — Truthful capability claims | **Unresolved** | `Exact` pushdown on `FixedSizeBinary(16)` keys unverified; "iterative SVD for large" has no library route; condition-number cost is O(n) sparse solves without a budget; "pseudo-inverse via sparse LU" is not a route (F17). Enum members without law bindings, unsupported opcodes and missing derivatives are typed refusals — aligned. | F17 |

No weighted score offsets these. G1 and G2 fail on narrow, correctable points; nothing found requires a change to the architecture.

---

## 7. Principle findings

Ranked: correctness and authority defects first, then semantic duplication and extension difficulty, then measured cost and proportionality.

| # | Finding | Principle IDs | Concrete evidence or gap | Consequence | Proposed correction | Verification |
|---|---|---|---|---|---|---|
| F1 | Authored identity is a function of the qualified name, so rename is undefined and destructive | DM-11, DM-12, DM-02 | §5.1 line 478: `semantic_id = blake3_128("pse:authored:v1" ‖ package_id ‖ qualified_name)`. D4 line 183: "Names … are never identity". §22.2 `change_ops.op ∈ {insert, update, delete}` — no rename. `case_specs.target` is a DSL path (§6.10). §24.1 lists "rename and reorder" among conformance tests that must pass. | Renaming `H101` to `heater_1` creates a new entity: every derived ID under it changes, every `case_specs` row targeting the old path fails at P12 with `case.unknown_target`, prior runs and derivations no longer join to the model, and the §22.4 diff report shows a deletion and an addition rather than a rename. | Assign authored entity IDs at creation (UUIDv7, or blake3 of a creation nonce) and persist the ID in the document; keep `qualified_name` as an attribute. Reserve name-derived IDs for reference-package entities where the name *is* the contract, and say so. Add `change_ops.op = rename` and resolve `case_specs.target` to `symbol_id` at commit, storing both. Surface area: §5.1, §6.1, §6.10, §22.2, the parser. | Conformance test: rename a unit; assert all derived IDs, case bindings, and derivation joins are unchanged except `qualified_name` columns; assert the diff report contains one rename row. |
| F2 | Template equations have two authored representations | DM-02, DM-23, DM-14 | §6.6 line 831: "parsed form lives in `authored.template_expr_*`"; §7.7: "the graph is the truth and the text is retained … for display and re-parse tests". Both are `authored` relations and both are valid `change_ops` targets. | A change set updating `template_equations.expression` without re-parsing leaves the graph stale: the display shows one equation and the compiler instantiates another; a change set updating graph rows directly bypasses the DSL's source spans. "Re-parse tests" is a test, not a reconciliation protocol. | Make the text the only authored fact; produce the graph in `normalized.template_expr_*` from P1/P3 with a derivation. Alternatively make the graph authored and the text a rendered projection — but then the DSL loses its source spans. Surface area: one namespace move plus P1's output list. | Governance test: no `authored.*` relation has a `producing_pass_id`; round-trip test parse→render→parse is the identity on the graph. |
| F3 | Policies and profiles are placed in `compiled` while referenced from `authored` | DM-13, DM-09, DM-07 | §6.11 lines 1044 and 1062 put `discretization_policies` and `solver_profiles` in `compiled` (written by P11/P14 per §14.1). §6.10 line 1000 `authored.case_policies` and §6.3 line 647 `continuous_domains.discretization_policy_id` reference them. §1.2: "No pass writes backward into `authored`." | P2 cannot check the foreign key (the target does not exist until compilation); a profile regenerated by P14 gets a new identity and every case that named it dangles; `ipopt.default` is described as "shipped", i.e. reference data, yet lives in a derived namespace. | Split: `reference.solver_profiles` / `authored.solver_profiles` (policy definitions), `authored.discretization_policies`, `reference.plan_templates`; keep in `compiled` only the *bound* instances per problem (`compiled.problem_profiles`, `compiled.bound_plans`). Surface area: §6.11, §14.1 P11/P14 output lists, Appendix B. | Governance test: no `authored.*` or `reference.*` column has `fk_relation_id` into `compiled`/`runtime`. |
| F4 | "Unknown is never silent exclusion" has no representation in the inferred relations | DM-08, DM-47 | §7.6 line 1249 and §14.2 rule 3 (line 2024): rows "are written to the head relation with `status`". No `status` column exists on `inferred.phase_species`, `instance_features`, `instances`, `ports`, `port_members`, `scope_members`, `boundary_crossings`, `topology_edges`, `state_flash_required` (§6.4, §6.7, §9.2). Only `method_resolutions` has one. | A rule evaluating `dynamic(I)` where the parent's `dynamic` is itself undecided has nowhere to write `unknown`; the implementer either drops the row (the forbidden outcome) or writes `false`, which then silently disables holdup terms downstream. | Either add `status : enum(TruthValue)` to every rule-produced head relation (generated from the registry, so one declaration), or add one relation `inferred.undecided(rule_id, head_relation_id, key, reason, derivation_id)` and require rules to write there. State which. | Fixture: a species with `valid_phase_types = null` and a phase of type `undefined`; assert an `unknown` row and a diagnostic, and assert the species is absent from `phase_species` rather than present with a guessed value. |
| F5 | The Mul/Div affine-kind rule rejects valid thermodynamics written in the reference templates | DM-06, DM-40, DM-24 | §8.3 line 1324: "affine kinds must be absolute-free (an absolute temperature may not be multiplied unless the author wraps it as `convert(T, K)`)". §9.4 line 1463: `dens_mol_phase[p] = P/(R·T)` with no wrapper; §9.3 NIST method `t = T/1000{K}`; every Arrhenius and EOS expression divides by `T`. | Either P10 rejects the entire reference property library, or every template wraps every temperature — noise that is itself an error source — and two implementers will pick different readings of "absolute". The conflation is between *point vs difference* (T vs ΔT, which governs addition) and *affine units* (°C, gauge), which is a unit property already handled by `is_affine` and P3's conversion to canonical units. | Redefine `affine_kind ∈ {point, difference}` on quantity types; restrict Mul/Div only to values still carried in an affine *unit* (impossible after P3 normalizes to canonical SI units); keep the addition algebra (point ± difference → point; point − point → difference; point + point → error unless declared as a weighted mean). Gauge pressure is a reference-state distinction, not an affine kind. Surface area: §8.1, §8.3, one enum. | P10 test set: `P/(R·T)` types without wrapping; `T_out − T_in` yields difference; `T + ΔT` yields point; `T1 + T2` errors; `25{degC}` literals convert to K at P3 before inference. |
| F6 | Index expansion (scalarization) has no pass and no contract; §7's pass numbers are stale | DM-22, DM-21, DM-18 | §7.1 line 1166: "one indexed equation declaration … until P13 lowers it"; §7.2 lines 1189–1196: "via discretization (P13)", "after P13/P14"; §7.1 line 1168: "fails explicitly at P14". §14.1: P11 is discretization, P13 is structural analysis, P15 is backend lowering. §14.1 lists no pass that turns indexed `math_equations` (P7) into one row per valid index tuple, yet `compiled.incidence` (line 1085), `equation_order` and DOF need scalar rows. §8.2 line 1311 says P8 inserts `UnitConvert`; §9.3 and §14.1 say P9. | Implementer A flattens in P7 (losing DM-18's compact form and making P7's per-instance memo huge); implementer B flattens in P15 (then P13's DOF and DM partition run over indexed rows and are wrong). The pass contracts in `reference.pass_specs` cannot be written from the document as it stands. | Add "P12a index expansion" (or fold into P12 with its own postcondition): input indexed equations + `valid_index_tuples` + meshes; output scalar `math_equations` rows with `parent_equation_id` and `index`, `Broadcast`/`SumOver` bodies rewritten per instance, IDs per §5.1. Renumber §7 and §8.2 to match §14.1. Surface area: §7.1, §7.2, §8.2, §14.1, §6.9 (add `parent_equation_id`). | Golden test on slice A: `math_equations` row count after expansion equals Σ over declarations of \|valid tuples\|; P13 preconditions assert every equation is scalar. |
| F7 | Whether parameter symbols are folded is undecided, and it changes static families, incidence linearity and the memo | DM-24, DM-40, DM-32 | §7.4 step 3 line 1218: folding "only among literal constants". §14.4 line 2050: "constant folding and simplification that used a parameter value record the value's hash". §19.4 line 2366: estimated parameters are symbols "switched from `parameter` to `free` by the case" — so parameters must survive as symbols to P12. §6.12 `incidence.linear : bool`; §7.5 `AFFINE_EQUALITY` requires constant coefficients; §18.7 selects solve plans "from static attributes only". | `flow_mass = mw·flow_mol` is `BILINEAR` if `mw` stays symbolic and `AFFINE_EQUALITY` if folded; DM partition, block order, nominal-value scaling and the solve-plan class differ between the two readings, and a case that estimates `mw` needs the unfolded form while a case that fixes it benefits from the folded one. | State: P10 never folds `role = parameter` symbols (the canonical graph is case-independent and hashes identically across cases). P12 produces a case-bound view in which `treatment = parameter` symbols are substituted, and P13's static attributes are computed on that view with the substitution recorded in the derivation. Surface area: §7.4, §14.4, §14.1 P12/P13 contracts. | Test: same model, two cases (mw fixed vs mw estimated): identical P10 hashes; different `math_static_analysis.family` for the affected equations; `incidence.linear` differs accordingly. |
| F8 | Rule bodies, invariant specs and selectors are opaque JSON text, contrary to D1 | DM-10, DM-31, DM-49, DM-01 | D1 (§2): "no JSON column that carries required behavior". §4.1 line 378 `schema_invariants.spec : Utf8 (JSON)`; §6.11 line 1042 `rule_specs.body : text (typed RulePlanSpec JSON)`; §6.7 line 878 `scopes.selector : text (typed JSON)`; §6.10 line 1008 `case_sets.generator`; §6.11 line 1057 `init_stages.target`; §6.13 lines 1115, 1123. The rule bodies are the compiler's inference logic — the most decision-relevant behaviour in the design. | Stratification ("a rule may negate only relations in a lower stratum") cannot be checked by a P2 relational invariant; "which rules read `authored.species`" cannot be queried; impact analysis for §22.4's diff report and §14.4's invalidation must parse text; a semantic diff of a rule change is a JSON diff. | Promote the load-bearing part: `reference.rule_plan_nodes` / `rule_plan_edges` typed like `math_expr_*` (scan, filter, project, join, anti-join, union, distinct with typed payloads), or at minimum a derived `reference.rule_dependencies(rule_id, relation_id, mode ∈ {read, negate}, stratum)` generated when the registry is built. Selectors: a small `selector_terms` relation. Leave `diagnostics_findings.values` as JSON (genuinely open-ended). Surface area: §4.1, §6.11, one generator. | P2 invariant `rules.stratified_negation` expressed as a RulePlanSpec over `rule_dependencies`; a query "rules reading relation X" used by the diff report. |
| F9 | The DataFusion engine is an undeclared input of every inferred relation: version, optimizer rule set and session config are outside the memo key, and the plan fingerprint has no mechanism | DM-31, DM-32, DM-48 | §14.2 rule 5 pins "config snapshot, function registry hash, catalog snapshot" and requires a plan fingerprint; §14.3 keys memos on "content hashes of its declared inputs and its own version". Neither names the DataFusion version or the optimizer/analyzer rule list. **Interface-checked** (Appendix A): `Optimizer::with_rules(vec![…])` accepts an explicit rule list; `information_schema.df_settings` exposes the full config as a table; `datafusion-proto` serializes a `LogicalPlan` to protobuf bytes with a `LogicalExtensionCodec` for UDFs and extension nodes. | A DataFusion patch upgrade changes a rewrite (or a kernel UDF's `simplify`), the pass record says "unchanged inputs", and the memo returns a stale `inferred.*` artifact whose hash no longer matches what recomputation would produce — an invalid reuse that nothing detects until a clean rebuild. | Add to every rule-executing pass's key: DataFusion crate version, the ordered optimizer/analyzer rule names (and pin the list with `Optimizer::with_rules` so an upgrade cannot add rules silently), and blake3 of `information_schema.df_settings`. Define the plan fingerprint as blake3 of the `datafusion-proto` encoding under the platform's `LogicalExtensionCodec`, re-verified per upgrade. Surface area: §14.2 rule 5, §14.3, `pass_specs.cache_key_inputs`. | Test: bump the pinned rule list; assert every rule-pass memo misses. Test: two sessions with different `batch_size` produce identical plan fingerprints (the fingerprint must not include execution-only settings) — state which settings are semantic. |
| F10 | Solver capability probing happens at an unstated time and the resolved options are not recorded per run | DM-19, DM-28, DM-48 | §18.3 line 2285: `linear_solver = ma57` "when HSL is available and `mumps` otherwise (a capability probe …)"; §26 line 2617: "capability probe at profile creation". `runtime.runs.environment` records `solver_version` but not the linear solver actually used; `solver_profiles.options` is a shared row. | `pse reproduce` on a host without HSL runs the same `solver_profile_id` with MUMPS, takes a different iterate path, and the manifest claims the same profile. The selection depended on ambient state that no record captured. | Make the probe an explicit runtime operation whose result is a `runtime.host_capabilities` row; record `runs.resolved_options` (the options actually passed to `CreateIpoptProblem`) on every run; a profile that names `ma57` on a host without HSL is a typed `capability.backend` failure unless the profile declares a fallback policy. | Reproduction test: replay a run's manifest on a host lacking HSL; assert a typed failure or, with a declared fallback, a run row whose `resolved_options` differ visibly from the original. |
| F11 | Dependency anchors are stale and the pin mechanism does not achieve the stated intent; one authoring-boundary crate is unmaintained; one dual-number split is unaddressed | DM-48, DM-51, DM-45 | §3.1 pins DataFusion 55.0.0 / Arrow 59.2.0; the decided anchors are **55.1.0 / 59.3.0** (project owner, this review) and the evidence extraction under `docs/design_review/evidence/` is at those versions. Both maps report (Interface-checked, capability map) that pinning only the umbrella crate lets sub-crates float and that `cargo tree -d` cannot detect a mixed family. The supporting map reports `serde_yaml 0.9.34+deprecated` ("no longer maintained") at the authoring boundary, and `feos-core 0.10.1` requiring `num-dual ^0.14` against `num-dual 0.15`. | Mixed-family `ArrayRef`s are distinct Rust types with identical names; the failure message never says "version". An unmaintained parser fed untrusted authoring documents is a DM-45 trust gap. Two `num-dual` copies make `DualNum` two traits and silently break §18.5's monomorphization. | §3.1: pin every `arrow-*` and `datafusion-*` crate with `=` at 59.3.0 / 55.1.0 (or commit and enforce a lockfile); add `num-dual` to the pin table with `cargo tree -d` coverage; replace `serde_yaml` (the supporting map names three candidates; choose on span support and panic-freedom); enable `ffi` and `canonical_extension_types` features. | CI: `cargo tree` asserts one version per family; a governance test that the YAML crate is the chosen one. |
| F12 | The canonical hashing contract leaves three byte-affecting choices to library defaults, and the native borrow path omits two preconditions | DM-15, DM-48, DM-37 | §5.3 step 3: "deterministic alignment" — no value; no `MetadataVersion`; schema and field metadata are `HashMap`s whose iteration order is unspecified, yet `pse.contract.fingerprint` hashes them. The Arrow map's probe (Interface-checked, capability map) measured that alignment changes the IPC bytes (872 vs 1032 for identical rows) and that concatenating batches is what makes split-invariant hashing true. §20.1 stores the same relation as `.arrow` and `.parquet` without saying which is hashed. The supporting map notes `serde_arrow` writes its own field-metadata key. D11/§18.2 borrow Arrow buffers without stating the `ptr_offset() == 0` and `null_count() == 0` preconditions. | A change of Arrow default alignment or a different metadata insertion order produces a different `content_hash` for identical semantics, so `snapshot_id`s in history stop reproducing; a sliced or nullable array borrowed by the tape reads wrong rows with plausible numbers. | §5.3: name the alignment constant and `MetadataVersion` and version them with the contract; sort metadata keys before hashing; state that content identity is defined over the canonical IPC encoding only and Parquet is never hashed; classify foreign metadata keys as volatile. §18.2: state the borrow preconditions. | Canonicalization property tests: identical relations under re-batching, metadata re-insertion order and dictionary re-encoding hash equal; a Parquet round trip reproduces the IPC hash. Negative test: a sliced array is copied, not borrowed. |
| F13 | `salsa` duplicates the memoization the artifact store already provides (over-construction) | DM-58, DM-57, DM-32 | §14.3 line 2032: passes are salsa queries "keyed by the content hashes of its declared inputs and its own version; the memo stores the output artifact hashes". `reference.pass_specs` already declares `inputs` and `cache_key_inputs` (§6.11). Salsa's distinctive value is *automatic* dependency tracking and backdating; here dependencies are declared by contract and backdating falls out of content-hash keying (an unchanged output hash is an unchanged input hash for the next pass). | A second incremental mechanism with its own database, macro dialect, unwind-based cancellation (which cannot cross the Ipopt FFI), pre-1.0 API churn and an unstated MSRV, whose safety net (catching an undeclared read) is precisely the case the contract already forbids. Sixteen passes do not need a query engine to memoize. | Defer salsa. Implement the pass engine as a map from (pass_id, pass_version, input hashes, engine key from F9) → output hashes, backed by the artifact store; per-instance memoization inside P7 uses the same key shape. Keep a seam so salsa can be adopted if sub-pass granularity proves necessary. Removes machinery; no verification needed beyond the existing incrementality tests (§24.1). | — |
| F14 | Per-row derivations for mechanically expanded rows are mandatory and unmeasured | DM-39, DM-58, DM-46 | §1.1 line 120: "every derived row carries a derivation"; §14.2 rule 4; `provenance.derivations.supporting : list<struct>` per row (line 1118). Discretization-expanded symbols and connection equations are pure functions already encoded in their IDs (§5.1). §24.3 lists no provenance-cost benchmark. | For a distributed dynamic model (10⁴ nodes × 10² symbols) the derivation relation is larger than the mathematics it explains; compile time and store growth carry a large constant nobody has measured, and the closure report depends on scanning it. | Declare derivation granularity per relation in the registry: `row` (law expansion, method resolution, inference — where negative completeness is the deliverable) or `rule` (discretization, connection expansion — reconstructible from the rule and the ID formula). Add a provenance-size and pass-time benchmark to §24.3. | Benchmark on slice C at increasing mesh size: derivation rows and bytes per equation; a reconstruction test that regenerates `rule`-granularity derivations from IDs. |
| F15 | Adversarial lifecycle tests are absent from the test plan | DM-54, DM-30 | §24.1 covers governance, canonicalization, rules, kernels, structural, conformance, diagnostics, initialization, incrementality, provenance, doctrine. It does not list: truncated or partially written artifact under a correct hash name; ref CAS conflict; cancellation mid-pass and mid-solve; NL subprocess kill; stale memo after an engine upgrade; concurrent change sets on one base revision. The Arrow map notes a `StreamWriter` dropped without `finish()` yields a truncated stream with no error at the call site. | The design's recovery story (§20.1, §17.1, §23.2) is asserted, not protected; the first regression will be discovered in production. | Add a "lifecycle" layer to §24.1 with the six cases above, each with a negative assertion (rejected at its declared boundary) and, for publication, a hash-verify-before-put check. | The tests themselves. |
| F16 | The Python boundary's loss profile is undeclared | DM-42 | §21.1 line 2455: bundle `variables` carry ordinal, id, name, bounds, initial, treatment, scale — no `quantity_type_id`. §4.4: Python consumers register matching extension types; unknown consumers see storage. Nothing states, per column, whether `pse.*` survives or degrades. | A parity mismatch on an enthalpy residual cannot be explained from the bundle because reference state and basis are absent; a consumer reading `FixedSizeBinary(16)` cannot tell a semantic ID from a content hash. | Add `quantity_type_id` and `unit_id` to bundle `variables`; publish a per-column loss declaration in the bundle manifest; register the ten `pyarrow.ExtensionType` classes in `python/pse`. | Round-trip test Rust → Python → Rust preserving extension names; parity harness test that reads reference state from the bundle. |
| F17 | Three numerical-analysis capabilities are claimed without a route or a budget | DM-43, DM-59, DM-39 | §15.5: "dense `faer` SVD for small problems, iterative for large" — the supporting map finds no iterative SVD in faer 0.24 (Interface-checked, capability map). §15.4: `‖J‖_F·‖J⁻¹‖_F` "via `faer` sparse LU (pseudo-inverse for non-square)" — a sparse LU is not a pseudo-inverse route, and the Frobenius inverse norm needs O(n) sparse solves. §5.4: `Exact` pushdown on `pse.semantic_id` keys — whether DataFusion presents `FixedSizeBinary(16)` equality and `IN` lists in a shape the provider can match is unverified; `Exact` causes the optimizer to delete the filter (Interface-checked, Appendix A). | An "iterative SVD" that does not exist blocks the SVD toolbox above dense size; a condition number that costs a full solve per column is a diagnostic nobody runs; a wrongly `Exact` filter returns rows the query excluded, silently. | §15.5: either implement Lanczos over faer's `matrix_free`, add a crate, or restrict the toolbox to dense-tractable sizes and say so. §15.4: use QR least-squares for non-square; add a Hager–Higham 1-norm estimator or declare the exact Frobenius form opt-in. §5.4: spike the filter shapes before claiming `Exact`; add the test-only wrapper provider that re-applies every `Exact` filter and fails on a survivor. | The spike; a CI wrapper-provider test over golden snapshots; a benchmark for the condition-number path. |

**Observations (not findings).** `normalized` holds "canonical copies of authored relations after P3" — a full duplicate namespace; consider views for relations P3 does not change. `pse.ordinal_ref` carries its target relation in platform metadata rather than in the extension type's own metadata, so the type cannot enforce it. `compiled.math_expr_nodes.scope_instance_id` is ambiguous for a hash-consed node shared across scopes. `runtime.runs` lacks an attempt ordinal for retried stages. `provenance.refs` and `refs/<name>.json` need one sentence naming the CAS object as authoritative.

### Applicability

| Group | Applied | Reason |
|---|---|---|
| 1 Semantic authority | Yes | The design's thesis is D1; G1 findings F1–F3 |
| 2 Types, schemas, invariants | Yes | Physical typing and four-valued inference; F4, F5 |
| 3 Identity, versions, consistency | Yes | Three identity forms and content hashing; F1, F12 |
| 4 Declarative composition | Yes | Templates, laws, plans; the extension journey is the design's strength |
| 5 Compilation and preservation | Yes | Sixteen passes; F6, F7 |
| 6 Planning, execution, effects | Yes | Overlays, runs, backends; F10 |
| 7 Dependencies and incrementality | Yes | Memo keys and salsa; F9, F13 |
| 8 Execution representations and performance | Yes, lightly | Native layouts and DataFusion's four roles are well placed; no measurements exist (§24.3 says so) |
| 9 Boundaries and providers | Yes | Kernels, backends, Python; F16, F17 |
| 10 Provenance and reproducibility | Yes | Derivations and manifests; F14 |
| 11 Evolution and verification | Yes | Registry migrations, test layers; F15 |
| 12 Leverage and proportionality | Yes | F13, F14 and the §8 alternative |

**Maturity assessment.** Not scored. Everything is at rating 1 (stated in prose) by construction; a numeric total would manufacture precision. The proposed target is rating 3 at the end of each vertical slice, which §24.2's acceptance criteria are designed to demonstrate.

---

## 8. Alternatives and architectural leverage

| Alternative | Semantic duplication and extension locality | Correctness and operational risks | Implementation / maintenance cost | Performance evidence | Why selected or rejected |
|---|---|---|---|---|---|
| **Current baseline** (IDAES on Pyomo) | Every unit re-expresses construction, validation, initialization and scaling in Python; five parallel maintenance surfaces (proposal §2.1) | Lazy construction by attribute access; inspection mutates; identity by name path; no snapshots | Established; large community | Known to be construction-bound at scale | The reason the project exists |
| **Proposed design** (blueprint rev. 1) | One declaration per unit, method, kernel, plan; adapters generated; inference as data | The findings above — all correctable within the architecture | High: ~150 relations, a registry-driven generator, a rule compiler, a native AD program, three backends | None yet (honestly stated) | Selected, subject to the §11 changes |
| **Simpler viable alternative** — typed relations and generated views as proposed, but compiler passes as ordinary Rust functions over those views (no `RulePlanSpec`, no fixed-point executor, no salsa); DataFusion only for the snapshot catalog, batch kernels and analytics; whole-pass memoization keyed by declared-input content hashes; provenance rows written by the pass code | Extension locality for units, methods, kernels and plans is **identical** (those are documents and `KernelSpec`s either way). Inference rules become Rust functions — a SHOULD-level deviation from DM-16 — but with typed views they remain readable and are still tested against golden inferred relations | Loses the guarantee that every inferred row's derivation is produced mechanically by the rule engine (it becomes a discipline); loses four-valued semantics as a framework property. Gains: no engine-version dependency inside the compiler (F9 largely disappears), no JSON rule bodies (F8 disappears), no second memo mechanism (F13 disappears) | Materially lower for phases 0–1; the rule compiler and fixed-point executor are the largest pieces of platform machinery with no analogue in IDAES | None — but it removes the unmeasured per-plan optimizer cost from the compile path | **Partially adopted in the recommendation:** keep the rule compiler for the closure passes P4–P6 and P8's contribution matching, where set semantics, stratified negation and per-row provenance are the deliverable and where agents are expected to add rules; drop salsa (F13); type the rule bodies (F8) so that what remains is genuinely data. For everything else the passes are already ordinary Rust in the blueprint. |

**Abstractions justified by current needs.** The schema registry and generator (≈150 relations; five generated artifact kinds — DM-52 leverage is real). The math IR with typed payload relations (DataFusion `Expr` has no place for a quantity type; the map confirms). `KernelSpec` with generated adapters (six bindings per kernel). The contributions substrate and law templates (replaces the largest duplicated construction logic in IDAES). Immutable case overlays (DM-29 by construction). Content-addressed snapshots with manifests (DM-14, DM-48).

**Abstractions not yet justified.** salsa (F13). Per-row derivations for mechanically expanded relations (F14). `egglog` is correctly gated to phase 4 and correctly required to be an optimization, never a correctness dependency (the supporting map reaches the same conclusion). `LogicalPlan::Extension` nodes for rule bodies (DataFusion map item 26) would be additional machinery; evaluate only after F8's typed rule relations exist, since they answer the same provenance question at the data level.

**What remains ordinary code.** Hopcroft–Karp, Dulmage–Mendelsohn, Tarjan, block triangularization; the evaluation program and reverse-mode adjoint; the NL writer and SOL reader; the Ipopt driver; the cubic-root and Helmholtz-state kernels; bubble/dew and Rachford–Rice estimators. The blueprint keeps all of these behind contracts and does not try to make them declarative — correctly.

---

## 9. Verification and measurement plan

| Claim or risk | Evidence label | Test / analysis / benchmark | Conditions and expected result | Current result or remaining gap |
|---|---|---|---|---|
| Rename preserves identity, bindings and provenance (F1) | Proposed | Conformance test "rename and reorder" (§24.1) | After the F1 correction: only `qualified_name` columns differ | Fails by construction today |
| One authority per template equation (F2) | Proposed | Governance test: no `authored.*` has `producing_pass_id`; parse→render→parse identity | Passes after moving the graph to `normalized` | Not yet expressible |
| No authored FK into `compiled` (F3) | Proposed | Registry query over `schema_columns.fk_relation_id` | Zero rows | Two violations today |
| Unknown ≠ false (F4) | Proposed | Fixture with an undecidable phase validity | An `unknown` row and a diagnostic; no guessed row | No column to assert on |
| Unit inference accepts reference physics (F5) | Proposed | P10 over every reference template expression | Zero `math.unit_inconsistent` on the shipped library; the five algebra cases in F5 | Rule contradicts §9.4 |
| Index expansion contract (F6) | Proposed | Golden row counts; P13 precondition | Scalar rows = Σ valid tuples | No pass |
| Static attributes independent of parameter treatment (F7) | Proposed | Two-case test | Identical P10 hashes; families differ only on the case-bound view | Undecided |
| Memo invalidates on engine change (F9) | Interface-checked (mechanisms) / Proposed (test) | Bump pinned rule list → all rule-pass memos miss; plan fingerprint stable across execution-only settings | As stated | Key incomplete |
| Canonical hash invariance (F12) | Interface-checked (capability map probe) / Proposed | Property tests: re-batch, metadata order, dictionary re-encode, Parquet round trip | Equal hashes | Alignment and key-order unspecified |
| `Exact` pushdown truthfulness (F17) | Interface-checked (semantics) / unverified (shapes) | Spike on `FixedSizeBinary(16)` filter shapes; CI wrapper provider re-applying `Exact` filters | Zero survivors | Spike not done |
| Interrupted publication and cancellation (F15) | Proposed | Six lifecycle tests | Rejected at the declared boundary; no partial artifact under a correct name | Not in the plan |
| Backend conformance (native vs Pyomo residuals and Jacobians) | Proposed | §24.1 parity harness, 1e-10 relative at random valid points | As stated | Requires Pyomo and IDAES installed; neither is in this repository |
| End-to-end performance | Hypothesis (DM-39) | §24.3 per-stage benchmarks at increasing mesh and component counts, plus provenance size (F14) | No claim before measurement — the blueprint already says this | Nothing measured |

**Cost accounting (material categories).** Construction and compilation: sixteen passes, the rule plans' optimization time, provenance rows (unmeasured; F14). Transfer: one bundle per Python crossing (bounded). Execution: native tape; Ipopt; NL subprocesses. Storage: dual `.arrow`/`.parquet` artifacts per relation plus derivations. Recovery: idempotent artifact writes by hash. Inspection: SQL over the snapshot catalog. Memory: DataFusion pool (choice unmade — the DataFusion map's item 19) plus native workspaces.

---

## 10. Exceptions and unresolved decisions

**Exception E1 — commit contract is P2-level validity.**
Principle IDs: DM-14, DM-07. Scope: authored model and case revisions. Reason: path resolution and unit inference need the whole snapshot and are legitimately compiler work; requiring P3–P10 success before commit would block incremental authoring. Consequence: a committed revision may not compile. Compensating control: the closure report (§14.5) gates *publication as a compiled problem*; the commit boundary must be documented as "structurally and referentially valid" and the UI/agent surface must show the last closure status per revision. Evidence: §22.2, §14.5. Owner: platform architecture. Revisit trigger: an agent workflow that needs compile-clean commits.

**Exception E2 — inference rules are relational data (DM-16 default retained) but only for the closure passes.**
Principle IDs: DM-16, DM-58. Scope: P4–P6 and P8 contribution matching. Reason: set semantics, stratified negation and per-row provenance are the deliverable there; elsewhere passes are ordinary Rust. Consequence: two ways to write a pass. Compensating control: `pass_specs.determinism` and the same contract shape for both. Owner: compiler lead. Revisit trigger: a third party needs to add inference outside P4–P8.

**Unresolved MUST-level items (the design is incomplete against these until revised):** F1, F2, F3 (DM-02/DM-11/DM-13); F4 (DM-08); F5 (DM-06); F6 (DM-22); F7 (DM-24); F9 (DM-31/DM-32); F15 (DM-54). Each has a correction with small surface area.

**Unresolved decisions the author must make (recorded, not resolved here):** the YAML crate; `num-dual` version; the `MemoryPool` implementation and limit; the `blake3` derivation mode for 128-bit IDs (`derive_key` vs truncation — the supporting map notes truncation safety is unverified in the docs it read); `arrow-pyarrow` vs `pyo3-arrow`; derivation granularity per relation; the iterative-SVD route; the condition-number budget; the Ipopt `-sys` binding strategy and linking.

---

## 11. Decision and implementation changes

**Decision: Revise** (document revision 2 before phase-0 code).

**Reason.** The architecture is sound and unusually well specified for a document at this stage: three representations with one identity system, typed relations with declared authority, contracted passes, immutable overlays, content-addressed snapshots, typed refusals for unsupported behaviour, and an honest stance on unmeasured performance. The failures are narrow and local: identity derived from names (F1), a duplicated authored representation (F2), policies in the wrong namespace (F3), a missing column for the design's own four-valued logic (F4), a unit rule that contradicts the design's own reference expressions (F5), a missing pass (F6), an undecided folding policy (F7), an incomplete memo key (F9), and a missing test layer (F15). None requires structural change; all must be fixed before the registry and pass contracts are generated, because each is cheap to change now and expensive after `tests/golden` exists.

| Priority | Change | Principle IDs | Acceptance evidence | Regression protection |
|---|---|---|---|---|
| 1 | Creation-time authored IDs; `rename` change op; case targets resolved to symbol IDs at commit (F1) | DM-11, DM-12, DM-02 | Rename conformance test passes | The conformance test |
| 1 | Move `template_expr_*` to `normalized`, produced by P1/P3 (F2) | DM-02, DM-23 | No authored relation with `producing_pass_id` | Governance test |
| 1 | Move profiles, discretization policies and plan templates to `reference`/`authored`; keep bound instances in `compiled` (F3) | DM-13, DM-09 | No authored FK into `compiled` | Registry governance query |
| 1 | `status` on rule-produced head relations or an `inferred.undecided` relation (F4) | DM-08 | Undecidable-fixture test | Fixture |
| 1 | Redefine affine kinds as point/difference; restrict Mul/Div by affine *unit* only (F5) | DM-06, DM-40 | Reference library passes P10; five algebra cases | P10 test set |
| 1 | Add the index-expansion pass and renumber §7/§8.2 (F6) | DM-22, DM-21 | Golden row counts; P13 precondition | Pass contract |
| 1 | State parameter-folding policy: symbolic at P10, substituted on the P12 case-bound view (F7) | DM-24, DM-32 | Two-case test | Test |
| 1 | Engine version, pinned optimizer rule list and `df_settings` hash in rule-pass keys; plan fingerprint via `datafusion-proto` (F9) | DM-31, DM-32, DM-48 | Rule-list bump invalidates memos | Test |
| 1 | Lifecycle test layer (F15) | DM-54 | Six tests exist and pass | The tests |
| 2 | Type the rule bodies and selectors, or at least generate `rule_dependencies` (F8) | DM-10, DM-31 | Stratification as a P2 invariant | Invariant |
| 2 | Runtime capability probe as a recorded operation; `runs.resolved_options` (F10) | DM-19, DM-48 | Reproduction test on a host without HSL | Test |
| 2 | §3.1: 55.1.0 / 59.3.0 family-wide `=` pins; `num-dual` pinned; replace `serde_yaml`; enable `ffi`, `canonical_extension_types` (F11) | DM-48, DM-51, DM-45 | `cargo tree` one version per family | CI |
| 2 | §5.3: alignment, `MetadataVersion`, sorted metadata keys, IPC-only identity, foreign-key classification; §18.2 borrow preconditions (F12) | DM-15, DM-48 | Canonicalization property tests | Tests |
| 2 | Bundle carries quantity types; per-column loss declaration; Python extension types registered (F16) | DM-42 | Round-trip test | Test |
| 2 | Resolve the three unbacked numerical routes and spike `Exact` shapes (F17) | DM-43, DM-59 | Spike report; wrapper-provider CI test | Test |
| 3 | Defer salsa; artifact-hash memo keyed by declared inputs (F13) | DM-58 | Incrementality tests unchanged | Existing tests |
| 3 | Declare derivation granularity per relation; add the provenance-size benchmark (F14) | DM-39, DM-58 | Benchmark on slice C | Benchmark |
| 3 | Adopt the capability maps' gate-closing items not already covered above: validate extension types in generated `try_from`; `RowConverter` for keys containing `pse.index_tuple`; `distinct`/Kleene kernels in the rule compiler; `CastOptions { safe: false }` at import; governance test that no `TableProvider` mutation method is implemented; bounded `MemoryPool`; `TimeProvider` injection; `conditional_arguments`/`short_circuits` on conditional kernels; no `config_options` reads in kernels; `Ipopt_get_curr_iterate` over text parsing; the never-unwind-through-FFI rule | DM-07, DM-15, DM-08, DM-42, DM-02, DM-30, DM-28, DM-24 | Each named test in the maps | As named |

**Final check.** The design's claims exceed its evidence in the places named above and nowhere else that this review inspected; the supported scope matches the phased delivery; the extension path for units, methods, kernels and plans is clear and validated by the journeys. After revision 2, re-review §5, §6.6, §6.10–§6.11, §7–§8, §14 only.

---

## Appendix A — Library capability verification ledger

Facts verified in this review through context7 against `/apache/datafusion` (High reputation, 5,032 snippets). Everything else about libraries is taken from the three capability maps and labelled *capability map*.

| # | Fact | Source seen | Used by |
|---|---|---|---|
| A1 | `datafusion-proto` converts logical plans to and from protobuf bytes; a `LogicalExtensionCodec` serializes UDFs and extension nodes (the FFI table-provider path uses `serialize_exprs` with such a codec) | `datafusion/proto/README.md`; `ffi/src/table_provider.rs` | F9 — plan fingerprint mechanism; supersedes the DataFusion map's "no stable plan hash" with a serialization contract |
| A2 | `Optimizer::with_rules(vec![…])` builds the optimizer from an explicit rule list; the physical optimizer likewise holds a `rules` vector (22 built-in) | `docs/source/library-user-guide/query-optimizer.md`; `physical-optimizer/src/optimizer.rs` | F9 — pin the rule set rather than merely record it |
| A3 | `information_schema.df_settings` exposes every session setting as a queryable table, including `datafusion.optimizer.*` | `docs/source/user-guide/sql/information_schema.md` | F9 — a hashable config snapshot for the pass record |
| A4 | Field metadata on expressions is explicit: `Expr::Literal(scalar, metadata)` (48.0.0), `with_metadata()`/`arrow_metadata()` functions, and `Expr::unalias_nested` preserving aliases with metadata (54.0.0) | `upgrading/48.0.0.md`, `upgrading/54.0.0.md`, `scalar_functions.md` | §4.3 — metadata does not propagate through computed expressions by itself; rule outputs must be re-typed against the head relation's generated schema (a rule to add to §14.2) |
| A5 | `TableProviderFilterPushDown::Exact` means "DataFusion will **not** add a `FilterExec`"; `Inexact` keeps one; the provider's guarantee is unverified by the engine | `library-user-guide/custom-table-providers.md` | F17, G7 |
| A6 | `UserDefinedLogicalNodeCore` carries `check_invariants(InvariantLevel)`, `prevent_predicate_push_down_columns()` (default: all columns), `supports_limit_pushdown()` (default false) | `datafusion/expr/src/logical_plan/extension.rs` | §8 — the DataFusion map's item 26 is confirmed as available; kept at *evaluate* |

Additional capabilities surfaced by this review that the maps did not list: A1 (plan serialization as the fingerprint contract), A2 (explicit rule-list pinning), A3 (config snapshot as a table), A4 (explicit metadata carriers, and therefore the head-schema re-typing rule). Capabilities from the maps that this review endorses as gate-closing are listed in §11 priority 3.
