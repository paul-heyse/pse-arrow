---
title: Efficiency alignment across the simulator codebase
date: 2026-10-11
tier: design
purpose: target
status: review
standard: Core 3.4
heuristics: Efficient Architecture 1.0
profile: Process Simulator 1.5
baseline: b01a45d71a5f5ab29991ad193679f5588ac3662f
---

# Efficiency alignment across the simulator codebase

## 1. Synthesis: decision, scope and what to retain

**Decision: Revise.** The semantic architecture is sound and its numerical library leverage is
strong. Its physical realization is not yet fit for the simulator's declared workloads. Several
supported operations perform structural work, crossings, coordination and publication that grow
with the wrong quantity: package size instead of edit size, retained history instead of live
participants, study size squared instead of ready work, index fields instead of demanded results.
These are architectural defects under AP-07/G9 even though current outputs are correct and no
speed has been measured.

The target is the best-in-class simulator of the Process Simulator profile and blueprint §0/§0.2.
An engineer authors physically typed, reusable models once. The engineer edits and re-solves
interactively; a value change must not rebuild structure. The engineer runs sweeps, sensitivities,
optimization and fitting, converges recycles and runs dynamics from the same definitions, and
extends the library with new units, properties and solvers. The development loop compiles and tests
one responsibility at a time. ADRs, blueprint decisions, plans, binding policies and pins are
treated as current design. Section 10 lists the rules these recommendations would change.

### What the design gets right, and must keep

The design carries a chain of correct and largely well-placed decisions:

1. Authored typed definitions produce a narrowly premised selected closure.
2. Immutable prepared products are keyed by complete semantic identity:
   - semantic body hashes;
   - artifact keys that exclude revision attribution;
   - program assembly memoized once per prepared structure.
3. Prepared solver views rebind values without rebuilding structure (`Unchanged`/`Sharing`/`Rebuild`).
4. Library-owned structural analysis. Matching, Dulmage–Mendelsohn and BTF come from
   pounce-presolve, and the witness is carried and revalidated linearly.
5. Class-specific native solvers keep their sessions:
   - Ipopt `ReOptimizeTNLP`;
   - KINSOL setup reuse and Anderson acceleration;
   - the full IDAS surface;
   - FERAL KKT inertia;
   - HiGHS tear selection with an independent petgraph witness.
6. An independent original-space PS-10 assessment runs on a fresh worker.
7. Fenced, closed and sealed coherent publication uses compact acknowledgments and exact
   uncertain-outcome settlement.
8. Results reach Python as a zero-copy Arrow C stream with the GIL released.

The study path already shows the right value shape. A study admits typed overlays through
`admit_overlay`, binds them as case members on a prepared basis, and rebinds the view per point. The
ephemeral study executor is event-driven and retains a predecessor's session for a single successor.
These strengths are preservation constraints for every remedy below. No finding recommends moving
numerical work into the store, replacing a qualified solver, or adding a general cache, scheduler or
accounting framework.

### The strongest causes

The findings group into six causes. Each cause is a structural choice. None is a slow
implementation detail.

