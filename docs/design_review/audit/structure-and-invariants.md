# Structure and invariants

These findings concern three things:
- whether distinctions, relationships, lifecycles and failure meanings have explicit,
  enforceable representations;
- whether those representations are declared once;
- whether they are re-spelled in strings and conventions.

The [collection README](README.md) gives the baseline and limits. Citations are `path::symbol`
at `cbf45ccc9`.

## <a id="da-007"></a>DA-007 — The canonical store's meaning lives outside the registry and is re-spelled in strings in every layer

**Principles:** AP-04, AP-05, DP-01, DP-02, DP-03, DP-04, DP-07, DP-24; G1, G2.

**Refresh status:** retained and widened.
- Plan 49 made attempt `phase` and `outcome` a store-enforced state machine
  (`pse-schema/src/catalog/substrate.rs::ATTEMPT_TRANSITIONS`, generated `ASSERT`s). It coded
  every refusal and guarded analysis-owner release (`ARTIFACT_HOLDERS`).
- Plan 51 then added projects, branches, drafts, contexts, imports and research using the old
  pattern.
- Several halves of other findings merge here:
  - the store-key half of DA-002;
  - the artifact-profile half of DA-032;
  - the canonical-key half of DA-005.

**Diagnosis.** The registry declares canonical record *shapes*: `catalog/substrate.rs` has
329 text columns and three enumerations, and `catalog/projects.rs`, `project_imports.rs` and
`project_contexts.rs` declare keys and kinds as text. The decision-relevant *meaning* of those
records is authored again in each layer that touches it:
- generator branches keyed on table names (`pse-codegen/src/codegen/surreal.rs`);
- about 4k lines of hand-written SurrealQL in Rust string constants
  (`codegen/surreal_*.rs`, concatenated into `generated/surreal.surql`);
- inline SurrealQL and pre-checks in `pse-operations`;
- `&str` store APIs;
- exchange documents;
- Python `str`.

1. **Identity kinds are string conventions recovered by parsing.**
   - Run keys are `format!("run:{run_id}")`, with `RunId` recovered by `strip_prefix` and
     `parse_hex` (`workflow/study_execution.rs`, `workflow/worker.rs`).
   - `AttemptId` is re-derived by hashing the key with a call-site literal
     (`FramedHasher::new(Frame::CanonicalPayloadV1).str("scientific.attempt.lineage.v1")…`,
     in `durable.rs` and `worker.rs`).
   - Admission and variant runs are `study-admission:{point}` and `study-variant:{point}`.
     The store's attempt state machine branches on
     `string::starts_with($this.run,'study-admission:')` (`codegen/surreal.rs` ≈233).
     The relationships run→point and problem→point exist only in these prefixes (DP-07).
   - The public boundary carries no key kind either.
     - Canonical keys are bare `str` throughout `python/pse/_workflow.py`, and Plan 51 extended
       this to project, branch, scope and operation keys.
     - The prior account's example persists: `study(study_id: StudyId)` versus
       `forget_study_results(study: str)`.
2. **Store vocabularies are hand-spelled per layer.**
   - Owner kinds (`product`, `analysis`, `artifact`, `study`, `run`, `project`, …) appear in
     about 55 literal comparisons, and `ARTIFACT_HOLDERS` embeds SurrealQL guard text inside
     the registry crate.
   - Persisted artifact kinds are five literals in the reader (`workflow/artifacts.rs`) but
     three in the registry (`ScientificArtifactKind`).
   - Plan 51 project commands are a typed Rust enum lowered to `kind` strings and re-dispatched
     by literal in `codegen/surreal_projects.rs`.
   - Templates hold 53 literal `IN [...]` vocabularies, including the registry enum
     `CanonicalTerminalOutcome` re-spelled in `surreal_execution.rs`.
   - Research roles are `String`, with the vocabulary in `workflow/research.rs` and in the
     template.
3. **Field rules have two homes.**
   - Attempt and study immutability and transitions are `pse-schema` constants.
   - Project, import, draft-version, context and knowledge immutability and state machines are
     18 `matches!(table, …)` branches in `codegen/surreal.rs`.
   - Artifact and analysis `active`/`retiring`, and study `active`/`terminal`, remain unasserted
     bool pairs.
   - File-instance `state` is a free string, tested against a hand Rust string set
     (`canonical_files.rs::collect_file`).
4. **New typed contracts re-stringify registry enums.**
   - `CanonicalResultOrigin` returns `phase: String` and `outcome: Option<String>` from a hand
     projection query.
   - `PackResultSelection.terminal_class` is `String`.
   - Exchange manifests carry `codec: String`, validated by a hand `matches!`, with an
     empty-string sentinel for "opaque schema".
   - Omission targets are `source:`/`result:`/`artifact:` prefixed strings.
   - The Plan 52l `ResultInspectionRequest` is `{run: String, attempt: String, relation: String}`.
