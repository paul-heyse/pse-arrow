# Inquiry briefs

The briefs given to the ten supporting inquiries of the [efficiency-alignment review](../../reviews/design_review_efficiency-alignment-codebase_2026-10-11.md),
recorded so later readers can see what each inquiry was asked and what it was not asked. The
library-catalog and capability-map exclusions were added after the maintainer reported those
sources stale. The design inquiries received a short launch prompt pointing at these briefs, the
[operation map](operation-map.md) and their role contracts; the library inquiries' launch prompts
additionally narrowed their write permissions to scratch space outside the repository.

## Common brief (given to every inquiry)

### Purpose and form

An independent, codebase-wide **DESIGN / TARGET** design review that identifies every material
opportunity to align the simulator's physical realization with AP-07/G9 of the design principles,
using *Heuristics for Efficient Architecture* (H1–H28). You are producing a **bounded supporting
assessment** for one inquiry. A separate fresh principal reviewer will own the combined judgment
and finding IDs; the coordinator reconciles inquiries. Use local candidate labels (e.g. `A-01`).

The maintainer selected a **full re-assessment**: no area is exempt because an earlier review
covered it. Prior findings — and their "Resolved"/"Implemented" dispositions — are leads and
contrary evidence, never conclusions. For each prior lead in your area state one position:
*confirmed still present*, *resolved (work actually removed)*, *residual* (partly removed),
*moved* (the work now happens elsewhere), *refuted* (the diagnosis was wrong), or *not examined*.

### Standard (read these; they own the criteria)

- `docs/design_review/design_principles/standard.toml` (Core 3.4, Process Simulator 1.5, binding)
- `docs/design_review/design_principles/core/design-principles.md` — especially §1, AP-01–AP-07,
  DP-08–DP-23, §A gates (G9), §C, §D evidence vocabulary, §F, §G false positives
- `docs/design_review/design_principles/core/efficient-architecture-heuristics.md` (H1–H28 and
  its *Interpretation safeguards*)
- `docs/design_review/design_principles/core/design-review-template.md` (finding standard)
- `docs/design_review/design_principles/profiles/process-simulator/principles.md` and `review.md`
  (functional target, numerical stage columns, PS-09/PS-10)
- `docs/design_review/design_principles/binding/pse-arrow.md` (scenarios PSE-S01–S07; defect shapes)
- The `design-review` and `design-review-process-simulator` skills (`.claude/skills/`)

### Grounding (important)

Judge only against the standard and the **functional target**: a best-in-class process simulator
in which an engineer authors physically typed, reusable unit and property models once, composes
flowsheets and studies, and solves them (square simulation, optimization, dynamics, parameter
estimation) with results that are physically meaningful, diagnosable, and fast enough for
interactive **edit → re-solve** loops (a value change must not rebuild structure), **studies**
(sweeps, sensitivities, optimization, fitting), **recycles/tears**, **dynamics** from the same
definitions, and **extension** (new unit/property/reaction model or solver).

ADRs, blueprint decisions (D1–D14), plans, AGENTS.md policies, binding policies, pins and memory
are the *current design*, not criteria. Read them to understand the subject; do not fit a
recommendation to them. If a recommendation would change one, name it as a candidate rule impact
(rule, where recorded, proposed change, what the recommendation becomes if the rule is kept).

### Workload premises (qualitative; no SLA, no cost model)