1. **Preparation and validity are owned per call, not for the lifetime of their premises.**
   - An interactive value edit cannot reach the value route studies use. It mints a revision and
     re-proves the whole selected closure twice ([F01](#f01)–[F03](#f03)).
   - Product descriptions are bound to every premise of the read, so any edit re-uploads every body
     ([F04](#f04)).
   - Study points re-establish preparation that their fixed revision already settles ([F06](#f06)).
   - Two reuse mechanisms exist, and neither stops propagation inside the closure ([F07](#f07)).
2. **Immutable validity is re-proved by reconstruction instead of carried.** The process image,
   managed executables, native generations, validation owners and IPC frames are re-hashed or
   re-validated at crossings that change no premise ([F05](#f05), [F23](#f23)).
3. **Coordination domains are broader than the effects they protect.**
   - One per-problem guard record, one exclusive local gate and one study-wide record serialize
     every lane of a study ([F08](#f08)).
   - Durable discovery rescans blocked work ([F09](#f09)).
   - Capacity is charged at grains that do not match demand ([F10](#f10)).
4. **Publication shape is set by mechanism, not by demand.**
   - A per-field scalar-cell index sets the block size ([F11](#f11)).
   - Durable finish re-reads what the writer holds ([F12](#f12)).
   - Study, horizon and sensitivity detail ignores consumer demand ([F13](#f13)).
   - Per-iteration telemetry is a guaranteed durable stream ([F14](#f14)).
   - Lifecycle residue is never collected ([F15](#f15)).
5. **The inner loop reshapes semantic representations per callback, and nested work runs at
   semantic grain.**
   - Callbacks bind through identity maps and materialize widest shapes ([F16](#f16)).
   - Programs are re-optimized per order and deep-cloned per attempt ([F17](#f17)).
   - The cubic accelerator runs exact isolation behind a cross-attempt mutex ([F18](#f18)).
   - Conditional units and initialization blocks each run a complete attempt ([F19](#f19)).
   - Dense, global or repeated numerical steps: [F20](#f20)–[F22](#f22).
6. **The development loop scales with retained history and couples tests to infrastructure**
   ([F25](#f25)–[F28](#f28)).
   - Every environment exit rewrites a growing ledger.
   - Every Runtime-constructing test provisions a canonical store.
   - Entry routes compile the whole product.
   - Admission partitions CPU statically.

The corrections fit together through a few shared contracts:

- one **preparation engagement** owner that holds protection, settlement and prepared views for a
  revision's working lifetime;
- one **coordination protocol** that registers conflicts where effects are;
- one **demand-shaped publication** unit;
- one **operation-shaped evaluation layout** behind the existing `CaseWorker` owner.

[§8](#8-alternatives-remedy-compatibility-and-order) explains their order and conflicts.

### Scope and decisions

| Field | Assessment |
|---|---|
| Subject and boundary | The whole simulator: authoring, admission and preparation; mathematical compilation and evaluation; the native solve route; multi-case workflows and shared capacity; publication, persistence and serving; the relational/columnar layer; identity and assurance; and the build/test/development loop. Includes the Rust/Python boundary, the SurrealDB canonical store and the supervisor scripts |
| Standard | Core principles and template 3.4, Heuristics for Efficient Architecture 1.0, Process Simulator principles and review additions 1.5, pse-arrow binding |
| Tier / purpose | DESIGN / TARGET |
| Reviewer / date | Fresh principal design reviewer, independent of the supporting inquiries, 2026-10-11. The reviewer re-read the decisive source for every finding |
| Baseline | `main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f`. The working tree also held an uncommitted `scripts/test_resources.py` change that defers reclaim while service admission is closed. It is outside the baseline and does not alter [F25](#f25) |
| Architectural fitness | **Fails G9.** AP-07 and AP-06 are violated, and AP-05 is violated narrowly in development tooling. AP-01–AP-04 are satisfied within the inspected scope |
| Behavioral/semantic adequacy | **Revise (narrow).** G7 and G8 fail narrowly ([F24](#f24)). G1–G6 pass on the inspected paths. PS-G1–PS-G3 were not re-qualified and stay unresolved for whole-simulator acceptance; no contrary evidence was found |
| Overall decision | **Revise** |
| Disposition owner | **Not yet scheduled.** The proposed owner is a **new plan created through `plan-creation`**. These findings are not added to Plan 28, and no completed plan (28a–28k, 33, 34, 35) is reopened. Rule impacts take effect only after the operator confirms them during plan creation |

Supporting documents, which carry bounded judgments (this review owns the combined decision):

- [Inquiry index, reconciliation, coordinator verification and probes](../evidence/efficiency-alignment-codebase-2026-10-11/README.md)
- [Operation map](../evidence/efficiency-alignment-codebase-2026-10-11/operation-map.md): warm edit→re-solve, study point N+1 and result reopen
- A: [authoring, admission and incremental preparation](../evidence/efficiency-alignment-codebase-2026-10-11/authoring-admission-and-incremental-preparation.md)
- B: [mathematical compilation and evaluation](../evidence/efficiency-alignment-codebase-2026-10-11/mathematical-compilation-and-evaluation.md)
- C: [single-solve native route](../evidence/efficiency-alignment-codebase-2026-10-11/single-solve-native-route.md)
- D: [multi-case workflows and capacity](../evidence/efficiency-alignment-codebase-2026-10-11/multi-case-workflows-and-capacity.md)
- E: [publication, persistence and results](../evidence/efficiency-alignment-codebase-2026-10-11/publication-persistence-and-results.md)
- F: [relational and columnar layer](../evidence/efficiency-alignment-codebase-2026-10-11/relational-and-columnar-layer.md)
- G: [build, test and development loop](../evidence/efficiency-alignment-codebase-2026-10-11/build-test-and-development-loop.md)
- H: [assurance, identity and observation](../evidence/efficiency-alignment-codebase-2026-10-11/assurance-identity-and-observation.md)
- L1: [numerical library utilization](../evidence/efficiency-alignment-codebase-2026-10-11/numerical-library-utilization.md)
- L2: [data and infrastructure library utilization](../evidence/efficiency-alignment-codebase-2026-10-11/data-and-infrastructure-library-utilization.md)

Inquiry candidate labels (A-01, D-02 and so on) are local to those documents. This review's
finding IDs are F01–F29.

## 2. Scope, workloads and coverage (slot 1)

**Analysis modes in scope:** square simulation, optimization, dynamics with sensitivities,
parameter estimation with profiles, rolling-horizon estimation and control, and studies over
sweeps. Recycles and tears are included.

**Workload premises.** These are qualitative; there is no SLA and no cost model.

- Interactive edit → re-solve on one flowsheet, with both value-only and structural edits.
- A thousand-point value sweep.
- Sixteen independent end-to-end cases on sixteen physical cores. The host has 32 logical CPUs and
  about 188 GiB, shared with other repositories. The reference execution configures 32 math jobs
  and 16 in-flight cases.
- Larger sparse structures, many small blocks, and growing expression populations.
- Long trajectories with sensitivities, and fitting with many observations and parameters.
- Growing retained history, and interruption before publication.
- A developer compiling and testing one responsibility.

**Inspected.** Ten bounded inquiries supply the supporting analysis: eight design inquiries (A–H)
and two library-utilization inquiries (L1, L2). The coordinator's operation map traced three warm
operations; the coordinator also made source checks V1–V21 and ran light probes M1, S1 and G-P1 to
G-P3. The principal reviewer re-read the decisive source for every published finding, at the
`path::symbol` locations each finding's **Evidence** line names. Where a count or search comes from
an inquiry rather than the reviewer, the finding says so.

**Not inspected, or read shallowly** (reconciliation §6 and the inquiry coverage notes):
`root_isolation.rs` (IBEX); the PETSc, SCIP, HiGHS, Uno and Clarabel adapter internals; `kkt/path/*`
continuation and the IDAS adjoint routes; the dynamics trajectory and shooting encoders;
`derived.rs`, `factorable.rs`, `collocation.rs`, `surrogate.rs` and `implicit_regimes.rs` proof
transport; the `pse-authoring` parser and `pse-quantity` inference; `pse-codegen` internals; the
`pse-engine` session internals (`round`, `traversal`, `cache/*`, `operation/*`, `provider/*`);
`python/pse` beyond the boundary modules; most of the roughly 7.3k-line supervisor; `scripts/tests/`
and `tests/`; teqp and parity.

No end-to-end measurement was made. These are unexamined breadth, not defects. [§9](#9-verification-and-evidence-limits-slot-10)
separates them from the premises that could change a remedy. None of them could reverse the G9
verdict, which several independent structural violations establish.

## 3. Operations, authority and change scenarios (slots 2–5)

### 3.1 Responsibilities and the complete operations

| Responsibility (owner) | Decision hidden | Contract consumed / exposed | Dependency direction | State and effect owner | Local test setup today |
|---|---|---|---|---|---|
| Authored definitions and checking (`pse-authoring`, `pse-modeling`) | Grammar, closure selection, checking | Declarations → checked selected package | Meaning; no backend | Immutable values | Pure |
| Preparation (`pse-compiler`; `pse-runtime/src/math*`) | Specialization, planning, body identity, retention | Selected admission → prepared basis, view and programs | Compiler → math library | `ModelingCache`, `DefaultCache` and a fresh Salsa workspace per call | Requires a canonical store when reached through `Runtime` ([F26](#f26)) |
| Evaluation (`pse-math` assembly, execution and implicit) | Layout, derivative programs, providers | `CaseValues` → `Evaluation` | Math → Symbolica/faer | Attempt-owned workers, except the cubic accelerator ([F18](#f18)) | Pure |
| Native solve (`pse-backend-native`) | Adapter lifecycle, presolve, callbacks | Oracle → typed solve report | Native → libraries | Session thread with a `Retained` session | Pure, or native-only |
| Workflow composition (`pse-runtime/src/workflow`) | Analysis routing, studies, durability | Package/study APIs → `RunResult` and handles | Workflow → math, native, operations | `Runtime`, `MathService` caches, session threads | Requires a canonical store |
| Canonical store (`pse-operations`, generated SurrealQL) | Revisions, protections, coordination, publication protocol | Typed store API, server functions | Operations → SurrealDB | Server guards, `ProblemTurns` | Requires a SurrealDB fixture |
| Relational layer (`pse-engine`, `pse-relations`) | Validation contexts, DataFusion planning, memory pool | `FieldCheckedBatch`, `ValidationContext`, pool | Engine → DataFusion/Arrow | Factory assemblies (weak) | Pure |
| Boundary (`pse-py`, `python/pse`) | Python API shape, GIL slicing | JSON documents and Arrow C streams | Python → runtime | Process executor | Store fixture |
| Development environment (`scripts/`, `justfile`) | Admission, test resources, native cache | `pse-env`, recipes | Scripts → systemd, store, caches | Test-resource ledger, operation records | Real host state |

The operation map gives the stage-by-stage trace. In summary:

- **A warm value edit runs as follows:**
  1. An inventory read and a JSON round trip.
  2. Complete-source persistence of a new revision.
  3. Two preparations. Each one qualifies selection, misses the basis on the new revision, and
     builds fresh compiler workspaces.
  4. Description settlement for every portable body, with receiving qualification.
  5. A view rebind.
  6. A native session thread.
  7. Durable publication of about 22 relations, with indexed scalar cells.
  8. A full re-read of the sealed record before `wait()` returns.
- **Study point N+1** reconstructs the package and reads the declaration. It makes two preparations;
  each protects and releases, and each acknowledges every body sequentially. It then rebinds, opens
  a new session thread, publishes the full detail and observes the point. All of a study's
  mutations share one problem gate.
- **Reopening a result** costs four RPCs plus, per page, a renewal and a page read. Each block then
  takes one RPC, a double digest, an IPC preflight and decode, a registry admission and an FFI batch.

### 3.2 Physical meaning, authority and well-posedness (profile slot 3)

The remedies below change representations at the boundaries in this table. Each remedy must carry
the listed meaning unchanged. Cells the reviewer could not establish are marked.

| Quantity or element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| Authored case or fixture value | Declared quantity type and unit | As declared | As declared | Admission rejects incompatible targets (§19) | Authored declaration (§22.2) |
| Study or overlay assignment | Admitted physically through `admit_overlay` | As target | As target | Refuses ambiguous, missing or incompatible targets | `AdmittedBinding` (study binding) |
| `CaseValues` scalar | Declared representation unit, converted to canonical by `CanonicalConversionPlan` (affine, precompiled) | Per target | Carried by quantity type | Bind-time missing and nonfinite refusal | `pse-math` binding |
| Solver coordinate | Normalized by `Normalization`; physical point recovered at each callback | — | — | Nonfinite trials refused | Resolved numerical policy (§16) |
| Result relation row | Physical units; basis and reference-state columns not re-established by this review | Not established here | Not established here | — | Registry schema (§4) |
| Warm seed | Exact IEEE bits (`u64`) | As state | — | Compatibility of layout, profile and data | Seed document (§20.6) |

**Well-posedness statement.** The authored role (fixed specification, free unknown, start) is
declared through fixtures and case bindings. A value overlay binds values only. Changes to fixed or
free status, bounds, rows or contributions remain structural: §14.4 routes them through the case
plan, and `admit_overlay` does not express them. The compiler computes matching, DM and BTF once
per structure (`pse-structural/src/incidence.rs::CaseIncidence::analyze`, through pounce-presolve).
Each native step revalidates the carried witness linearly (`pse-backend-native/src/structural.rs::check`)
before the solver runs. This review did not re-examine whether rejection diagnostics name model
elements. The value route proposed in [F01](#f01) preserves well-posedness by construction:
it cannot change roles, so the structural witness stays valid.

### 3.3 Change scenarios (slot 4)

| ID | Stimulus and conditions (binding scenario) | Expected response | Observed impact | Findings |
|---|---|---|---|---|
| S01 | Interactive value edit, then re-solve, on the default durable runtime (the target workload; PSE-S07) | Bind values onto retained structure; publish only what is demanded | Mints a revision and prepares twice over the whole closure. Re-describes every body, re-hashes the image, publishes full detail with cells, then re-reads | F01–F05, F11, F12, F14, F16 |
| S02 | Structural edit or text reload | Work proportional to the changed declarations | Package-proportional ingress; no cutoff inside the closure; reload mints a new problem | F02–F04, F07 |
| S03 | Thousand-point value sweep (PSE-S03) | Prepare once; bind per point | Per point: about 25 + 2B crossings, a new session thread, full publication | F06, F08, F11, F13, F17, F22 |
| S04 | Sixteen cases on sixteen cores (PSE-S07) | Lanes proceed independently within shared capacity | All mutations of a problem pass through one gate; guard bumps abort concurrent reads; idle native capacity during I/O | F08, F10, F14, F18 |
| S05 | Continuation or dependency study | Discover ready work in O(ready) | O(N²) rescans of blocked points; per-round barrier | F09 |
| S06 | Recycle with conditional units; initialization of many small blocks | Native session and factor reuse; block-sequential execution | One complete cold attempt per fixed-point evaluation and per block | F19, F21 |
| S07 | Long trajectories with sensitivities; fits with profiles; long horizons | Demand-shaped retention and admission | Full state sensitivities at every sample; peak charge held for the whole fit; horizons retained in memory | F10, F13, F20 |
| S08 | Growing retained history; interruption before publication | Residue bounded by live participants; coherent recovery | Receipts, guards and protection tombstones grow without bound; recovery is sound | F15 |
| S09 | Add a unit or property model (PSE-S01); replace or upgrade a library (PSE-S02, PSE-S06) | One authoritative addition; library change absorbed by its integration owner | The extension stays local in semantics. Its preparation cost grows with closure and body count (F04). Unused engine machinery is carried through every DataFusion upgrade (F24) | F04, F07, F24 |
| S10 | Compile or test one responsibility (PSE-S05) | Only the responsibility's dependencies | A per-test canonical store; ledger rewrites at every exit; whole-product entry routes; static CPU quota | F25–F28 |
| S11 | Change a result storage representation (PSE-S04) | Persistence owners absorb it | Local to persistence owners: codegen schema, projection, reader. The [F11](#f11) remedy is itself this substitution | F11 |

**Domain extension (S09).** Adding a unit model that uses existing physics stays semantically local.
Its bodies compile through the shared pipeline, and admission and diagnostics are not re-authored.
The physical cost of the extension is the problem: [F04](#f04) settles descriptions per body against
the whole closure's premises, so each added body enlarges every later edit.

**Mechanism substitution (S11, S09).** Replacing the scalar-cell index with key-ordered blocks
changes only `pse-codegen` SurrealQL, `result_projection` and `connected_results`, plus the §19.2
text. Replacing the Salsa owner ([F07](#f07)) changes only `pse-compiler` and the `MathService`
retention. Both substitutions are well localized, which is a strength of the decomposition.

## 4. Findings by cause (slot 7)

Every finding records three things. **Diagnosis evidence** is source-traced, which this review
labels Interface-checked, unless a measurement is named. **Remedy maturity** is Proposed unless
stated otherwise. **Priority** is consequence-based; [§8](#8-alternatives-remedy-compatibility-and-order)
gives prerequisite order separately.

### 4.1 Preparation and validity at the wrong lifetime

#### <a id="f01"></a>F01 — An authored value edit has no value route; every value change rebuilds the selected structure

**Cause.** The only interactive edit intent is complete-source replacement. That is
`crates/pse-runtime/src/workflow/modeling.rs::ModelingPackage::with_declarations`, exposed to Python
as `with_declarations(source: bytes)` in `python/pse/_native.pyi`. Case and fixture values are
authored inside declarations: `workflow/modeling/declared.rs::declared_execution` reads
`model.compiled().model.fixtures`. A value tweak therefore mints a revision, and the consequences
follow in order:

1. The retained selected admission misses: the same-revision adoption fails.
2. The basis misses, because `BasisKey` includes `SelectedDependencies`.
3. Selection rehydration, complete checking, specialization, projection, planning and completion
   rerun in fresh compiler workspaces.
4. Only at `workflow/modeling/views.rs::bound_case` does the edit reach value-level reuse
   (`rebind_scoped`), after the rebuilt structure hashes equal.

The value machinery already exists. `workflow/bindings.rs::ModelingPackage::admit_overlay` resolves
targets and admits values physically, and `prepare_bound_operation` binds them as
`case.members` on the unchanged basis. Only studies and initialization overlays reach it.

**Consequence.** On S01 every feed or parameter tweak pays closure-proportional preparation twice
([F03](#f03)), whole-body settlement ([F04](#f04)), receiving qualification ([F05](#f05)) and
package-proportional ingress ([F02](#f02)), before reaching the cheap rebind. This is the target's
explicit anti-goal ("a value change must not rebuild structure"). It is also the heuristic case
where proving an input unchanged repeats most of the work (H14).

**Principles.** AP-07/G9; PS-11 (SHOULD, not met, with no exception record); DP-09 and DP-10
(separate structural from value inputs); H8 (the interface hides the unit of intent); H12; H13.

**Corrective contract.**

- **Consumes:** a parent revision's held preparation engagement (see [F03](#f03) and [F06](#f06))
  and typed assignments by member or path with quantities.
- **Decides:** target resolution, physical admission and duplicate refusal (all from `admit_overlay`),
  and classification. Assignments the overlay contract cannot express fall to the structural route:
  fixed/free changes, structural literals, domain sizes and discretization counts.
- **Produces:** a bound case on the parent's basis and view (`rebind_scoped`), with result lineage
  "parent revision + overlay identity". A separate, explicit commit operation can later persist the
  values as a delta source edit ([F02](#f02)).
- **Consumers stop:** minting revisions in order to solve, and re-checking structure for value changes.

**Preserve.**

- Results name the exact values used.
- Structural literals stay structural.
- Unit and physical admission are kept.
- PS-10 post-solve checks are kept.
- A warm start is recorded as a dependency of the result (the PS-11 MUST).
- The overlay never silently diverges from the authored revision a user later reopens. Lineage says
  which one a result belongs to.

**Uncertainty and settling evidence.** It is not yet established which authored value fields
`admit_overlay` addresses: parameters and fixed variable values are clear, while starts and bounds
need checking. Probe A-P1 (edit versus warm `prepare_solve`, with frontier/basis load counts)
sizes the gap.

**Evidence.** Diagnosis: Interface-checked (the reviewer read `with_declarations`, both `prepare`
call sites, `admit_overlay` and the Python surface). Remedy: Proposed. Sources: A-01,
operation-map a.2–a.3.

#### <a id="f02"></a>F02 — Edit ingress does package-proportional work, and text reload discards problem lineage

**Cause.** `ModelingPackage::with_declarations` calls `workflow/modeling/canonical.rs::full_source`,
which rehydrates every row, data document, package and fit of the previous revision. It then
discards the rows (`let (_, scope, documents, sources, _)`) and rebuilds the documents batch.

`canonical.rs::persist` then does all of the following:

- builds a fully validated `modeling_declarations` batch and drops it;
- frames every row with BLAKE3 (`math::modeling::source_revision`);
- reparses each row's expressions through `SourceIndex::references`;
- serializes and hashes every row in `edit()`;
- admits every data table in a fresh compiler workspace whenever the package has any data rows
  (`math().workspace` + `modeling_revision`);
- scans every member of the previous revision (`membership_page`).

Only after all of that does it drop unchanged edits. The store protocol is already a delta:
`store.edit(problem, parent, …)` applies edits to a parent. Python adds about four whole-document
JSON passes. Text reload (`modeling_from_documents` → `persist_modeling(previous = None)`) mints a
new problem, which loses problem-scoped selection, basis and product reuse and duplicates the object
set. Membership paging adds a smaller cost: one protected transaction per 64 rows, with the version
interval post-filtered after the `(problem, key)` index (L2-08; Proposed; probe L2-P3).

**Consequence.** On S01, S02 and S09 the cost of one edit grows with package size and data volume,
not with the edit. Each reload also enlarges retained history (S08).

**Principles.** AP-07; H5; H12; H16; H25.

**Corrective contract.** Ingress consumes a delta against a parent: changed, added and removed
declarations or documents. It decides identities only for changed objects, and recomputes references
only when a name, parent, kind or import changes. It re-derives record suppliers only when the data
closure changes, and makes the source-revision identity derivable from member versions (a frame
version change). Reload passes the parent and reparses only documents whose checksum changed.

**Preserve.** The exact unchanged-publication return. Deletions of absent members. Complete
validation of the rows that actually cross the boundary (H18).

**Evidence.** Interface-checked: the reviewer read `with_declarations`, `persist` and its
previous-revision diff. Remedy: Proposed. Sources: A-02, L2-08.

#### <a id="f03"></a>F03 — One solve prepares twice, and every preparation re-qualifies and re-settles per call

**Cause.**

- `declared.rs::declared_execution` calls `prepare` to read the fixture and procedure.
  `workflow/modeling/cases.rs::resolve_case` then calls `prepare` again, with `bindings.demand`
  extended by the fixture's value and variable names. That is a different `BasisKey`.
- Each `prepare` takes a protection and walks retained candidates
  (`canonical.rs::qualified_selected_candidate`, newest first; on a new revision each costs at
  least one RPC before it misses, A-06).
- Each `prepare` then builds or loads the basis, runs `settle_preparation`, and releases.
- `prepare_flow` builds an uncached fresh workspace on every call.
- Every fresh workspace reserves the full `workspace_bytes` (4 GiB by default) up front
  ([F10](#f10)).

**Consequence.**

- On S01 and S02, an edit pays the whole preparation and settlement twice.
- In an editing session, the miss cost grows with the number of distinct edits retained for the
  same request (H10).
- The per-call shape is also what makes study points expensive ([F06](#f06)).

**Principles.** AP-07; H2 (two semantic steps become two physical preparations); H12; PA02 residual.

**Corrective contract.**

- Compute the final demand before preparation. The fixture's member names are authored on the root
  declaration, and a semantic-only `prepare_semantic_modeling_revision` exists. Then prepare once.
- A **preparation engagement** owner holds the protection and the settled descriptions for its
  lifetime; it is shared with [F06](#f06) and [F01](#f01).
- Candidate qualification resolves the union of the retained candidates' premises in one batch and
  compares them locally.

**Preserve.** Binding validation against the product actually solved (at least as strict). Every
premise of an accepted candidate is rechecked. A/B/A reuse is kept.

**Evidence.** Interface-checked: the reviewer read both `prepare` call sites,
`qualified_selected_candidate` and `MathService::workspace`. Remedy: Proposed. Sources: A-04, A-06.

#### <a id="f04"></a>F04 — Product descriptions bind the whole selected closure, so any edit re-describes and re-uploads every body on the critical path

**Cause.**

- `pse-operations/src/canonical_selection.rs::ProductDescription::prepare` snapshots and serializes
  the read's complete dependency premises.
- `ProductDescription::matches_read` requires the description's premises to equal every premise of
  the current read, kind by kind: Name, Scope, Interpretation, Logical, KindScope and References.
- `canonical_staging.rs::product_blob` hashes `payload ‖ dependencies` twice.

A body's semantic identity (`grouped::semantic_body`) already frames its own semantic dependencies,
yet an edit to any declaration in the closure invalidates every body's description. The chain is:

1. `workflow/modeling/preparation.rs::settle_preparation` calls `describe_body`, then
   `publish_description` (staging upload and admission), for every portable body.
2. The bodies are processed sequentially.
3. `prepare_selected` awaits all of this before returning.

The dependency premises are identical for every body in a read, yet they are encoded once per body
(H-05).

**Consequence.**

- On S01 and S02, roughly B × (payload + P) bytes are uploaded and B admissions made per edit, where
  B is the number of portable bodies and P the premise size. Both grow with flowsheet size, so the
  upload grows roughly quadratically.
- On S08, B new product roots accumulate per revision.
- On S09, every added body enlarges every later edit.

**Principles.** AP-07; H13 (incidental provenance propagated as a result-affecting dependency); H16;
H23 (publication coupled to private preparation); H26.

**Corrective contract.**

- (i) **Content and reachability.** Upload a content-addressed payload once per (body semantic
  identity, producer). Record reachability in small per-revision or per-selection root records.
- (ii) **Premise scope.** Encode premises once per read. Where identity frames are shown complete,
  record only the premises a body consumed.
- (iii) **Settlement timing.** Settle once per engagement and off the solve's critical path, draining
  on cancellation. Skip settlement on ephemeral runs that publish nothing.

**Preserve.**

- ADR-0164's receiving admission before a new eligible publication.
- Detection of released roots and expired protections before reuse is claimed.
- Exact settlement of uncertain acknowledgments.
- No cached permission (the PA03 counterexample).

**Uncertainty.** Item (ii)'s narrowed premises are safe only if `SemanticBodyHash` frames every
meaning-affecting dependency, including absent names a body probed. That needs a formal argument or
a decision; until then, (i) and (iii) still remove the re-upload and the critical-path coupling.
Probe A-P3 (count products and blocks before and after a one-parameter edit) sizes the effect.

**Evidence.** Interface-checked: the reviewer read `matches_read`, `prepare` and the
`settle_preparation` loop. Remedy: Proposed. Sources: A-03, H-05; PA03 moved.

#### <a id="f05"></a>F05 — Immutable runtime, executable and generation validity is re-proved by complete content hashing at every use

**Cause.** There is one cause with four manifestations. Each re-establishes a property whose
premises have not changed, by full SHA-256 reconstruction, at a crossing rather than at the trust
transition.

- **(a) Receiving qualification.** The call chain is:
  1. `settle_preparation`
  2. `math/portable.rs::qualify_replay`
  3. `ReplayAdmission::qualify_receiving`
  4. `pse-buildinfo/identity.rs::LocalRuntimeObservation::verify_scoped`
  5. a full `capture_scoped`, which hashes every executable-mapped ELF and then re-hashes each loaded
     file in a closing `verify_scoped` loop.

  This runs under the glibc loader write lock. It is triggered once per `prepare` that must publish,
  so a value edit triggers it twice ([F03](#f03), [F04](#f04)). Under memory pressure, retention
  bypass (`ModelingCache::put`) makes study points repeat it.
- **(b) Runtime construction.** `pse-py/src/workflow.rs::NativeRuntime::new` calls
  `pse_buildinfo::observe_deployment` → `observe_outer` outside `py.detach`, so the GIL is held.
  That function reads every file under `crates`, `vendor`, `xtask`, `python` and `scripts` and
  spawns `git rev-parse`. A full local capture follows. This happens for every `pse.Runtime` object,
  and Python tests construct at least one per test. The resulting outer digest describes the
  checkout at construction, not the source that built the loaded artifact.
- **(c) Managed primary.** `workflow/worker.rs::Runtime::ensure_managed_primary` does the following:
  1. Captures the worker and supervisor script (`NativeExecutionSelection::from_store`).
  2. Spawns `scripts/surreal_server.py ensure-primary`. There, `_ensure_primary` calls
     `checked_primary`, which runs `recorded_primary` → `verify_generation` plus two `file_digest`
     calls. `primary_ready` calls `checked_primary` again, and `_primary_process` hashes the
     process's executable.
  3. Calls `require_current`.

  That is about seven to eight full passes over the 727 MB development `pse-worker` per durable
  study start, before any point runs. Worker start adds more.
- **(d) Native cache.** `scripts/native_cache.py::prepare` → `valid` → `installation_files` hashes
  every installed file of each family on every top-level `pse-env --native`. `collect` →
  `pinned_generations` scans every operation record (1,190 at baseline, Measured by `ls`), and
  settled records are not retired.

**Measured.**

- M1: 0.30–0.39 s per SHA-256 pass over a 0.73–0.79 GB binary, hot cache.
- G-P3: 0.58–0.71 s per solver generation (1.2 GB).
- Arithmetic only, not an end-to-end measurement: about 1.2–1.6 s of hashing per value edit if two
  qualifications occur (probe H-P1 confirms the count), and about 2–3 s per durable study start.

**Consequence.**

- On S01 and S03 these are seconds of serialized CPU on the interactive path, with loader exclusion
  that blocks any concurrent `dlopen`.
- `FileObservation::capture` is non-interruptible, and in (c) it runs on an executor thread.
- On S10, every Python test pays (b) and every native command pays (d).
- The same incident class as 2026-10-08 has moved, not disappeared.

**Principles.** AP-07; DP-03 ("reuse immutable established validity while its premises hold");
H18, H19, H20, H26, H28; DP-20 (a blocked executor).

**Corrective contract.** Validity is owned at its trust transition.

- **Process image.** The startup observation is the image's owned validity. Later "current
  admission" is a premise check:
  - the executable-mapping set from `/proc/self/maps`;
  - loader add/sub counters;
  - per-file `(dev, ino, size, mtime_ns, ctime_ns)` with racy-clean handling;
  - effective configuration bytes.

  Full re-observation happens only when a premise changes, and the trigger no longer depends on
  basis residency.
- **Generations and executables.** Content is verified at publication and at exec or launch.
  Readiness and per-operation checks compare stat identity against the stored digest. Rust passes
  its observation to the supervisor instead of both recomputing.
- **Outer provenance.** Recorded once per process, outside the GIL. Alternatively, the executable
  identity serves as run provenance, with git's stat-cached index status as checkout context.

**Preserve.**

- Replaced or modified bytes are refused. Atomic replacement changes the inode, and an unprivileged
  process cannot set ctime.
- A late load or unload invalidates.
- Strict producer receipts are kept.
- The configuration observer is kept.
- Interruption and drain behavior is kept.
- Privileged in-place mutation stays outside the documented contract.

**Uncertainty.** Probe H-P4 (late executable mappings) matters either way. If MKL or libffi map
executable pages after startup, `qualify_receiving` returns `None` and persisted reuse is silently
disabled for the process. That is a different defect requiring the same owned-validity redesign, so
the probe changes the remedy's controls, not the verdict.

**Evidence.**

- Diagnosis: Interface-checked (the reviewer read `capture_scoped`, `verify_scoped`,
  `NativeRuntime::new`, `observe_outer`, `ensure_managed_primary`, `require_current`, the supervisor
  call sites and `native_cache.valid`/`collect`).
- Per-pass costs: Measured (M1, G-P3).
- Totals: arithmetic.
- Remedy: Proposed.
- Sources: H-01, H-02, H-03, E-06, G-05; PA01, PE05 and PE06 residual or moved.

#### <a id="f06"></a>F06 — Study and horizon occurrences re-establish study-invariant preparation per point

**Cause.** Per point, `workflow/worker.rs::work_once_scoped` calls
`workflow/study.rs::prepare_study_candidate`. That function reads the point and the study header,
calls `physical_source` and `store.revision`, and reopens `modeling_revision` (protect and release).
It then calls `bindings.rs::prepare_bound_operation`:

- `declared_execution`: a declaration read (about five RPCs) and `prepare` #1;
- `prepare_declared`: `prepare` #2, a rebind job and `prepare_solve`.

Each `prepare` protects and adopts the same-revision admission. It acknowledges every portable body
sequentially (`canonical.rs::product_acknowledged`, which takes `retention:<problem>` `FOR UPDATE`)
and then releases. All of this happens against a revision the study has pinned. `DeclaredStudyAdmission`
is built twice and dropped. Horizons repeat the shape per sample
(`horizon/driver.rs::prepare_solve` → `prepare_analysis_attempt`).

**Consequence.**

- On S03, each point makes about 25 + 2B sequential crossings before its claim, about seven of them
  mutations paced under [F08](#f08).
- A 1,000-point sweep of a flowsheet with tens of bodies re-proves fixed premises tens of thousands
  of times.
- S07 horizons repeat the same work per sample.

Compiler products are cached, so the waste is canonical re-establishment, not compilation.

**Principles.** AP-07; PS-11; DP-10; H12; H14; H19. This is the profile's "structure rebuilt per
case" shape in canonical rather than compiler form.

**Corrective contract.** A study engagement consumes the study's pinned revision and operation
descriptor. For as long as a worker is engaged with the study, it holds:

- one protection, renewed with the lease;
- the settled basis, view and descriptions.

Per point, the engagement performs only binding and seed-need admission (`validate_binding`), the
value rebind or compare, and `prepare_solve`. The same owner serves horizon samples and the
interactive value route ([F01](#f01)).

**Preserve.**

- Per-point admission.
- The independent PS-10 assessment of each new numerical state.
- Exact revision and context identity.
- Protection never outlives the engagement, and inactive studies remain reclaimable.

**Evidence.** Interface-checked: the reviewer read `prepare_study_candidate` and
`prepare_bound_operation`. Remedy: Proposed. Sources: D-04, A-07; PE04 residual.

#### <a id="f07"></a>F07 — Two reuse mechanisms, neither giving early cutoff inside the closure; the documented backdating does not occur

**Cause.**

- `pse-runtime/src/math.rs::MathService::workspace` constructs a fresh `CompilerWorkspace`, with a
  new `salsa::Storage`, on every call. The reviewer counted 24 non-test construction sites in
  `pse-runtime`.
- No production path publishes a second revision into an existing workspace. Each
  `publish_modeling_revision` site runs in a fresh context.
- Salsa's rebuild, trim, LRU and request-bucket machinery, including the hand-rolled
  `modeling_request_in_bucket` dedupe (L1-06), therefore serves a long-lived lifetime that never
  occurs.
- Cross-call reuse lives in `math/retention.rs::ModelingCache`, keyed on complete
  `SelectedDependencies`. Any in-closure change misses everything between selection and bodies.

Blueprint §14.4 states that "unchanged results backdate so unrelated edits do not propagate", and
its reuse table lists "case plan, structure" as reused on value changes. Both hold only for study
bindings. For authored edits, "reused" means "recomputed, then found equal".

**Consequence.** On S02 and S09, structural edits cannot become proportional to the change under
either mechanism. Two overlapping reuse owners violate DP-09, and Salsa machinery without reuse
benefit is carried (H27). The architecture text over-claims ([§6](#6-architectural-foundations-gates-and-numerical-stages-slots-5-and-6), G7).

**Principles.** DP-09; DP-16; H4; H13; H14; H27; AP-07.

**Corrective contract.** This is a design decision.

- (a) Make Salsa the owner. Use a long-lived single-writer database per (problem, physical context,
  provider set), with per-declaration inputs, tracked check, specialization and plan, and tracked
  membership and name lookups, so that absence is a real dependency. Then retire the `Frontier` and
  `Basis` caches (probably `Selected` too) and the bespoke flights for pure work.
- (b) Drop Salsa, keep plain functions and keyed caches, and correct §14.4.

[F01](#f01) does not depend on this choice, because value edits bypass structure either way.

**Preserve.** Handles stay local. No effects inside tracked queries. Attempt-owned state stays
outside. Cancellation is kept. Interned slot reuse must not leak IDs across revisions.

**Uncertainty.** Option (a) needs a decomposed checker; the EI04 domains of absence, duplicates and
cycles are exactly what tracked membership must cover. Which option wins overall is unmeasured.

**Evidence.** Interface-checked: the reviewer counted construction sites and publication call sites.
Remedy: a Proposed decision. Sources: A-05, L1-06; US05 residual.

### 4.2 Coordination and shared capacity

#### <a id="f08"></a>F08 — One per-problem guard, one exclusive local gate and one study-wide record serialize every lane of a study

**Cause.** Four mechanisms combine.

- **Local pacing.** `pse-operations/src/canonical.rs::ProblemTurns::run` gives every
  `QueryEffect::Mutation` the per-problem exclusive `RwLock` write guard. The retry loop of
  `protected_query` (up to `GUARDED_ATTEMPTS` = 32 with `conflict_backoff` sleeps) runs inside that
  gate.
- **The retention guard.** Every protected transaction begins with `PROTECTED_BEGIN`, which reads
  `canonical_guards:retention:<problem>` `FOR UPDATE`. The same record is bumped by:
  - run begin, claim and seal (`fn::pse_execution_v1::touch`);
  - protection acquisition and release;
  - every result read: `canonical_results.rs::read_results` and `ResultRead::renew` are both
    Mutations that `UPSERT … generation+1`, which the reviewer verified;
  - stage and edit activation;
  - product admission.
- **The study-wide record.** The generated `fn::pse_execution_v1::writer` calls `study_cancelled`,
  which `touch`es `study:<id>` on every append, renew, close, seal and seed of a study point's run.
- **Fence and problem.** `canonical_execution.rs::protected_execution_query` reads the run's
  immutable problem by RPC on every mutation, because `AttemptFence` does not carry it. All of a
  study's point runs share the study's problem.

**Consequence.**

- On S04 and S03, all lanes' protects and releases, claims, heartbeats, appends (about 22 or more
  per point), closes, seals and observes proceed single file through one in-process gate, whatever
  the lane count.
- Across processes, each guard bump aborts every concurrent protected read and mutation of that
  problem (optimistic concurrency, `FOR UPDATE` per SB091). The aborted work retries with sleeps while
  holding the gate.
- A Python reader opening results invalidates worker transactions.
- An interactive re-solve of the same flowsheet queues behind sweep publication, with no priority
  between the two classes.

Plan 28e recorded a 1,000-point managed sweep that failed on a result-ingestion conflict. The
correction serialized the work; it did not narrow the conflict domain.

**Principles.** AP-07; DP-20; H21, H22, H23, H24; Parallel F01 residual.

**Corrective contract.** This must be one protocol design, not a set of piecemeal edits:

1. **Cancellation.** Register the conflict on the rare side: writers read the cancel record
   `FOR UPDATE` without writing it, and cancel writes it. Alternatively, cancellation marks the
   bounded set of in-flight runs, and each writer checks its own run.
2. **Retention.** Split the guard by the conflicts it must create:
   - root creation and lifecycle transitions write a key only the reclaimer locks;
   - the reclaimer writes a key that readers lock;
   - renewal of an existing pin conflicts through the pin row the reclaimer must lock.
3. **Local pacing.** Key pacing by the guard domains a transaction actually writes: run and attempt
   for execution writes, and the problem only where `retention:<problem>` is written.
4. **Fence.** Carry `problem` in `AttemptFence`.
5. **Retries.** Back off outside the gate.

**Preserve.**

- Cancellation wins against late appends.
- Reclamation never deletes rows visible to a live pin.
- Phantom protection between root creation and the reclaimer's range scan. SurrealDB 3.3 has no
  predicate locks and no shared lock mode; `FOR UPDATE` covers record ids only.
- Named-absence premises (US04).
- Uncertain-response settlement.
- Server guards stay authoritative, and local pacing stays an optimization.

**Uncertainty and settling evidence.**

- Probe D-P1: do two `FOR UPDATE` readers of one record conflict with each other on RocksDB 3.3?
  The answer selects the variant for item 1. It does not change the diagnosis, because writers
  already serialize locally.
- Probe L2-P1 counts retries.
- Probe D-P3 measures gate wait relative to solve time.

**Evidence.** Diagnosis: Interface-checked (the reviewer read the generated `writer`,
`study_cancelled` and `touch`, `PROTECTED_BEGIN`, `ProblemTurns` and the retry placement, and the
guard bump in `read_results`/`renew`). The failure history comes from the Plan 28e record and was
not re-run. Remedy: Proposed. Sources: L2-02, D-02, E-02 and the E-03 fence part.

#### <a id="f09"></a>F09 — Durable discovery revisits every blocked candidate and rounds behave as barriers

**Cause.**

- `worker.rs::work_once_scoped` restarts at the first study and at ordinal −1 on every call.
- `canonical_studies.rs::study_candidates` excludes only settled or assigned points; the composite
  index is used (V2). Points waiting for predecessors are therefore returned.
- Each candidate costs one `study_scope` RPC, then `ActionKind::Wait(_) | Reconcile => continue`.
- `Runtime::serve` latches `idle` and stops refilling lanes until every lane in the round drains.
- The finalization probe `touch`es the study guard even when the study cannot finalize.
- `finalize_canonical_study` re-reads each point (D-09).

**Consequence.**

- On S05, a continuation chain of N points served by w lanes makes about (w−1)·N²/2 scope reads
  over the study. For 16 lanes and 1,000 points that is on the order of 10⁷; this is source
  arithmetic, not a timing claim.
- Chains with skewed solve times wait each round for the slowest one.
- Finalization probes contend with publication ([F08](#f08)).

**Principles.** AP-07; DP-20; H5, H9, H10, H22.

**Corrective contract.**

- The settle and observe server functions maintain readiness on dependents: a blocked-predecessor
  count, or a ready flag in the candidate index. Discovery then examines O(ready).
- Lane affinity for a single-successor continuation, with session retention and in-memory seed
  hand-off, as the ephemeral executor already does.
- Continuous refill instead of the idle latch.
- A pending-count read before any guarded finalization.
- Finalization pages project the fields they need.

**Preserve.** The claim still fences the point and its predecessors. The pure `study_policy` stays
the authority: a ready flag is a hint that the claim re-establishes. Retries, cancellation, refusal
settlement and ordinal result placement are kept.

**Evidence.** Interface-checked: the reviewer read the discovery loop, the `serve` latch and the
candidate index. Magnitude: arithmetic (probe D-P2). Remedy: Proposed. Sources: D-01, D-09.

#### <a id="f10"></a>F10 — Shared capacity is charged and coordinated at grains that do not match demand

**Cause.** Five manifestations, each with its own closure obligation:

- **(a) Lanes.** `Runtime::serve` lanes hold their slot through discovery, RPC-bound preparation,
  claim, solve and publication. Native job permits use `try_acquire_owned` (`math/jobs.rs`,
  `math/staged.rs`) and refuse instead of queueing. Width is therefore statically at most `jobs/2`.
- **(b) Fits with profiles.** `workflow/run.rs` charges `threads × profile_workers` CPU permits and
  `bytes × workers` for the entire fit (`fitting.rs::profile_workers`), although only the profile
  phase uses that width. The fair semaphore then blocks smaller requests queued behind it.
- **(c) Upper-bound reservations.** `MathService::workspace` reserves 4 GiB per fresh workspace.
  `math/artifacts.rs` reserves `worker_bytes` (8 GiB) for every non-flat body, because
  `execution.rs::selected_compilation_allocation_bound` returns `None` when `!is_flat_arithmetic()`.
  A failed `try_grow` raises a pressure demand that trims retention.
- **(d) faer parallelism.** `pse-math/src/implicit.rs::Problem::new` uses
  `faer::get_global_parallelism()`, the global rayon pool sized to logical CPUs, outside admission.
  Every other faer site passes an admitted `Par`.
- **(e) Accounting.** `result_export.rs` registers a `MemoryConsumer` per result row
  (`"result:row-copy"`). `pse-engine/src/resources.rs::ReleaseNotifyingPool` calls `notify_waiters`
  on every departure, waking every admission waiter to retry and possibly trim.

**Consequence.** On S04 and S07:

- native capacity sits idle during I/O;
- new work is refused instead of paced;
- fit studies serialize;
- phantom maximum reservations evict useful retention, which makes the next preparation colder;
- inner factorizations can oversubscribe beyond admitted permits;
- wake storms occur exactly when memory is scarce.

**Principles.** AP-07; DP-20; H11, H17, H21, H26; Parallel F02 residual.

**Corrective contract.** Admission at the grain of actual demand:

- **Lanes:** separate stage admission. Preparation and publication are bounded by memory and
  connections. Native entry waits on job and CPU semaphores under the original deadline and
  cancellation.
- **Fits:** admit at estimation width and grow for profiles, or run profile chains as their own jobs.
- **Reservations:** construction bounds for guarded and provider bodies, and incremental workspace
  reservation. A reservation that exists only as a cap does not raise pressure.
- **faer:** `Par` comes from the admitted scope.
- **Accounting:** chunk-grain consumers, with notification only on actual release.

**Preserve.** Finite admission; honest refusal of genuinely oversized work; drain on stop; stack and
foreign-allowance reservations; deterministic results; exhaustion diagnostics.

**Ordering.** The benefit of (a) appears only once [F08](#f08) narrows serialization.

**Uncertainty.** For (d), materiality depends on faer's supernodal path (probes D-P4 and L1-P1).
For (e), contention is unmeasured (probe F-P2).

**Evidence.** Interface-checked: the reviewer read `serve`, the permit sites, fit charging, the
allocation-bound fallback, `get_global_parallelism` and the per-row consumer. Remedy: Proposed.
Sources: D-03, D-06, D-08, B-10, L1-02, F-03.

### 4.3 Publication, results and history

#### <a id="f11"></a>F11 — A derived per-field scalar-cell index sets the physical publication unit; empty relations each cost a transaction

**Cause.**

- `workflow/result_projection.rs::scalar_cells_at` emits one `canonical_result_cells` record per
  declared scalar field per row: ten per `solve_variables` row and eight per `solve_constraints`
  row, among others.
- `store_result_chunk` limits rows per block with `metadata_prefix` against `EXECUTION_METADATA_BYTES`
  (`RESULT_INDEX_BYTES` = 128 KiB). The index metadata therefore sets block size, not the 512 KiB
  payload bound.
- Five indexes cover `canonical_result_cells`.
- Each block is one append (two RPCs and a guarded server transaction) plus a permanent receipt
  ([F15](#f15)).
- An empty relation still encodes and appends a schema-only block: the `num_rows() == 0` path, while
  `store_result_table` refuses ordinal 0. That cost is paid for all 22 relations of a modeling run.
- Trajectory blocks are cut at least once per symbol group.

The cells' only consumer is `connected_results.rs::Runtime::output_results`, exposed as Python
`Runtime.output_results`. It selects one (relation, owner, field, partition), which names at most
one row under the relation key, then fetches and decodes the whole block and re-derives the cell.

**Consequence.**

- Every durable solve writes about 10n + 8m cell records, each maintained by five indexes. It makes
  roughly 2·(n/r + m/r′) sequential append RPCs, where r and r′ are tens of rows by source estimate
  (unmeasured; probe E-P1), plus 22 or more fixed appends.
- Reclamation deletes 64 cells per RPC.
- Reads pay one RPC, schema decode, registry admission and FFI batch per small block.
- S01 waits on all of it ([F12](#f12)); S03 multiplies it by points; S08 retains it.

**Principles.** AP-07; H2 (a semantic field set dictates the transaction and frame unit); H6; H8;
H16; H27; PE02 residual.

**Corrective contract.**

- Store result relations in primary-key order and let blocks fill to the payload bound.
- Carry per-block key bounds in the existing descriptor fields (`coordinate_min`/`coordinate_max`),
  or at most one locator per row.
- Evaluate the scalar selection with Arrow `filter`/`take` on the decoded candidate block.
- Declare empty relations in the close transaction; the manifest still covers every advertised
  relation.
- Append small relations together within the 4 MiB message bound.
- Group many symbols per trajectory block.

**Preserve.**

- Exact IEEE bits, signed zero, and missing versus finite stay in the original rows.
- Readers verify selection against decoded rows.
- Selection never crosses attempts.
- The 512 KiB bound and preflight are kept.
- A crash before close never makes a partial relation look empty.

**Interaction.** This changes Plan 33 EI02's selected-scalar verification, which is kept in
substance (selection verified on decoded rows), and §19.2's derived scalar cells ([RC08](#rc08)).
A future cross-output numeric predicate is better served by Arrow or DataFusion over decoded blocks
([F24](#f24), alternative 1) than by an entity-attribute-value index.

**Evidence.** Interface-checked: the reviewer read `scalar_cells_at`, `metadata_prefix`, the
empty-block path, the index definitions and the consumer search. Remedy: Proposed. Sources: E-01, E-04.

#### <a id="f12"></a>F12 — Durable completion re-proves facts the writer holds, and stored-result handles re-read the whole record on every access

**Cause.**

- `workflow/durable.rs::Operations::settle` calls `reconcile_closed_attempt`, which pages back every
  batch's key, digest and row count per result set, although the writer holds every compact append
  acknowledgment.
- `settle` then ends with `self.record(...)`:
  - `read_results`: four RPCs, including `protect` and a guard-bumping transaction;
  - the completion chunks;
  - a seed page loop that renews on every page.
- `closed_response` issues `closure_operation` even after a successful close.
- `pse-py/src/workflow.rs::NativeStoredResult::{usable, completion, diagnostics}` each call
  `Runtime::stored_completion` → `Operations::record`. Every property access is therefore a full
  reopen, including the seed inventory.

**Consequence.**

- On the default durable runtime, the S01 waiter returns only after this re-proof, which adds guard
  bumps ([F08](#f08)) and keeps a second deserialized completion copy live.
- Scanning `result(i).usable` and `.completion` across a 1,000-point study costs about 2,000 complete
  reopenings and about 10⁴ retention-guard bumps.

**Principles.** AP-07; H1; H6; H12; H16; H19.

**Corrective contract.**

- Build the durable record from writer-held acknowledged bytes: the digest-acknowledged appends and
  the seal acknowledgment.
- Read back only after an uncertain acknowledgment, or during recovery.
- Compute the reconcile set from the acknowledgments, checked server-side against
  `canonical_result_sets` counters in one transaction.
- Handles hold the immutable acknowledged completion after their first read. Alternatively,
  `stored_completion` reads only the attempt and its completion; a terminal study page is read in
  bulk.
- Option: return the in-memory scientific result before durability completes, exposing durability as
  its own state. This is conditional on the seed dependency: cross-run warm starts read stored seeds
  (`run.rs::with_stored_start` → `latest_seed`).

**Preserve.** A truthful terminal class. Manifest and seal fencing. Lost-acknowledgment settlement
through receipts. Recovery reconciling from server state. Retired or reclaimed runs still refuse.

**Evidence.** Interface-checked: the reviewer read the `settle` tail, `stored_completion` and the
Python handle methods. Remedy: Proposed. Sources: E-03, E-05.

#### <a id="f13"></a>F13 — Result detail and retention are fixed at full single-solve detail regardless of consumer demand

**Cause.**

- `DurableAttempt::store_tables` publishes every relation in `RunResult::table_ids()` for every
  study point, and the study contract has no output demand.
- `workflow/study_execution.rs::study_inner` retains every point's `Arc<RunResult>` until the study
  returns.
- `horizon/driver.rs::Loop::execute_solve` keeps every step's prepared clone and result.
- `pse-backend-native/src/dynamics/idas.rs::Session::sample` stores n×np state sensitivities at every
  sample. Consumers read output sensitivities (fitting and trajectory publication) or only window
  ends (shooting continuity, `shooting.rs`).

**Consequence.**

- S03 sweeps write millions of indexed cells for a few key values per point.
- Ephemeral studies hold O(N × full report) live, which the budget can predictably refuse (H11).
- S07 horizons grow their live state and final publication linearly with steps.
- Long trajectories with sensitivities hold samples × n × np floats that nothing consumes.

**Principles.** AP-07; DP-10; H6, H16, H17, H27.

**Corrective contract.**

- **Studies:** a study-level output demand: selected relations or members, a compact per-point
  summary, and full detail for failed or flagged points. Unselected ephemeral detail is released
  once the point is recorded.
- **Horizons:** stream per-step results, retaining only what later steps consume (the last accepted
  step per role, priors and seeds).
- **Integrators:** the sample demand reaches the integrator profile. State sensitivities are kept
  only at window ends or at requested samples; IDAS can interpolate the rest.

**Preserve.**

- Outcome facts, diagnostics, PS-10 evidence summaries, start provenance and result identity.
- Detail that is not retained is reproducible only within the declared determinism class, and is
  labeled as such.
- Shooting continuity derivatives; step identity and order.

**Evidence.** Interface-checked: the reviewer read the horizon pushes, the ephemeral retention and
the IDAS sample consumers. Remedy: Proposed. Sources: D-05, D-07, C-04.

#### <a id="f14"></a>F14 — Per-iteration telemetry is a guaranteed durable stream that back-pressures the solver and cancels it on overflow

**Cause.**

- `pse-backend-native/src/solve.rs::Progress::push` sends every event to the tap before applying the
  bounded local retention.
- Events include each Ipopt or POUNCE iteration (`tnlp.rs::Adapter::intermediate_callback` and the
  Uno/Ipopt bridge) and each failed trial (`callback.rs::CallbackState::evaluate`). Each is a
  `BTreeMap<String, Metric>`.
- `workflow/durable.rs::Tap::observe` uses `try_send`. When the channel is full, it blocks the native
  thread for up to the heartbeat interval. On failure it sets overflow and cancels;
  `Streamer::finish` then reports "durable progress admission failed; scientific run cancelled".
- Writes are mutations under [F08](#f08)'s gate.

**Consequence.**

- On S01 and S04, the solver's inner loop is coupled to store latency and per-problem serialization.
- Overflow of a diagnostic stream aborts a scientifically sound solve. The local history already
  accepts bounded partiality with drop counts.
- The rules conflict. §19.2 says diagnostic capture "cannot change a scientific outcome". §20.6 says
  progress failure "cancels the scientific run".

**Principles.** AP-07; H1 (completeness of an observation stream is enforced as if it were the
result); H23; H26.

**Corrective contract.** Two streams:

- a guaranteed small channel for phase, failure, terminal and seed-relevant events;
- per-iteration telemetry, sampled or coalesced at the tap (latest state plus dropped counts) and
  published at a bounded rate.

Overflow records progress partiality and does not cancel the run. Per-iteration keys are static.

**Preserve.** The terminal outcome record. The order of kept events. Cancellation responsiveness.
The claim fence before writes.

**Evidence.** Interface-checked: the reviewer read `Progress::push`, `Tap::observe` and the
cancellation message. Remedy: Proposed. Sources: C-07, H-04 (events).

#### <a id="f15"></a>F15 — Lifecycle residue grows with every operation and is never collected

**Cause.**

- Every append, claim, close, seal, seed, study start and study claim creates a permanent
  `canonical_execution_operations` record and a permanent `execution-operation:` guard.
- `fn::pse_retention_v1::collect_run` deletes cells, outputs, blocks, batches, sets and seeds. It does
  not delete receipts or guards; the reviewer verified this.
- The generated SurrealQL contains no `DELETE` of `canonical_protections`, so released protections
  remain as tombstones.
- RECLAIM's protection-interval range scan walks released history.

**Consequence.** On S08, receipts and guards grow by one pair per append. That makes them
proportional to [F11](#f11)'s block count. Protections grow by about 6–8 per study point,
indefinitely, and reclamation checks scan that history.

**Principles.** AP-07; DP-16; H25; H27.

**Corrective contract.**

- Collect append receipts and operation guards with the run's batches, once retirement refuses
  replay.
- Collect released or expired protections after the acquisition deadline has passed.
- Keep run-level lifecycle receipts.

**Preserve.** Lost-acknowledgment settlement while the run is live. Late acquisitions refused by
deadline. Run-level lineage.

**Evidence.** Interface-checked: the reviewer read `collect_run` and searched for protection
deletion. Magnitude: unmeasured (probe E-P4). Remedy: Proposed. Source: E-07.

### 4.4 Evaluation inner loop and implicit properties

#### <a id="f16"></a>F16 — Evaluation callbacks rebind through semantic-identity maps and materialize widest shapes on every call

**Cause.** On every residual, gradient, Jacobian and Hessian callback:

- **Value transport.** `pse-backend-native/src/assembled.rs::AlgebraicOracle::trial` inserts each
  coordinate into `CaseValues.scalars: BTreeMap<SemanticId, f64>`. Dynamics does the same in
  `workflow/dynamics.rs::DynamicWorker::assign`.
- **Gathering.** For each instance, `pse-math/src/assembly.rs::CaseWorker::evaluate` calls
  `binding.rs::InstanceBinding::values_selected`. That function allocates a `BTreeSet` to re-prove
  that the immutable formals contain no duplicates, does one `BTreeMap` lookup and conversion per
  formal, and collects a `Vec`. A bits `Vec` is then built for the memo.
- **Assembly.** `CaseWorker::constraints` looks up `self.assembly.rows[&id]` per contribution, and
  the Jacobian and Hessian paths binary-search the outputs per term.
- **Guard and applicability stages.** `execution.rs::evaluate_stages` copies every body slot into a
  local frame sized `vec![f64::NAN; parameters.len()]` for each guarded stage
  (`for (slot, value) in local.iter_mut().enumerate()`) before running a small nested program.
- **Frames and results.** `Worker::evaluate_seeded` fills the whole widest-order frame with NaN on
  every call. An `Evaluation` with dense both-triangle Hessians is allocated per call, and finiteness
  is re-scanned at several layers.
- **Applicability evidence.** `pse-model/src/applicability.rs::Node::observe` clones the claim, input
  names and filtered permissions into a complete `Observation` on every evaluation, although only the
  last is read.
- **Accounting.** `callback.rs::CallbackState::evaluate` allocates two `String` keys and takes the
  admission lock twice per callback.

All of the mappings these steps consult are fixed once the bound case exists.

**Consequence.** Identity and bookkeeping work per callback runs beside the arithmetic. It is roughly
O((n + Σ formals)·log N + nnz·log R + guarded stages × body slots + slots × W_max). It recurs on every
native route and on every right-hand-side call of long trajectories. It is plausibly dominant for
many small bodies (S06), multiplied by sweeps (S03), and recurs throughout dynamics (S07). For a few
large PC-SAFT bodies the arithmetic probably dominates. The share is unmeasured.

**Principles.** AP-07; DP-10 ("keep wide provenance out of inner loops"); H12; H15; H16; H19; H26;
EF08 moved from preparation into evaluation.

**Corrective contract.** At worker construction, compile an operation-shaped binding inside the
existing `CaseAssembly`/`CaseWorker` owner:

- a dense coordinate slice;
- a fixed and parameter vector, refilled per rebind;
- per-instance gather lists (dense index plus resolved affine conversion);
- per-term precomputed (cache slot, local offset, global row);
- guard and applicability stages that read only their compile-time read set;
- fills of only the requested-order prefix, reusable evaluation buffers, and packed Hessian access;
- one finiteness check per publication boundary;
- an applicability outcome code and a compact record per trial, with full `Observation`s hydrated
  only for the accepted point or a refusal;
- static accounting keys.

`CaseValues` remains the semantic boundary representation for rebind, the PS-10 original-case
evaluation and diagnostics, which hydrate by identity.

**Preserve.**

- Repeated sources stay aliases, and admitted conversions stay exact.
- A missing scalar is refused at bind time, and nonfinite trials are refused.
- Per-instance bit-exact memo semantics and instance-attributed errors.
- Applicability refusal at the exact offending trial, with attribution.
- Guards precede the algebra they protect (PS-06).
- The fresh, independent PS-10 worker.
- Atomic output publication and attempt-owned scratch.

**Uncertainty.** Probes B-P1, C-P1 and H-P5 measure the share. The diagnosis does not depend on them;
the priority does.

**Evidence.** Diagnosis: Interface-checked (the reviewer read `trial`, `values_selected`,
`CaseValues`, `evaluate`, `constraints`, both guard arms, the frame sizing, `observe` and
`DynamicWorker::assign`). Magnitude: unmeasured. Remedy: Proposed. Sources: B-01, C-01, B-02, B-03,
B-04, H-04.

#### <a id="f17"></a>F17 — Evaluator programs are re-optimized per order and deep-cloned per attempt

**Cause.**

- `execution.rs::compile_scope` compiles one program per order (Value, First, Second). Each program
  goes through `compile_stages` → `library/admission.rs::bounded_evaluator` →
  `library.rs::exact_evaluator`, so Horner and CPE optimization run over the same expressions up to
  three times per block.
- Constraint and objective demand groups are compiled to the plan's order, although only
  `Demand::All` is evaluated at Second.
- `CompiledBody::worker_scoped` does `programs: self.programs.as_ref().clone()`, copying all orders.
  It is called through `CaseAssembly::worker_scoped` for every attempt and step: multistart seeds,
  the new session per study point, re-solve oracles, and every conditional-unit evaluation
  ([F19](#f19)).
- Symbolica 3.0.1 `ExpressionEvaluator` clones carry one heap allocation per Add, Mul or ExternalFun
  instruction.

**Consequence.**

- Cold preparation and structural edits triple symbolic optimization per block.
- Each attempt copies every instruction stream, including Second-order programs on routes that never
  request Hessians.
- Sixteen concurrent cases hold sixteen charged copies, and a 1,000-point sweep copies them 1,000 times.

**Principles.** AP-07; H6; H12; H14; H17.

**Corrective contract.**

- Build the exact evaluator once per block and clone it before each order's `vectorize`.
- Compile each group only to the order it is evaluated at.
- Clone programs lazily per order on first use.
- Optionally reuse a reset scratch worker across sequential attempts on one lane, with charges
  transferred. That option depends on [RC13](#rc13).
- An upstream Symbolica split of the immutable program from the per-worker stack is the
  library-owned end state.

**Preserve.** Per-order admission and allowances. Artifact-key completeness. No mutable semantic
state across attempts. The fresh PS-10 worker. Charges ending at drain.

**Evidence.** Interface-checked: the reviewer read the per-order loop and the `programs.clone()`.
Remedy: Proposed. Probes B-P2 and L1-P4. Sources: B-07, B-08, L1-03.

#### <a id="f18"></a>F18 — The cubic accelerator runs exact rational root isolation per inner solve behind a cross-attempt mutex; implicit providers re-verify on memo hits

**Cause.** On every memo miss, `pse-math/src/implicit_cubic.rs::CubicRoots::solve`:

1. evaluates the four coefficients under `Mutex<ExpressionEvaluator<f64>>`;
2. converts each `f64` to a `Rational`;
3. builds a `UnivariatePolynomial` over Q and calls `isolate_real_roots()`;
4. refines every real root to 2⁻⁵⁴ in exact arithmetic;
5. verifies each refined root against the original residual.

`implicit_accelerators.rs::Cubic::admit` returns one `Arc<CubicRoots>`, held by the prepared
`Factory`. `implicit.rs` `create_scoped` clones that `Arc`, so every attempt on the prepared view
shares one mutable evaluator. That breaches the existing rule that "mutable evaluator and solver
state belongs to the attempt" (§14.3; DP-19).

Separately, `Nested::evaluate` repeats work even on a memo hit: the anchor, configuration
resolution, a BLAKE3 key, `problem.verify` and `selection.verify`. `Problem::derivatives` verifies
again, and one provider stage per partial calls the same provider at identical inputs.

**Consequence.**

- Peng–Robinson density sits in the property inner loop of flash, recycle and sweep workloads, so
  certified bignum isolation runs at every new outer point.
- In-process lanes and concurrent cases (S04) serialize on the shared mutex.
- Redundant verifications run per instance per callback.

**Principles.** AP-07; DP-19 (conformance); H9; H19; H20; H22.

**Corrective contract.**

- Give each attempt its own coefficient evaluator. This is a conformance fix with no rule change.
- Bracket in `f64` within the declared branch interval: critical points from the quadratic, sign
  tests on monotone segments, then safeguarded Newton or bisection. Fall back to the existing exact
  isolation when a rigorous rounding bound makes a sign test ambiguous.
  - Alternatively, use Symbolica `roots_hot_start` (Aberth) as candidates, with an exact fallback
    wherever completeness is in doubt.
- Treat a memo hit as already verified for its complete key.
- Derive the derivative residual check from the jet.
- Evaluate a provider once per distinct (spec, inputs) per body evaluation.

**Preserve.**

- "Exactly one original-residual-verified root in the declared branch interval".
- Refusal on ambiguity or degeneracy.
- Time limits and cancellation.
- The derivative contract and determinism.
- Complete memo keys.
- Independent verification of every new point.

**Uncertainty.** Call frequency, per-call cost and contention: probes B-P3 and L1-P3. Near-critical
behavior needs explicit controls.

**Evidence.** Interface-checked: the reviewer read `solve`, `admit` and the `Arc` clone in
`create_scoped`. Remedy: Proposed. Sources: B-05, L1-04, B-06.

### 4.5 Numerical stage grain and algorithm fit

#### <a id="f19"></a>F19 — Conditional units and initialization blocks each run a complete cold attempt

**Cause.**

- For every fixed-point evaluation and every unit,
  `workflow/strategies/conditional.rs::UnitWorker::evaluate` clones the whole case's values.
- A conditional unit then calls `math/initialization.rs::MathService::evaluate_conditional_unit`,
  which:
  - builds a new worker (cloning all programs, [F17](#f17));
  - clones the values into `AlgebraicOracle::new`;
  - hashes every value (`values.identity()`);
  - creates `execution::Retained::default()`, which means a new native session, KINSOL memory and
    KLU symbolic analysis;
  - solves with `warm: None`;
  - qualifies the result, commits and drops everything.
- `initialization.rs::stages` runs a full `attempt` per BTF block: rebind job, conditional
  preparation, task, session step, `admit_step`, worker and oracle, structural check, native solve,
  qualification and commit. It clones whole `CaseValues` several times per block.

**Consequence.**

- On S06 recycles, the work is K iterations × U conditional units × (worker instancing + O(V) clone
  and hash + native setup + a cold Newton solve). Near convergence each unit could finish in zero or
  one step.
- Initializing many small blocks is dominated by per-block orchestration.

**Principles.** AP-07; H2 (each semantic block gets its own physical lifecycle); H10; H12; PS-11
(factor and session reuse).

**Corrective contract.**

- Unit solver state lives with the `CausalMap` that already owns the units: one attempt-owned worker
  and oracle (`AlgebraicOracle::update_values` exists), one retained session (KINSOL
  `KINSetNoInitSetup`), a unit-local binding, and compatibility computed over the unit's inputs only.
- A warm start from the unit's last committed solution is a declared, recorded start, used only where
  uniqueness or branch screening permits it.
- Block sequences execute inside one admitted native operation: one original-case worker, the dense
  binding ([F16](#f16)), and per-block library root solves or direct evaluation of explicit 1×1
  assignments. Larger or failing blocks escalate to the full strategy step.

**Preserve.**

- The fixed-point map stays deterministic in the tear values: session and factor reuse is
  unconditional, and warm starts are screened.
- Typed refusal and teardown.
- Charges released on every path.
- `commit_block` qualification.
- PS-08 transactional block commits and restoration.
- Cancellation between blocks.
- PS-09: KINSOL owns the iteration.

**Evidence.** Interface-checked: the reviewer read `evaluate_conditional_unit` (fresh worker,
`Retained::default`, `warm: None`, identity) and the per-block loop. Remedy: Proposed. Probe C-P2.
Sources: C-02, C-06; solver-acceleration F08/F15/F17 residual.

#### <a id="f20"></a>F20 — Root parameter response is dense O(n³) and duplicates the sparse predictor's factorization

**Cause.**

- `math/solves.rs` densifies the sparse Jacobian (`faer::Mat::zeros(n, n)` and `(n, np)`) and calls
  `pse-backend-native/src/square_response.rs::response`. That function computes a dense SVD
  (`spectrum`) for rank, a dense partial-pivot LU and a dense backward error.
- Its workspace is O(n²); beyond the allowance it refuses with `Withheld::Memory`.
- `fresh_root_predictor` factors the same Jacobian with sparse faer LU
  (`square_response/actions.rs::SparseFactor`).
- `fitting/oracle.rs::response_rank` also uses the dense path.

**Consequence.** For larger sparse structures, a supported Root sensitivity becomes either serial
cubic work or an honest refusal (H11), and it repeats a factorization the predictor already has.

**Principles.** AP-07; DP-10; H5; H9; H11; H14.

**Corrective contract.**

- One sparse factorization serves the response, the predictor and multiple right-hand sides.
- Rank evidence comes from a sparse rank-revealing QR. SuiteSparse SPQR is available through
  `suitesparse_sys` feature `spqr` but is not enabled. Alternatively, rank evidence comes from the
  structural matching plus a sparse backward error.
- The dense SVD is retained as an explicit, size-bounded certification mode.

**Preserve.** §15.5.1's rank, cutoff, backward-error and neighborhood evidence, or its explicit
relabeling. Withholding independent of the base root. Physical back-mapping.

**Uncertainty.** SPQR rank is a tolerance on R, not singular values, so the evidence semantics change
([RC14](#rc14)). Request frequency is unestablished (probe L1-P2).

**Evidence.** Interface-checked: the reviewer read the densification and the SVD and LU calls.
Remedy: Proposed. Sources: C-03, L1-01.

#### <a id="f21"></a>F21 — Tear selection solves one global MILP without SCC decomposition

**Cause.** `pse-backend-native/src/tears.rs::compile` creates:

- one order variable per node, bounded to [0, n−1];
- one row `o_from − o_to − n·x_d ≤ −1` per connection, across the whole flowsheet.

`pse-structural/src/flowsheet.rs::FlowGraph::components` (`kosaraju_scc`) exists, but it is not used
to decompose the problem. Inter-SCC connections can never lie on a cycle.

**Consequence.** On larger flowsheets with several loops (S06), the big-M relaxation is weak and
branch-and-bound runs globally, so independent loops are solved jointly. A time limit then truncates
to "no usable incumbent". This happens once per structure.

**Principles.** AP-07; H5; H9; H11.

**Corrective contract.** Decompose by SCC, merging SCCs coupled by grouped decisions. Only intra-SCC
connections enter, with M = |SCC|. Trivial SCCs need no tear. Lazy cycle constraints are optional.
All of this stays within HiGHS and petgraph.

**Preserve.** Mandatory and forbidden policies; decision grouping; the independent witness DAG
check; recording of the native status.

**Evidence.** Interface-checked. Magnitude: Proposed (static probe C-P4). Low priority. Source: C-05.

#### <a id="f22"></a>F22 — Value-independent numerical-policy resolution is recomputed per prepared solve with quadratic scans

**Cause.** `math/solves.rs::MathService::prepare_solve` calls `pse_math::numerics::resolve` on every
preparation, including every sweep point and every edit, although none of its inputs is a case value.
Inside it:

- validation runs `targets.iter().any(...)` for each candidate declaration, although
  `target_access` (a `CheckedInventory`) was built just before;
- each target filters over all declarations.

**Consequence.** O(targets × declarations) per prepared solve, repeated identically across points
(S03).

**Principles.** AP-07; H9; H12; PE01 residual.

**Corrective contract.** Index declarations once, and memoize `ResolvedNumericalPolicy` with the
prepared view, keyed on the policy key, declaration identities, targets and physical context.

**Preserve.** Conflict detection, strict-nominal refusal, rule checks with physical preconditions,
provenance and caller order.

**Evidence.** Interface-checked: the reviewer read both loops. Low priority. Source: B-09.

### 4.6 Relational layer: assurance and machinery

#### <a id="f23"></a>F23 — Validation is re-proved at internal crossings, and the interactive runtime is built force-validated

**Cause.**

- **Validation owner identity.** `pse-engine/src/session/factory.rs::EngineFactory::validation_context`
  stores the context in a `OnceLock` on a `ModelAssembly` that `models` holds only weakly
  (`Weak<ModelAssembly>`). `workflow/mod.rs::Runtime::validation_context` keeps no strong holder.
  Each call therefore rebuilds the assembly — session-state builder, extension registry, inspection
  binding, function inventory — and returns a fresh context. The absence of a production strong
  holder rests on inquiry F's search; probe F-P1 confirms it.
- **Reuse test.** Reuse compares a per-instance `Arc<()>`
  (`LocalFieldAdmission::matches` → `same_native_owner`), so admitted validity never crosses a
  context boundary.
- **Collection finish.** `pse-relations/src/columnar/collection.rs::Collection::finish` rebuilds with
  `FieldCheckedBatch::from_parts` and drops the admission it just minted.
- **Success-path work.** `validate/prepared.rs::PreparedLocalContract::evaluate_selected` runs
  `validate_full()` on every column, including arrays from safe builders and the validating IPC
  reader. It materializes a location per row per field before any predicate fails.
- **IPC frames.** Result blocks run the bespoke `preflight_block` on the writer's own output and
  copy 3–4 times per crossing (L2-03).
- **Interactive build.** `justfile` `py-sync`/`py-sync-native` build the interactive extension with
  `--features force-validate`, so every Arrow construction is re-validated in the operator's
  runtime.

**Consequence.** Every relation built or admitted, including about 22 relation encodes per durable
run (and so about 22k per 1,000-point sweep, inferred from call sites), re-prepares unchanged
contracts and re-proves validity. Diagnostic work runs on the success path. The measured production
build differs from the runtime the operator uses.

**Principles.** AP-07; DP-03; DP-10; H12; H14; H19; H20; H26; EF02 residual in effect.

**Corrective contract.**

- One immutable validation owner per registry-implementation owner and factory implementation
  identity, held strongly by the factory.
- Admission is keyed by semantic implementation identity (function hash, settings, generation)
  instead of the instance.
- `Collection::finish` carries the admission.
- Predicates evaluate on visible slices, and locations are built only for failing rows.
- `validate_full` runs at raw ingress only.
- Decode with `StreamDecoder` over the stored `Bytes`, keeping only the semantic checks the decoder
  lacks.
- The interactive extension is built without `force-validate`; correctness runs use a separately
  built force-validated extension ([RC15](#rc15)).

**Preserve.** Contexts with differing function bindings or settings still re-evaluate. Exact
diagnostics (rule, path, row). Masked children are never evaluated. FFI structural safety.
ADR-0170's guarantee for correctness closures.

**Evidence.** Interface-checked: the reviewer read `model_assembly`, `validation_context`,
`Collection::finish`, the `validate_full` site and the justfile recipes. Remedy: Proposed.
Sources: F-01, F-02, F-06, L2-03.

#### <a id="f24"></a>F24 — Engine and file-cache machinery outlived its role; a documented inspection capability has no route

**Cause.**

- No production caller exists for any of these: `pse_engine::cache_service::{bind_state,
  bind_state_with_generation}`, `ListingTable`, `register_parquet`, `ParquetSource`,
  `pse_rules::invariants::run_invariants`, the SQL session entry points, DML commands,
  `schema_transform` and the inspection UDTFs. The reviewer confirmed this by search over `crates/`,
  excluding tests and `cache_service/` itself. The Delta route these served was removed in Plan 28.
- Production nevertheless builds `NativeCacheService` with envelopes that reserve pool capacity up
  front (`cache_service/envelope.rs::Envelope::new` `try_grow`; up to 50 MiB of metadata and 16 MiB
  of statistics under `policy.rs::for_memory`). DataFusion itself receives `unbound_config()`, so
  those envelopes can never fill.
- `ResourceBudget::hashing_may_use_pool` is a user-visible setting that nothing reads.
- The architecture overview lists "invariant checks, inspection" as DataFusion roles, and §14.2 says
  registry invariant queries "support explicit inspection". No Python or runtime route reaches them.

**Consequence.**

- Every DataFusion upgrade (PSE-S06), every `pse-runtime` build and every review carries the layer.
- Each runtime pool starts with about 66 MiB reserved for nothing.
- A capability claimed in the architecture does not exist for users (G7).
- The workload a relational engine would genuinely serve has no relational route: inspecting results
  across a study's points.

**Principles.** G7; G8 (DP-14, DP-16: wrapper machinery that rebuilds DataFusion's cache features,
kept after its stated reason lapsed); H26; H27.

**Corrective contract.** The principal does not choose between these; both conform.

1. **Commit.** A DataFusion `TableProvider` over canonical result blocks that pushes run, relation,
   partition and range filters into `result_block_page` and exposes relational inspection of
   studies and invariants through Python.
2. **Retire.** Remove the unreachable machinery under DP-16, keeping physical inventory, validation,
   the pool and containers, and narrow the documented roles.

Either way, delete the dead envelopes and the dead setting.

**Preserve.** Full library eligibility: a library feature may remain without a consumer. The defect
is bespoke wrapper machinery without an architectural role.

**Evidence.** Interface-checked: the reviewer ran the caller searches and read the envelope
reservation. Internal reachability of `round`, `traversal`, `cache/*` and `operation/*` was not
traced. Remedy: a Proposed decision. Sources: F-04, L2-01, L2-05.

### 4.7 Development loop

#### <a id="f25"></a>F25 — Test-resource coordination processes retained history on every fixture and every environment exit

**Cause.**

- `scripts/test_resources.py::resource_metadata` delegates to `host_admission.metadata`. That
  function takes a blocking exclusive `flock`, parses and validates the whole `allocations.json`,
  and rewrites it.
- Every fixture `register` and `drain` passes through it. The Rust side spawns Python for each
  (`pse_operations::testing::resource_bridge`), and the runner's `finalize` and per-resource
  `reclaim` use it too.
- `scripts/pse_env.py`'s `finally` block runs `test_resources.reclaim_reports()` on **every**
  top-level `pse-env` exit, including compile checks and `just --list`.
- An unguarded `ResourceError` there replaces the command's own status (125). The lock wait has no
  deadline (G-08).

Measured state on 2026-10-11, from `ls` and inquiry G's read-only census:

- the ledger is 1.71 MB and holds 1,971 owners, most of them retained or pinned;
- `invocations/` holds 639 files and has no pruning path.

Probe G-P1 measured 9.3 ms per parse-and-dump cycle, excluding validation, locking and writes, so a
fixture's five or more cycles hold the host-wide lock serially.

**Consequence.** On S10, fixture cost grows with retained history rather than live participants, and
it serializes host-wide. Each fixture pays:

- two Python interpreter starts (about 17 ms and 13 ms of imports, G-P2);
- two `systemctl` and `/proc` observations;
- at least five full rewrites.

Every unrelated command also rewrites the ledger at exit. A passing `cargo check` can report a
boundary failure, or hang, because of unrelated ledger state (AP-05).

**Principles.** AP-07; AP-05; DP-21; H8; H9; H25; H26; DP-13 (indexed concurrent records are a
library capability: stdlib SQLite in WAL mode). 10-09 F03 residual and moved.

**Corrective contract.**

- Live coordination is one small record per resource, created exclusively, updated under a
  per-record lock and looked up exactly. SQLite with indexed state, invocation and disposition is
  equivalent.
- Pinned and failure diagnostics move to cold storage.
- Rust writes registrations directly, or batches them per invocation.
- Disposal is batched.
- `reclaim_reports` runs from runners or an explicit sweep only.
- Invocation files are pruned after reconciliation.
- Cleanup errors are reported without changing the command's status, and the lock gets a bounded
  deadline.

**Preserve.** Exact test association; service-generation and namespace checks; drain proof before
eligibility; no-recreation; retention of pinned failures; the runner's terminal outcome required for
disposal.

**Evidence.** Diagnosis: Interface-checked. Census and per-cycle cost: Measured under the stated
conditions. Remedy: Proposed. Sources: G-01, G-08.

#### <a id="f26"></a>F26 — Every Runtime-constructing test provisions a canonical store, whatever its subject

**Cause.**

- `crates/pse-runtime/src/workflow/tests.rs::runtime_on` always calls `canonical_deployment()` →
  `pse_operations::testing::canonical_fixture_store()` before `.with_durability(Ephemeral)`.
- `Runtime::from_shared` eagerly constructs durable `Operations`.
- Each fixture creates a UUID database and installs the complete schema: 479 DEFINE statements,
  including 101 indexes and 42 functions. It also pays [F25](#f25)'s coordination, and registration
  can start the SurrealDB service as a hidden side effect.
- By inquiry G's count, about 231 call sites across 59 files reach these helpers. They cover math
  solves, strategies, modeling, fitting and data documents.
- Measured, from the maintainer's JUnit report under a 16-thread run on a shared host: a median
  1.31 s (range 1.09–4.05 s) of cold per-test setup before the store was consulted.

**Consequence.** On S10 and PSE-S05, policy and algorithm assertions cannot run without a supervised
server. They inherit fan-out failure and per-test DDL, network and Python work. The 2026-10-11
workspace-test review's F02 is confirmed and broader than that review stated: the coupling sits in
the `Runtime` constructor.

**Principles.** AP-06; AP-07; H12.

**Corrective contract.** Each option keeps the canonical-source rule.

- (a) The runner pre-provisions a pool of initialized fixture databases in one batched operation,
  with one compatibility preflight before fan-out (10-11 F01).
- (b) Durable `Operations` are constructed lazily, or bound at `with_durability`.
- (c) Local owners are exercised directly (10-11 F02).

**Preserve.** Per-test isolation where store state is asserted. Late interpretation checks.
Scientific source admission for ephemeral runs. Drain and retention ownership.

**Uncertainty.** How many tests genuinely consume canonical semantics (probe G-P5).

**Evidence.** Diagnosis: Interface-checked (the reviewer read `runtime_on` and
`canonical_deployment`). Setup time: Measured as stated. Remedy: Proposed. Sources: G-02; 10-11
F01–F03 confirmed.

#### <a id="f27"></a>F27 — Entry routes and dependency placement compile far more than the responsibility under test

**Cause.**

- **py-test.** `just py-test` builds `xtask` with the default `package-fixtures` feature. That is
  818 units, including Symbolica, DataFusion, `aws-lc-sys` and egobox, force-validated. The build
  exists only so `xtask/src/python_tests.rs::run` can spawn `.venv/bin/python -m
  scripts.python_tests`, which `py-unit` already calls directly.
- **codegen.** `just codegen` step 3 runs under `--native=compiler,solver` with `package-fixtures`.
  Every regeneration therefore builds the whole runtime and pays [F05](#f05)(d), even for registry
  edits unrelated to Ipopt bindgen.
- **pse-math.** `pse-math/Cargo.toml` takes egobox and argmin as normal dependencies, used only by
  `pse_math::surrogate`, whose sole consumer is `pse-runtime`. Under workspace unification (ADR-0122)
  a feature gate cannot narrow this. Inquiry G measured that pruning removes 78 of `pse-math`'s
  262 crates and a second nalgebra/ndarray major.
- **workspace-hack.** The workspace-hack adds 35–39 crates to small roots such as `pse-codegen`,
  `pse-schema` and `pse-columnar`, on top of the mandated feature unification.

**Consequence.** On S10, cold, worktree and dependency-bump builds, and the critical path of pure and
generator roots, wait on unrelated crates. A narrow registry change pays for a full-product native
build.

**Principles.** AP-06; AP-07; H5; H27; DP-16; EF04 residual.

**Corrective contract.**

- `py-test` calls the Python module directly after `--store`.
- The native solver step of codegen runs only when the headers or the bindgen generator change.
- The surrogate code moves into `pse-runtime` (no rule change), or into its own crate (ADR).
- The authoring driver and physical inventory move below `pse-runtime` (ADR).
- Hakari is retired or its membership narrowed.

**Preserve.** Regeneration from one authoritative declaration; production-path physical admission;
byte-preserving writes; `codegen-check` coverage; `family-check`.

**Evidence.** Interface-checked: the reviewer read the recipes, `python_tests.rs` and
`pse-math/Cargo.toml`. Unit and crate counts come from inquiry G's read-only `cargo` resolution.
Remedy: Proposed. Sources: G-03, G-06, L1-07.

#### <a id="f28"></a>F28 — Host admission partitions CPU statically, with hard quotas and command-shape classification

**Cause.**

- `.config/agent-capacity.toml` gives functional work cores 8–15 and reserves cores 0–7 for timing,
  even when no timing work is admitted.
- `pse_env.py` sets `CPUQuota=len(cores)·100%`, which is 400% for a four-core slot, while Cargo runs
  eight jobs and nextest runs sixteen test threads.
- Acquisition fails after 30 s instead of queueing.
- `host_admission.classify` inspects the first words of the command. Every `bash -c` recipe line is
  therefore classed `functional`, a 40 GiB slot, and the functional lane allows two slots host-wide.

**Consequence.** On S10, an edit-compile loop uses at most four cores of CPU time on an otherwise idle
16-core host. A third concurrent agent's trivial recipe line fails after 30 s. Throttling stretches
sub-second test controls.

**Principles.** AP-07; DP-20 (bounded but not work-conserving); H21; H25.

**Corrective contract.**

- Admit by memory budget, and share CPU by weight.
- Reserve timing cores only while a timing allocation is live.
- Declare a class per recipe, and queue fairly.
- Derive nextest threads from the admitted width.

**Preserve.** Per-command memory caps; aggregate and non-agent headroom; timing isolation during
measurement; exclusive full campaigns; drain before release.

**Uncertainty.** The maintainer may reserve cores 0–7 for other repositories' workloads (probe G-P7).

**Evidence.** Interface-checked, plus inquiry G's executed read-only classification. Low priority.
Source: G-04.

### 4.8 Behavioral hazard

#### <a id="f29"></a>F29 — Exact parsing of JSON floats depends on a feature that only a solver crate enables

**Cause.**

- `serde_json`'s `float_roundtrip` is enabled only transitively, by `clarabel 0.11.1` (inquiry L2's
  `cargo tree -e features`).
- `[workspace.dependencies] serde_json = "=1.0.151"` declares no features.
- Numerical policy `f64` fields decoded from Python-sent JSON (`pse-py/src/documents.rs`) enter
  request identities.

**Consequence.** If the solver dependency set changes, identities and tolerances could change by one
ULP, silently. This is a correctness hazard, not an efficiency one: exact parsing is a relied-upon
fact that nothing declares.

**Principles.** DP-15; DP-01 (prime directive 3: one authoritative declaration); G2 hazard.

**Corrective contract.** Declare `float_roundtrip` once, on the workspace dependency.

**Evidence.** Interface-checked: inquiry L2's feature tree, and the reviewer confirmed the declaration
in `Cargo.toml`. Remedy: Proposed, a single-line change. Source: L2-07.

## 5. Finding index

Priority is by consequence: High means a target workload is materially unfit; Medium is material
amplification; Low is a bounded or rare cost or a hazard. The diagnosis is Interface-checked in every
row, with measured magnitudes named where they exist. Remedies are Proposed.

| ID | Finding | Cause | Scenarios | Principles | Priority | Supporting inquiries |
|---|---|---|---|---|---|---|
| [F01](#f01) | No value route for authored value edits | Preparation lifetime | S01 | AP-07, PS-11, DP-09/10 | High | A |
| [F02](#f02) | Package-proportional edit ingress; reload loses lineage | Preparation lifetime | S01, S02, S09 | AP-07, H5/H12 | Medium | A, L2 |
| [F03](#f03) | Double preparation; per-call qualification and settlement | Preparation lifetime | S01, S02 | AP-07, H2/H12 | Medium | A |
| [F04](#f04) | Descriptions bound to the whole closure | Preparation lifetime | S01, S02, S08, S09 | AP-07, H13/H23 | Medium | A, H |
| [F05](#f05) | Immutable validity re-proved by full hashing (Measured per pass) | Validity carriage | S01, S03, S10 | AP-07, DP-03, H18–H20 | High | H, E, G |
| [F06](#f06) | Study and horizon points re-establish fixed preparation | Preparation lifetime | S03, S07 | AP-07, PS-11 | High | D, A |
| [F07](#f07) | Two reuse mechanisms without cutoff; §14.4 over-claims | Preparation lifetime | S02, S09 | DP-09, DP-16, G7 | Low (decision) | A, L1 |
| [F08](#f08) | One problem guard, gate and study record serialize lanes | Coordination | S03, S04 | AP-07, DP-20, H22/H23 | High | L2, D, E |
| [F09](#f09) | Discovery rescans blocked candidates; round barrier | Coordination | S05 | AP-07, H10/H22 | High (continuation) | D |
| [F10](#f10) | Capacity charged at mismatched grains | Coordination | S04, S07 | AP-07, DP-20, H21 | Medium | D, B, L1, F |
| [F11](#f11) | Per-field cell index sets publication unit; empty relations | Publication | All durable; S03, S08 | AP-07, H2/H6/H27 | High | E |
| [F12](#f12) | Durable finish re-proof; handle re-reads | Publication | S01, S03 | AP-07, H19 | Medium | E |
| [F13](#f13) | Result detail and retention not demand-shaped | Publication | S03, S07 | AP-07, H6/H17 | Medium | D, C |
| [F14](#f14) | Per-iteration telemetry is guaranteed and cancels on overflow | Publication | S01, S04 | AP-07, H1/H23 | Medium | C, H |
| [F15](#f15) | Lifecycle residue never collected | Publication | S08 | AP-07, H25/H27 | Low | E |
| [F16](#f16) | Callbacks bind through identity maps; widest shapes | Inner loop | S01, S03, S06, S07 | AP-07, DP-10, H15 | High (share unmeasured) | B, C, H |
| [F17](#f17) | Per-order re-optimization; per-attempt program clones | Inner loop | S02–S04 | AP-07, H12/H17 | Medium | B, L1 |
| [F18](#f18) | Exact cubic isolation behind a cross-attempt mutex; memo-hit re-verification | Inner loop | S01, S03, S04, S06 | AP-07, DP-19 | Medium | B, L1 |
| [F19](#f19) | Full cold attempt per conditional-unit evaluation and per block | Stage grain | S06 | AP-07, H2/H10 | Medium | C |
| [F20](#f20) | Dense Root response duplicates the sparse factor | Stage grain | S07, larger structures | AP-07, H9/H11 | Medium | C, L1 |
| [F21](#f21) | Global tear MILP | Stage grain | S06 | AP-07, H9 | Low | C |
| [F22](#f22) | Per-solve numerical-policy resolution | Stage grain | S03 | AP-07, H12 | Low | B |
| [F23](#f23) | Validation re-proved at crossings; force-validated interactive build | Validity carriage | All; S10 | AP-07, DP-03, H19/H20 | Medium | F, L2 |
| [F24](#f24) | Machinery without role; unrouted inspection claim | Machinery | S09, S10 | G7, G8, DP-16 | Low | F, L2 |
| [F25](#f25) | Ledger history processed on every fixture and exit (Measured census) | Dev loop | S10 | AP-07, AP-05, H25 | High (dev) | G |
| [F26](#f26) | Per-test canonical store for every Runtime test (Measured setup) | Dev loop | S10 | AP-06, AP-07 | High (dev) | G |
| [F27](#f27) | Entry routes and dependency placement widen builds | Dev loop | S10 | AP-06, H5/H27 | Medium | G, L1 |
| [F28](#f28) | Static CPU partition and command-shape classes | Dev loop | S10 | AP-07, DP-20 | Low | G |
| [F29](#f29) | Exact float parsing depends on a transitive feature | Hazard | S01 boundary | DP-15, DP-01 | Low | L2 |

## 6. Architectural foundations, gates and numerical stages (slots 5 and 6)

### 6.1 Numerical stage columns (profile slot 5)

| Stage (owner) | Formulation policy | Derivative source and order | Scaling | Class and solver capability | Status → outcome | Tolerances and post-solve check | Execution fit |
|---|---|---|---|---|---|---|---|
| Compile (`pse-compiler`, `pse-math` execution) | Domain and branch guards kept as typed stages outside the Symbolica program; guards recorded before rewriting | Symbolica exact evaluator with HyperDual `vectorize`, orders ≤ 2, structural zero components | Resolved numerical policy (§16) | Class discovered per prepared solve (`discover_class`) | — | — | F07, F17, F22; F10(c) |
| Evaluate (`CaseWorker`, `AlgebraicOracle`) | Guards and applicability evaluated per call with typed refusal | Exact forward-mode jets | Callback normalization (`tnlp.rs`) | — | Nonfinite trial refused | — | F16 |
| Implicit providers (`pse-math/implicit*`) | Declared branch interval; selection proofs | Implicit-function derivatives via retained faer LU | — | Inner cubic (exact), KINSOL or affine | Refusal on ambiguity or degeneracy | Original-residual verification per root | F18, F10(d) |
| NLP/Root step (`pse-backend-native`) | POUNCE presolve with FBBT where facts allow | From the oracle | Normalization | Ipopt (`ReOptimizeTNLP`), POUNCE, KINSOL | Typed reports; full mapping not re-traced here | `quality::attach_nlp` on the unpresolved oracle; FERAL KKT inertia; independent original-model `Assessment` | F14, F17 |
| Recycle (`recycle.rs`, `tears.rs`) | Declared fixed-point map | — | — | KINSOL fixed point with Anderson; HiGHS tear MILP with petgraph witness | Native status recorded | Witness DAG check | F19, F21 |
| Dynamics (`dynamics/idas.rs`, diffsol) | Integration policy (§13) | Forward sensitivities | — | IDAS/KLU and diffsol | Not re-traced | Not re-traced | F13 (state sensitivities); KLU refactor (opportunity) |
| Response (`square_response`) | — | Dense SVD and LU | Physical back-mapping | Root parameter response | `Withheld` verdicts | Rank, cutoff and backward-error evidence | F20 |

### 6.2 Foundation verdicts

| Foundation | Scenario and evidence | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | Authored physics, preparation, evaluation, native execution, workflow, persistence and boundary have distinct owners with explicit dependency direction (§3.1). Representative changes stay within owners: the result-representation substitution (S11), Salsa replacement (F07) and solver replacement (PSE-S02) | **Satisfied** (inspected scope) | Keep the owners. Remedies must not merge them |
| AP-02 Stable contracts | Consumers depend on typed contracts: prepared views, oracles, typed solve reports, closed manifests, Arrow streams. Library mechanics stay inside integration owners. The vendored SurrealDB patch is a justified library gap, but no pins hold records it (§7) | **Satisfied** | Record the hold (§7) |
| AP-03 Composition | Interactive and study workflows compose the same primitives (`declared_execution`, `prepare_bound_operation`, `study_policy`, the strategy owner). The defects of [F03](#f03) and [F19](#f19) lie in physical grain, not in entangled composition | **Satisfied** | — |
| AP-04 Domain model and authority | Bindings and overlays are typed and physically admitted; body identity is semantic; a single study policy authority; registry schema authority. Model adequacy holds for the inspected distinctions. Specification and implementation diverge in §14.4 ([F07](#f07)), a defect to reconcile that does not establish an inadequate model | **Satisfied** (inspected scope) | Reconcile §14.4 (RC02) |
| AP-05 Explicit structure and constraints | In product paths, failure outcomes and effects are explicit. In development tooling, a command's exit status and liveness depend on unrelated ledger state ([F25](#f25)), and a fixture constructor can start a service ([F26](#f26)) | **Violated (narrow: development environment)** | F25, F26 |
| AP-06 Local reasoning and testability | Runtime-level policy and algorithm tests require a supervised canonical store per test ([F26](#f26)). Entry routes and dependency placement compile the whole product for narrow work ([F27](#f27)) | **Violated** | F26, F27 |
| AP-07 Execution fits the workload | Established by static structure for every target workload: edit→re-solve (F01–F05, F11, F12), studies (F06, F09, F11, F13), concurrency (F08, F10, F18), recycles and initialization (F19, F21), dynamics and fitting (F13, F20), history (F15), development (F25–F28) | **Violated** | All High and Medium findings |

### 6.3 Gates

| Gate | Verdict | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | Pass (inspected) | No second writable authority found. Caches and retained products are keyed on semantic identity | — |
| G2 Semantic fidelity | Pass, with a latent hazard | Exact float parsing currently holds, but only through an undeclared transitive feature ([F29](#f29)) | F29 |
| G3 Validity | Pass (inspected) | Bounded admission before allocation; IPC preflight; physical admission of overlays | Preserve in F11, F16, F23 |
| G4 Hidden behavior | Pass (product); note | No undeclared effects found in product inspection paths. The development fixture constructor's service start is recorded under AP-05 and AP-06 | F26 |
| G5 Consistency and recovery | Pass (inspected) | Fenced ingestion, close, reconcile and seal; exact uncertain-outcome settlement; recovery without science rerun. Progress overflow cancellation is declared (§20.6), though misplaced ([F14](#f14)) | Preserve in F08, F11, F12 |
| G6 Transformation and reuse | Pass (inspected) | Reuse keys are complete or conservative (descriptions, validation identity, artifact keys). No unsound reuse found | Remedies must keep keys complete (F01, F04, F05, F23) |
| G7 Truthful capability claims | **Fail (narrow)** | Explicit inspection through registry invariant queries is claimed (architecture overview library table; §14.2) with no user-reachable route. The §14.4 reuse table and backdating text over-state the mechanism for authored edits ([F07](#f07)) | F24, F07 (RC02, RC16) |
| G8 Library leverage | **Fail (narrow)** | Numerical and solver leverage is strong (PS-09 conformance). The bespoke file-cache, store and IO wrapper machinery duplicates DataFusion `CacheManager` and `LimitStore` capability, and its stated reason lapsed with the Delta route ([F24](#f24)) | F24 |
| G9 Architectural fitness | **Fail** | AP-05 (narrow), AP-06 and AP-07 are violated | Findings above |
| PS-G1 Physical consistency | Unresolved (not re-assessed) | Not in this review's assessed scope. No contrary evidence on inspected paths; overlay admission and affine conversions are preserved by every remedy | Not required by this review |
| PS-G2 Well-posedness | Unresolved (not re-qualified) | The pre-solve structural witness is carried and revalidated per step (§3.2). Diagnostic naming was not re-examined | — |
| PS-G3 Numerical integrity | Unresolved (not re-qualified) | The independent original-space assessment exists and is a preservation constraint. The status → outcome mapping was not re-traced | — |

Architectural fitness and behavioral adequacy are judged separately. Passing tests and correct
outputs do not offset the G9 failure, and the G9 failure says nothing against the scientific results.

## 7. Library fit and utilization (slot 8)

The library picture is mostly strong. Problem classes have library owners that are used through
their native lifecycles. The remaining issues are capabilities left unused where they would remove
work, and bespoke machinery around a library that has lost its role.

| Library at pin | Integration owner | Use and fit | Unused capability that would remove work | Bespoke duplication | Recommendation |
|---|---|---|---|---|---|
| Symbolica/Numerica 3.0.1 | `pse-math` | Exact evaluators with Horner/CPE; HyperDual `vectorize` with structural zeros; bounded admission. Good fit | One exact build cloned per order; float `roots_hot_start` as a candidate generator | Exact rational isolation per inner call where float bracketing suffices with an exact fallback (F18) | F17 and F18. JIT/SIMD (`native_code_generation`, `ExpressionEvaluator<f64x4>`) is a measured future option, not a finding; it needs a rule change if pursued |
| faer 0.24.4 | `pse-math`, native, runtime | Symbolic LU retained with numeric refactor; fixed patterns refilled | Sparse response factor; explicit admitted `Par` everywhere | Dense SVD and LU over sparse systems (F20); global parallelism in implicit blocks (F10d) | F10(d), F20 |
| SUNDIALS (IDAS, KINSOL) | `dynamics/idas.rs`, `kinsol.rs` | Full surface: reinitialization, sensitivities, adjoint, Anderson acceleration, `KINSetNoInitSetup`. Strength | `IDAGetSens` interpolation in place of retaining sensitivities at every sample | — | F13 |
| SuiteSparse (klu, amd, colamd, btf) | `dynamics/linear.rs` | KLU with symbolic analysis retained | `klu_l_refactor` with an rcond guard; `spqr` for rank evidence | Full factor per linearisation (opportunity) | F20; KLU refactor is an opportunity |
| POUNCE, FERAL 0.18 | native, structural | Retained FERAL leases; presolve, DM and BTF; KKT inertia; sensitivity. Strength | `SparseLu::factor` with retained symbolic data for KINSOL blocks | Markowitz factor on every block setup (opportunity) | Opportunity |
| Ipopt (C++ bridge) | `ipopt/sequence.rs` | `ReOptimizeTNLP` warm structure. Strength | — | — | Keep |
| HiGHS, petgraph, rustworkx | `tears.rs`, `pse-structural` | Tear MILP with independent witness; SCC and topological order from libraries. No bespoke graph algorithms found in L1's search | SCC decomposition before the MILP | — | F21 |
| Salsa 0.28.4 | `pse-compiler` | Tracked queries, durability, LRU and interning, all inside a per-call database | Long-lived database (cutoff); interned LRU for requests | Hand-rolled request dedupe and full rebuild (L1-06) | F07 decision |
| egobox, argmin | `pse-math::surrogate` | Function fits | — | — | Move (F27) |
| SurrealDB 3.3.0 (vendored and patched) | `pse-operations` | Server functions; one-RPC transactions; bulk insert; typed codecs. Strong native use | Record-id `FOR UPDATE` on the rare side; key-partitioned guards (there is no shared lock mode); LIVE SELECT (rare path) | About 2.8k-line request-lifecycle patch; justified by a real library gap, but no pins hold records it | F08. Record a hold with a revisit trigger (upstream request contexts) |
| Arrow 59.3 | `pse-columnar`, `pse-relations`, results | `RowConverter`, `take`, filter, concat, casts; C stream export. Strength | `StreamDecoder` over `Buffer::from(Bytes)` (zero-copy decode); `filter`/`take` for cell selection | Bespoke IPC preflight on both encode and decode (F23); a per-field cell index (F11) | F11, F23 |
| DataFusion 55.1 | `pse-engine`, `pse-rules`, `physical.rs` | Physical-inventory admission; expression validation; memory pool | `TableProvider` pushdown over result blocks for study inspection; `with_sort_order` for sorted sources (cold path) | File-cache envelopes, store generation bindings and `SharedIo` with no consumer (F24); per-call assembly rebuild (F23) | F23, F24 |
| pyo3 / pyo3-arrow | `pse-py` | Zero-copy streams; `detach` on native calls. Strength | — | Outer observation with the GIL held (F05b) | F05 |
| blake3, sha2 | `pse-ids`, `pse-buildinfo` | Framed semantic identity (strength); streamed SHA-256 | `update_mmap_rayon` (secondary to removing repetition) | Dead `hashing_may_use_pool` setting (F24) | F05, F24 |
| serde_json 1.0.151 | many | Documents and receipts; seeds encoded as bits (strength) | — | `float_roundtrip` undeclared (F29) | F29 |
| quick_cache vs `Flights` | `math/retention` | `Flights` carries native cancel and completion ownership, which the library's guard API cannot express. Adequate | — | — | Keep |

**Composed execution.** The numerical stack keeps PS-09's division correctly. Libraries own
iteration, globalization, factorization and native model management, and the simulator owns
scientific families and bounded composition. No finding recommends re-implementing solver machinery.
Several remedies use more of the adopted libraries: SPQR, `klu_l_refactor`, admitted faer `Par`,
`IDAGetSens` interpolation, Arrow `StreamDecoder`, record-id `FOR UPDATE` on the rare side, and
DataFusion pushdown if F24's first option is chosen. FeOS stays a reference oracle. Inquiry L1's
note that symbolic size grows superquadratically with component count is a measured-scaling question
for the knowledge-boundary decision (probe L1-P5), not a finding.

## 8. Alternatives, remedy compatibility and order

### 8.1 Alternatives (slot 9)

| Alternative | Scenarios served and change locality | Contracts, composition, test isolation | Meaning or machinery carried | Tradeoffs | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | All supported, at amplified cost | Good semantic contracts; Runtime tests coupled to the store | Two reuse mechanisms; problem-wide coordination; index-shaped publication; per-callback map binding | Correct, but S01, S03, S04, S05 and S10 are unfit | Rejected (G9) |
| **Proposed design** | S01–S11 | A preparation engagement owner; one coordination protocol; demand-shaped publication; an operation-shaped evaluation layout; per-record development coordination | Adds one lifetime owner; removes caches, guards, cells, receipts and machinery | Removes the structural work in §4. Benefits are Proposed until measured | **Selected direction.** Revisit if probes show the engagement cannot hold protections across a lease, or the readiness flag cannot be maintained server-side |
| Library-owned: Salsa as preparation owner | S02, S09 (structural edits) | Tracked inputs per declaration; cancellation on write | Needs a decomposed checker (EI04) | Large change; true cutoff | Decision under F07; independent of F01 |
| Library-owned: DataFusion over result blocks | Study inspection and cross-output predicates | Gives F24 a real role | Provider and pushdown into `result_block_page` | Adds a user-facing capability | Choose if relational inspection is in scope; otherwise retire (F24) |
| Library-owned: Symbolica JIT/SIMD | Many identical bodies; long trajectories | Needs determinism qualification (FMA) and W^X ownership | A new evaluator profile | Unmeasured; a rule change (§14.5) | Not recommended now. Revisit after F16 and F17 are fixed and B-P1 is measured |
| Library-owned: SQLite for test coordination | S10 | Indexed per-record state | Stdlib | Small | An acceptable realization of F25 |
| Platform-owned: fs-verity for generations | F05(c)(d) | Kernel-enforced immutability with O(1) digest | Needs the ext4 `verity` feature | Unknown availability on the host | Revisit if stat identity is judged insufficient |
| Simplest viable | S01, S03, S10 | Drop Salsa (F07b); defer durability off the waiter's path (F12 option); delete dead machinery (F24); hash once per invocation (F05 if RC04/RC05 are kept) | Least new machinery | Leaves F08 and F11 unaddressed | Use as fallback steps where a rule is kept |

**Reference practice, read for behavior only.** Equation-oriented environments built on Pyomo keep a
built model resident and change fixed values through mutable parameters, which persistent solver
interfaces push to the solver without regenerating the model. That is the behavior [F01](#f01) asks
for. Sequential-modular simulators re-solve only affected blocks, reusing tear guesses; that is
[F19](#f19)'s behavior. Neither is evidence about this codebase's cost.

### 8.2 How the corrections fit together

**Shared contracts.** Four contracts carry most of the remedies.

1. **The preparation engagement** ([F01](#f01), [F03](#f03), [F04](#f04), [F06](#f06), and the
   [F05](#f05)(a) trigger). This owner holds one protection, the settled basis, view and descriptions
   for a (process, revision, selection) working lifetime, and drains on cancellation. Interactive
   value edits bind onto it ([F01](#f01)). Study points and horizon samples bind onto it
   ([F06](#f06)). Settlement happens once per engagement ([F04](#f04)iii), and receiving
   qualification happens once per engagement rather than per `prepare` call ([F05](#f05)a). It is
   the prerequisite of F01 and F06. It conflicts with §14.3's "each consumer … explicitly settles
   canonical effects" ([RC03](#rc03)).
2. **The coordination protocol** ([F08](#f08), [F09](#f09), [F10](#f10)a, [F12](#f12) fence).
   The three shapes the inquiries proposed — splitting guards by role, registering the conflict on
   the rare side, and pacing by the written guard domain with the problem carried in the fence — are
   compatible only as one design. Adopting them piecemeal could reopen phantom or write-skew
   exposure, because SurrealDB 3.3 offers record-id `FOR UPDATE` only. Probe D-P1 selects the
   cancellation variant. [F10](#f10)(a)'s lane decoupling has no benefit until this lands.
3. **Demand-shaped publication** ([F11](#f11), [F13](#f13), [F14](#f14), [F15](#f15), [F12](#f12)).
   Output demand ([F13](#f13)) reduces volume upstream of the unit change ([F11](#f11)). Both
   interact with Plan 33 EI02, which [F11](#f11) keeps in substance by verifying selection against
   decoded rows, and with [F24](#f24)'s first option, which makes the same move as F11's alternative.
   Receipt collection ([F15](#f15)) shrinks once there are fewer appends.
4. **Operation-shaped evaluation** ([F16](#f16), [F17](#f17), [F18](#f18), [F19](#f19)). All four
   touch `CaseWorker`/`AlgebraicOracle`, and one owner should take them together. The shared
   preservation constraint is PS-10 independence: the original-case worker stays fresh and
   semantically keyed. [F19](#f19)'s block-sequential execution depends on [F16](#f16)'s dense binding.

**Conflicting or rule-bounded remedies.**

- [F04](#f04)(ii) narrows premises. It is safe only if the body identity frames all meaning-affecting
  dependencies; until that is decided, keep complete premises encoded once.
- [F05](#f05)'s stat-identity checks change the documented meaning of "current admission" (ADR-0164)
  and Plan 35's executable-association rule ([RC04](#rc04), [RC05](#rc05)).
- [F12](#f12)'s early return conflicts with §20.6 and with cross-run seed reads.
- [F14](#f14) resolves a conflict between §19.2 and §20.6.
- [F17](#f17)'s worker reuse needs [RC13](#rc13). Lazy cloning does not.
- [F23](#f23)'s non-validated interactive build needs [RC15](#rc15) and must keep ADR-0170's
  correctness closures.
- [F26](#f26)'s options (a)–(c) keep the canonical-source rule.

**Priority versus prerequisite order.**

- **By consequence**, the most urgent are F01, F06, F08, F11, F05, F16 and F09 in the product, and
  F25 and F26 in development.
- **By prerequisite order**:
  1. Decide the preparation engagement contract (RC03, RC04).
  2. Decide the coordination protocol (RC06, after D-P1).
  3. The value route (F01), study engagement (F06) and settlement changes (F04, F05) then follow from
     the first decision.
  4. Lane decoupling (F10a), the fence (F12) and discovery readiness (F09) follow from the second.
  5. Publication demand (F13) precedes the publication unit (F11).
- **Independent**, and implementable at any time: F16–F18 (the inner loop), F19–F22 (numerical
  stages), F23 and F24, F25–F28 (development) and F29.

## 9. Verification and evidence limits (slot 10)

### 9.1 What the evidence establishes

| Claim | Label | Basis | Conditions | Result or gap |
|---|---|---|---|---|
| All diagnoses F01–F29 | Interface-checked | The principal re-read decisive source at the cited symbols; inquiries traced them independently | Baseline commit | Structural amplification established; magnitudes open |
| SHA-256 cost per pass | Measured | M1: `hashlib` over the dev worker and extension | Hot cache; `sha_ni`; light resource class | 0.30–0.39 s per 0.73–0.79 GB |
| Native generation verification | Measured | G-P3: `sha256sum` over the newest solver generation | Warm cache; nice 19 | 0.58–0.71 s per 1.2 GB (2,305 files) |
| Ledger cycle | Measured | G-P1: parse and dump of `allocations.json` | 1.71 MB; excludes validation, lock and write | 9.3 ms per cycle |
| Bridge imports | Measured | G-P2: `-X importtime` | — | About 17 ms and 13 ms cumulative |
| Per-test cold setup | Measured | Maintainer JUnit report | 16 threads; shared host; before store refusal | Median 1.31 s (1.09–4.05 s, n = 107) |
| Physical packages | Measured (static) | S1 | Shipped plus fixtures | 2. A single-slot physical cache is low materiality |
| Binary and state sizes | Measured | `ls`, 2026-10-11 | — | Extension 785.6 MB; worker 727.0 MB; 1,190 native operation records; 639 invocation files |
| Per-edit and per-study-start hashing totals | Arithmetic only | Multiplication of the counts above | — | Not an end-to-end measurement |
| Any speed-up from a remedy | Not claimed | — | — | Requires the probes below and end-to-end measurement |

### 9.2 Settling probes

These settle magnitudes or select remedy variants. None is needed to establish a diagnosis.

- **Settle a remedy variant:**
  - D-P1: `FOR UPDATE` reader semantics on SurrealDB 3.3 with RocksDB; selects F08's cancellation
    variant.
  - H-P4: late executable mappings; selects F05's controls and may expose a silent loss of reuse.
  - Whether `SemanticBodyHash` framing is complete: decides F04(ii), by formal argument or decision.
  - Which fields `admit_overlay` covers: bounds F01's value route.
- **Set priority:**
  - B-P1, C-P1 and H-P5: callback-path profiles (F16).
  - A-P1: edit versus warm preparation (F01, F03).
  - H-P1: the receiving-observation count per edit (F05).
  - E-P1: rows per block and empty blocks (F11).
  - D-P2 and D-P3: discovery counts and gate wait (F09, F08).
  - L2-P1: conflict retries (F08).
  - B-P3 and L1-P3: cubic cost and contention (F18).
  - C-P2: conditional-unit recycle (F19).
  - L1-P2: dense versus sparse response (F20).
  - D-P4 and L1-P1: the faer supernodal path (F10d).
  - F-P1: validation context identity (F23).
  - F-P2: pool contention (F10e).
  - E-P4: residue per run (F15).
  - G-P5: per-fixture lifecycle (F26).
- **Uncertainties that are not findings yet:**
  - G-P4: whether native and non-native routes invalidate the `pse-backend-native` build-script unit
    (inquiry G's G-07, unconfirmed).
  - C-P3: the share of presolve per attempt.
  - L2-P2: the pool reserved at startup (F24 magnitude).

Breadth that was not examined (see §2) is not a defect. The open probes can change priorities and
remedy variants; none can reverse a foundation verdict. Several independent High findings establish
AP-07 from structure alone, and the AP-06 and AP-05 findings rest on read source plus measured setup
and census.

### 9.3 Positions on prior dispositions

The maintainer chose full re-assessment. Earlier dispositions were treated as leads.

| Lead | Position | Evidence and current owner |
|---|---|---|
| 10-09 [F01](design_review_efficiency-principles-codebase_2026-10-09.md#f01) native-local journey | Resolved (residual default) | `select.py --execution-effects`; `unit-native-package` still defaults to canonical |
| 10-09 [F02](design_review_efficiency-principles-codebase_2026-10-09.md#f02) applicability inputs | Resolved | `validation_scope.py` input scope v3 |
| 10-09 [F03](design_review_efficiency-principles-codebase_2026-10-09.md#f03) coordination history | Residual and moved | F25; F05(d) |
| 10-09 [F04](design_review_efficiency-principles-codebase_2026-10-09.md#f04) narrow targets, force-validate | Resolved for test closures; moved to the interactive build | `arrow_validation.resolve`; F23 |
| 10-09 [F05](design_review_efficiency-principles-codebase_2026-10-09.md#f05), [F06](design_review_efficiency-principles-codebase_2026-10-09.md#f06) occurrence render; closure copies | Resolved (minor residual clones) | Positional `ExpressionOccurrence`; `CheckedPackage::select` |
| 10-09 [F07](design_review_efficiency-principles-codebase_2026-10-09.md#f07) conditional coordinate copies | Resolved (minor residual per-block row filter) | `CoordinateUniverse` shared |
| 10-09 [F08](design_review_efficiency-principles-codebase_2026-10-09.md#f08)–[F11](design_review_efficiency-principles-codebase_2026-10-09.md#f11) | Resolved | Inquiries B, D and E |
| Plan 33 EI02 / EI04 / EI05 / EI06 | Changed by F11 / confirmed (sound for structural edits; unfit as the value route, F01) / adequate / implemented with a quota interaction (F28) | — |
| [PE01](design_review_production-execution-efficiency_2026-10-07.md#pe01) | Residual | F22 |
| [PE02](design_review_production-execution-efficiency_2026-10-07.md#pe02) | Residual: the unit is now index metadata (verified `metadata_prefix`) | F11 |
| [PE03](design_review_production-execution-efficiency_2026-10-07.md#pe03) | Resolved; double read digest is minor | — |
| [PE04](design_review_production-execution-efficiency_2026-10-07.md#pe04) | Residual | F06 |
| [PE05](design_review_production-execution-efficiency_2026-10-07.md#pe05) / [PE06](design_review_production-execution-efficiency_2026-10-07.md#pe06) | Residual / moved to runtime observation | F05 |
| [PA01](design_review_preparation-assurance-and-reuse_2026-10-08.md#pa01) | Residual and moved: from per body to per prepare and per Runtime | F05 |
| [PA02](design_review_preparation-assurance-and-reuse_2026-10-08.md#pa02) | Residual | F03 |
| [PA03](design_review_preparation-assurance-and-reuse_2026-10-08.md#pa03) | Moved: the memo is pure; publication now sits in settlement | F04 |
| [PA04](design_review_preparation-assurance-and-reuse_2026-10-08.md#pa04) | Resolved for local replay; residual for non-interruptible deployment captures | F05 |
| Parallel [F01](design_review_parallel-execution-architecture_2026-10-08.md#f01) | Confirmed still present and broader (study touch; read bumps) | F08 |
| Parallel [F02](design_review_parallel-execution-architecture_2026-10-08.md#f02) | Residual; new manifestation in fit profiles | F10 |
| Parallel [F03](design_review_parallel-execution-architecture_2026-10-08.md#f03) | Not examined (process-global locks remain) | — |
| Parallel [F04](design_review_parallel-execution-architecture_2026-10-08.md#f04) | Resolved | — |
| Parallel [F05](design_review_parallel-execution-architecture_2026-10-08.md#f05) | Resolved for dispatch; the throughput limit moved | F08, F09, F10 |
| [EF01](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef01), [EF03](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef03), [EF07](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef07) | Resolved | — |
| [EF02](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef02) | Resolved as a mechanism; residual in effect | F23 |
| [EF04](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef04) | Residual | F27 |
| [EF05](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef05) / [EF06](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef06) | Resolved at build level, moved to runtime / residual | F05 |
| [EF08](design_review_execution-efficiency-and-surrealdb_2026-10-05.md#ef08) | Moved: preparation compaction exists; guard stages still copy whole-body frames | F16 |
| [US01](design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us01), [US02](design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us02) | Resolved | — |
| [US03](design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us03) | Implemented; not re-tested | — |
| [US04](design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us04) | Implemented; its realization is F08's contention | F08 |
| [US05](design_review_surrealdb-unified-simulation-substrate_2026-10-05.md#us05) | Residual | F04, F07 |
| State [F01](design_review_unified-state-management-and-persistence_2026-10-10.md#f01)–[F04](design_review_unified-state-management-and-persistence_2026-10-10.md#f04), [F09](design_review_unified-state-management-and-persistence_2026-10-10.md#f09) | Resolved. For F02, retention yields on job and session paths, although reservation caps still trigger trims (F10c) | — |
| State [F05](design_review_unified-state-management-and-persistence_2026-10-10.md#f05)–[F08](design_review_unified-state-management-and-persistence_2026-10-10.md#f08) | Not examined | — |
| 10-11 [F01](design_review_workspace-test-admission-and-interruption_2026-10-11.md#f01)–[F03](design_review_workspace-test-admission-and-interruption_2026-10-11.md#f03) | Confirmed still present. F02 is broader: it is structural in `Runtime::from_shared`. F03: `nextest.toml` names `…preserve_external_and_deployment_databases` while the test is `…preserve_deployment_database` | F26 |

Plan 28's rows marked "Implemented, E3/E4/E5 pending" are not closed by this review. Where the work
above is residual or moved, the new plan should own the correction. Plan 28 keeps its own acceptance
campaign.

## 10. Rule impacts and disposition (slot 11)

The recommendations would change the rules below. **None takes effect until the operator confirms
it in plan creation.** Routes follow the binding: an ADR for D1–D14, crate and governance changes; a
`design:` PR with a revision row for blueprint text; plan scope for sequencing.

| ID | Rule and where recorded | Proposed change | Findings | If the rule is kept |
|---|---|---|---|---|
| <a id="rc01"></a>RC01 | The package edit contract: §22.2 "`with_declarations` admits a new revision"; §19 case binding text; `modeling_from_documents` minting a problem per load | Add a value-overlay solve bound to the parent's engagement, with recorded lineage; delta ingress; lineage-preserving reload | F01, F02 | Value edits still rebuild structure. Only F02–F04 cheapen the path, and PS-11 needs an exception record |
| <a id="rc02"></a>RC02 | §14.4 Incrementality: "unchanged results backdate"; the reuse table; "no persistent Salsa store" | Either make it true (a long-lived workspace) or correct the text to recompute-and-compare | F07 | The text over-claims (G7) |
| <a id="rc03"></a>RC03 | §14.3 preparation amendment ("each consumer rechecks … and explicitly settles canonical effects"); ADR-0164 descriptions bound to complete read premises | An engagement owner settles once per lifetime; content-addressed payload plus reachability roots; premises encoded once per read (narrowed only if framing is proven complete); settlement off the critical path; skipped for ephemeral runs | F03, F04, F06 | Only removal of the second `prepare` and deduplication of encodings remain; per-point acknowledgment stays |
| <a id="rc04"></a>RC04 | ADR-0164 and §14.3 "Deployment-local replay": "absent acknowledgment requires current admission"; "complete dirty outer observation remains at deployment/run admission" | Process-image validity owned from startup and re-established by premise checks; outer provenance once per process | F05(a)(b) | One qualification per engagement; outer observation memoized per process and moved outside the GIL |
| <a id="rc05"></a>RC05 | Plan 35 RC01 executable association ("not weakened to … digest-shaped strings"); the `NativeExecutionSelection::require_current` contract; supervisor readiness checks; per-operation native generation verification | Content verification at publication and at exec or launch; stat identity plus stored digest at readiness and per operation | F05(c)(d) | Hash once per ensure-primary invocation and once per native operation, shared between Rust and the supervisor |
| <a id="rc06"></a>RC06 | §14.3 local pacing ("transactions changing protection, publication or retention premises take exclusive access"); §20.4 "Read acquisition, renewal, admission and reclamation share the retention guard"; §20.6 "Study cancellation participates … in renewal and append fences"; generated `writer`/`study_cancelled` | One coordination protocol (F08 items 1–5) | F08, F09, F10a, F12 | Per-problem serialization becomes a declared capacity limit for concurrent cases; only the fence and retry placement improve |
| <a id="rc07"></a>RC07 | `MathPolicy.jobs` as an immediate-refusal ceiling (`math.rs`); `conformance.toml` lanes at `jobs/2` | Queued native entry under deadline and cancellation; stage-separated lanes | F10a | Idle native capacity during I/O is accepted |
| <a id="rc08"></a>RC08 | §19.2 "Derived scalar cells and dense output descriptors reference these original batches"; Plan 33 EI02; the code rule that every advertised relation stores at least one block | Key-ordered blocks with key bounds or one locator per row; selection on decoded rows; empty sets declared at close; multi-output trajectory blocks | F11 | Decouple index staging from IPC framing. Write and reclaim amplification remain |
| <a id="rc09"></a>RC09 | §20.6 "Durable records and study facts consume the acknowledged stored completion" (implemented as a read-back) | Writer-held acknowledged bytes satisfy acknowledgment; read back only on uncertainty or recovery; optionally return before durability, with an explicit durability state | F12 | Only the fence and reconcile-from-acknowledgment parts apply |
| <a id="rc10"></a>RC10 | Study result contract (each occurrence a full single-solve run with all relations); "a horizon is one run and one attempt" with in-memory history (ADR-0110) | A study-level output demand; streamed horizon steps | F13 | Only ephemeral retention is bounded; horizon length becomes a documented limit |
| <a id="rc11"></a>RC11 | §20.6 "Failure cancels the scientific run" (progress); §19.2 "Retention failure is an operational failure" | Per-iteration telemetry sampled with recorded drops; a guaranteed channel for phase, failure and terminal events; overflow marks progress partial | F14 | Move per-iteration events off the guaranteed stream |
| <a id="rc12"></a>RC12 | §20.4 "preserving lifecycle and lineage receipts"; the protection-tombstone test assertion | Collect append receipts and guards with run batches; collect released protections after the acquisition deadline | F15 | Residue growth becomes a documented limit with periodic compaction |
| <a id="rc13"></a>RC13 | §14.3 "Mutable evaluator and solver state belongs to the attempt" (conditional) | Allow scratch-only workers to transfer between sequential attempts on one lane, with charges transferred | F17 (optional part) | Lazy per-order cloning only |
| <a id="rc14"></a>RC14 | §15.5.1 Root response rank, cutoff, backward-error and neighborhood evidence | Allow sparse rank-revealing evidence, with dense SVD as an explicit bounded certification mode | F20 | One sparse factor still serves predictor and solve; the dense SVD remains, size-bounded; large n is withheld |
| <a id="rc15"></a>RC15 | AGENTS.md force-validation and ADR-0170 as applied by `justfile` `py-sync`/`py-sync-native` | The interactive extension is built without `force-validate`; correctness runs use a force-validated build | F23 | Record the cost as accepted; interactive measurements must use the production build |
| <a id="rc16"></a>RC16 | Architecture overview library table and §14.2 DataFusion roles ("invariant checks, inspection") | Commit to an exposed relational inspection role, or narrow the roles and retire the machinery | F24 | The claim stays unbacked (G7) |
| <a id="rc17"></a>RC17 | A crate addition requires an ADR (AGENTS.md, `.claude/rules/decisions.md`); ADR-0122's workspace-hack | ADRs for an authoring/physical crate below `pse-runtime` and optionally a surrogate crate; retire or narrow hakari | F27 | Move the surrogate code into `pse-runtime`, remove the py-test hop, narrow `final-excludes` |
| <a id="rc18"></a>RC18 | Host capacity policy (`.config/agent-capacity.toml`; the agent environment guide: "timing uses cores 0–7 … kernel quotas bound CPU time") | Memory-based admission; CPU weights; timing reservation only while timing runs; declared recipe classes; queueing | F28 | At least classify by declared recipe class and derive nextest threads from the allocation |
| <a id="rc19"></a>RC19 | Test-resource coordination realization (the Plan 33 EFF09 live-coordination contract as documented for the agent environment) | Per-record live coordination; retention kept outside the hot artifact; reclaim only from runners or a sweep | F25 | Skip reclaim on commands that create no test resources; bound the lock wait |

No rule change is needed for F09's readiness flag, for F16, F18 (a conformance fix), F19, F21, F22,
F26 options (a)–(c), or F29. RC13 applies only if its option is chosen. JIT/SIMD admission (§14.5)
is not recommended, so it is not listed.

**Disposition.** The disposition is not yet scheduled. The proposed owner is a new plan created
through `plan-creation`, which records each finding as `finding | scenario | disposition |
decision/work owner | evidence or revisit trigger`. The new plan presents RC01–RC19 for operator
confirmation and orders the work by §8.2's prerequisites. Plan 28's open acceptance rows and the
completed Plans 33, 34 and 35 are not reopened.

## 11. Decision and next consequential choices (slot 12)

**Behavioral/semantic adequacy: Revise (narrow).** G7 and G8 fail narrowly ([F24](#f24),
[F07](#f07)). G2 carries a latent hazard ([F29](#f29)). Other core gates pass on the inspected
paths. Scientific gates PS-G1–PS-G3 were not re-qualified.

**Architectural fitness: Fails G9.** AP-07 is violated across every target workload, AP-06 in
testability, and AP-05 narrowly in the development environment. AP-01–AP-04 are satisfied within the
inspected scope.

**Overall: Revise.** The strongest evidence is source-traced structure:

- the complete-replacement edit route beside an unused value route (F01);
- per-call settlement against whole-closure premises (F03, F04);
- per-point re-establishment on a pinned revision (F06);
- one problem gate and one study record for every mutation (F08);
- one index record per scalar field setting the publication unit (F11);
- full content hashing of immutable artifacts at every use, Measured at 0.3–0.7 s per pass (F05);
- a per-test canonical store with a measured median of 1.31 s of cold setup (F26), together with
  history-proportional ledger rewrites (F25).

The open uncertainty concerns magnitudes and remedy variants, not the diagnoses. This decision
accepts no remedy as implemented.

| Priority | Change | Findings and scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| 1 | Preparation engagement and value route | F01, F03, F04, F06; S01, S03 | One preparation and one settlement per engagement. A value edit performs no compiler workspace build and no description publication (A-P1, A-P3). Lineage records parent plus overlay | New plan |
| 1 | Owned validity at trust transitions | F05; S01, S03, S10 | Zero full-image hashes per value edit and per warm study start (H-P1, H-P2). Replaced bytes, late load and configuration change refuse | New plan |
| 1 | One coordination protocol | F08, F09; S04, S05 | D-P1 result. No study-wide write per append. Mutations paced by written domain. Discovery reads O(ready) (D-P2). Cancellation, reclamation and phantom controls pass | New plan (ADR route if a D1–D14 decision moves) |
| 1 | Demand-shaped publication | F11, F13; S01, S03 | Blocks fill to the payload bound. No per-field cells. Empty relations declared at close. Selection results identical on exact-bit controls (E-P1 before and after) | New plan |
| 1 | Development loop | F25, F26; S10 | `pse-env -- true` does not open the ledger. Fixture cost flat as history grows. Runtime policy tests run without a store or with a pooled fixture | New plan |
| 2 | Operation-shaped evaluation | F16–F18; S03, S06, S07 | Bit-identical residuals, Jacobians, Hessians, errors and applicability; B-P1 shares before and after; contention controls (B-P3) | New plan |
| 2 | Nested and staged grain; response; capacity grain | F19, F20, F10; S06, S07, S04 | C-P2, L1-P2, D-P4; deterministic fixed-point maps; refusal semantics preserved | New plan |
| 2 | Finish, handles, progress, validation carriage | F12, F14, F23; S01, S03 | RPC counts per durable run; overflow yields partial progress, not cancellation; F-P1 | New plan |
| 3 | Salsa owner decision; machinery; residue; build closures; host admission; numerics; tears; float feature | F07, F24, F15, F27, F28, F22, F21, F29 | Decision records; deletions; E-P4; G-P6, G-P7 | New plan |

**Next consequential choices.**

1. **Preparation engagement contract.** Define its lifetime, protection renewal, drain, the owner of
   settled descriptions, and how the value route records lineage. This needs RC01, RC03 and RC04.
2. **Coordination protocol.** Choose the cancellation variant (probe D-P1), the guard split, and the
   pacing key. This needs RC06.
3. **Publication unit and output demand.** Decide the study output-demand contract and the
   replacement for the scalar-cell index (RC08, RC10). Decide [F24](#f24)'s commit-or-retire at the
   same time, because a DataFusion provider over result blocks is the natural home for cross-output
   predicates.
4. **Salsa owner versus plain functions** for structural edits ([F07](#f07), RC02). This is
   independent of the value route.
5. **Inner-loop owner.** Assign one owner for F16–F19, and decide RC13.

**Artifact path:**
`docs/design_review/reviews/design_review_efficiency-alignment-codebase_2026-10-11.md`, with
supporting documents under
`docs/design_review/evidence/efficiency-alignment-codebase-2026-10-11/`.
