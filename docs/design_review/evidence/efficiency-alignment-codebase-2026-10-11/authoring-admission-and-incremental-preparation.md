# Authoring, admission and incremental preparation: bounded supporting assessment

Inquiry **A** of the [principal efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md). This is supporting
evidence, not an independent principal review, finding ledger or qualification receipt. The
principal review owns the combined judgment, finding identifiers (`F01`…) and rule impacts
(`RC01`…); local labels below (`A-nn`) are investigation labels only. Coordinator source
checks, reconciliation and probe results are in the [evidence README](README.md).

Baseline: `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11), production source clean.
Assessed by a fresh read-only `design-reviewer` against Core 3.4, Heuristics for Efficient Architecture
1.0, Process Simulator 1.5 and the [pse-arrow binding](../../design_principles/binding/pse-arrow.md).
No builds, tests or services were run by the inquiry; probes it proposes are listed for later
settlement unless the README records them as executed.

---

Bounded supporting assessment for the 2026-10-11 efficiency-alignment review. Baseline: `main` at
b01a45d71a5f5ab29991ad193679f5588ac3662f. Standard: Core 3.4 with the Heuristics 1.0, Process
Simulator 1.5 and the pse-arrow binding. The work was read-only. Nothing was built, run or probed.
The labels A-nn are local; the principal reviewer owns the final IDs.

## 1. Scope examined and coverage limits

**Read in full or at the operation level:**

- **Edit ingress.**
  - `pse-runtime/src/workflow/modeling.rs`: `with_declarations`, `prepare`, `prepare_flow`, `modeling_from_documents`, `persist_modeling`, `document_inputs`.
  - `workflow/modeling/canonical.rs`: `persist`, `edit`, `SourceIndex::{insert,data_closure,references}`, `full_source`, `declaration_inventory`, `declaration`, `checked_selection`, `qualified_selected_candidate`, the head of `select_into`, and `SourceRevision::open`.
- **Preparation.**
  - `workflow/modeling/preparation.rs`: `prepare_selected`, `settle_preparation`.
  - `workflow/modeling/declared.rs::declared_execution`.
  - `workflow/modeling/cases.rs`: `resolve_case`, `prepare_solve_seed`, `discrete_assignment`.
  - `workflow/modeling/views.rs::bound_case`.
  - `workflow/bindings.rs`: `prepare_bound_operation`, `validate_binding`.
  - `workflow/study.rs::prepare_study_candidate`.
  - `workflow/worker.rs::package_from_sources`.
- **Math service.**
  - `math.rs`: `MathService::workspace`, `assemble`.
  - `math/preparation.rs` in full.
  - `math/modeling.rs`: `source_revision`, `modeling_revision`, `prepare_modeling_view`, `rebind_scoped`.
  - `math/portable.rs`: `encoded_body`, `describe_body`, `publish_description`, `qualify_replay`, `product_candidate_presence`.
  - `math/retention.rs`: `selected`, `retain_selected`.
- **Compiler.**
  - `pse-compiler/src/workspace.rs`: the database, inputs, `new`, `with_events`, `rebuild`, `trim_queries`.
  - `workspace/modeling.rs` in full.
  - `workspace/modeling/executable.rs`: `plan_modeling_cancellable`, `complete_modeling_cancellable`, `admitted`.
  - `executable/grouped.rs`: `semantic_body`, `plan`.
  - `executable/implicit.rs::SupplierTopology`.
- **Modeling and authoring.**
  - `pse-modeling/src/check.rs`: `CheckedPackage::select`, `occurrences_of`.
  - `expression/admission.rs::ExpressionOccurrence`.
  - `selected_source.rs`: `occurrences`, `record_sources`.
  - `authoring_driver/document/{edit,owned,owned_reparse,data}.rs` (relevant parts).
- **Canonical store (`pse-operations`).**
  - `canonical_selection.rs`: `SelectedRead::{snapshot_dependencies,dependencies}`, `ProductDescription::{prepare,matches_read,publication}`, `adopt_dependencies_from_same_revision`, `recheck_dependencies`, `acknowledge_description_until`, `publish_description_inner`.
  - `canonical.rs`: `product_acknowledged`, `release_protection`.
  - `canonical_staging.rs::product_blob`.
  - The `canonical_products` indexes in `generated/surreal.surql`.
- **Python boundary.**
  - `pse-py/src/workflow/modeling.rs`: `with_declarations`, `declarations`, `prepare_solve`, `from_documents`.
  - `pse-py/src/workflow.rs`: producer construction.
  - `python/pse/_modeling.py`: `with_declarations`, `declarations`.
- **Architecture and leads.**
  - Architecture §14.3 and §14.4 (`mathematics-and-compilation.md`), and identity-and-publication §20.
  - Plan 33 EI04, review 10-09 F05 and F06, PA01–PA03, PE04, EF02, EF08, US05, graph-review F01 and F03, and state-review F03.
- **Salsa skill:** behaviours B001, B004, B012, B013, B019 and B021.

**Not read, or only skimmed:**

- the rest of `select_into` and `insert_inventories`
- `stage_edits` / `store.edit` (RPC count unknown)
- `pse_modeling::check_with` internals
- `specialize.rs`
- `implicit::admit`
- `conditional.rs`
- the `pse-authoring` grammar and parser (performance)
- `workflow/physical.rs` and `physical_cache.rs`, beyond confirming that a process-level admission cache exists
- `_workflow.py`

I did not check whether the SurrealDB planner uses `product_discovery` for the acknowledgment query (probe P2). No quantity in this report was measured.

## 2. Complete operations examined

**(a) Authored value edit → re-solve (Python).** This is the only interactive edit route at baseline. The entry `pse-py::NativeModelingPackage::with_declarations` is a complete-source replacement. No value-only edit API exists. Study bindings and initialization overrides are the only paths that do not mint a revision.

1. **Read the inventory.** `declarations()` → `canonical.rs::declaration_inventory`.
   - Protect, then a `membership_page` loop and grouped `selected_objects` per 64 rows.
   - Every declaration is JSON-decoded, JSON-encoded to Python, then structured by cattrs.
   - Owner: the runtime. Crossings: the RPCs plus a Python↔Rust JSON round.
2. **Re-encode in Python.** `_modeling.py::with_declarations` unstructures the rows, then runs encode → decode (validation) → encode. Rust then decodes `DeclarationEdit`.
3. **Rehydrate the previous revision.** `ModelingPackage::with_declarations` → `full_source`.
   - Fetches every modeling row, every physical-scope document, every data document (bytes cloned and parsed by `decode_selected_data`), every package and every fit.
   - The rows are then discarded (`let (_, scope, documents, sources, _)`), and the fit batch is built but unused.
   - It rebuilds the `authored.documents` batch without `source_text`.
4. **Persist.** `canonical.rs::persist`:
   - Builds a validated `modeling_declarations` batch for every row and drops it.
   - Computes `math::modeling::source_revision`: BLAKE3 over every row frame.
   - Builds a `SourceIndex`, then for every row runs `references(row)`, which parses each row's expression text through `selected_source::occurrences` → `expression::occurrences::collect` → `dsl::parse_*`.
   - Builds a serde_json payload per row, plus BLAKE3 in `edit()`.
   - Runs `data_closure()`. If any data rows exist, it creates a **fresh compiler workspace** and calls `modeling_revision`, which runs a full check of the data rows and Salsa `document_table` admission of every data table, only to derive `record:` edits.
   - Hashes every data document's bytes in `edit("data:")`.
   - Scans every member of the previous revision with `membership_page`.
   - Only then drops the unchanged edits and calls `store.edit`. The problem is kept.
5. **Prepare (first pass).** `prepare_solve` → `declared_execution`:
   - `declaration(root)` costs about 5 RPCs.
   - `prepare(root, analysis bindings)` → `checked_selection`. Each retained candidate for this `SelectedRequest` is rechecked newest-first, with at least one `resolve_logicals` RPC each, and all of them miss.
   - Under the qualification turn, `select_into` rehydrates the complete selected closure in rounds.
   - A flight job creates a **fresh Salsa database** (`MathService::workspace` → `CompilerWorkspace::new` → `salsa::Storage::new`). `publish_modeling_with` → `check_modeling` → `pse_modeling::check_with` checks the whole selected closure.
   - `prepare_selected`: the `BasisKey` contains the new `SelectedDependencies`, so it misses. Another fresh database runs `plan_modeling_cancellable`: Salsa `specialized`, `projection`, then `grouped::plan`, which normalizes every consumer body and runs `render_expr` + BLAKE3 for its `SemanticBodyHash`.
   - `complete_modeling_cancellable`: a body hits through `BodyRetention` → `ModelingCache::Body` when its hash is unchanged.
   - `settle_preparation`: the new basis has no descriptions. That triggers `qualify_replay` (a full local runtime re-observation, see PA01), then `describe_body` and `publish_description` for **every** portable body.
6. **Prepare (second pass).** `prepare_declared` → `prepare_analysis` → `prepare_solve_seed` → `resolve_case`.
   - This makes a **second `prepare`** with `bindings.demand` extended by the case value and variable names.
   - Selection adopts the same-revision admission (1 RPC).
   - The basis misses (different demand), so there is another fresh database, specialize/project/plan, completion, another `qualify_replay`, and `describe_body` + acknowledgment RPC for every body.
7. **Bind the view.** `views.rs::bound_case`: `view_key` = BLAKE3 over the bound structure and every body key. If only `CaseValues` changed, the view hits and `rebind_scoped` returns `Unchanged`, `Sharing` or `Rebuild`. The assembled program `OnceLock` is shared.

**Live state.** Each pass reserves `policy.workspace_bytes` per fresh workspace; there are three or more per edit. Each `BodyDescription` holds a full premise snapshot. New `Selected`, `Frontier` and `Basis` entries are retained per revision.

**(b) Text document reload.** `pse-py::modeling::from_documents` reparses every document. `modeling_from_documents` → `persist_modeling(previous = None)` mints a **new problem**. `SelectedRequest.problem` and product acknowledgment are scoped by problem, so selection, basis and descriptions all miss; only in-memory body and view keys can hit. The exact-bytes ingress cache `worker.rs::package_from_sources` is not on the Python path.

**(c) Warm study point (same revision).** `study.rs::prepare_study_candidate`:

- Reads the point and study, `physical_source` and `revision`.
- `modeling_revision` → `SourceRevision::open` (protect, context, release).
- `prepare_bound_operation` → `declared_execution`: `declaration()` plus a first `prepare`.
- `prepare_declared` → a second `prepare`.

Each `prepare` costs protect + adopt + B × (`matches_read` locally + `acknowledge_description_until` RPC) + release. The basis and view hit, and values enter as `case.members`.

**(d) Flow composition.** `ModelingPackage::prepare_flow` runs `checked_selection`, then a fresh `pure_workspace` and `prepare_semantic_modeling_revision` (specialize + `SemanticModeling::admit`) on every call, with no product cache.

## 3. Candidate findings

### A-01: An authored value edit has no value-only route; it takes the complete structural chain

**Cause.**

- The only edit intent is complete-source replacement: `ModelingPackage::with_declarations`; `_modeling.py::ModelingPackage.with_declarations`, documented as "Admit edited generated declarations".
- Case and parameter values are authored inside declarations: the fixture is read from `model.compiled().model.fixtures` in `declared_execution`.
- Any value edit therefore creates a new revision. The selected admission (`qualified_selected_candidate`) and the basis (`BasisKey.dependencies`) miss, which re-runs:
  - selection rehydration
  - the complete check of the closure
  - two specializations, projections and plans
  - full description settlement (A-03)

  The edit reaches value-level reuse only at `bound_case` → `rebind_scoped`.
- The machinery for a true value path already exists: `bindings.rs::admit_overlay` and `AdmittedBinding`, then `prepare_bound_operation` sets `execution.analysis.case.members = binding.values()`. With it the `BasisKey` stays equal and the view rebinds. It is reachable only through studies.

**Consequence.** Interactive edit → re-solve (PS-11 "a value change must not rebuild structure"; PSE-S07):

- Every tweak to a feed or parameter value pays work proportional to the selected flowsheet closure, plus package-proportional ingress (A-02). The view rebind that the design describes as the value path (`views.rs` header, §14.4 table "Fixed or parameter value → … reused: bodies, case plan, structure") is reached only after the structure has been rebuilt and found equal.
- This is "proving unchanged repeats most of the work" (H14).
- Principles: DP-09 and DP-10 (separate structural from value inputs), H8 (the interface hides the unit of intent), H12 and H13.

**Simpler conforming alternative (Proposed).**

1. Expose a value-edit intent on a package, for example `with_values({member/path: quantity})`.
2. Admit it through the existing `admit_overlay` physical and reference contract against the parent revision's prepared model.
3. Bind it as case members onto the parent basis and view: `rebind_scoped`. That is no new mechanism.
4. If the edit must become authored source, persist it as a **delta** edit of the changed declaration objects (A-02), with the revision lineage recording the parent plus the overlay. Persisting can happen outside the solve's critical path.
5. Edits that `admit_overlay` refuses fall back to the structural route: structural literals such as `x*x==2`, fixed/free changes, domain or index sizes, discretization counts.

**Preservation.**

- The provenance of a result must name the exact values it used. Study runs already record binding values.
- A literal inside an equation stays structural (`reuse_tests::checked_selected_admission_rechecks_fresh_protection_and_retains_a_b_a`: "structural literal changes must re-admit").
- Unit and physical admission of the new value is preserved (`admit_overlay`).
- PS-10 post-solve checks are unchanged.
- An overlay must not silently diverge from an authored revision the user later reopens. The lineage must say which one the result belongs to.

**Uncertainty.** I did not establish which authored value fields `admit_overlay` can address: parameters and variables, yes; fixture `fix` and start values are likely. How much of the chain's cost lies upstream of body admission is unmeasured.

**Evidence:** Interface-checked (source-traced); the remedy is Proposed. **Verification:** probe P1.

### A-02: Edit ingress does package-proportional work regardless of edit size

**Cause.**

- `canonical.rs::full_source` reloads and parses the entire previous revision, including every data document (`decode_selected_data`, `bytes.clone()`), and then discards the rows.
- `persist` then does all of the following before diffing against stored member versions:
  - validates a complete Arrow batch and drops it
  - hashes all rows (`source_revision`)
  - runs `references()` on every row, which reparses expression text
  - serializes every row to JSON and BLAKE3-hashes it (`edit`)
  - BLAKE3-hashes every data document's bytes
  - re-admits all data tables in a fresh compiler workspace whenever the package has any data rows (`runtime.shared.math().workspace(...)` + `modeling_revision` on `index.data_closure()`)
  - scans all members (`membership_page` loop)
- `SourceIndex::data_closure` also linear-scans `self.rows` per kept row (`candidate.parent_id == row.parent_id && candidate.name == row.name` and the import scan), although the `named_children` index exists. That is O(kept × rows).
- Python adds roughly four complete JSON passes per edit cycle: `declarations()` decode and structure, then `with_declarations` unstructure, encode, decode and encode.
- **Text reload** (`modeling_from_documents` → `persist_modeling(..., None)`) reparses every document and mints a new problem. That discards problem-scoped reuse (selected candidates, basis, product acknowledgments) and adds a full new object set to the store on every reload.

**Consequence.** For edit → re-solve on a package with libraries or property data (PSE-S01 extension, growing packages), the cost of one edit grows with package and data size, not with the edit (H5, H12, H14, H16). Data-heavy packages pay table admission on every unrelated edit. Reloading text multiplies durable history (H25, H27; seam with E).

**Simpler alternative (Proposed).**

- Make the edit a **delta** (changed, added and removed declarations, or document texts) against a parent revision.
- Compute identities and payloads only for the changed objects. Unchanged objects keep their stored `version` keys, which are already content identities (`edit()`).
- Recompute `references` for a changed row. Recompute other rows' references only when a name, parent, kind or import changes (a conservative trigger).
- Re-derive `record:` suppliers only when data-closure rows or data documents change.
- Make the source-revision identity derivable from member versions, so that unchanged rows are not re-framed. That is a frame change; see RC-A5.
- For text reload, pass the parent package so the problem lineage and membership diff are kept. Diff documents by checksum and reparse only the changed documents. The `package_checksum` mechanism already exists.

**Preservation.**

- `persist`'s exact unchanged-publication return (`edits.is_empty() && identity == previous.identity`).
- Deletions of absent `modeling:`, `record:` and `physical_scope_document:` members.
- Complete validation of rows that actually crossed the Python boundary. That validation is legitimate (H18) for the changed rows.

**Uncertainty.** `stage_edits` RPC count. The parse cost relative to the rest is unmeasured.

**Evidence:** Interface-checked. **Verification:** P1, plus the count of store requests per one-row edit at two package sizes.

### A-03: Body descriptions bind the whole selected closure, so any in-closure edit re-uploads every unchanged body on the critical path

**Cause.**

- `ProductDescription::prepare` (`canonical_selection.rs`) snapshots the **complete** read premises (`read.snapshot_dependencies()`) and serializes them (`BorrowedDependencies(read)`).
- `canonical_staging::product_blob` concatenates `payload ‖ dependencies` and hashes the whole blob twice: `version` and `digest`.
- A body's `SemanticBodyHash` already frames its complete semantic dependencies (`grouped::plan`: normalized expression, formals, quantity, physical identity, function IDs, provider identities, limits). Yet its description depends on every `Logical`, `Name`, `Scope`, `KindScope` and `References` premise of the closure.
- An edit to any declaration in the closure therefore changes every body's blob:
  - `matches_read` fails → `describe_body` → `product_acknowledged` misses (it matches by descriptor) → `stage_product_blob` re-uploads payload and premises → `admit_product`
  - this happens for every portable body inside `settle_preparation`, which `prepare_selected` awaits before returning
- The second `prepare` (A-04) recomputes `ProductDescription::prepare` per body (premise JSON plus blob hashing) and acknowledges each one.
- Each `BodyDescription` retains its own premise copy, so B × P stays live per basis (H17).
- On a runtime with a producer, which is the default for Python, every settling `prepare` first runs `qualify_replay` → `LocalRuntimeObservation::verify_scoped`: a full SHA-256 re-capture of the executable closure (PA01, owned by H).
- The ephemeral runtime does this too (operation-map a.5).
- That descriptions carry "fully merged acquisition premises" is an asserted design property: `reuse_tests::ordinary_preparation_rechecks_additional_acquisition_absence_before_basis_reuse`. I did not run it.

**Consequence.**

- Edit → re-solve: about B × (payload + P) bytes uploaded and B admissions per edit, plus two runtime observations. B is the portable body count and P the closure premise size; both grow with flowsheet size, so the upload grows roughly quadratically.
- Growing history: B new product roots per revision (seam with E on retention).
- H13 (incidental provenance propagated as a result-affecting dependency), H16, H23 (publication coupled to private preparation), H26, H1 (cache-population effect versus the solve's guarantee).
- This is the edit-path residual of PA03.

**Simpler alternative (Proposed).**

- (i) **Body-scoped premises.** Key a description on the body's semantic identity and the producer. Record the selected premises that body actually consumed, or none beyond the `SemanticBodyHash` frame, if identity frames are judged complete.
- (ii) **Separate content from reachability.** A content-addressed payload blob per (body identity, producer) is uploaded once. Small per-revision or per-selection root records establish reachability.
- (iii) **Settle off the critical path.** Settle after the solve, or in a bounded background owner that drains on cancel. Skip settlement for ephemeral runs. Settle once per (process, revision, protection) rather than per `prepare`.

**Preservation.**

- New eligible publication still requires current receiving admission (ADR-0164).
- Released roots and expired protection must still be detected before reuse is claimed.
- Uncertain acknowledgments settle by exact request.
- Never cache permission (PA03 counterexample).
- If (i) narrows premises, the narrowed set must still include everything that can change the body's meaning: absent names it probed and functions' bodies. `SemanticBodyHash` already frames function IDs that "frame body, signature, validity…"; whether that is complete is the key premise.

**Uncertainty.**

- Whether per-body premises can be proven complete. If they cannot, (ii) and (iii) still remove the re-upload and the critical-path coupling.
- Payload sizes are unknown.

**Evidence:** Interface-checked. **Verification:** P3 (count `canonical_products` rows and blob bytes before and after a one-parameter edit).

### A-04: A declared-case solve prepares the same root twice

**Cause.**

- `declared.rs::declared_execution` calls `self.prepare(root, instance, Bindings::default().with_analysis(route), …)` to read the fixture and procedure.
- `cases.rs::resolve_case` then calls `self.prepare(...)` again with `bindings.demand` extended by `case.values` and `case.variables` names.
- The `BasisKey` covers the bindings, so the two are separate bases. Each `prepare` owns:
  - protect, selection qualification, settlement and release
  - on a miss, a fresh database with specialize, project, plan and complete, plus per-body description work
- The first product is used for the fixture and for study binding validation (`validate_binding` reads `model.admitted.inputs` and `model.model.symbols`).

**Consequence.**

- Edit and cold paths: two full preparations per edit, so A-03's per-body settlement is paid twice.
- Warm study points: two RPC re-qualification sets per point (A-07).
- H2 (the physical unit follows two semantic steps), H12. This is PA02's "repeated shared basis" residual.

**Alternative (Proposed).**

- Read the route, procedure and fixture from the checked declaration plus a semantic-only specialization. `prepare_semantic_modeling_revision` / `specialize_modeling` already exist without projection, plan or publication.
- Then run one complete preparation with the final demand.
- Alternatively, compute the demand before the first `prepare`. Fixture member names are authored on the root declaration.

**Preservation.** Studies validate bindings against the admitted inputs of the product actually solved. The demand-extended product is the one solved, so validating against it is at least as strict.

**Uncertainty.** Whether the fixture extraction needs specialized values (units or expressions). Whether the demand changes admitted inputs.

**Evidence:** Interface-checked. **Verification:** count `MathService::preparations()` frontier and basis loads per `prepare_solve` after an edit.

### A-05: Salsa is a per-operation memo; neither reuse mechanism gives early cutoff inside the selected closure

**Cause.**

- Every `MathService::workspace` call constructs a new `CompilerWorkspace` with fresh `salsa::Storage` (`workspace.rs::with_events`). It publishes one revision and is dropped. Three or more are made per edit (A-01).
- No production path ever sets a second revision into a database. `publish_modeling_revision`'s `set_checked` and the `rebuild` / `trim_queries` / request-bucket / LRU / retention-accounting machinery serve a long-lived lifetime that never occurs.
- Salsa inputs are monolithic: one `Catalog { checked: Arc<CheckedPackage> }`. `selected`, `specialized`, `projection` and `admitted` are keyed on whole-package and whole-request inputs, and checking (`check_with`) runs outside Salsa.
- Cross-call reuse is instead owned by `ModelingCache` (`Selected`, `Frontier`, `Basis`, `Body`, `Solver`). Its `Selected` and `Basis` keys include the complete `SelectedDependencies`, so any in-closure change misses everything between selection and bodies.
- Architecture §14.4 states that "unchanged results backdate so unrelated edits do not propagate" and gives a reuse table. Physically, backdating never spans an edit, and "reused" means recomputed and then found equal: the §G "it is incremental" false positive in the documentation, not in correctness.

**Consequence.**

- DP-09: two reuse mechanisms over overlapping scopes, neither able to stop propagation inside the closure.
- H4: library capability carried but unused.
- H14 and H27: Salsa machinery, rotation and accounting maintained without cross-edit benefit.
- Edit → re-solve and the S01 extension cannot become proportional to the change under either mechanism.

**Alternatives (Proposed, a design decision).**

- **(a) Make Salsa the owner.**
  - A long-lived single-writer database per (problem, physical context, provider set), with per-declaration inputs or per-selected-object inputs.
  - Tracked check, specialization and plan at declaration, instance and body granularity.
  - Scope-membership and name lookups as tracked queries, so absences and deletions are real dependencies (H13).
  - Concurrent preparations on cloned handles. A write cancels in-flight readers (salsa B012); same-key requests coalesce (B013); durability on physical and provider inputs (B004).
  - Then retire `Frontier`, `Basis` and probably `Selected` caching, along with the bespoke flights for pure work.
- **(b) Drop Salsa** for plain functions plus the existing keyed caches. DP-09 allows that "immutable full rebuilding can be simpler". Correct §14.4 to match.

**Preservation.** Handles stay local. Interned slot reuse across revisions (B019, B021) must not leak IDs. Attempt-owned mutable state stays outside. Effects stay outside tracked queries (PA03: skipped queries must not skip required effects). Cancellation and drain.

**Uncertainty.** (a) needs a decomposed checker. EI04's unresolved domains (absent lexical candidates, duplicates, cycles) are exactly what tracked membership queries would have to cover. Which alternative wins overall is unmeasured.

**Evidence:** Interface-checked. **Verification:** confirm that no production caller publishes two revisions into one workspace (`rg 'publish_modeling_revision|publish_modeling_with'` under `crates/pse-runtime/src`). I believe this but did not exhaustively prove it.

### A-06: A changed revision's miss cost grows with retained candidate history

**Cause.**

- `math/retention.rs::retain_selected` appends one `SelectedAdmission` per distinct dependency set to the `SelectedRequest` list, bounded only by bytes (to support A/B/A).
- `qualified_selected_candidate` walks the list newest-first. For a new revision, `adopt_dependencies_from_same_revision` returns false locally, then `recheck_selection_dependencies` issues at least one `resolve_logicals` window RPC per candidate before rejecting it.
- On a miss, `select_into` rehydrates the whole closure, even though the newest candidate's `versions` map already identifies which objects are unchanged.

**Consequence.** In an editing session of E distinct value edits to the same case, each new edit costs at least E sequential RPCs before its unavoidable miss (H10, H25), plus a full closure re-read (H13).

**Alternative.** Resolve the union of the candidates' logical premises once (one batched set). Compare all candidates locally. Rehydrate only the changed or new objects and re-establish the name, scope and absence premises.

**Preservation.** Every premise of an accepted candidate is still rechecked. A/B/A reuse is kept.

**Evidence:** Interface-checked. **Verification:** RPC count for the E-th edit, E ∈ {1, 8}.

### A-07: Warm study points re-qualify preparation per point; acknowledgment uses a history-proportional, exclusively guarded path (PE04 residual; seam with D and E)

**Cause.**

- Per point (operation c):
  - package reconstruction (`SourceRevision::open`, about 4 RPCs)
  - `declaration()` (about 5)
  - 2 × [protect, adopt, B × `acknowledge_description_until`, release]
- `canonical.rs::product_acknowledged`:
  - runs `SELECT … FROM canonical_guards … FOR UPDATE` on the problem's retention guard
  - filters `canonical_products` by `problem, request, payload, dependencies, producer` with root joins
  - the only applicable index is `product_discovery(problem, producer)`, and products accumulate per revision (A-03)

**Consequence.**

- Thousand-point sweep: roughly 2,000 × B acknowledgment transactions.
- Concurrent lanes serialize on one guard record, and on `ProblemTurns` behind other lanes' result-append write guards.
- Each acknowledgment examines rows proportional to the problem's product history.
- H8, H22, H23, H25.

**Alternative.**

- Hold one protection per lane or study for the study's fixed revision.
- Treat descriptions settled by this process under a live protection of the same revision as settled.
- Make acknowledgment a keyed lookup: an index on the request/descriptor identity, or the store-issued key.
- Use a snapshot read rather than `FOR UPDATE` if retention exclusion is not needed for a read.

**Preservation.** Released or expired roots must be detected. Exact uncertain-outcome settlement.

**Uncertainty.** The planner plan (P2). Whether `FOR UPDATE` serializes readers in SurrealDB 3.3 transactions.

**Evidence:** Interface-checked.

## 4. Opportunities and uncertainties that are not findings

- **EI04: a full incremental checker.**
  - Complete checking of the selected closure is a sound baseline for structural edits; duplicates, cycles and absence are closure-global.
  - Given A-01 (values bypass it) and A-05 (nothing downstream cuts off anyway), a checker redesign is not the first lever. Its benefit for structural edits is unmeasured.
- **`prepare_flow`** builds an uncached fresh workspace with semantic specialization on every call. How often it is called (recycles, inspection) is unknown.
- **Expression text is reparsed** in `persist`/`references`, `select_into`, and the checker's `occurrences::collect`. A retained parsed form could serve all three; parse cost is unmeasured.
- **`CheckedPackage::select`** clones `selection_closures` and `lowered_functions` whole. This is minor.
- **`grouped::plan`** renders each consumer body once for identity. This is acceptable: one render per body, not per subtree.

## 5. Strengths and preservation constraints

- **Selected-closure acquisition with narrow premises.** Name, Logical, Scope, KindScope and References premises, with windowed early rejection (`recheck_dependencies`). Package ancestors do not force unrelated members (`selected_source::requires_members`). Out-of-closure edits reuse the admission, basis and descriptions. This is asserted by `reuse_tests::checked_selected_admission_rechecks_fresh_protection_and_retains_a_b_a` ("a changed unconsumed body preserves checked selected input"); I did not run it.
- **Parametric bodies keyed on complete semantic identity** (`grouped::semantic_body`, `ModelingCache::Body`). Parameter values are ports, so value edits do not change bodies.
- **View reuse and value-only rebind** with the shared program memo (`bound_case`, `rebind_scoped` Unchanged/Sharing/Rebuild, `Preparation.executable`). Study bindings already deliver PS-11 within one revision.
- **Pure frontier and completion are separated from protection and publication effects**, and concurrent identical preparation is coalesced through flights.
- Physical admission and the exact-bytes package ingress are cached per process.
- Exact unchanged-publication detection in `persist`.
- Fresh attribution per revision.
- Cancellation drains through `receiving` and `scoped`.

## 6. Positions on prior leads

| Lead | Position | Evidence |
|---|---|---|
| 10-09 F05 (occurrence render/hash) | Resolved | `ExpressionOccurrence::Node{body,position}`; `in_body` = `inventory::positions` |
| 10-09 F06 (selection scans and copies the universe) | Resolved (minor residual clones) | `CheckedPackage::select` uses `occurrences_of` ranges and selected-only maps |
| EI04 (complete checking retained) | Confirmed still present; reassessed as sound for structural edits, unfit as the value-edit route | A-01, A-05; `workspace/modeling.rs::check_modeling` |
| PE04 (study occurrences reacquire source/admission) | Residual: compiler work removed on warm hits; RPC re-qualification remains | A-07 |
| PA02 (warm composition rebuilds the compiler generation) | Residual: the warm same-revision hit avoids it; every new revision builds 3+ fresh databases; the double `prepare` remains; `prepare_flow` is uncached | A-04, A-05 |
| PA03 (memo hit does full encoding/publication) | Residual on the edit path: encoding is cached, but blob re-upload happens per revision | A-03 |
| PA01 (observation per body) | Moved: per settling `prepare`; H owns | `settle_preparation` → `qualify_replay` |
| EF02 | Resolved | `modeling_validation_required`, `FieldCheckedBatch::readmit_context` owner `matches` |
| EF08 | Not examined (B) | — |
| Graph F01/J3 | Resolved | `PreparedBasis::description` binary search |
| Graph F03/B8/N13 | Resolved | `AdmittedModeling.supplier_topology` `get_or_try_init` |
| State F03 | Resolved | `declaration_inventory` and `fit_declarations` use `selected_objects`; conformance uses `selected_declarations` |
| US05 | Residual: selected closure and premises exist; in-closure edits invalidate the whole chain | A-03, A-05 |

## 7. Candidate rule impacts

- **RC-A1.**
  - **Rule:** blueprint §14.4, "Salsa reuse … unchanged results backdate", the reuse table, and "no persistent Salsa store".
  - **Change:** under A-05(a), make it true with a long-lived workspace; under (b), remove the claim.
  - **If the rule is kept:** A-05 reduces to correcting the description.
- **RC-A2.**
  - **Rule:** the §14.3 preparation amendment, "each consumer rechecks … and explicitly settles canonical effects"; the §14 supplement, "selected dependencies … enter the products that consume them"; ADR-0164 product descriptions.
  - **Change:** body-scoped premises, content and reachability separated, and settlement off the critical path, skipped for ephemeral runs (A-03, A-07).
  - **If the rule is kept:** only deduplicating encoded material and removing the second `prepare` remain.
- **RC-A3.**
  - **Rule:** Plan 33 EI04 disposition.
  - **Change:** none needed for A-01. A future incremental checker reopens it.
- **RC-A4.**
  - **Rule:** the edit API contract (`with_declarations` complete replacement; `modeling_from_documents` "persist once" minting a problem per load).
  - **Change:** add value-overlay and delta intents, and a lineage-preserving reload (A-01, A-02).
- **RC-A5 (optional).**
  - **Rule:** the `ModelingSourceRevisionV4` frame.
  - **Change:** derive it from member version keys so that delta edits avoid reframing all rows. This is a new frame version.

## 8. Proposed light probes

These are for the coordinator to run under `scripts/pse-env --resource-class light`.

- **P1 (A-01, A-02, A-04).**
  - **Question:** on one flowsheet fixture with ≥10 portable bodies, does a one-parameter edit + `prepare_solve` cost far more than a warm `prepare_solve`?
  - **Method:** a Python script under scratch, `python -I`. Time `pkg.with_declarations(rows')` and `pkg2.prepare_solve(case, s)` against a repeated `pkg.prepare_solve(case, s)`, and read `ModelingRunReport.preparations` / `MathService::preparations()` counts.
  - **Discriminating outcome:** edit ≫ warm, with frontier and basis loads of 2 per edit.
- **P2 (A-07).**
  - **Question:** does the acknowledgment examine rows proportional to product history?
  - **Method:** against an isolated test store, run the query body of `product_acknowledged` with `EXPLAIN FULL`, after creating N ∈ {10, 1000} products for one (problem, producer).
  - **Discriminating outcome:** examined rows grow with N.
- **P3 (A-03).**
  - **Question:** after a one-parameter edit and `prepare`, how many new `canonical_products` and payload blocks does the problem have?
  - **Method:** run `SELECT count() FROM canonical_products WHERE problem=$p GROUP ALL` before and after, using an existing `canonical-tests` fixture.
  - **Discriminating outcome:** an increase of B confirms A-03; ≈0 refutes it.