5. **Lifecycle projections are partial or misplaced.**
   - `durable.rs` maps unknown attempt states to `TerminalClass::Failed` with a wildcard.
   - The projections from terminal outcome to attempt state and study-point state live in the
     *code generator* (`pse-codegen/src/codegen/rust/enums.rs`), keyed on the enum's name.
   - `AttemptState` declares members that are never produced.
   - Exported selections always write `PackScientificState::Unknown`, a hand Rust enum
     (`connected_results.rs::pack_selection`). The public `pack_selection` takes usability from
     the caller, and `validate_pack_selection` checks every field except usability. A recipient
     therefore never sees the producer's typed decision (PS-12).

**Consequence.**
- *Change:* add a durable owner, a project command or an admission-only run kind, or version the
  project exchange format (PSE-S04). Each needs coordinated hand edits to a catalog constant or
  SurrealQL fragment, a generator branch, a template, inline queries, Rust `&str` APIs, exchange
  structs and Python. No regeneration or compile check ties the spellings together. Plan 51 is
  the demonstrated instance.
- *Supported behavior:* most mismatches fail closed, as an absent key or unknown kind is refused.
  Two classes can be silent:
  - a misspelt owner-kind filter in a reclamation count returns zero rows (hypothesis);
  - versioned exchange documents freeze string vocabularies and prefix-encoded references that a
    later registry change cannot be checked against (DP-24).
- *Agents and Python callers* cannot discover which string a parameter expects or which kind an
  id is.

**Footprint.**
- *Decisive:*
  - `pse-schema/src/catalog/{substrate,projects,project_imports,project_contexts,products}.rs`;
  - `pse-codegen/src/codegen/{surreal,surreal_projects,surreal_execution,surreal_lifecycle}.rs`;
  - `pse-operations/src/{canonical_execution,canonical_studies,canonical_projects,canonical_files}.rs`;
  - `pse-runtime/src/workflow/{durable,study,study_execution,worker,artifacts,project_portability,result_selection,result_inspection}.rs`;
  - `python/pse/_workflow.py`.
- *Suspected:*
  - evidence, knowledge and maintenance tables (`kind`, `phase`, `state` text);
  - diagnostic `source_ref("canonical.run", …)` keys.