- Interactive edit → re-solve on one flowsheet (value-only edits and structural edits).
- A thousand-point value sweep (an existing supported journey).
- Sixteen independent end-to-end cases on sixteen physical cores with internal library parallelism
  coordinated under finite CPU/memory (host: 32 logical CPUs, ~188 GiB RAM, shared with other
  repositories' workloads).
- Larger sparse structures; many small blocks; increasing declaration/expression populations.
- Long trajectories with sensitivities; fitting with many observations/parameters.
- Growing retained history (runs, analyses, artifacts, test resources).
- Interruption/cancellation before publication; crash recovery.
- Local development: compiling/testing one changed responsibility.

### How to assess

- Assess the **complete physical operation**: necessary vs repeated work, access paths, crossings
  (Python↔Rust, FFI, threads, SurrealDB WebSocket RPC, files), live state, reuse validity,
  resource/transaction lifetimes, failure/recovery, under growth/skew/concurrency where material.
- A useful candidate finding names: the avoidable work; why it arises (the design cause); the
  consequence under a named workload/scenario; a simpler conforming alternative (including
  library-owned alternatives); preservation constraints (guarantees that must survive); and
  material uncertainty. Static evidence can establish structural amplification; any quantitative
  speed/capacity claim needs measurement — do not claim speedups.
- Preserve correctness, physical/numerical integrity, exactness/partiality, PS-10 independent
  post-solve checks, truthful outcomes, source protection and coherent publication. Removing a
  check is not a remedy unless its guarantee is preserved elsewhere or provably unnecessary.
- Avoid false positives (principles §G; heuristics *Interpretation safeguards*): full-input work
  is legitimate for global computations; small fixed inputs may use simple code; a copy can be
  justified; caching adds maintenance. Multiple files or a large module do not establish a defect.
- Also record **strengths/preservation constraints** worth keeping, and anything you checked and
  found adequate (an adequate area is a valid result; do not manufacture findings).
- Cite evidence as `path::symbol` (file path plus function/type name). Never cite line numbers
  alone. Label claims with the evidence vocabulary (§D): *Proposed*, *Interface-checked*,
  *Implemented*, *Tested*, *Measured*. Source-traced diagnoses are typically *Interface-checked*
  or source-traced; say which.

### Permitted effects

Read-only. Do not edit tracked files, build the workspace, run test suites, start services, or
mutate any store. You may use `rg`, `ast-grep`, `cargo metadata`/`cargo tree` (read-only),
`git log/show/diff`. For library capabilities use, in this order of preference: the library
skills (repository-linked in `.claude/skills/` and the shared store
`~/.local/share/library-skills/skills/`: datafusion, datafusion-tracing, symbolica-faer-oximo,
native-solver-libraries, salsa, rust-graphs, neo4j-surrealdb, pyo3, serde-arrow, rust-reasoning,
stats-metrics, fixedbitset, …; each has task routes and pinned sources), context7, GitHub
search of upstream repositories (`gh search code`, `gh api`), and pinned upstream source under
`~/.cargo/registry/src/` or `vendor/`. Check version applicability against the pins in
`Cargo.toml` `[workspace.dependencies]` / `pyproject.toml`. Do NOT use
`docs/library-utilization.jsonl`, the library-catalog MCP or `docs/capability-maps/`: they are
stale and their generators have been removed. If a **light probe** would settle a doubt that could change a verdict, *propose* it in your
report (question, exact command, expected discriminating outcome); the coordinator arranges
execution under `scripts/pse-env --resource-class light`. Read-only static queries (`rg`,
`cargo metadata`, `cargo tree`, `git`) are fine to run yourself. Another heavy campaign may be
waiting for machine capacity: never start builds, tests or services.


### Baseline

`main` at `b01a45d71a5f5ab29991ad193679f5588ac3662f` (2026-10-11). Only dirty tracked file:
`docs/plans/28e-rebuild-retirement-and-qualification.md` (documentation). Production source is
clean at that commit.

### Lead sources (prior reviews and dispositions)

- `docs/design_review/reviews/design_review_efficiency-principles-codebase_2026-10-09.md`
  (F01–F11; all marked Resolved by `docs/plans/33-efficiency-principles-remediation.md`, see its
  *Finding dispositions*, *Investigation decisions* EI01–EI06 and *Outcome*), and its evidence
  directory `docs/design_review/evidence/efficiency-principles-codebase-2026-10-09/`.
- `docs/design_review/reviews/design_review_production-execution-efficiency_2026-10-07.md`
  (PE01–PE06), `design_review_preparation-assurance-and-reuse_2026-10-08.md` (PA01–PA04),
  `design_review_parallel-execution-architecture_2026-10-08.md` (F01–F05),
  `design_review_execution-efficiency-and-surrealdb_2026-10-05.md` (EF01–EF08),
  `design_review_plan-28-remaining-design-enhancements_2026-10-07.md`,
  `design_review_unified-state-management-and-persistence_2026-10-10.md` (F01–F09, Plan 34 done),
  `design_review_workspace-test-admission-and-interruption_2026-10-11.md` (F01–F03, unrouted).
- `docs/plans/28-surrealdb-unified-substrate.md` (*Finding dispositions* and the review-integration
  sections) and companions 28a–28k: most rows are "implemented, assembled acceptance E3/E4/E5
  pending". `docs/plans/34-…`, `35-…` (done).
- Note: these dispositions are claims by the implementing plans, mostly without measurement.

### Architecture entry points

- `docs/authoritative_design/README.md`, `sections/reading-guide.md`,
  `sections/architecture-overview.md`, and the owning sections (`numerical-execution.md`,
  `mathematics-and-compilation.md`, `models-and-composition.md`, `workflows-and-results.md`,
  `identity-and-publication.md`, `schema-and-relations.md`, `workspace-and-dependencies.md`,
  `operations-and-validation.md`).
- Operation map (warm edit→re-solve, study point N+1, result reopen) produced for this review:
  [operation-map.md](operation-map.md)
  — facts only; verify anything decisive yourself.

### Report format (return as your final message; the coordinator publishes)

1. Scope actually examined and coverage limits (what you did not read).
2. Complete operations examined (short stage → owner → crossing → live state notes).
3. Candidate findings (`X-nn`): title; observed cause with `path::symbol` evidence; consequence
   under a named workload/scenario and the heuristic/principle (AP/DP/H) involved; simpler
   conforming alternative (and library-owned option); preservation constraints; uncertainty;
   evidence label; suggested verification.
4. Opportunities/uncertainties that are not findings (and why not).
5. Strengths/preservation constraints observed.
6. Positions on prior leads in your area (table: lead ID → position → evidence).
7. Candidate rule impacts (if any).
8. Proposed light probes (if any), with command and discriminating outcome.

Be thorough but concise; aim for roughly 2,500–5,000 words. Precision over volume.

## Inquiry-specific briefs (A–H, L1, L2)

Sibling ownership: A authoring/admission/incremental preparation; B mathematical compilation,
evaluation and properties; C single-solve native route and structural analysis; D multi-case
workflows and shared capacity; E publication, persistence and results serving; F relational and
columnar layer (DataFusion/Arrow); G build, test and development loop; H cross-cutting assurance,
identity and observation. Follow dependencies into a sibling's area when your question needs it,
but leave that sibling's own questions to them; flag seams you see.

### A — Authoring, admission and incremental preparation

Scope: `crates/pse-authoring` (dsl, language, grammar), `crates/pse-runtime/src/authoring_driver/`
(document load/owned/hydrate/edit), `crates/pse-modeling` (check, specialize, selected_source,
portable, contextual), `crates/pse-compiler/src/workspace*` (Salsa database and queries,
modeling, flow, conditional), `crates/pse-runtime/src/workflow/modeling.rs` and
`workflow/modeling/{canonical,preparation,cases}.rs`, `workflow/physical.rs`, `physical/`,
`crates/pse-operations/src/canonical_{staging,selection}.rs` and `canonical.rs` source paths,
`crates/pse-py/src/workflow{,/modeling}.rs` ingress, `python/pse/_modeling.py`, `_workflow.py`.

Revealing questions:
- Value-only edit vs structural edit: what complete work runs from the edit to the solve? Is any
  parse, check, specialization, lowering, hashing, staging or canonical-revision work repeated
  although its inputs are unchanged (H12, H13)? Does change propagation stop when a recomputed
  value is substitutable (early cutoff)?
- Is the physical unit of checking/publication right? Plan 33 EI04 deliberately retained complete
  authored checking on changed publication; assess that choice afresh against the target.
- Does selected source/closure follow the smallest semantically sufficient scope (H5)? Are
  inventory consumers given selection and bulk intent (H8)?
- Are established properties (checked package, admitted revision, physical context) carried
  through hand-offs or re-established (H19)? Does Python↔Rust ingress copy or re-validate?
- Extension scenario PSE-S01: adding a unit/property model — what preparation work scales with
  package size rather than with the change?

Leads (re-assess each): 10-09 F05 (occurrence preparation renders/hashes subtree text), F06
(selected closure scans/copies checked universe), EI04; PE04 (study occurrences reacquire
selected source/admission); PA02 (warm composition rebuilds compiler generation); EF02, EF08;
graph review F01/J3 (warm description settlement linear search), F03/B8/N13 (topology rebuilt
per registration); state F03 (inventory consumers discard selection/grouped retrieval intent);
US05 (compile input contract). Library skill: `salsa`.

### B — Mathematical compilation, evaluation and properties

Scope: `crates/pse-compiler/src/typed_math*.rs`, `physical_identity`; `crates/pse-math`
(execution, derived, implicit_*, jets, factorable, assembly, presolve, collocation, binding,
numerics, normalization, library, typed*); `crates/pse-runtime/src/math.rs` (MathService),
`math/{modeling,preparation,products,retention.rs,retention/,artifacts,portable,functions,
prediction,surrogate}.rs`;
`crates/pse-kernels` (external functions/FeOS references) and property-model evaluation paths;
`crates/pse-quantity` where it appears in evaluation.

Revealing questions:
- Evaluator lifecycle: how often are Symbolica evaluators built, optimized, compiled, cloned,
  instantiated per worker/attempt/case? What is reused at its stable lifetime vs rebuilt (H12,
  H14)? What does a memo hit still cost?
- Derivatives and sparsity: are Jacobian/Hessian structures, symbolic analysis and sparse
  patterns prepared once per structure and refilled per value (DP-10)? Structural zeros (e.g.
  Taylor zeros) — is avoidable work generated?
- Property/thermo evaluation per solver iteration: physical form, crossings, allocations,
  repeated unit conversions or identity lookups in inner loops (H15, DP-10 "keep wide provenance
  out of inner loops").
- Implicit blocks/conditional regimes: representation fit and repeated coordinate projection.
- Combined live working set: evaluators, workspaces, retained products, caches (H17); how the
  memory pool charges them.
- Library fit (H4, DP-13/14): Symbolica/Numerica, faer (symbolic factorization reuse), num-dual,
  FeOS capabilities that would remove bespoke machinery or that bespoke wrappers duplicate. Use
  the `symbolica-faer-oximo` skill and context7.

Leads: 10-09 F07 (conditional blocks materialize original coordinate metadata), F08 (difference
inference lacks physical facts), F10 (fitting response search — shared with D); PE01 (coordinate
projection quadratic identity lookup); PA03 (memo hit performs full encoding/publication); Enh F05
(dynamic workers rebuild fixed layouts); production review §8 (Taylor zeros, reusable numeric
factor storage); parallel F02 (max capacity charged as immediate demand); state F02 (disposable
retention does not yield to useful work); EI05; RC02/RC03 (durable fast hashing, persisted
incremental runtime — unselected).

### C — Single-solve native route

Scope: `crates/pse-runtime/src/math/{solves.rs,solves/,staged.rs,jobs.rs,strategy/,
initialization.rs,flows.rs}`; `crates/pse-backend-native` (routing, solve, execution.rs and
execution/{runner, ipopt, pounce, kinsol, highs, clarabel, scip, uno, petsc, sos, factorable},
tnlp, callback, transport, kkt/, quality, square_response, tears, recycle, root_isolation,
presolve/, initialization, dynamics/ and dynamics.rs incl. IDAS integrator); `crates/pse-structural`
(incidence, flowsheet SCCs, projection); the `*-sys` crates only as needed.

Revealing questions:
- Per-callback work: what happens on each residual/Jacobian/Hessian callback across the FFI
  boundary — allocations, copies, index translation, validation, locking, unit conversion?
- Attempt lifecycle: are native workspaces, symbolic factorizations, solver objects reused across
  multistart, homotopy/continuation, staged initialization and retries, while keeping DP-19
  attempt-owned state? Is repeated setup a consequence of semantic boundaries (H2)?
- PS-09 / H4: do application-level loops (tears/recycle, globalization, continuation, strategy
  decisions between attempts) re-create capabilities that the native library owns, or is the
  bespoke composition justified by a missing consumed contract?
- Post-solve assessment (PS-10): do KKT/quality/square-response checks repeat factorization or
  full evaluation where an established property could be carried (H19/H20), while preserving
  independence of the check?
- Structural analysis (pse-structural; tear selection in math/flows.rs): algorithm and access
  path fit for larger/denser flowsheets (H9, H10); repeated computation across solves.
- Dynamics: integrator setup, event handling, sensitivity, trajectory capture — repeated work per
  step/sample; output detail vs demand (H6).

Leads: Enh F04 (IDAS refactorizes the same sample matrix — reported implemented); parallel F03
(native process coexistence/exclusion), F04 (POUNCE team stack reservation); solver-acceleration
review 2026-10-03 (`design_review_solver-acceleration-and-globalization_2026-10-03.md`: "nothing
owns decisions between attempts"); graph F02/N12 (flow admission scans every decision per edge);
integrated-solve-pipeline review 2026-10-03. Library skills: `native-solver-libraries`,
`rust-graphs`, `rust-reasoning`.

### D — Multi-case workflows and shared capacity

Scope: `crates/pse-runtime/src/workflow/{study.rs,study_operations.rs,study_execution.rs,
worker.rs,fitting/,uncertainty.rs,dynamics*.rs,modeling/dynamics*.rs,trajectory.rs,integrated.rs,
shooting.rs,horizon/,strategies/}`, `crates/pse-operations/src/{study_policy,canonical_studies}.rs`,
`crates/pse-model/src/study*`; shared capacity across crates: `crates/pse-runtime/src/{budget,
math,cancel,env}.rs`, `math/jobs.rs`, `crates/pse-columnar/src/resource*`, the DataFusion
`MemoryPool`/`RuntimeEnv`, Tokio runtime and blocking pools, native thread teams (POUNCE, faer,
rayon, BLAS/MKL threads).

Revealing questions:
- Across N cases of a sweep/sensitivity/fitting/MHE horizon: what is prepared once per study vs
  repeated per case (PSE-S03, S07)? Does each case cross the store/compiler/admission boundary?
- Capacity: are nested pools (Tokio, case lanes, native teams, BLAS threads, DataFusion pool)
  coordinated so that downstream capacity constrains upstream admission (H21, DP-20)? Is memory
  reserved at real demand? What happens at the memory limit and on cancellation (drain)?
- Ordering: are independent cases serialized by broad barriers, global ordering or shared
  ownership (H22)? Deterministic reduction preserved?
- Demand: is result detail per case (trajectories, sensitivities, diagnostics) matched to what
  the study consumer needs (H6)?
- Recovery: checkpoint/retry granularity vs failure unit (H24): does a failed case replay the
  whole study? Durable dispatch cost per case?
- Fitting: response assembly, covariance, uncertainty — repeated searches, dense intermediates.

Leads: parallel F01 (broad retention conflict domain), F02, F05 (serial frontier); PE04; EF03
(each durable dispatch evaluates whole study); Enh F01 (stored-study creation repeats admission),
N7 (fitting demand); 10-09 F10 (fitting response search); state F01 (rebinding retains obsolete
binding ancestry), F02. Library skill: `native-solver-libraries` for thread teams.

### E — Publication, persistence and results serving

Scope: `crates/pse-runtime/src/workflow/{durable,result_blocks,results,simulation_results,
modeling_results,connected_results,result_projection,result_export,retention,run}.rs` and their
fitting/trajectory result encoders; `crates/pse-operations/src/canonical*.rs` (incl.
canonical_execution, canonical_results, canonical_analyses, canonical_transport,
canonical_staging) and the generated SurrealQL/queries
(`crates/pse-operations/src/generated/`, generators in `crates/pse-codegen/src/codegen/surreal_*`);
`scripts/surreal_server.py` (server lifecycle, ensure-primary); `crates/pse-py` TableStream and
result handles; `python/pse/_runs.py`.

Revealing questions:
- Critical path of edit → re-solve: how many sequential SurrealDB RPCs (claims, staging, append,
  manifests, acknowledgments) precede the user seeing results? Which are required for the
  guarantee and which only for the mechanism (H1, H8, H23)? Ephemeral vs durable path choice.
- Movement: does each encode/frame/transfer/persist/reload step serve a purpose (H16)? Payload
  sizes of acknowledgments; block sizes; full-result materialization before selective transport.
- Selective reads: does a reopened subset read/verify/decode only what it needs (H5, H7)?
- Lifecycle: retention, retirement, cleanup, history growth, analysis payloads; does ordinary work
  scan history (H25, H27)? Atomic visibility/exclusion matched to effects (H23)?
- Is SurrealDB used through its native bulk/streaming/query capabilities or through item-by-item
  coordination (H4, H8)? Use the `neo4j-surrealdb` skill.

Leads: PE02 (tiny IPC blocks), PE03 (ack returns payload; two sequential RPCs per read); 10-09 F09
(selective transport builds complete Arrow map), F11 (retired analyses retain payloads), EI01
(gather forecast), EI02 (scalar-cell verification), EI03 (analysis size); EF01 (superseded
bundles kept); US01–US04; state F04 (source protection abandonment), F06, F08 (readiness
exclusion), F09 (cancellation/export settlement); Plan 35 (SurrealDB lifecycle remediation).

### F — Relational and columnar layer (no prior efficiency review)

Scope: `crates/pse-engine` (session/ incl. preparation, cache, schema_transform, admission,
engine_session; operation/; cache_service/; provider/; stores), `crates/pse-relations`
(validate/, columnar/, physical/, native, resolved_contract), `crates/pse-rules` (invariants,
references), `crates/pse-schema` (registry, builder, product, fingerprint, catalog),
`crates/pse-columnar` (owned_buffer, resource, flight, native_value/field), generated relation
code (`crates/*/src/generated/` — read only), and the consumers in pse-runtime/pse-py of
`pse_engine::session` and `cache_service`.

Revealing questions:
- Where is DataFusion actually on an operational path (admission, inspection, export,
  validation, invariants)? Does it pay its way — optimizer visibility, predicate/projection
  pushdown, plan reuse, statistics — or is it used as a container with bespoke iteration around
  it (H4, H8, H12)? Are custom providers/UDFs/rewrites duplicating native capabilities?
- Validation: is Arrow/relation validation repeated on immutable batches whose validity was
  already established (H19, H20)? Does `force_validate` (or equivalent full validation) reach
  production paths? Is `FieldCheckedBatch`/`readmit_context` carrying validity correctly?
- Schema transforms and conversions: copies, casts, metadata re-derivation (H16).
- Accounting: memory-pool reservations and resource tracking overhead vs protection (H26).
- Proportionality: is the size and generality of the session/cache layer matched to the
  capabilities its consumers use (H27, DP-14/DP-16)? Identify unused or duplicated machinery.
Leads: 10-09 evidence README columnar notes (registry `OnceLock`, `ContractArena`,
`FieldCheckedBatch`, `readmit_context`, `take_reserved`); 10-09 F04 (force-validate selection —
build side owned by G); `design_review_scoped-arrow-validation-target_2026-10-09.md`,
`design_review_relation-local-result-transport-target_2026-10-09.md`; ADR-0170. Library skills:
`datafusion`, `datafusion-tracing`, `serde-arrow`.

### G — Build, test and development loop

Scope: `Cargo.toml` workspace (features, profiles, `[workspace.dependencies]`), `.cargo/config.toml`,
`rust-toolchain.toml`, `crates/pse-workspace-hack`, `.config/hakari.toml`, `.config/nextest.toml`,
`.config/agent-capacity.toml`, `justfile`, `scripts/` (pse-env, select.py, host_admission.py,
test_resources.py, native_operation.py, build_measurements.py, arrow_validation.py,
validation_scope*, codegen drivers, native installation/verification, surreal test fixtures),
`scripts/tests/`, `crates/pse-codegen` (as a build step), `crates/pse-testkit`, Rust test fixtures
(e.g. `crates/pse-runtime/src/workflow/tests.rs`) and Python `tests/` (conformance, engine,
fixtures, lifecycle; conftest) plus `scripts/{python_tests,native_tests,test_run,validation*,
preflight,affected,compiler_cache,native_cache,native_pipeline_cache,build_cache_probe}.py`, the
`*-sys` build scripts and native installation.

Revealing questions:
- Does compiling/testing one changed responsibility build and start only its actual dependency
  closure (H5, AP-06, PSE-S05)? Do feature unification/workspace-hack choices give small roots
  heavy dependencies? Native features/installation verification on narrow builds?
- Test setup: is expensive shared setup (canonical store, services, native workers) paid before
  compatibility or relevance is established (fan-out before admission)? Do local assertions
  inherit canonical setup unnecessarily?
- Coordination: do host admission/test-resource/native-operation records scan growing history
  on hot paths (H25)? Locks/leases scaled to real participants?
- Codegen: does a no-op regeneration do or write work; does a narrow registry change regenerate
  broadly (H13)?
- Qualification applicability/evidence reuse: complete consumed inputs, and cost of computing
  applicability (H14).

Leads: 10-09 F01 (native-local tests acquire worker/store journey), F02 (applicability omits
inputs), F03 (active coordination processes history), F04 (narrow checks build unrelated relation
targets), EI06; PE05 (native installation re-verification), PE06 (outer attestation broadens
invalidation); EF04, EF05, EF06, EF07; Enh F03; `design_review_workspace-test-admission-and-
interruption_2026-10-11.md` F01–F03 (unrouted). Note the maintainer's 2026-10-11 nextest change
(`.config/nextest.toml`, commit b01a45d71).

### H — Cross-cutting assurance, identity and observation

Follow this responsibility across all components (runtime, operations, math, compiler, pse-py,
scripts). Scope: `crates/pse-ids` (frame, id, float, preimage; BLAKE3), every digest/receipt/
fingerprint computation (rg for `blake3`, `sha2`/`Sha256`, `digest`, `fingerprint`, `receipt`,
`attest`), runtime/executable observation (`pse-buildinfo`, runtime trust roots, loader locks),
admission re-checks, provenance/evidence payloads, diagnostics/logging/tracing volume.

Revealing questions:
- Where does each check enforce relative to where uncertainty actually enters or changes (H18)?
  Are internal crossings re-checked although nothing changed?
- Are established properties carried through operations that preserve their premises, via types,
  ownership or snapshots, or re-proved (H19)? Is any digest used as if it established semantics
  (H20 safeguard)?
- Auxiliary proportionality (H26): hashing/encoding of whole payloads on hot paths, provenance
  copies, per-primitive events, wide payloads in inner loops.
- Identity framing: canonical preimage construction costs (render-then-hash, repeated
  serialization), cache key construction vs recomputation cost (H14).
- The 2026-10-08 incident showed SHA-256 compression at ~97% of sampled cycles during a Python
  flash sweep: at the current baseline, where could whole-artifact/executable hashing still run
  per body, per case or per run?

Leads: PA01 (complete runtime observation repeated per body), PA04 (observation waits ignore
interruption); PE05, PE06; 10-09 F02, F05; EI05 (semantic BLAKE3, bucket prehash, interner);
graph U01 (FlowProjectionV2 framing, ADR-0167); RC02 (durable fast hashing — unselected);
`docs/design_review/evidence/preparation-assurance-and-reuse-2026-10-08.md`.

### L1 / L2 — Library utilization and fit (library-research role)

These two inquiries give the cross-cutting library view: how the codebase actually uses each pinned
major library, and which library-owned capabilities would remove bespoke work or are being
duplicated by wrappers (H4, DP-13/DP-14/DP-16, AP-07 "compare a credible simpler or native
realization"). Area inquiries A–H assess library fit inside their own operations; L1/L2 own the
per-library inventory and the comparison against each library's current capability surface.
Full library eligibility applies: no library or licence is refused; integration cost is assessed
on its own merits, and a capability is not withheld for lack of a consumer.

Method for each major library:
1. Pin and features: `Cargo.toml` `[workspace.dependencies]` (and member manifests), `pyproject.toml`.
2. Integration owners: which crates/modules depend on it (`cargo tree -i <crate> -e normal --offline`,
   `rg` for its paths). Is the integration owned in one place or spread across consumers?
3. Capabilities used: which APIs/features are actually called, on which operational path (hot vs
   cold), and how (bulk vs item-by-item, prepared/reused vs rebuilt).
4. Capability surface: from the library skill (task routes, pinned source), context7 and GitHub
   (upstream source, examples, issues, release notes at or after the pin), identify capabilities
   relevant to the simulator's workloads that would replace bespoke code, remove repeated work,
   preserve optimizer/native batch visibility, or that existing wrappers recreate (planning,
   caching, validation, retries, transactions, parallelism).
5. Version applicability: confirm a capability exists at the pinned version; if it needs an
   upgrade, say so (bumps are ordinary agent judgment here; a family major needs an ADR).
6. Report bespoke-vs-library pairs with evidence on both sides, semantic fit, integration owner,
   what machinery would be removed or introduced, and uncertainty. A library capability is a
   candidate, not a finding, unless the bespoke path causes material avoidable work or burden.

#### L1 — Numerical, symbolic and scientific libraries

Symbolica 3.0.1 (+Numerica via Symbolica, gmp-mpfr-sys); faer 0.24.4; nalgebra; ndarray;
num-dual (if present transitively via FeOS); FeOS/feos-core 0.10.1 (PC-SAFT); quantity 0.14;
Ipopt (pse-ipopt-sys), POUNCE family (pounce-nlp/-presolve/-linsol/-feral/-sens-core/-qp/-rs,
feral) at rev ff638694; Uno (pse-uno-sys); PETSc (pse-petsc-sys); SUNDIALS (sundials-sys:
KINSOL/IDAS); diffsol 0.16.2; HiGHS 2.4 / highs-sys; Clarabel 0.11.1; SCIP (scip-sys 0.1.28);
suitesparse_sys; egobox (ego/moe/gp/doe), argmin, finitediff, statrs; Salsa 0.28.4;
petgraph 0.8.3, rustworkx-core 0.18.1; rayon; teqp (Python, parity). Library skills:
`symbolica-faer-oximo`, `native-solver-libraries`, `salsa`, `rust-graphs`, `rust-reasoning`,
`stats-metrics`, `fixedbitset`, `pyomo-and-solvers` (IDAES behaviour reference only).
Particular questions: Symbolica evaluator construction/optimization/compilation reuse and
instancing; faer symbolic factorization reuse and sparse pattern refill; SUNDIALS/IDAS/KINSOL
workspace and linear-solver reuse; Ipopt/POUNCE warm start, sensitivity and reuse across attempts;
whether bespoke globalization/continuation/strategy loops duplicate solver capabilities (PS-09);
Salsa durability/early-cutoff/LRU features versus bespoke reuse; graph algorithms used for tears
and SCCs versus library offerings.

#### L2 — Data, persistence, boundary and infrastructure libraries

Arrow 59.3 family (arrow-ipc/-select/-cast/-row/-ord, parquet); DataFusion 55.1 family
(+datafusion-tracing, instrumented-object-store); object_store 0.13.2; SurrealDB 3.3.0
(vendored at `vendor/surrealdb`, protocol-ws) and tokio-tungstenite; pyo3 0.29.2,
pyo3-async-runtimes, pyo3-arrow 0.19; tokio/tokio-util; blake3 1.8.7 (rayon), sha2; serde/
serde_json/schemars/serde-saphyr/toml; quick_cache, rustc-hash, typed-index-collections,
enum-map; tracing/tracing-subscriber; winnow and sqlparser (authoring); uuid; Python: pyarrow
25, numpy, attrs/cattrs/msgspec. Library skills: `datafusion`, `datafusion-tracing`,
`neo4j-surrealdb`, `pyo3`, `serde-arrow`, `arrow-polars-duckdb`, `pydantic`/`python-oracles` if
relevant. Particular questions: DataFusion used as an engine (planning, pushdown, statistics,
prepared plans, streaming) vs as a container; Arrow kernels/IPC options (compression, dictionary,
`take`/`filter`, row format) vs bespoke equivalents; SurrealDB bulk insert, transactions, live
queries, streaming/export, server-side functions and indexes vs item-by-item RPCs; pyo3/pyo3-arrow
zero-copy and GIL release; BLAKE3 incremental/parallel/mmap hashing vs repeated whole-payload
hashing; quick_cache usage and coalescing; whether tracing instrumentation cost is proportionate.