**Counterevidence.**
- §5 deliberately makes canonical lookup keys opaque and distinct from scientific ids.
- ADR-0164 and ADR-0214 place atomic enforcement in server-side functions.
- Both choices are sound for representation and enforcement placement.
- What does not follow from them:
  - that each layer owns the kind and vocabulary;
  - that relationships are prefixes.

  Overview §0.6 states the opposite rule ("every decision vocabulary that crosses a boundary is
  a registry enumeration with one Rust type").
- The mechanism to derive the missing pieces exists: the `__ARTIFACT_HOLDERS__` substitution and
  enum `TYPE` literals. It is applied to a handful of cases only.

**Evidence:** *Implemented* for the duplication, prefix parsing and re-stringification. The
silent-filter consequence is a hypothesis; mutating one owner-kind spelling under the
`pse-operations` retention tests would settle it.

**Improvement direction.**
- Canonical entity kinds declared once, each with a typed key and one mapping to its semantic id.
- Closed store vocabularies and table classifications declared in the registry and lowered into
  SurrealQL assertions and functions, Rust, exchange documents, backup inventory and Python.
- Relationships carried as typed fields.
- Field immutability and transitions declared in one home and lowered generically.

**Related:**
- [DA-033](boundaries-and-composition.md#da-033): the same templates re-author the protocol
  envelope.
- [DA-019](boundaries-and-composition.md#da-019): the backup inventory is a Python copy of table
  classification.
- [DA-005](semantic-model-and-authority.md#da-005): the boundary consequence.

## <a id="da-009"></a>DA-009 — Analysis kinds have no owned operation contract; lifecycle, replay, publication and host owners each re-project kind internals

**Principles:** AP-01, AP-03, DP-02, DP-08, DP-16; §E; G9 (PSE-S03).

**Refresh status:** reframed. Request and result pairing is now structural: `RunOutcome` variants
pair them, and one `RunSupervisor::start_native` serves every kind, including horizon. This
resolves DA-012's copied supervisor and cancellation divergence. The consequence that a new kind
threads through every consumer persists. DA-012's surviving strand, declared-procedure dispatch
re-implemented per consumer, merges here.

**Diagnosis.**
- The closed set of analysis kinds (declared case and modeling, simulation, fit, horizon, and
  host-only shooting) appears in several layered enums with different variant sets:
  - `RuntimeOperationRequest` and `PreparedOperation` (`workflow/operation_preparation.rs`);
  - `OperationRequest` and `PreparedStudyOperation` (`workflow/study_operations.rs`);
  - `RunRequest`, `RunReport` and `RunOutcome` (`workflow/run_outcome.rs`).
- The only per-kind contract is `PairedOperation { request, finish }`.
- Every consumer that needs a fact about a kind matches the variant and reaches into that kind's
  private structure:
  - **Sources, identity, provenance and inputs.** `durable.rs::{identities, request_inputs,
    request_context, request_provenance}` (≈2345–2480). Modeling has its own frame,
    `Frame::DurableModelingRequestV3`.
  - **Replay constructors.** `replay.rs::ReplayDocument` carries parallel optional
    `case_constructors`, `simulation_constructors`, `fit_constructor`, `horizon_constructor` and
    `shooting_constructor`. "Exactly one is present" is a convention.
  - **Numerics.** `numerics.rs`.
  - **Native report, strategy trace and warm start.** `attempt_documents.rs` and
    `completion.rs::RunResult::available_start`, written with `_ => None` wildcards.
  - **Rule mapping.** `diagnostics.rs`.
  - **Study tables and bindings.**
  - **Per-kind accessors.** pse-py `workflow.rs` (≈3700–3980).
- `RunResult` still carries horizon-only and modeling-only `Option` fields.
- Declared procedures are dispatched three times with different subsets:
  - conformance (`modeling/conformance.rs` ≈1386–1589);
  - `modeling/declared.rs`;
  - study bindings (`bindings.rs`, Solve only).
- The two study executors sequence the predicted-start flow twice (`study_execution.rs`,
  `study.rs`). The decisions inside it are pure and shared.

**Consequence.** *PSE-S03* (a new dynamic analysis runnable in studies) requires:
- about 45–50 per-kind arms in about 19 production files, by a regex scan of the Fit and
  Simulation variants;
- new fields and version bumps in two persisted documents;
- four registry vocabularies;
- host methods;
- possibly both study executors.

Wildcard arms let a new kind compile while silently lacking:
- its native report extent and projection in attempt summaries;
- its warm-start availability;
- its residual qualification fields.

That is G2/G6 risk where the compiler would otherwise force the decision. An extender must also
learn how durable, replay and review owners read other kinds' internals.

**Counterevidence.**
- A small closed set of analysis modes in exhaustive Rust enums is idiomatic and
  compiler-checked.
- The kinds' payloads genuinely differ.
- The process-simulator profile says no universal plugin framework is needed.
- §E allows a genuinely new mode to change core contracts.
- The closed set is not the defect. The defect is that about ten per-kind questions are answered
  again in every consumer, some by wildcard. The shared run supervisor shows that consolidation
  works once a contract exists.

**Evidence:** *Implemented* for the match sites, wildcards and persisted shape. The arm counts
are approximate (a regex scan that excludes `Self::` arms). Silent omission is inferred; no fifth
kind was built.

**Improvement direction.**
- Each kind answers its lifecycle questions behind one owned contract: sources, identity frame,
  provenance, numerics, native report, warm start, replay constructor and memory extent.
- Lifecycle owners consume that contract without naming kinds.
- Persisted documents carry one tagged per-kind constructor.
- Declared-procedure dispatch and start composition are shared.

**Related:**
- [DA-021](local-reasoning-and-assurance.md#da-021): the flat workflow visibility that makes the
  reach-ins possible.
- [DA-015](boundaries-and-composition.md#da-015): host-side projection of the same operations.

## <a id="da-039"></a>DA-039 — The constructor, not the meaning, classifies most workflow and math failures as invalid models

**Principles:** DP-21, AP-05, DP-03, PS-04, PS-10; G3, G7, PS-G2 (attribution).

**Refresh status:** new. This is DA-008's cause in the workflow and math owners; DA-008 itself is
resolved in the persistence owner.

**Diagnosis.** §23.2 separates three things:
- request admission;
- internal defects;
- typed source failures.

It also forbids reconstructing detail from a coarse class. The realization collapses them.

1. **The workflow path.**
   - `workflow/mod.rs::contract(message)` returns `WorkflowError::Input(String)`, coded
     `DiagnosticCode::WorkflowInput`.
   - The vocabulary declares that code with class `CompileMath`
     (`pse-diagnostics/src/vocabulary.rs`).
   - `pse-model/src/diagnostic.rs::disposition` maps `CompileMath` to
     `BoundaryClass::InvalidModel`.
   - About 2,050 production `contract(` calls in pse-runtime take this path. That is an estimate
     that excludes trailing test modules; a raw grep that includes them gives about 2,340.
   - `WorkflowError::Internal` exists but has about ten uses.
2. **The math path.** In pse-math, `MathError::Contract(String)` (about 650 production sites)
   maps to `MathContract`, also of class `CompileMath`. `MathError` has no internal-invariant
   variant. pse-backend-native is better separated: `ProblemError` distinguishes `Contract`,
   `Internal` and `Limit`.

Sampled sites in all of the following categories surface as `workflow.input` / `invalid_model`,
with the model element only in prose and empty `sources`:

| Category | Example |
|---|---|
| Internal invariant | `run.rs` "completion has not been captured" |
| Resource extent | `horizon/driver.rs` deadline extent; `pse-math/src/implicit_reconstruction.rs` refinement allowance |
| Store integrity | `study.rs` "lazy physical receipt absent" |
| Unsupported capability | `fitting/modeling.rs` "residual LM requires …" |
| Genuine specification error | `cases.rs` "unknown case specification"; "missing case value or resolvable start" |

One condition, implicit-provider lock poisoning, receives three classes: `runtime.infrastructure`
(`pse-math/src/implicit.rs` via `MathError::Library`), `invalid_model` (`implicit_products.rs` via
`Contract`) and a provider terminal error.

**Consequence.**
- **Durable studies.**
  - Lazy point admission treats any failure whose class is not Infrastructure, Internal or
    Cancelled as a scientific `StudyAdmission` refusal and persists it
    (`workflow/study.rs` ≈1061–1083).
  - `durable.rs::failure` marks it deterministic.
  - The clearest manifestation needs no misclassified constructor.
    `pse-model/src/diagnostic.rs::disposition` maps `RuntimeTimeout` and `RuntimeResourceLimit`
    to `BoundaryClass::ResourceLimit`, which the branch does not exempt. A typed timeout or
    resource refusal during lazy admission is therefore persisted as a scientific admission
    refusal for that occurrence.
  - An internal defect or resource extent raised through `contract()` ends the same way, as
    "invalid model".
- **Agent and Python boundary.** Under ADR-0224, source-bearing messages are private and agents
  receive code, stage and class. A platform bug, an exhausted allowance and a missing initial
  condition then look identical, so remedy and retry decisions cannot be made.
- **Well-posedness attribution (PS-04).** `diagnose` reports `VariableMissingValue` and
  structural over- or under-determination as typed findings naming every element
  (`modeling/diagnostics.rs`). The solve path's admission (`cases.rs::require_inputs`) reports
  only the first missing input, in prose, contrary to §23.2.
- **Extension.** Every new refusal in a workflow or math owner defaults to the stringly
  constructor, and nothing asks which class its meaning has.

**Counterevidence.**
- The most consequential scientific refusals are typed with sources: domain, structural
  deficiency, applicability and native outcomes.
- About 145 `ProblemError::internal(...)` sites in `pse-runtime/src/workflow` (for example
  in `dynamics.rs`) classify internal invariants correctly. The separation exists; the
  convenient default undermines it.
- Many `contract()` sites guard practically unreachable checked-arithmetic extents.
- "Invalid model" is correct for the genuine specification errors.
- The share of reachable misclassified sites is unknown.

**Evidence:** *Implemented* for the mapping chain and sampled sites; the coordinator re-read the
`contract` → `WorkflowInput` → `CompileMath` → `InvalidModel` chain and the lazy-admission
branch. The durable and agent consequences are *Interface-checked*. Either of these would settle
the reachable share:
- a static categorization of the constructor sites;
- injecting a resource failure during lazy admission.

**Improvement direction.**
- Constructors whose class follows the meaning: specification refusal with affected identities
  and rule, internal invariant, resource limit, integrity or infrastructure.
- Formulation-time specification and well-posedness refusals projected through the same typed
  findings `diagnose` produces.
- One classification per failure condition across math and runtime.

## Resolved identities

| ID | Refresh outcome | What establishes it |
|---|---|---|
| <a id="da-008"></a>DA-008 | Resolved | Store refusals become `Refused{code, class, stage, effect}` only from thrown registry codes (`pse-operations/src/canonical.rs`, `From<surrealdb::Error>`). Every generated `THROW` is a `PSE_REFUSAL:` code. Worker loss is `CanonicalError::WorkerLost`, with effect taken from facts. Residues: summary recovery still recognizes a lost writer by a field conjunction (`study.rs` ≈1786). The development stores' hand SurrealQL (`pse-operations/src/{evidence,knowledge}.rs`) throws free text, which is a [DA-020](boundaries-and-composition.md#da-020) manifestation. |
| <a id="da-010"></a>DA-010 | Resolved | `CovarianceOrigin::{QualifiedFit, External}`. `ParameterCovariance::verified_fit` requires a reconciled certified content hash. External covariance is explicitly uncertified and checked for PSD, with units carried (`math/settings.rs`; `uncertainty/psd.rs`). |
| <a id="da-011"></a>DA-011 | Resolved | The generated document catalogue and versions (`pse-py/src/generated/documents.rs`). The boundary-schema fingerprint is checked at import (`python/pse/_build.py`). |

The removal commit of this refresh preserves their 2026-10-21 text.
