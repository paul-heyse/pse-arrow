# Design review: in-use-library remediation of the first design-principle audit

| Field | Content |
|---|---|
| Subject and boundary | The whole pse-arrow simulator core at source `a791c533e` (diagnostic publication `0530dcaa1`; recommendation-audit input `e08b1f1dd`): canonical store (`pse-operations`, SurrealDB 3.3), result storage and selection (`pse-runtime` result owners, Arrow IPC, DataFusion 55.1), preparation (`pse-modeling`, `pse-compiler`/Salsa 0.28.4, `pse-math`/Symbolica 3.0.1), native seam (`pse-backend-native`), runtime orchestration (`pse-runtime`), Python boundary (`pse-py`, `python/pse`). Suppliers and consumers are included where a correction depends on them. |
| Standard | Core principles and template 3.4 with Heuristics for Efficient Architecture 1.0; process-simulator profile 1.5; binding `pse-arrow` (selected by [`standard.toml`](../design_principles/standard.toml)). |
| Tier / purpose | Design tier; target purpose (best-in-class design for the functional target). |
| Focus | **Library-targeted, in-use libraries only** (maintainer selection, 2026-10-21): direct workspace dependencies at their pins, all their cargo features, and already-resolved sibling crates. Principle scope covers every audit finding DA-001–DA-032. New third-party libraries are out of scope. |
| Audit input | [Design principle audit](../audit/README.md), publication `0530dcaa1`, source baseline `a791c533e`. The diagnoses are challengeable input. |
| Reviewer / date | Revised judgment: independent recommendation auditor, 2026-10-21, continuing the same engagement. Exact input: `e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05` at this path. The original principal reviewer did not author or approve this revision. See the [net-change companion](audits/design_review_in-use-library-remediation_2026-10-21_audit.md) and [new evidence](../evidence/in-use-library-remediation-2026-10-21/recommendation-audit/README.md). Original receipts retain their original scope and date; new source checks and the selective codec probe are distinguished below. |
| Decisions | Behavioural adequacy: **Revise**. Architectural fitness: **Revise**. Overall: **Revise** ([§14](#14-decision)). |
| Disposition owner | [Proposed Plan49](../../plans/49-library-native-remediation.md) owns current finding/inquiry routes and rule decisions. Its implementation packages remain proposed; authoring and audit dispositions do not establish implementation or execution approval. |
| Intended path | `docs/design_review/reviews/design_review_in-use-library-remediation_2026-10-21.md` |

## Summary

**What the system is for.** pse-arrow lets an engineer author physically typed unit and property
models once and solve them as square systems, optimizations, dynamics and fits. It serves the
profile's five workloads: edit → re-solve, studies, recycles, dynamics and extension. The libraries
it already holds cover almost every generic responsibility involved:
- SurrealDB for canonical state and lane coordination;
- Arrow/DataFusion for relational and result access;
- Salsa for incremental compilation;
- Symbolica for mathematics;
- the native solvers for iteration and factorization;
- tokio-util, tracing, rayon, pyo3, pyo3-arrow, schemars and msgspec for execution and the
  boundary.

**What already supports the target, and stays.** The audit's strengths survived independent
re-reading and constrain every recommendation below:
- physical typing has one owner;
- domain guards are created before Symbolica sees an expression;
- derivatives are library-owned and refuse rather than approximate;
- conservation is by construction;
- solver outcomes are typed and independently re-qualified in original space;
- initialization never mutates the specification;
- cancellation drains native work;
- the fenced durable lifecycle protects truthfulness (claim before effects, close freezes
  membership, atomic seal, cancellation wins);
- value changes rebind instead of rebuilding.

**Where it falls short.** The 32 audit findings and ten additional ones reduce to seven
structural causes. In most of them a library the repository already links owns the missing
responsibility, but the repository either restates that responsibility beside the library or
switches the library's mechanism off.

1. **C1. Canonical lifecycle meaning lives outside the store's own type system**
   ([§4.1](#41-c1-canonical-lifecycle-meaning-lives-outside-the-stores-type-system)).
   - State is text and flag bags, and refusals are prose that Rust re-parses.
   - SurrealDB 3.3 already enforces literal and tagged-union field types, `READONLY` and
     `$before` transition assertions on every write path. Its `THROW` carries a byte-stable
     payload. Both were *Tested* by probe.
   - Guard rows stay: `REFERENCE … ON DELETE REJECT` was shown to dangle under races.
   - Recommendations R1–R3.
2. **C2. Result selection re-implements an index beside the engine that reads the payload**
   ([§4.2](#42-c2-result-selection-re-implements-an-index-beside-the-engine-that-reads-the-payload)).
   - Every scalar result row is written as a transactional SurrealDB locator with three index
     entries.
   - DataFusion's `PruningPredicateBuilder` over block statistics, with the canonical predicate
     lowered to a plain filter, replaces it. Equivalence was *Tested* on 63/63 cases.
   - The benefit is removing per-row locators and pruning on key columns. Value pruning was
     shown to be mostly ineffective. Recommendation R4.
3. **C3. Identity is carried by spelled names and rendered text**
   ([§4.3](#43-c3-identity-is-carried-by-spelled-names-and-rendered-text)).
   - This applies to the specialized model, derived identities, expression fields and relation
     policies.
   - The correction is project-owned: a resolved, identity-typed representation. In-use
     libraries supply only the representation's machinery.
   - It changes blueprint §7.1's "no intermediate IR" rule. Recommendations R5–R6.
4. **C4. Preparation repeats and then discards its own work**
   ([§4.4](#44-c4-preparation-repeats-and-then-discards-its-own-work)).
   - At default limits, every publish of the reference package rotates the whole Salsa
     workspace. The package's retained footprint exceeds the limits (*Measured*, F09).
   - The correction separates the admitted live working set from disposable query results and
     bounds the metadata, inputs, interned roots and retained values that remain. Salsa already
     applies count-based LRU to granular queries. Its public memory report does not expose a
     value-holding memo count or a byte-weighted eviction policy; memos needed for backdating
     stay resident.
   - Body evaluation should extend the existing symbolic product routes region by region over
     the guarded stage program. Probe A's savings are an upper bound measured on a guard-free
     program, and they refute a whole-group Dualizer.
   - Recommendations R7–R9.
5. **C5. Orchestration copies the run lifecycle and reinterprets owners' rules**
   ([§4.5](#45-c5-orchestration-copies-the-run-lifecycle-and-reinterprets-owners-rules)).
   - The copies have diverged: horizon caller cancellation is ignored (*Tested*, F01).
   - The tokio-util cancellation tree the runtime already wraps, plus the runtime's own
     `OperationRequest`/`PreparationSettings` types, own the remedy. Recommendations R10–R13.
6. **C6. Solver capability and composition knowledge sit outside the adapter seam**
   ([§4.6](#46-c6-solver-capability-and-composition-knowledge-sit-outside-the-adapter-seam)).
   - KINSOL fixed-point, the POUNCE second-opinion ladder and arclength suitability are named
     in the runtime. Capability records overstate what is admitted.
   - Uno maps a small-step stall to infeasibility (*Tested*, F02).
   - The Uno patch is mostly legitimate integration. Four hunks carry numerics.
   - Recommendations R14–R16.
7. **C7. Boundary vocabularies, identities and documents are hand-paired across Rust and
   Python** ([§4.7](#47-c7-boundary-vocabularies-identities-and-documents-are-hand-paired)).
   - The pyo3 `INPUT_TYPE` machinery, pyo3-arrow input types, schemars keywords and msgspec
     hooks already linked close the gaps. Recommendations R17–R21.

Three audit findings fall outside these causes: tooling placement (DA-020), documentation drift
(DA-023) and unconsumed bespoke machinery (DA-024). They are treated in [§4.8](#48-findings-outside-the-seven-causes).

**Decision.** Revise. Several core and profile gates fail, and all seven foundations are violated
somewhere in scope. The failed gates are G1, G2, G3, G5, G7, G8, G9 and PS-G3. The scientific
strengths are real, and none of the corrections needs to weaken them. Two defects are cheap and
independent: F01 (one line) and F02 (one mapping and named constants). They should not wait for
the larger remedies.

## 1. Scope, drivers and coverage

### 1.1 Functional target, workloads and premises

The target is the profile's *Functional target* together with blueprint §0/§0.2's capability
families. The analysis modes in scope are:
- square simulation;
- optimization, including NLP via Ipopt, POUNCE and Uno;
- dynamics, including horizons;
- parameter estimation through the LM residual route and NLP fits.

The workloads are:
- **edit → re-solve**, examined on the reference `ThreeUnits` composition and the BT_PR flash;
- **studies**: local studies and durable studies with up to 16 SurrealDB-coordinated worker lanes;
- **recycles**: declared fixed-point maps on KINSOL;
- **dynamics**: horizon cancellation;
- **extension**: a new lifecycle state, a new indexed relation, a new backend capability.

Scale premises come from the declared limits: 100,000 rows, instances and scalars per case
(`crates/pse-math/src/binding.rs:1161-1170`), and 100,000 specialization items. Concurrency comes
from P1.

The maintainer's fixed premises bind every recommendation:
- **P1.** SurrealDB stays canonical and is the enabler of 16-lane parallelism through guarded
  claim/started/settle functions, generation fences and "cancellation wins". Lane parallelism must
  be kept or improved.
- **P2.** Durable work can outlive its caller, and multi-process workspaces are supported, without
  a daemon.
- **P3.** Other formats are tactical payloads referenced from SurrealDB, never a second state,
  commit or lifecycle authority.
- **P4.** Vendored integration hooks are legitimate integration, not forks. Only hunks that change
  numerical behaviour are judged numerically.

Scope rules from the brief:
- Improve existing functionality. Minor additions are allowed only as clear wins; no major
  additions with tradeoffs.
- A correction that would regress another part of the system is recorded as **remedy open**, with
  the conflict stated.

### 1.2 Inputs and evidence

**Read.**
- The standard: core 3.4, heuristics 1.0, profile 1.5 and the binding.
- The audit: README and five topic pages.
- The coordinator's brief and checkpoint notes.
- The census and six capability inventories.
- Three probe reports (`probe-worktree`, `probe-surreal`, `probe-pruning`) and their logs.
- These inputs, the brief, the checkpoint notes and the three challenge reports are published as
  [supporting evidence](../evidence/in-use-library-remediation-2026-10-21/review-brief.md) in
  `docs/design_review/evidence/in-use-library-remediation-2026-10-21/`, with probe diffs, scripts
  and logs under `probe-artifacts/`.
- Repository source at `a791c533e`.
- Pinned library sources:
  - `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`: surrealdb-core/types 3.3.0,
    datafusion-pruning/-common/-physical-expr-common 55.1.0, arrow-ipc/-array/-buffer 59.3.0,
    salsa 0.28.4, symbolica 3.0.1, rayon-core 1.13.0, tokio-util 0.7.19, pyo3 0.29.2,
    pyo3-arrow 0.19.0;
  - the levenberg-marquardt checkout `e67066c`;
  - the POUNCE/FERAL checkout `7f5e2e9`;
  - the Uno `805a27ec` C API header;
  - `vendor/sundials-sys`.

**Excluded, per the brief.** Earlier reviews, review evidence, plans, the discarded earlier review
attempt, capability maps and the library catalog.

**Original executed evidence retained.** The following receipts belong to the first review;
they were not rerun by this recommendation audit. Their *Tested* and *Measured* claims apply
only to the named cases and original conditions.
- **probe-worktree** (profile `test`, pinned nightly, private test-resource state, sequential runs):
  - **A**, *Measured*: Symbolica operation counts on `bt_pr_flash_nested`, 19 instance groups,
    118 outputs; test `probe_a_body_granularity_pr_flash`.
  - **B1, B2**, *Tested* and *Measured*: demand diff and unreached-edit re-execution on the
    reference `ThreeUnits` composition; tests `probe_b1_demand_diff_on_reference_flowsheet` and
    `probe_b2_unreached_edit_reexecutes_projection`.
  - **C**, *Tested*: horizon caller cancellation before and after the one-line fix; test
    `probe_c_caller_cancel_after_admission_stops_horizon`, a 20-step antiwindup NMPC loop. Two
    existing horizon controls still passed.
  - **D**, *Tested* with synthetic inputs: the Uno latch and native mapping; test
    `probe_d_bridge_latch_and_native_status_mapping`.
- **probe-surreal**, *Tested*: Q1–Q4 on SurrealDB 3.3.0 `surreal start memory`, over HTTP `/sql`
  and the CLI. Q4 timings are *Measured* (indicative).
- **probe-pruning**, *Tested*: a standalone crate pinned to arrow 59.3.0 and datafusion 55.1.0,
  running probes 2, 4 and 8; in-memory batches with one Float64 column.

**Recommendation-audit evidence, 2026-10-21.** Source and pinned upstream re-reading supports
the remedy corrections and F10 below. The coordinator's selective
[`identity-hook-probe.py`](../evidence/in-use-library-remediation-2026-10-21/recommendation-audit/identity-hook-probe.py)
ran through `scripts/pse-env -- .venv/bin/python` on Python 3.14.8/msgspec 0.21.1,
exit 0 against a zero-failure baseline. Both the existing bytes-subclass identity and a non-bytes
alternative invoked `enc_hook` and `dec_hook` and round-tripped hex. This settles that codec
mechanism, not assembled product decoding. No product build, native solver, store service or
comprehensive qualification was run for this recommendation audit. Context7's Salsa API account
was checked against the pinned 0.28.4 source for the consequential retention claims.

Everything else is *Interface-checked* (library source or API read), *Implemented* (repository
path read) or *Proposed* (recommendation). No speed or capacity claim is made without one of the
measurements above.

### 1.3 Not examined, and limits

- **Scientific correctness of reference packages and property envelopes (PS-02).** Not examined,
  so PS-G1 is unresolved.
- **Structural analysis and degree-of-freedom diagnostics (PS-04/PS-05).** Not examined beyond
  confirming that HK/DM/BTF is library-owned through pounce-presolve. PS-G2 is unresolved.
- **SurrealDB behaviour on the repository's RocksDB server engine.** The probes used the memory
  engine, so conflict timing may differ.
- **Remedy magnitudes at supported scale.** Quantities such as write volume, evaluation time and
  16-lane throughput are unmeasured except where a probe is named.
- **Breadth.** IDAS/Diffsol internals, SCIP, HiGHS diagnostics, conic lowering, IBEX, surrogates
  (beyond pool lifetime) and the `TableStream` internals were not inspected.
- **Unexamined is not aligned.** An unexamined area is not a defect, and it is not evidence of
  alignment either.

## 2. The system as it bears on the findings

### 2.1 Decomposition (slot 2)

| Responsibility | Decision hidden | Contract consumed / exposed | Library owning the generic part | State / effect owner | Local test setup today |
|---|---|---|---|---|---|
| Authored physics and specialization (`pse-modeling`) | Instantiation, accumulators, closure from original terms | Authored DSL → `SpecializedModel` (still `dsl::Expr`, names `s_<hex>`) | none for the resolved form; petgraph, rpds for structure | Pure | Unit tests over authored sources |
| Incremental compilation (`pse-compiler`) | What to recompute after an edit; body identity | `Catalog`/`Request` inputs → `Projection`, `semantic_body` | Salsa (tracked, input, interned, LRU, `memory_usage`) | Salsa database inside `CompilerWorkspace` | Workspace tests |
| Library mathematics (`pse-math`) | Guards, regions, derivative order, admission budgets | `BodyBuilder` → admitted bodies, evaluators | Symbolica (`evaluator_multiple`, `Dualizer`, `derivative`, `export_cpp`) | `MathService` retention | Unit tests |
| Native seam (`pse-backend-native`) | Problem class, capability, status → typed outcome, qualification | `BackendExecution`, `Capability`, `SolveReport` | Ipopt, POUNCE/FERAL, Uno, KINSOL/IDAS/KLU, HiGHS, Clarabel, SCIP, PETSc, Diffsol | Native sessions under drained jobs | Per-adapter tests with linked solvers |
| Orchestration (`pse-runtime::workflow`) | Run lifecycle, study execution, strategies | `Runtime` composition root, `RunResult`, `StudyOperation` | tokio/tokio-util (cancellation tree), rayon | Runtime + canonical store | Often a full runtime ([DA-021](../audit/local-reasoning-and-assurance.md#da-021)) |
| Canonical store (`pse-operations`) | Lifecycle transitions, fences, retention | 116 generated `fn::pse_*` functions over 62 SCHEMAFULL tables | SurrealDB 3.3 (vendored SDK with integration patch; official server) | SurrealDB | Server or mock peer |
| Results (`pse-runtime` result owners) | Which rows a selection returns, exact bits | Arrow IPC files + SurrealDB block/row metadata; `SnapshotProvider` | Arrow IPC, DataFusion | `ProtectedFile` + store | Store-backed tests |
| Python boundary (`pse-py`, `python/pse`) | None in principle ("mechanical projection") | PyO3 classes, msgspec documents, Arrow streams | pyo3, pyo3-arrow, schemars, msgspec, cattrs | Runtime handles | Built extension only (`test = false`) |

Dependency direction follows these responsibilities except where the causes below say otherwise:
the host composes runtime operations, the runtime names backends, and orchestration restates
compiler rules.

### 2.2 Physical semantics at the affected boundaries (slot 3, profile)

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| Authored variable, row, member | Full `QuantityType` (kind, unit algebra) | Declared in type | Declared in type (`pse-quantity`) | Not examined (PS-02) | `crates/pse-quantity/src/quantity_type.rs:9-45` |
| Primary typed input (`BindingAssignment`) | Quantity and unit ids carried | Carried | Carried | n/a | Generated contracts; "bare numbers are not input values" |
| Side-route starts, block-initialization stages, `diagnose_samples` | **Bare `float`** keyed by `SemanticId`; unit implied by the target's canonical unit | Implied | Implied | n/a | Convention only ([DA-005](../audit/semantic-model-and-authority.md#da-005)) |
| Dynamics `atol`, `out_atol`, `parameter_scales` | Bare floats in **normalized state order** | n/a | n/a | n/a | Solver order ([DA-005](../audit/semantic-model-and-authority.md#da-005), PS-12) |
| Result scalar rows (`runtime.solve_variables`, `runtime.solve_constraints`) | `Float64` with **optional** untyped `quantity_id`, `unit_id` | Through the ids | Through the ids | n/a | Registry `native_math.rs:1175-1205`; no Python resolver |
| `ParameterCovariance` document | Bare values, no units | n/a | n/a | Validity not carried | `math/settings.rs:91-149` ([DA-010](../audit/structure-and-invariants.md#da-010)) |

No row was invented; cells marked "not examined" are unresolved, not passes.

### 2.3 Well-posedness statement (slot 3, profile)

- Variable roles are declared in the authored model.
- Structural analysis runs before solving through library-owned matching and decomposition
  (pounce-presolve `hopcroft_karp`, `DulmageMendelsohnPartition`, `BlockTriangularForm`, in
  `pse-structural`). How diagnostics name model elements was not examined.
- Two paths matter to this review. Declared fixed-point maps reach KINSOL through a runtime-built
  session (`math/initialization.rs:1271`; `workflow/strategies/conditional.rs:912`) rather than
  through `execution::roots`. Whether every admission check of the roots runner applies on that
  path is therefore **unresolved** ([DA-016](../audit/boundaries-and-composition.md#da-016); R14
  closes it structurally).
- PS-G2 is not settled by this review.

### 2.4 Strengths verified and how they constrain remedies

| Strength | Re-read at | Constrains |
|---|---|---|
| Guards survive rewriting: admission before Symbolica; `Require`/`Domain`/`Applicability`/`Branch`/`Provider` are host stages | `crates/pse-math/src/execution.rs:4802-4870` (regions stop at host stages and dependent blocks); `guarded.rs:96`, `:166` (order-qualified `Require`; no read before a domain-safe producer) | R7 is defined over the stage program, never over the flattened per-output expression; it may fuse only arithmetic between barriers; Symbolica `IfElse` is rejected |
| Library-owned derivatives with no finite-difference fallback | audit; `execution.rs:4237-4313` | R7's symbolic route is library-owned and exact; order admission must cover it |
| Typed outcomes and original-space qualification | `crates/pse-backend-native/src/quality.rs:686-700` (assurance only with a certificate capability) | R15 corrects one mapping; R14 must keep each recovery rung separately qualified |
| Fenced durable lifecycle; cancellation wins | `surreal.surql` `fn::pse_execution_v1::append` and siblings | R1–R4 add types and shrink transactions; they must not move authority out of the guarded functions |
| Value rebind outside structure | `PreparedCase::rebind`; `IntegratedExperiment::bind` (`workflow/integrated.rs:63-71`) | R8 must keep values out of Salsa; R12 extends rebind and never makes values inputs |
| Cancellation drain (compute released after native join) | `math/jobs.rs:1428-1490` | R10 bridges, then awaits; never `JoinSet` abort-on-drop for lanes |
| Python performs no science; strict decoding | `python/pse/codec/__init__.py:185-260` | R17–R19 keep strict, version-first refusal for inputs |

## 3. Change scenarios and simulator journeys (slot 4)

| ID | Stimulus and conditions | Expected response | Observed today | Correction / acceptance |
|---|---|---|---|---|
| <a id="s01"></a>S01 | Edit → re-solve: edit a declaration the live root does not reach, at production default limits ([PSE-S07](../design_principles/binding/pse-arrow.md#pse-s07)) | Unrelated granular checks reused; `projection` re-runs at most | Every granular query re-runs on publish; `selected`, `specialized`, `projection` and 143 `semantic_body` re-execute; 0 bodies reused; generation 1→2 (*Measured*, probe B2) | R8; acceptance: the same probe at the redefined defaults shows no rotation, 143/143 bodies reused and `specialized` backdating |
| <a id="s02"></a>S02 | Edit → re-solve with no edit | No re-execution | 79 `semantic_body` re-run (LRU 64 < 143 bodies) (*Measured*, probe B2) | R8 |
| <a id="s03"></a>S03 | Cancel a local study while it runs horizon points ([PSE-S07](../design_principles/binding/pse-arrow.md#pse-s07), interrupt) | The running point stops promptly; native work drains | Ignored: 20 of 20 steps complete (*Tested*, probe C) | F01 one-line fix (*Tested*); R10 makes it structural |
| <a id="s04"></a>S04 | Add a lifecycle state or an owner kind ([PSE-S04](../design_principles/binding/pse-arrow.md#pse-s04), [PSE-S06](../design_principles/binding/pse-arrow.md#pse-s06)) | One registry declaration; store, Rust and Python follow | Hand edits across substrate text columns, template literals, Rust enums and Python literals; projections default silently ([DA-007](../audit/structure-and-invariants.md#da-007)) | R1, R17; acceptance: one declaration plus regeneration |
| <a id="s05"></a>S05 | Durable study, 16 lanes over one problem ([PSE-S07](../design_principles/binding/pse-arrow.md#pse-s07)) | Run starts proceed in parallel apart from necessary ordering | Every point's `begin_run` takes the problem's exclusive in-process turn and writes one guard key (F03) | Necessary sequence order retained; breadth of local pacing unresolved pending the 16-lane probe ([§12](#12-verification-and-settling-evidence)) |
| <a id="s06"></a>S06 | Select results by owner, field and partition for a large solve; append its results ([PSE-S04](../design_principles/binding/pse-arrow.md#pse-s04)) | Write work scales with payload blocks; reads prune by key | One transactional locator per scalar row, plus three index entries each ([DA-026](../audit/execution-fit.md#da-026)) | R4 |
| <a id="s07"></a>S07 | Serve declared recycles with another library, or give another NLP backend the recovery ladder ([PSE-S02](../design_principles/binding/pse-arrow.md#pse-s02), mechanism substitution) | One adapter plus capability record | Runtime initialization, conditional strategies, solves, paths and the binding change ([DA-016](../audit/boundaries-and-composition.md#da-016)) | R14 |
| <a id="s08"></a>S08 | Python sweep supplying starts and per-state tolerances (boundary round trip) | Units and identities carried; no solver order | Bare floats and positional tolerances ([DA-005](../audit/semantic-model-and-authority.md#da-005)) | R18 |
| <a id="s09"></a>S09 | Dynamic or fit sweep with values only ([PSE-S03](../design_principles/binding/pse-arrow.md#pse-s03)) | Prepare once, bind per point | Refused with `StudyOperationUnsupported` ([DA-014](../audit/boundaries-and-composition.md#da-014)) | R12 |
| <a id="s10"></a>S10 | Test LM route eligibility in isolation ([PSE-S05](../design_principles/binding/pse-arrow.md#pse-s05)) | Policy with its actual inputs | Needs a runtime, admission and compilation ([DA-021](../audit/local-reasoning-and-assurance.md#da-021)) | R11 |
| <a id="s11"></a>S11 | Add a unit model on existing physics ([PSE-S01](../design_principles/binding/pse-arrow.md#pse-s01), domain extension) | Package-only change | Holds (audit strength); C3's field-grammar inventories matter only when the IR itself gains a field | Preserve |

A domain extension (S11) and two mechanism substitutions (S06 storage representation, S07 solver)
are included.

## 4. Structural causes and corrections

Each cause below gives its diagnosis, the correction built from in-use library capabilities,
alternatives, preserved strengths, what consumers stop interpreting, and the reverse check against
P1–P4. Recommendations carry *Proposed* evidence. Their enabling library facts carry the label
of how they were established.

### 4.1 C1. Canonical lifecycle meaning lives outside the store's type system

**Diagnosis.** Explains [DA-007](../audit/structure-and-invariants.md#da-007) and
[DA-008](../audit/structure-and-invariants.md#da-008), the store side of
[DA-004](../audit/semantic-model-and-authority.md#da-004), the bypassing-writer part of
[DA-019](../audit/boundaries-and-composition.md#da-019), and F03/F07.

The canonical store is the authority under P1, yet its lifecycle facts are declared below the
level at which the library can enforce them:
- **Untyped state.**
  - The substrate catalog declares attempts as independent `ingestion_open`, `closed` and
    `terminal` flags plus an optional `outcome` text
    (`crates/pse-schema/src/catalog/substrate.rs:530-548`), and `substrate.rs` has zero
    `enumeration(` calls.
  - The SurrealQL generator maps Arrow types only (`crates/pse-codegen/src/codegen/surreal.rs:71-91`),
    so registry enumerations never reach the store.
- **Restated value sets.** Value sets are re-spelled as template literals; for example, artifact
  owner kinds are `NOT IN ['study','user','project','analysis','audit']` in
  `fn::pse_artifact_v1::attach`.
- **Unguarded release.** `attach` guards `analysis` owners, but `release` has no `analysis`
  branch (`crates/pse-operations/src/generated/surreal.surql` `fn::pse_artifact_v1::release`).
  Owner-kind meaning is spread across function bodies rather than declared once.
- **Prose refusals.**
  - 306 `THROW` lines refuse with prose.
  - `CanonicalError` classifies everything except `Codec` as `RuntimeInfrastructure`
    (`crates/pse-operations/src/canonical.rs:176-183`).
  - A production retry decision strips `"An error occurred: "` and compares three literal
    messages (`crates/pse-runtime/src/workflow/worker.rs:1697`).
  - Worker loss is minted as `CanonicalError::Configuration("worker authority expired …")`
    (`crates/pse-runtime/src/workflow/durable.rs:505-508`).
  - `TerminationCause::Infrastructure` is matched but never constructed.

Two inventory claims did not survive re-reading:
- **The second prose match is test code.** The match the inventory cited
  (`canonical_files.rs:1470`) is in a `#[cfg(test)]` module, so only `worker.rs:1697` is in
  production.
- **"Retries exhausted" is never produced.** Every guarded retry loop returns from inside the loop
  on its final attempt (`canonical.rs:962-975`, `:2091-2108`, `:1383-1424`, `:1737-1785`).
  - So the `Configuration("… retries exhausted")` and `IncompleteResponse` after each loop cannot
    be reached.
  - Exhausted retries surface as the raw `Driver(TransactionConflict)` and then collapse into
    `RuntimeInfrastructure` like everything else. That is part of DA-008's collapse, not a
    separate mislabel.

**Correction.**

*R1, typed lifecycle in the store, generated from the registry.*
- **Generator.** Extend the SurrealQL generator to read `FieldContract::enum_name_of` and emit:
  - literal-union field types (`TYPE 'a' | 'b'`, `option<…>` when nullable);
  - `READONLY` only on columns verified immutable, table by table (`key`; identity columns such as
    `run` on attempts). It must **not** go on columns the fenced lifecycle rewrites in place:
    - guard `touch` (`generation += 1dec`, surql `:799`);
    - study `cancel` (`cancelled = true, generation += 1dec`, `:2013`);
    - the retention guard (`:2142`);
    - `fn::pse_files_v1::recover_intent` (`owner=$new_owner`, `:1437`), which is P2 recovery;
    - operation compaction (`request=type::bytes('')`, `minimal=true`, `:2807`).
  - per-column transition rules for those columns instead of `READONLY`:
    - fence counters get a monotonic `ASSERT $before = NONE OR $value = $before OR
      $value = $before + 1dec`;
    - an owner rewrite is admitted only from a recovery state;
    - compaction may go only to empty bytes with `minimal = true`;
  - one **tagged-union lifecycle field** per state machine in place of flag bags, with the
    states derived from all supported transitions rather than an illustrative three-state
    simplification. Attempts must distinguish ingestion, closed-awaiting-manifest,
    closed-with-manifest and terminal, or represent the manifest's optionality explicitly.
    `close`, `cancel` and `recover` close ingestion before `manifest` installs the descriptor
    (`pse-codegen/src/codegen/surreal_execution.rs`, functions `close`, `cancel`, `recover`,
    `manifest`). The generation remains the existing u64-compatible decimal domain; a
    SurrealQL `int` must not narrow its range;
  - allowed transitions declared once in the registry and emitted as field assertions.
    A phase-pair assertion is only one part: same-phase changes, generation/fence ownership,
    immutable payloads and cancellation/recovery transitions keep their existing checks.
    A schema backstop never replaces the guarded cross-record protocol.
- **Registry.** Declare the store's physical lifecycle phases separately from observational
  scientific states. Existing `AttemptState` includes planned/queued/running/stale/superseded
  (`pse-schema/src/catalog/operations.rs`) and is not the ingestion/closed/terminal phase set;
  store `succeeded` also projects to observational `completed`. Generate store types and
  explicit projections from their respective declarations rather than retargeting or merging
  these different vocabularies. Apply the same distinction to `StudyPointState`.
- **Owner kinds.** These become either a literal union or typed `record<canonical_studies |
  canonical_analyses>` holders. In both cases, one generated release guard covers every holder
  table, which closes the `analysis` gap by construction.
- **Library facts** (SurrealDB 3.3.0; *Interface-checked* in core sources, *Tested* by probe-surreal
  Q1):
  - The tagged union, literal unions, `option<literal>`, `READONLY` and the `$before` transition
    rule were enforced on CREATE, INSERT, `INSERT … ON DUPLICATE KEY UPDATE`, UPDATE and UPSERT.
  - A multi-row UPDATE with one illegal row rolled back entirely.
  - `DEFINE INDEX … FIELDS state.phase` is used by the planner.
- **Conditions.**
  - Schema refusals arrive as `kind:"Internal"` prose.
  - `UPSERT <id> SET/MERGE` on an existing row names the wrong field in its refusal text.
  - `int` inside an object literal accepts but does not coerce whole floats.
  - So the schema is a **backstop**: the guarded functions keep their domain pre-checks (R2), and a
    schema refusal is classified as an invariant breach, never decoded from text.

*R2, typed refusals through `THROW`.*
- **Vocabulary.** Declare a registry enumeration of canonical refusal codes, with classes such as
  `stale_premise`, `fence_lost`, `worker_authority_lost`, `bound_exceeded`, `identity_changed` and
  `unavailable`.
- **Store side.** Templates refer to codes by generated constant and throw a code-prefixed string,
  for example `THROW 'pse.refusal:study_decision_generation_changed'`.
- **Rust side.** `CanonicalError` gains `Refused { refusal, class }`. It is decoded once, from
  `Error::is_thrown()` (types `error.rs:449`) plus the pinned display prefix (core
  `exec/error.rs:27`, `"An error occurred: {0}"`).
- **Retry and diagnostics.**
  - The retry decision at `worker.rs:1697` matches the class.
  - The two other DA-008 consumers read the typed class and termination cause rather than shape:
    - retry classification by wrapper shape (`durable.rs:2062-2099`) uses the class;
    - study recovery's lost-writer inference from completion = None, Transient and Absent
      (`study.rs:1694-1700`; `study_policy.rs:554-560`) becomes a match on
      `TerminationCause::Infrastructure` with the `worker_authority_lost` class.
  - The `detail()` helper stops hard-coding `effect: Absent` wherever the producer can establish
    effect state.
  - Worker loss becomes `worker_authority_lost` and maps to `TerminationCause::Infrastructure`.
  - Exhausted conflict retries become a typed `ConflictRetriesExhausted` and stop being an
    anonymous driver error.
  - `TypedDiagnostic` maps classes to distinct §23.2 codes.
- **Library fact** (*Tested*, probe-surreal Q2): thrown values are byte-stable across top level,
  `IF`, `FOR`, nested `fn::`, `BEGIN…COMMIT` and `SET` with `kind:"Thrown"`, and a failed
  transaction's siblings report `NotExecuted`/`Cancelled`, with all writes rolled back.
- **Payload form.** The code-prefix form is selected because it needs no parser in the default
  build. The object form (`THROW {refusal, class, …}`) is equally stable and round-trips through
  `surrealdb_syn::value`, but that parser sits behind `canonical-maintenance` today. It is the
  alternative where structured detail is needed.
- **Remaining Rust work.** Carry operation identity/stage and truthful effect state. Unknown
  refusal codes fail closed. A stable class does not establish that effects are absent after
  transport loss or timeout, and retryability is not automatic replay permission. Preserve the
  server's exact acknowledged study-cancellation distinction and the worker's fresh-premise
  reread; do not synthesize acknowledgment from an incidental cancellation/fence shape.

*R3, use the store's own keys and adjacency (candidate).*
- **Keys.** Every row's id is `type::record(table, key)`, and the decoder checks this only
  client-side (`codegen/surreal.rs:148`). The server never asserts it. It does maintain the 62
  `<t>_key … UNIQUE` indexes the generator emits (`surreal.rs:135`).
  - Replace each index with `DEFINE FIELD key … READONLY ASSERT $value = record::id($this.id)`.
    `record::id` exists at core `fnc/mod.rs:336`. `$this.…` is already used in production field
    asserts (surql `:426-430`), and field clauses evaluate against the current document, whose id
    is fixed at record-id generation (core `doc/field.rs:613-619`, `:655`). This is
    *Interface-checked*; the UPSERT and `INSERT … ON DUPLICATE KEY` forms still need a probe.
  - That makes key–id agreement a server invariant, and record-id uniqueness then implies key
    uniqueness.
  - **Condition: some readers still use the key index.** Production paging uses compound indexes
    and is unaffected, but whole-table keyset paging `WHERE key > … ORDER BY key LIMIT 64` in
    `scripts/surreal_server.py` (`:5749`, `:7275`) relies on the `key` index. So do the remaining
    `WHERE key = $k` reads. These must move to record-id ranges and record fetches, with
    `EXPLAIN` checked on RocksDB, before an index is dropped.
- **Relation tables.**
  - Mark `canonical_study_dependencies` `ENFORCED` like the other relation tables (it is
    `TYPE RELATION … SCHEMAFULL` without it, surql `:596`).
  - Where direction indexes duplicate adjacency, readiness scans can use traversal. The pageable
    forms are in-idiom `->(deps LIMIT n START m)` or keyset `->(deps WHERE id > $c LIMIT n)`,
    never outer `FROM <traversal> LIMIT/START`, which loaded every edge (≈300 ms at 20k edges
    against ≈0.35–6 ms; *Measured*, indicative, probe-surreal Q4).
- **Status.** The traversal half is a candidate pending a RocksDB check of ordering and cost.

**Alternatives.**

| Alternative | Assessment |
|---|---|
| Current baseline | Untyped columns, prose refusals; correct today because the functions check most transitions, but the meaning is unowned and exhaustive projections default silently. |
| Simplest viable | Literal unions per column without the tagged union: removes unknown values but keeps undefined flag combinations. Acceptable first step; weaker than R1. |
| `REFERENCE … ON DELETE REJECT` for owners | **Rejected as a replacement** for guard rows. REJECT is checked only at delete time. A racing reference insert left a dangling owner even with `FOR UPDATE` on the referrer side, and a dangling owner reads as "not retired" (*Tested*, probe-surreal Q3). Only a referrer write that changes the target protected both orders, which is today's guard-row discipline. Typed `record<…>` holders remain useful for typing (R1). |
| `DEFINE EVENT` for readiness fan-out | **Remedy open.** Asynchronous events delete the queued work with only a warning when retries are exhausted (core `doc/event.rs:483-503`), which violates DP-19. Synchronous events put sibling fan-out into every settle transaction and would add conflicts under P1. |
| `DEFINE SEQUENCE` for run sequence | **Remedy open** (F03). The user-facing `sequence::nextval` promises consecutive values while its node stays up,
  but its durable cursor allocation is separate from the caller's run-publication commit
  (surrealdb-datastore 3.3.0 `sequences.rs`, `nextval`/cursor commit). It does not establish
  commit order for the visible-run paging contract, and its i64 extent narrows the current
  u64 domain. Internal allocator batch comments alone are not the decisive premise. |
| gRPC engine of the same SDK | **Not recommended.** It would bypass the vendored WebSocket integration (bounded admission, deadlines, replay checkpoints, response limits) that P4 makes legitimate, and its equivalents are unverified. |

**Preserved.** Generated guarded functions remain the runtime lifecycle mutation owner; privileged
installation/restore/activation keeps its explicit maintenance owner (R21). Claim, close, seal and fence
order are unchanged, guard rows stay, and SCHEMAFULL stays.

**Consumers stop interpreting:**
- prose refusals;
- flag conjunctions such as "terminal while ingestion open";
- wildcard projections of `AttemptState` (`durable.rs:1875-1880`; `study.rs:1570-1574`);
- per-template owner-kind lists.

The operator mutations in `scripts/surreal_server.py` (DA-019) move into generated guarded
maintenance functions (R21). Until then, its f-string SQL is refused at the table when it writes an
illegal value.

**Reverse check.**
- **P1.** R1 and R2 add no keys to transactions. R3 removes one index entry per canonical row, so
  conflict sets shrink.
- **P2 and P3.** Unchanged.
- **P4.** No vendored hunk is touched.
- **DP-19.** Strengthened (worker loss becomes truthful), on condition that R1 keeps every in-place
  fence, cancellation, recovery and compaction rewrite legal through transition rules rather than
  `READONLY`. A blanket `READONLY` would refuse "cancellation wins", guard touches and P2 file
  recovery, which regresses P1, P2 and DP-19.

### 4.2 C2. Result selection re-implements an index beside the engine that reads the payload

**Diagnosis.** Explains [DA-026](../audit/execution-fit.md#da-026), the result-side sites of
[DA-032](../audit/semantic-model-and-authority.md#da-032), and the result write volume in
[DA-025](../audit/execution-fit.md#da-025).

The guarded `fn::pse_execution_v1::append` inserts one `canonical_result_rows` locator per scalar
result row inside the fenced transaction. Each locator carries three index entries:
`canonical_result_rows_key` UNIQUE, `result_row_selection` and `result_row_batch` (surql
`:458-473`, `:745-746`). Appends are capped at 18,724 rows and 131,072 encoded metadata bytes. The
existing `canonical_result_blocks` and `canonical_result_block_outputs` rows already hold
`[start,end)` ranges and typed `coordinate_min/max {bits, projection}` fields, but scalar blocks
write `coordinate_min: None` (`result_projection.rs:1156`).

Reads then:
- decode through `SnapshotProvider`, which reports every filter as `Unsupported` and exposes one
  streaming partition behind a mutex-held decoder (`result_snapshot.rs:39-50`; provider `scan`);
- check each locator against its row;
- apply a hand-written `scalar_predicate`.

The heavy write work and the bespoke predicate sit beside a DataFusion engine that already decodes
the payload.

**Correction.**

*R4, pruned result selection over record-batch statistics.*
1. **Write.**
   - Within each IPC record batch, order rows by the **sort key the registry declares per
     relation**, using `lexsort_to_indices` and `take` (both already used,
     `pse-schema/src/builder/native.rs:108`):
     - solve variables: `(run_id, step, symbol_id)`;
     - solve constraints: `(run_id, step, row_id)`;
     - fit parameters: `(run_id, parameter_id)`;
     - fit observations: `(run_id, experiment_id, observation_id)`;
       these are the existing supported key shapes in `selected_scalar_row`
       (`pse-runtime/src/workflow/result_projection.rs`), not a universal `symbol_id`;
     - symbol-major for dense trajectories, whose projection is symbol-major and refuses
       non-contiguous symbols (`result_projection.rs:1195-1218`). Trajectories keep their dense
       path; R4's locator removal concerns the scalar relations.
   - Each canonical result batch is one IPC record batch of a packed file (`file_batch_index`),
     and `canonical_result_blocks` holds one row per batch (surql `:1044`).
   - Commit that record batch's key statistics, in the same guarded append, on its **existing**
     block row: min/max for that relation's declared owner and partition keys, an exact row
     count and exact null counts. Optional `step` and experiment keys retain their meaning.
   - These are `Precision::Exact` facts, which is what `PrunableStatistics` reads
     (datafusion-common `pruning.rs:342`, `:377-415`). Missing-value (`IS NULL`) pruning needs the
     exact null counts.
   - The append contract states that the statistics are exact, and the existing block-row
     equality check on replay (surql `:1026-1031`) covers them.
   - Stop writing per-row locators after all four scalar relations and their portable
     selection consumers have a replacement. A row's coordinate remains `(batch, offset)`;
     deterministic sorting happens before authoritative row positions are assigned, or an
     explicit mapping preserves them. Existing snapshots/review packs record row ranges and
     must never be silently reinterpreted in a new sort order (`connected_results.rs`,
     `pack_selection`, selected-row reconstruction).
   - Derive exact statistics from the admitted original batch and bind them to its schema,
     digest and interpretation in the guarded append. Replay equality proves that supplied
     metadata is unchanged, not that it was originally true of the payload.
2. **Snapshot.** SurrealDB still returns the committed, fenced set of batches with their
   statistics. It stays the authority on what exists (P1, P3).
3. **Provider.**
   - `supports_filters_pushdown` returns `Inexact`.
   - `scan`/`scan_with_args` builds `PrunableStatistics::new(Vec<Arc<Statistics>>, schema)`
     (datafusion-common `pruning.rs:314-330`) from the store's statistics, and prunes with
     `PruningPredicateBuilder::new().with_file_schema(..).try_build(expr)` (datafusion-pruning
     `pruning_predicate.rs:413-483`; `PruningPredicate::try_new` is deprecated at `:597`).
   - The scan physically plans pushed filters, retaining required key/predicate columns.
     `Inexact` leaves residual filtering to the engine; a scan limit must not truncate
     false-positive batches before that filter can fulfill the requested limit. Unknown or
     unsupported statistics retain the batch conservatively, never mean an empty answer.
   - It decodes only surviving record batches **inside the existing single protected decoder**.
     That decoder is already a `FileReader<ProtectedFile>` (`result_snapshot.rs:89`), so it uses
     `FileReader::set_index` (arrow-ipc `reader.rs:1417`; already called at
     `result_snapshot.rs:390`); a `FileReaderBuilder::with_projection` projection (`:1183`)
     is used only after defining a sound
     projected-read admission boundary. Today the decoder checks equality with the full
     relation schema and returns a fully `FieldCheckedBatch`-admitted value
     (`result_snapshot.rs`, `ProtectedResultDecode::decode`). A projected batch cannot pass
     that full-schema check or masquerade as a fully checked relation. The simplest selected
     step retains full decode/admission and applies output projection afterward; input
     projection remains a candidate where required validation columns and trust guarantees
     can be preserved. `FileDecoder` is not needed.
   - Arrow IPC decodes whole record batches. Decoding is saved only when a whole record batch is
     pruned. Pruning a sub-batch range (`canonical_result_block_outputs`) saves filter work, not
     decoding.
4. **Canonical predicate.** `output_results(field, minimum, maximum, missing)` lowers to a
   plain DataFusion `Expr` matching all existing modes in `scalar_predicate`:
   - no `missing`, minimum or maximum: `TRUE`, retaining nulls, NaNs and infinities;
   - `missing = true`: `col IS NULL`; combining this with bounds stays invalid;
   - `missing = false` or either bound: `col > -inf AND col < +inf`, then the requested
     `col >= minimum` / `col <= maximum` comparisons;
   - signed zeros compare numerically equally while original payload bits remain unchanged.

   It runs under the full optimizer. The finite guard is conditional; applying it to an
   unfiltered selection would silently remove supported rows.
5. **Declared fields.** Which fields are indexable is declared in the registry instead of the
   hand lists at `result_projection.rs:28-140` and `canonical_execution.rs:2345-2356`.

**Library facts** (*Tested*, probe-pruning, standalone crate on the pins):
- **Equivalence receipt.** The original standalone lowering/reference probe agreed on 63/63
  grid combinations and retained selected bits through `DataFrame::filter`. This is not a
  differential test against every production mode: production's all-None predicate returns
  all rows. Adoption must compare the revised conditional lowering directly with production
  `scalar_predicate`, including all-None and `missing = true` cases.
- **Soundness.** There were zero soundness violations over 3,391 containers and 20,107 rows,
  covering ±NaN with payloads, ±0, ±∞, subnormals and nulls.
- **Zeros.** DataFusion 55.1 normalizes ±0 in comparisons (`normalize_float_zero` in `apply_cmp`,
  physical-expr-common `datum.rs:26`, `:100`), so `-0.0 = 0.0` is true. Only the NaN half of the
  totalOrder trap remains, and the finite guard handles it. The signed-zero literals are therefore
  redundant but harmless.
- **Key pruning works.** With `step` given, a lookup kept exactly one container (probe 8).
- **Value pruning mostly does not.** It pruned 0% of containers for unordered values at 16 rows
  per batch or more.
- **Absent ids.** Min/max cannot reject every absent owner id. Pinned DataFusion also exposes
  `PruningStatistics::contained` for literal guarantees (`datafusion-pruning-55.1.0`,
  `pruning_predicate.rs`); `PrunableStatistics` itself returns unknown. **Conditional omitted
  opportunity:** if an existing exact block-owner summary is cheap to reuse, a custom adapter
  could reject absent ids. Do not build another per-row membership index or unbounded owner
  set merely to realize this capability.

So the benefit of R4 is the removal of per-row locators and their index writes, plus key pruning.
"Every Float64 column becomes prunable" is sound but is not claimed as a benefit.

**Conditions and limits.**
- Live-sample files keep their one-sealed-file-per-batch shape; statistics travel in the same
  acknowledged append.
- Pruning granularity is the IPC record batch, so record-batch row count becomes a tunable.
  Probe 8's containers were record batches.
- §20's single-decoder choice is kept. Parallel multi-partition decode would be an addition with
  an admission tradeoff, so it is not recommended here.

**Alternatives.**

| Alternative | Assessment |
|---|---|
| Current baseline | Correct and bounded; write work and guarded round trips scale with variables × constraints × steps rather than with payload blocks. |
| Array record ids `[result_set, output, partition, row]` with range scans | Keeps per-row writes; secondary. Useful only if some per-row locator must survive. |
| Parquet result payloads (page index, bloom filters on declared owner keys, `ParquetAccessPlan`) | **Not selected.** The selected locator-removal benefit does not require a format change; replacing live-sample IPC adds an unqualified storage/decoder transition. Parquet float-statistic NaN/zero handling needs an explicitly conservative pruning contract, not an assumption that an `Exact` label proves IEEE selection equivalence. Native page/bloom access remains a credible alternative if later workload evidence warrants that transition. Bloom filters are not the only possible absent-id mechanism; see `PruningStatistics::contained` above. |
| Simplest viable | Keep the locators, lower the predicate to DataFusion and set `Inexact`. This removes the bespoke predicate but none of the write volume. A partial step. |

**Preserved.** Exact bits, missingness and signed-zero semantics are kept. SurrealDB remains the
membership and commit authority. The protected file, ticket and preflight are unchanged.

**Consumers stop:** maintaining locators; replay comparison over locators; the 18,724-row extent
caps; the hand-built Arrow predicate kernels; the SurrealDB refusal branch for scalar predicates once all replacement consumers are verified.

**Reverse check.**
- **P1.** Guarded append transactions remove O(rows) locator and index writes in favor of O(blocks)
  metadata. The effect on 16-lane contention and throughput is unmeasured.
- **P3.** IPC stays a tactical payload referenced from SurrealDB.
- **DP-19.** Unchanged.

### 4.3 C3. Identity is carried by spelled names and rendered text

**Diagnosis.** Explains [DA-001](../audit/semantic-model-and-authority.md#da-001),
[DA-002](../audit/semantic-model-and-authority.md#da-002),
[DA-003](../audit/semantic-model-and-authority.md#da-003) and the non-result sites of
[DA-032](../audit/semantic-model-and-authority.md#da-032). It is also the identity half of
[DA-027](../audit/execution-fit.md#da-027).

- **Specialized model.** Specialization hands the compiler authored-syntax `Expr` trees whose
  resolved references are names `s_<32 hex>`. These are renamed to `local_<n>` and then to
  `input_<n>`/`member_<n>`, and body identity frames *rendered text*. Blueprint §7.1 chooses this
  deliberately ("no intermediate IR is produced or stored").
- **Derived identities.** They are minted from call-site literal preimages, and other crates
  re-derive them.
- **Expression fields.** They are plain `Utf8`, so grammar and role are restated in four
  inventories.
- **Relation policies.** They name relations by literal strings although generated
  `RELATION_ID`/`NAME` handles exist.

The common cause is that the model does not carry the identities and relationships its consumers
need, so each consumer reconstructs them from spellings. AP-04's adequacy half fails here, not
because there is no owner, but because the owned representation omits distinctions (resolved
reference vs binder vs parameter) that consumers depend on.

**Correction (project-owned; in-use libraries supply only the machinery).** No in-scope library can
own a pre-admission resolved representation:
- Symbolica atoms cannot carry physical types, binders or host guard semantics.
- Admission must precede Symbolica, and that guard-survival strength must stay.

*R5, a resolved, identity-typed specialization representation.*
- **Representation.** Typed variants for resolved symbol reference, binder, parameter and literal,
  over a `typed-index-collections` `TiVec<SymbolIx, Symbol>` symbol table.
- **Typing environment.** Use an identity-aware lookup behind the existing `TypeLookup`
  contract (`crates/pse-modeling/src/types/environment.rs`). A borrowed view over the
  authoritative symbol table is the simplest candidate; an `rpds` persistent environment is
  appropriate if snapshots/overlays need persistence (the checker already uses it in
  `check/phases.rs`). Both remove per-site `BTreeMap<String, Type>` rebuilding. Container
  choice is implementation design, not a second type authority.
- **Dependency edges.** The compiler's petgraph member graph is built from identities rather than
  parsed names.
- **Body identity.** Frame the canonical resolved form rather than `render_expr` text,
  preserving binder scope, coordinate/parameter roles, physical/refinement contracts,
  consumed functions, guards, validity and premises. Use scope-local positional roles for
  cross-instance sharing and bind actual instance identities separately; hashing each
  instance's global SymbolIds into the shared program would destroy that sharing. A pure
  renderer change then stops re-keying mathematical bodies.
- **Derived identities (DA-002).** Each derivation gets its own catalogued `pse_ids` frame variant,
  as §5.3 already asks, instead of `named_id(parent, "<literal>")`. The specialized model records
  the relationships consumers now re-derive (closure → conservation row), so the
  `let … else { continue; }` at `workflow/modeling/cases.rs:2695-2700` can tell "not a numerical
  target" from "lookup failed".
- **Expression fields (DA-003).** The registry's existing `pse.expr_dsl` field extension
  (`crates/pse-schema/src/model/extension.rs`) is extended to carry grammar and role as Arrow
  field extension metadata (arrow `canonical_extension_types`, `try_extension_type`, already
  used). Today `pse.expr_dsl` is `VersionOnly`, not a grammar/role schema. Its metadata contract
  and generated consumers therefore evolve explicitly; merely attaching the existing marker
  does not establish those distinctions. The collector, selector inventory and binding rule
  are then generated from the new declaration.
- **Identity evolution.** New `pse_ids` frame variants and resolved body framing need explicit
  version/domain separation. Existing durable artifacts must retain their recorded interpretation
  or become explicitly incompatible; no current frame silently acquires new meaning.

*R6, generated relation handles and declared membership.*
- Relation membership, scalar indexing, serde lists and analysis dispatch use the generated
  `RELATION_ID`/`NAME` handles.
- Artifact kinds declare their member relations through the registry's existing `artifact_profiles`
  (`crates/pse-schema/src/catalog/products.rs:12-58`) rather than hand lists
  (`workflow/artifacts.rs:786-826`).
- A misspelled relation then fails to compile instead of silently skipping a coverage check.

**Alternatives.**

| Alternative | Assessment |
|---|---|
| Current baseline | Correct where rendering is canonical and injective; every renderer or unit-spelling change is an identity change; misresolution refuses rather than mis-binding. |
| Symbolica function-atom formals (`Indeterminate::Function`; Symbolica's own `Dualizer` builds its parameters as `function!(dual_arg, index)`, `evaluate/dual.rs:558-570`) in place of the registered slot pool | **Candidate**, post-admission only. It removes the global symbol-pool growth but not the pre-admission naming problem. **Condition:** the pool exists to fix Symbolica's registration-order-dependent canonical term order (`pse-math/src/library.rs:36-39`), so the candidate must keep term order independent of registration order across processes (DP-11). Probe needed for `Dualizer`, `derivative` and `export_cpp` acceptance. |
| Reserve the `s_` prefix in the grammar | Simplest viable guard against collisions; does not remove re-derivation or rendered identity. A stop-gap. |

**Preserved.** Guard survival, positional cross-instance sharing, conservation by construction,
value-free bodies (rebind) and `member_id`'s well-layered frames.

**Consumers stop:** stripping `s_` and parsing hex; re-deriving conservation rows; restating field
grammars; string relation lists.

**Reverse check.** No effect on P1–P4. Derivatives are untouched. This changes blueprint §7.1
([RC01](#rc01)).

### 4.4 C4. Preparation repeats and then discards its own work

**Diagnosis.** Explains [DA-030](../audit/execution-fit.md#da-030) (re-diagnosed by F09),
[DA-027](../audit/execution-fit.md#da-027), [DA-028](../audit/execution-fit.md#da-028),
[DA-029](../audit/execution-fit.md#da-029) and the Salsa part of
[DA-022](../audit/local-reasoning-and-assurance.md#da-022).

*Retention is the dominant cause at defaults; the audit's granularity diagnosis is secondary.*
- **Rotation.** After each request, `trim_queries` triggers LRU eviction. If retained entries or
  known bytes still exceed `WorkspaceLimits`, it **rebuilds the whole database** from inputs
  (`crates/pse-compiler/src/workspace.rs:1737-1752`). The defaults are 16,384 retained entries,
  256 MiB known bytes and `query_values: 64` (`workspace.rs:95-104`; §14.3.2).
- **Footprint.** On the reference `ThreeUnits` package, measured with the limits lifted, the
  retained-usage estimator reported about 54.5k entries and about 606 MB right after publish. The
  footprint is dominated by granular checking of the whole package library: 2,743 declarations,
  each with `parsed`, `typed` and phase memos. That figure comes from the estimator (Salsa
  `memory_usage` plus the hand census), not from resident memory.
- **Consequence at defaults.** Every publish rotates, and an unrelated edit re-runs every granular
  query, `selected`, `specialized`, `projection` and all 143 `semantic_body`s, reusing 0 bodies.
  A no-edit re-prepare re-runs 79 bodies because 143 exceeds the LRU of 64 (*Measured*, probe B2).
- **Consequence with limits lifted.** Only `selected` and `projection` re-execute and `specialized`
  backdates. That confirms the second, smaller cause: `projection` reads the package-wide `Catalog`
  input directly (`executable.rs:459`, `:1033`).
- **AP-07.** An edit's cost scales with the size of the whole package library, not with the edit.

*Evaluation granularity: a probe reversed the inventory's mechanism.*
- Bodies are admitted one per scalar output, with transitive members inlined
  (`executable/grouped.rs:956`). Dispatch groups are expanded back per instance by `bound_bodies`
  (`executable.rs:401`) and re-identified by a second hash.
- On the BT_PR flash (*Measured*, probe A, Symbolica `count_operations` after construction, values
  / first / second order, totals over 19 instance groups):
  - **Whole-group Dualizer.** One `evaluator_multiple` over all substitutable outputs of a group
    with a Dualizer cost 177 / 6,206 / 505,003 operations and about 19 s to build at second order.
    Taylor work follows the 75-coordinate union, not actual dependence.
  - **`merge`.** It reproduces those counts exactly.
  - **Symbolic per output, then joint.** Symbolic `derivative` per output over its own coordinates,
    then one joint `evaluator_multiple`, cost 177 / 292 / 750. The per-output inlined baseline is
    223 / 901 / 4,449. Today's census is 303 / 705 / 1,843 but uses a different counter.
- **No shared members.** The fixture had zero `let` members read by two outputs, so DA-028's
  "members recomputed k times" did not appear; the value-order saving is cross-output CSE. The
  audit's own example (PR mixing-rule `let`s read by fugacity rows) sits in the 16 non-substitutable
  PR phase instances, all single-output in this fixture. DA-028's magnitude is therefore
  unresolved, not disproved.
- **Where the cost is.** It is in the non-substitutable provider and branch bodies, which no
  grouping reaches: 13,854 / 57,250 / 201,392.
- **Limits of the evidence.**
  - These are operation counts only. Nothing was evaluated, and nested implicit bodies were
    outside the hook.
  - Variants (b)–(d) were built from flattened, obligation-free expressions, and (d) computed the
    full upper Hessian triangle. The figures are an upper bound for a guard-free program (see R7).
  - The probe did not log whether the substitutable outputs carry `Domain` or `Require` stages.

*Scans.* DA-029's nested whole-model scans are confirmed in source:
- a per-row fixed point over member bindings by name (`executable.rs:614-655`);
- `rows.iter().find` per output;
- the accumulator-term scan per contribution;
- the full symbol-table re-render per contribution × coordinate.

**Correction.**

*R7, evaluation granularity: extend the existing symbolic product routes over the stage program.*

R7 is defined over the **guarded stage program**, never over the flattened per-output expression.
`PreparedBody::expression` can exist across `Domain`, `Applicability` and `Require` stages
(`crates/pse-math/src/execution.rs:1983-1991`). In that flattening:
- `Domain` and `Applicability` tokens become constants (`:6357-6420`);
- only value-order `Require` conditions become obligations (`:6553`);
- an order-qualified `Require` (`guarded.rs:96`) disappears.

A joint evaluator built over flattened expressions would therefore lose guard semantics.

- **Existing routes to extend.** The repository already owns symbolic routes:
  - `SelectedFirstRegion` (`execution.rs:4486-4535`);
  - direct `WeightedHessian`, Jacobian and transpose-Jacobian action products
    (`products.rs:186-330`), requested by `ProductKind` (`jets.rs:31-42`) over consumed support
    pairs only.

  Today these are restricted to all-`Block` or `is_flat_arithmetic` bodies (`execution.rs:2270-2274`,
  `:1638`), and `SelectedFirstRegion` requires every region input to be an original scalar
  (`execution.rs`, `inputs.iter().all(|&slot| slot < allowance.scalar_inputs)`). R7 is a
  **proposed region-by-region extension**, not a mechanical removal of these restrictions:
  - within an eligible arithmetic region of a dispatch-group instance, build the requested
    product symbolically (Symbolica `derivative`) and one joint `AtomCore::evaluator_multiple`
    (symbolica-3.0.1 `atom/core.rs:1042`) over the region's demanded outputs;
  - downstream of a provider, branch or intermediate-slot boundary, preserve existing
    Taylor/reverse chain propagation, or build local symbolic jets/actions with respect to
    region inputs and compose them through the existing derivative layouts and host stages.
    Treating intermediate slots as independent constants when differentiating with respect
    to original coordinates would lose first and mixed second derivatives;
  - host stages keep splitting regions and run before anything that depends on them;
  - `Require` is evaluated at the requested order;
  - the consumed-pair restriction and the product kinds are kept unchanged.
- **Demand.** The joint unit is the **demanded output closure per product**. It is charged under the
  existing `EvaluationLimits`, so observation-only outputs (terms, contributions) are not folded
  into solver-iteration programs, and residual/Jacobian programs are partitioned from
  observation programs.
- **Route choice.** Choose symbolic versus Taylor from deterministic eligibility and
  conservative work bounds before differentiation, and record the choice and its relevant
  premises in product identity. This is a prerequisite for reproducibility, not evidence of
  bit identity between different numerical routes.
- **Admission before differentiation.** Add the deterministic preflight needed for that choice;
  the current APIs do not already supply it. `SelectedFirstRegion::new` decrements
  `selected_partial_calls` inside its differentiation loop, while `construction::additional`
  calls a thread-local owner's `try_grow` (`pse-math/src/construction.rs`), which can refuse
  because of transient capacity. Transient capacity, cancellation, or a construction failure
  remains a typed refusal and never selects a different retained program. Preserve the existing
  operation, scratch and flattening limits (`products.rs:70-83`). Expression swell on deep
  expressions is unprobed beyond the original fixture.
- **Shared-program identity.** One program shared across the instances of a dispatch group must
  carry each instance's validity and closure `Require` stages and consumed physical premises in its
  identity, or take the bounds as instance parameters while keeping the stages.
  - The dispatch-group hash covers coordinate types, expressions, equations and functions only
    (`specialize/group.rs:123-200`).
  - Per-instance closure validity intervals (`executable.rs:427-452`) and per-output premises
    (`grouped.rs:178-196`, `:976-978`) are added later.
  - Per-instance checked-member attribution is kept.
  - So R7 removes `bound_bodies` re-expansion and the per-output re-normalization walk, but the
    body identity remains distinct from the group hash: it is re-framed over the resolved form
    (R5) plus validity and premises.
- **What probe A measured.** Probe A (*Measured*) built variants (b)–(d) from flattened
  expressions with no obligations (`probe_math_probe.rs:171`, `:209-215`), and (d) differentiated
  the full upper Hessian triangle. Its 41% and 58–59% reductions are therefore an **upper bound for
  a guard-free program**, measured against a different counter for "today". The second-order
  baseline that matters for NLPs is the `WeightedHessian` product.
- **Do not adopt** a whole-group `Dualizer` or `merge`; probe A refutes both.
- **Non-substitutable bodies (candidate, unprobed).** Inside the arithmetic between host stages,
  `arithmetic_region` stops at the first dependent `Block` (`execution.rs:4831-4835`), so each
  dependent `let` is its own evaluator.
  - Registering intermediate slots through `FunctionMap::add_aliases` (`function_map.rs:156`; the
    aliases "always inline in the scope where they are used"), with
    `EvaluatorBuilder::direct_translation(true)` (`:457`), would let Horner and CSE cross those
    boundaries while every host stage still splits regions. Aliases inline for evaluator
    construction; they do not automatically make `derivative(expression, x)` expand alias
    definitions. The candidate must establish derivative-chain handling and retain source-check
    attribution for eliminated producers. Direct translation is already the pinned builder's
    default; the opportunity is safe fusion, not merely setting that option.
  - Never use `InliningPolicy::Never`: a sub-evaluator cannot be vectorized
    (`library/admission.rs:1048`).
  - This targets where the cost actually is, but needs the alias probe before adoption.
  - Larger fused programs make one existing condition more pressing: `exact_evaluator` aborts the
    Horner search on cancellation without refusing the result (`library.rs:272-296`). A truncated
    evaluator must never be retained.

*R8, truthful bounded retention and a firewall for the catalog.*
- **Pinned mechanism and its limits.** Salsa 0.28.4 already has LRU on heavy granular queries:
  `parsed`, `signature_syntax`, `resolved`, `typed`, `complete`, `occurrence` and the checker
  phase queries (`pse-compiler/src/workspace/modeling/granular.rs`, tracked declarations and
  `configure`). They currently share `query_values` with the other query families.
  - Eviction removes values but keeps dependency metadata and revisions; `memory_usage` still
    counts the memo (`salsa/src/function/memo.rs`, `Memo::memory_usage`).
  - `IngredientInfo` exposes aggregate counts and bytes, not a public value-present memo
    count (`salsa/src/database.rs`, `memory_usage`, `IngredientInfo`). Zero reported heap is
    not a reliable value-presence test.
  - `set_lru_capacity` is count-based, not byte-weighted (`function/eviction/lru.rs`).
  - Changed evicted values cannot backdate without the previous value
    (`function/backdate.rs`, `old_memo.value()`); unchanged evicted memos keep their
    dependency verification behavior.
- **Retention policy.** Keep finite bounds on actual retained input/interned roots, memo
  metadata and known value bytes. Use the existing aggregate byte report plus the generation's
  project-owned shared-input census; do not replace real metadata growth with an unavailable
  value-only counter. Separate live-root/firewall retention from disposable results and give
  query families appropriate count capacities. Deriving capacities from a byte allowance
  needs conservative per-family sizing or adaptation; Salsa itself does not supply that policy.
- **Resident memos.** Accessors needed for backdating, `specialized`, and the admitted live
  root's semantic bodies stay resident under that root's retention allowance. An oversized
  live set still needs an explicit refusal/rotation policy; setting an unbounded LRU or retaining
  arbitrarily many roots is not bounded retention.
- **Immediate defaults.** Adjust documented defaults to cover the admitted reference workload
  and its live-root body set, subject to the enclosing memory budget. This fixes the demonstrated
  default mismatch but does not establish a scalable eviction policy for every admitted package.
  Rotation remains an honest last resort for finite non-evictable or metadata growth; it must
  stop being routine for the supported ordinary workload.
- **Measurement candidate.** `heap_size` on `#[salsa::input]` can contribute to the library's
  aggregate report, but replacing the shared `RetainedInputs` census requires explicit
  accounting ownership. Independent callbacks receive one input's fields and can double-count
  shared `Arc` roots or omit generation-owned maps/environment. Keep the existing census until
  equivalence or a deliberately conservative bound is established; callbacks stay project-written.
- **Catalog firewall.** Route `projection`'s two direct `catalog.checked(db)` reads through
  tracked accessors keyed by the selected package (`physical_scope(selected)`,
  `function_contracts(selected)`). Equal results backdate, provided the firewall values remain
  resident, so unrelated edits stop propagating into projection (original probe B2 with limits lifted).
- **Demand.** Move observation demand of **existing** symbols out of `Request`. Original probe
  B1 found it bookkeeping-only. Demand of a **new** `let` member changes the dispatch group
  (42→44 coordinates) and remains in specialization identity.
- **Acceptance.** At the explicitly selected new defaults, rerun probe B2: no routine publish
  rotation, 143/143 live bodies reused after the unrelated edit, and resident `specialized`
  backdating. Also exercise retained generations/membership growth and an oversized live root
  to show that metadata and values remain bounded without stale reuse. Do not label the old
  ≈606 MB estimator as resident memory or assume the new policy has already passed.
- **Second stage (Proposed, contingent on scale measurement).** Per-instance specialization
  through Salsa tracked structs with `#[tracked]` fields could localize structural edits.
  Accumulators and closure must still assemble at the root from original terms
  (`specialize.rs:3015-3083`), preserving conservation by construction. This larger change is
  unnecessary for the immediate F09 default mismatch.

*R9, index-based preparation scans.*
- Build a dependency index including derivative-rate symbols, free inputs, expression members
  and affected rows, then traverse consumers forward from rate symbols with petgraph `Bfs`/`Dfs`.
  The existing graph has dependency→consumer edges and only expression-valued symbol nodes
  (`pse-compiler/src/workspace/modeling/executable.rs`, graph construction); rates can have no
  node. `Reversed` traversal from rates on that graph is therefore not a drop-in replacement.
  A reverse search from each row is an alternative only with the corresponding explicit edge
  orientation. Keep raw numerical dependency distinct from validity/guard dependency in
  `MemberReads` (`executable.rs:1276+`).
- Replace repeated `iter().find`/`position` with local indexes built from existing identity
  maps once per projection; these local improvements can precede R5. R5 supplies the final
  resolved representation and an authoritative lookup that removes symbol-table rebuilding.
  Preserve cancellation, topological ordering, guard dependencies and diagnostic lineage.

**Cross-lane caches (DA-030's second half).**
- **Same-lineage lanes.** These lanes serialize on one `Arc<Mutex<CompilerWorkspace>>`. Salsa
  `Storage: Clone` handles share memos and block on identical in-flight queries, but any setter
  cancels every other handle (`salsa-0.28.4/src/storage.rs:160-190`).
  - Sharing one database is a **candidate** for live lanes of one revision.
  - It is **remedy open** for durable lanes pinned to an older revision: a head publication would
    cancel them and regress P1.
- **Hand-maintained keys.** Cross-lane keys built from process-pointer addresses
  (`math/preparation.rs:23-63`) and field-listed keys (`BasisKey`, `view_key`) get no library
  remedy here. They stay a G6 audit item for the plan, so DA-030 is addressed **in part**.
- **Pure products.** `quick_cache::sync::Cache::get_or_insert_async` can fold flights and weighted
  retention into one structure, but only for pure preparation products. Its caller-owned loader
  must not replace the detached completion that native-work flights need for the drain.

**Alternatives.**

| Alternative | Assessment |
|---|---|
| Current baseline | Correct and conservative; rotates on every publish at defaults; reuse depends on hand-keyed runtime caches. |
| Simplest viable | Raise `WorkspaceLimits` defaults above the footprint. That removes F09 at once but scales memory with library size rather than with use, and leaves rotation as the eviction policy. Acceptable as an interim setting. |
| Salsa `persistence` | **Not selected for the immediate remedy.** P3 forbids a second state/commit/lifecycle authority, not a reconstructible cache whose premises remain canonical. Persistence would still need a justified reuse benefit, serialization for Symbolica-holding values, version/invalidation and retention ownership; no such simplification was established here. |

**Preserved.** Values stay out of Salsa (rebind). Persisted dependencies remain the durable reuse
authority (§14.4). Premise-checked retention reuse is kept, and so is the cancellation drain.

**Consumers stop:** relying on routine whole-generation rotation for ordinary reuse;
observation-only demand in specialization identity for existing symbols. Shared-input accounting
stays until a sound replacement is established.

**Reverse check.**
- **P1.** Unaffected; the shared-database variant is gated as above.
- **Guards and derivatives.** Preserved **only** with R7 defined over the stage program: host
  stages split regions, `Require` is evaluated at the requested order, and per-instance validity
  and premises enter the shared-program identity. Under those conditions R7 changes the derivative
  *construction* route, not the derivative *source* vocabulary, and PS-07 holds. A joint evaluator
  over flattened expressions would regress guard survival and is not recommended.
- **Rule impacts.** §7.1 body granularity ([RC02](#rc02)) and §14.3.2/§14.4 retention
  ([RC03](#rc03)) change.

### 4.5 C5. Orchestration copies the run lifecycle and reinterprets owners' rules

**Diagnosis.** Explains [DA-012](../audit/boundaries-and-composition.md#da-012) (with F01),
[DA-009](../audit/structure-and-invariants.md#da-009),
[DA-013](../audit/boundaries-and-composition.md#da-013),
[DA-014](../audit/boundaries-and-composition.md#da-014),
[DA-015](../audit/boundaries-and-composition.md#da-015),
[DA-021](../audit/local-reasoning-and-assurance.md#da-021),
[DA-006](../audit/semantic-model-and-authority.md#da-006) and
[DA-031](../audit/semantic-model-and-authority.md#da-031).

The runtime writes the public run lifecycle out four times inline plus a horizon version, and the
copies have diverged:
- **Horizon cancellation.** The horizon driver shadows the caller's `CancelSource` with
  `crate::CancelSource::new()` (`workflow/horizon/driver.rs:142`). Caller cancellation after
  admission was ignored: 20 of 20 steps completed (*Tested*, probe C).
- **Paired outcomes.** Outcomes travel as two independent five-variant enums whose pairing is not
  structural (`run.rs:201-233`).
- **Hand-composed operations in `pse-py`.** Thirteen `ModelingPackage` methods are chains composed
  by hand. Eleven of them hard-code default `Profile`/`NumericalInputs`, and the binding
  hard-codes KINSOL for recycles. Meanwhile the runtime already has a **versioned,
  document-driven operation descriptor**: `StudyOperation { version: Version<7>, preparation:
  PreparationSettings, operation: OperationRequest, … }`, with `OperationRequest` covering
  DeclaredCase, Simulation, Fit and Horizon (`workflow/study_operations.rs:305-349`). `pse-py` uses
  neither `StudyOperation` nor its `OperationRequest`, and has its own three-kind
  `PreparedOperation` (`crates/pse-py/src/workflow.rs:2427-2431`). `StudyOperation` is the study's
  persisted record; the reusable part for hosts is `OperationRequest` plus `PreparationSettings`.
- **Restated owner rules.**
  - The predicted-start policy exists twice.
  - Annotation lowering exists in three modes.
  - Steady view choice restates the compiler's promotion predicate.

**Correction.**

*R10, one run supervisor on the cancellation tree the runtime already wraps.*
- **Supervisor.** A generic supervisor holds the skeleton once: mint, attempt, register, dispatch,
  admission wait, join, publish. Each operation kind supplies only its native closure and finish,
  and horizon is one kind of it.
- **Run token.** The run's token is `parent.child_with_deadline(deadline)` (`crates/pse-runtime/src/cancel.rs:48-52`),
  which uses tokio-util `CancellationToken::child_token` (`cancellation_token.rs:204`).
- **Lease.** The lease holds a `drop_guard` (`:275`) wrapped so that `run.stop_requested` is still
  recorded.
- **Native bridge.** Exactly one bridge projects the tree onto the flat native `Arc<AtomicBool>`
  leaf: on `token.cancelled()` it calls `leaf.cancel()`, awaits the drain, then returns
  `Cancelled`.
  - This mirrors `math/jobs.rs:1461-1466`: a result that completes after cancellation is
    discarded, and "cancellation wins".
  - The flat leaf stays, because `CancellationToken::is_cancelled` takes a mutex and native
    callbacks poll it.
  - The publish step labels a partially completed horizon as partial.
- **Immediate fix (F01, *Tested*).** `driver.rs:142` becomes `cancel.child_with_deadline(None)`.
  With it, the run was cancelled after one step and the join took 0.5 ms. It must be a child,
  because the lease cancels its token on drop and that must not cancel the caller.
  - The child also inherits the caller's enclosing deadline (`cancel.rs:44-51`), which probe C did
    not exercise.
  - F01's acceptance therefore adds a deadline-expiry horizon: a typed TimeLimit and a trajectory
    marked partial.
- **Lane drain.** The worker's `drain` (`worker.rs:377`) becomes a child of `stop`. Stop already
  cancels drain (`worker.rs:439`, `:446-448`), so behaviour is preserved.
- **Explicit-strategy family (DA-012's second family).** Conic, recycle and block-initialization
  strategies return through the math-layer `SolveHandle`. They have no `RunResult`, durable attempt,
  completion record or replay.
  - Running them through the supervisor would add durable lifecycle to them, which is a
    functionality addition with a cost. That is a **maintainer decision**, not recommended here.
  - The recommended correction is to state the exception in §17/§19 ([RC13](#rc13)), so the
    supervisor stays the one owner of what *is* a run.

*R11, the runtime-owned operation request is the one operation input, and outcomes are paired by
construction.*
- **Operations.** `pse-py` and studies share `OperationRequest` plus `PreparationSettings`, **not**
  `StudyOperation`.
  - `StudyOperation` is the persisted occurrence record "replayed by either executor"
    (`study_operations.rs:336-352`). It carries `Version<7>`, `deny_unknown_fields` and study-only
    `output`/`source` fields, and it stays study-owned.
  - Adding Shooting to `OperationRequest` would change that persisted format and make Shooting a
    study operation, which it is not today. That is a **maintainer decision**. If it is taken, it
    needs a version bump with a stated replay or regeneration path ([RC12](#rc12)). The
    in-process superset may wrap shared request structs plus host-only Shooting, with an
    explicit typed projection/refusal into the study-admitted set. Adding Shooting directly to
    the serialized `OperationRequest` does change the persisted operation format.
- **Runtime entry.** A runtime entry composes `declared_execution` and the operation under
  `PreparationSettings`. The binding's chains, hard-coded defaults, deadline rule, synthesized
  progress event and KINSOL recycle route move into testable runtime code; the route is read from
  the prepared strategy.
- **Declared-procedure dispatch.** One dispatch over the request's declared procedure kind replaces
  the per-consumer subsets (`modeling/declared.rs:502-545`; `modeling/conformance.rs:1386-1600`;
  `bindings.rs:219-229`). Studies keep their narrower admitted set as an explicit typed refusal.
- **Admission and deployment (rest of DA-015).**
  - A runtime-owned document-admission entry replaces the composition `pse-py` performs
    (`workflow.rs:977-1012`). Ingress re-validation of new external input stays as a deliberate
    trust transition.
  - One deployment-sequence owner replaces the copies in `pse-py` (`workflow.rs:692-760`) and
    `pse-worker` (`xtask/src/bin/support/deployment.rs:30-110`). ADR-0164's per-host deployment
    assertion is kept as a call into that owner.
- **Paired outcomes.** `RunResult` holds one enum whose variants carry both request and
  `Result<report>`. That is plain Rust, which no library replaces.
- **Operation set.** Add strum 0.28 `EnumDiscriminants` (`strum_macros-0.28.0/src/lib.rs:950`; not
  derived anywhere today) with `#[strum_discriminants(derive(enum_map::Enum))]`, so the
  operation-kind set is named once. `enum_map::EnumMap` (enum-map 3.1.0, already a direct
  dependency) then gives exhaustive per-kind tables.
- **Validated constructors (DA-015 invariants).**
  - Use nutype `PositiveCount` (`crates/pse-model/src/scalars.rs:65`) for `Limits`/`LimitsWire`.
  - Validate `LeasePolicy` through `#[serde(try_from = …)]`, which schemars follows, so
    "heartbeat shorter than lease" and its defaults have one owner.
- **Explicit policy inputs (DA-021).** Route policies take explicit inputs (for example, the
  foreign allowance as a value) instead of reading `Runtime` service handles. That lets LM
  eligibility and covariance withholding be tested with their actual inputs.

*R12, value binding for parameter-valued dynamic and fit study points.*
- **Today.** Study overlays bind values only for steady DeclaredCase Solve and refuse everything else
  (`workflow/bindings.rs:146-154`).
- **The primitive that exists.** `IntegratedExperiment::bind` writes converted consumer values into
  integration-vector **parameter columns** without re-preparing (`workflow/integrated.rs:61-70`).
  Fitting, shooting and horizon build it (`fitting.rs:370`, `fitting/modeling.rs:730`,
  `shooting.rs:437`, `horizon/driver.rs:919`); plain Simulation builds none.
- **Recommendation.** Admit **parameter-valued** overlays for Fit and Simulation points through
  R11's request:
  - for Simulation, build the parameter bindings at preparation the way fitting does;
  - each point's start is the declared start, or a seed recorded as that point's input dependency
    (PS-08, PS-11);
  - consistent DAE initial conditions are recomputed per point.

  This completes an existing study mode, as the brief allows.
- **Left out.** Binding initial states, schedules or fit data per point is a **lead**; it is not
  covered by the existing primitive.
- **Lead only.** Retaining IDAS memory across points through `IDAReInit`/`IDASensReInit` (in use)
  must either be shown bit-identical to a fresh integrator or be recorded as a dependency of the
  result.

*R13, single owners for start policy, annotation lowering and view choice (project-owned).*
- Predicted-start eligibility and fallback move into the pure `pse-model::study_policy`. Executors
  keep only mechanics.
- The compiler publishes one numerical-annotation lowering per analysis mode, with deliberate
  per-mode differences recorded, and one view choice, as Salsa tracked outputs consumed by steady,
  implicit and dynamic orchestration.
- No library owns these decisions; Salsa only hosts them.

**Alternatives.** The baseline keeps five lifecycle copies, so every lifecycle change is made five
times. The simplest viable step is F01 alone, which fixes the observed divergence but not the
cause. No library-owned workflow engine is in use, and none is needed: the supervisor is a short
Rust skeleton over tokio-util.

**Preserved.**
- The drain: compute is released only after the native join.
- Lanes keep in-task `FuturesUnordered` draining. Do not use `JoinSet`/`AbortOnDropHandle` for
  lanes, since abort-on-drop breaks "no return drops running lanes".
- Fences and "cancellation wins" still decide durable seals.
- Typed refusal before effects.

**Consumers stop:** forwarding closures per kind; `_ => None` request/report mismatches; binding
defaults; re-implementing predicted-start fallback.

**Reverse check.**
- **P1.** Unchanged.
- **P2.** Durable work started through the durable route belongs to the worker lane's token tree
  (`stop` → `drain`), not to the caller's lease, so a dropped caller handle still does not cancel
  durable work.
- **Value rebind.** Strengthened: shared request/preparation semantics serve common in-process and study modes through explicit admission, while `StudyOperation` stays the study-owned persisted record (P2 replay unchanged unless the operator takes the Shooting decision).

### 4.6 C6. Solver capability and composition knowledge sit outside the adapter seam

**Diagnosis.** Explains [DA-016](../audit/boundaries-and-composition.md#da-016),
[DA-017](../audit/boundaries-and-composition.md#da-017),
[DA-018](../audit/boundaries-and-composition.md#da-018), the native part of
[DA-022](../audit/local-reasoning-and-assurance.md#da-022), and F02.

- **Static capability records.** Capability is one static record per backend.
  - Uno advertises `ExactHessianOrLimitedMemory` (`execution/uno.rs:20`) but admits only
    `HessianMode::LimitedMemory` (`:73-81`).
  - KINSOL lists `DeclaredFixedPoint` as automatic (`execution/kinsol.rs:25-26`), but routing
    never produces that class (no occurrence in `routing.rs`).
- **Runtime-built sessions.** The runtime therefore builds `kinsol::Session` directly and calls
  `quality::qualify` itself (`math/initialization.rs:1271`, `:1288`;
  `workflow/strategies/conditional.rs:912`, `:922`).
- **Named backends.** POUNCE's second-opinion ladder is wired by `Backend::Pounce` in two runtime
  owners (`math/solves.rs:1127`, `:1278-1284`; `math/strategy/target.rs:97`, `:401`).
- **Uno status mapping.** It uses bare integers (`uno.rs:114-131`) and maps iterate 5 to
  `Termination::Infeasible`.
  - In the Uno C API, 5 is `UNO_INFEASIBLE_SMALL_STEP`, and only 3
    (`UNO_INFEASIBLE_STATIONARY_POINT`) is a stationary point of the constraint violation
    (`Uno_C_API.h:52-59`).
  - Ipopt's analogue maps to `Numerical` (`ipopt.rs:59-61`).
  - An `Infeasible` category with a candidate becomes a least-infeasible label
    (`quality.rs:558-570`). A small-step stall is therefore published as a local infeasibility
    conclusion (*Tested* mapping, probe D, F02).
  - Probe D refuted the inventory's second Uno claim, a masked bridge latch: the latch block and
    `CallbackState::finish` override the native category.

**The Uno patch, hunk by hunk** (P4; `docker/solvers/uno-pse.patch`, read in full):

| Hunk | Content | Class |
|---|---|---|
| `HiGHSSolver.cpp` preamble | Thread-local scope callback; deadline (`reason 1`) and memory (`reason 2`) throws; feasibility and accuracy counters and readback; materialization admission (`count*64 + n*16` against the foreign allowance) | **Integration** (cancellation, deadline, admission, observation) |
| Constructor: `output_flag` bool fix, `threads=1`, `parallel=off` | Serial controls under the admitted thread budget | **Integration**. Determinism-bearing: R15's declaration names it |
| Constructor: `primal/dual_feasibility_tolerance`, `optimality_tolerance` from Uno's options | HiGHS QP solves to the outer accuracy instead of its defaults | **Numerics** |
| Constructor: `qp_regularization_value = 0` | Removes HiGHS's 1e-7 diagonal perturbation | **Numerics** |
| `set_remaining_time`, time-limit status checks | Task clock propagated to each HiGHS owner | **Integration** |
| `seed_feasible_point` + `qp_allow_hot_start` + a second `Highs` | A feasibility LP before every constrained QP; its solution and basis seed the QP; LP infeasibility sets the subproblem status | **Numerics** (an added phase-1 step; changes iteration path and infeasibility detection) |
| `HiGHSQuadraticProgram`: remove the "no explicit Hessian" refusal; materialize the lower triangle of Uno's positive Hessian operator through n Hessian-vector products | Enables HiGHS-QP SQP with L-BFGS; O(n²) storage | **Numerics-bearing capability extension** (the curvature is Uno's; the representation and the admitted route are new) |
| `bad_alloc` catches; non-finite action check | Memory refusal and validation | **Integration** |
| `NoRelaxation.cpp` option override | Publishes the actual globalization selection through the readback route | **Integration** (observation). Verified at Uno `805a27ec`: `option_overrides` flow into `Result` and are read only by the C API readback (`interfaces/C/Uno_C_API.cpp:1187`) |

So the audit's "fork of library internals" framing is **challenged in part**: most hunks are P4
integration. Four hunks carry numerics. The dense route bounds SQP at about 1,450 variables under
the default 64 MiB foreign allowance. That bound is source arithmetic and is declared in neither
the record nor the architecture.

**Correction.**

*R14, settings-dependent capability and seam-owned compositions.*
- **Capability.** Replace the single static record with `capability_for(settings)`, published as
  the union plus per-profile rows, so Uno's SQP-LBFGS and SLP profiles state their own curvature.
- **Fixed-point problems.** Add `Problem::FixedPoint`/`Problem::Picard` representations so
  `routing::problem_classes` emits `DeclaredFixedPoint`. The seam then owns admission and
  qualification through a **map-specific route**, not the square-root route.
  - The roots runner's residual normalization, Jacobian-pattern check, warm transport and retained
    reuse (`execution/runner.rs:305-356`) do not fit maps as they stand.
  - The map and its variables use one shared scaling.
  - FixedPoint and Picard make no derivative demand: the Picard operator is never advertised as a
    derivative (`kinsol.rs:222-262`).
  - The declared restart schedule is carried over. Restarts accept only authorized starts and
    refuse warm starts (`kinsol/restarts.rs`).
  - Qualification uses `FixedPointOracle::original_residual`.
  - The session stays fresh per application, as both runtime paths do today, until the KINSOL
    neutral-reset candidate is shown bit-identical. Otherwise reuse is recorded as a warm-start
    dependency.
  - The KINSOL adapter maps the variants onto its existing `kinsol::Function` variants
    (`kinsol.rs:243-258`), and KINSOL keeps iterating (PS-09).
- **Recovery profiles.** Add `BackendExecution::recovery_profiles(.., permission)`, defaulting to
  empty. POUNCE implements it with the rungs it already lowers from
  `pounce_algorithm::second_opinion::second_opinion_rungs` (checkout
  `crates/pounce-algorithm/src/second_opinion.rs:241`; `pounce/profiles.rs:36-131`).
  - The replacement-start permission (`StartOrigin::Auxiliary`) stays with its semantic owner and
    is passed in. The adapter never decides it.
  - Perturbed starts are recorded as dependencies of the result they produce.
  - Every rung stays a separately admitted and separately qualified candidate.
  - Do not adopt the public `pounce_restoration::second_opinion_driver::run_second_opinion_ladder`
    (pounce-restoration `second_opinion_driver.rs:139`). It drives an `IpoptApplication` and would
    hide the attempts inside one native call.
- **Capability facts.** Separator consumption and KINSOL JVP demand become capability facts the
  adapter answers.
  - For the arclength corrector, only the backend-suitability part (Hessian modes) moves into the
    adapter.
  - The path preconditions stay with the path owner (`math/solves/paths.rs:974-984`): explicit
    physical start, equal numerics key, bit-equal scale, presolve off.
  - The runtime stops naming backends, and the `pse-py` KINSOL route goes (R11).
- **Composition-only methods.** PETSc NASM and pseudo-time are reported as "composition-only" until
  the seam carries a decomposition input.

*R15, Uno truthfulness and a declared binding.*
- **Constants.** The bridge `static_assert`s the `UNO_*` constants (`Uno_C_API.h:44-59`) against
  its exported codes, and `uno.rs` matches named constants.
- **Mapping.** Iterate 5 maps to `Numerical`, matching Ipopt's small-step analogue (F02).
- **Declaration.** The Uno record and ADR-0157 declare the patch as an integration binding with
  four named numerics hunks and the determinism-bearing serial controls, and declare the dense SQP
  size scope.
- **Remedy open.** Moving those numerics out of the patch has no in-use route that keeps the
  SQP-LBFGS capability:
  - every trust-region SQP route in Uno `805a27ec` sends curvature subproblems to BQPD or HiGHS,
    BQPD is not in the image, and HiGHS QP needs an explicit Hessian;
  - Uno's `"ipopt"` interior-point preset over MUMPS/SPRAL would be a different method, which is a
    major addition, so it is not recommended here.
- Upstreaming the HiGHS tolerance and operator-materialization changes is a process lead, not a
  library capability.

*R16, library-reported native memory where libraries expose it (candidates).*
- **FERAL, direct KKT path (candidate).**
  - Use `SymbolicFactorization::work_estimate().peak_bytes` (factor plus transient contribution
    blocks; `vendor/feral/src/symbolic/mod.rs:873-935` in the POUNCE checkout) as **pre-admission
    only**: after symbolic analysis, before numeric factorization.
  - It is an a-priori figure, since delayed pivots are a runtime property, so the post-factor
    accounting (`KktFactor::bytes()` with `factor_bytes`, `kkt.rs:762-764`) and the hard limit stay.
  - It refines, for this path only, the source-owned bounded FERAL profile of §18.8, which
    continues to cover POUNCE-internal FERAL ([RC15](#rc15)).
  - A stricter pre-admission could refuse cases admitted today, so it is not a clear win.
- **Candidates** (each needs its probe):
  - PETSc `PetscMallocSet` before `PetscInitialize` can track the `PetscMalloc`/`PetscFree`
    allocator domain (PETSc 3.24.0 `src/sys/memory/mal.c`), not automatically the entire foreign
    heap. Current `petsc.rs` already serializes PETSc through `PROCESS`, and process/init
    allocations can outlive attempts. Each tracked allocation must retain its original owner
    and size, so later free/realloc updates that owner even on another thread or after a
    different lane becomes current. Thread-local current-owner lookup alone is insufficient;
    process/init allocations need a separate process owner. MKL/BLAS and any bypassing
    allocations remain under the existing envelope until their coverage is established.
  - SuiteSparse `SuiteSparse_config_*_func_set` for KLU remains a candidate only if the actual
    allocation route is covered. The same original-owner provenance and lifetime rules apply.
  - Do not replace structural admission or the complete foreign envelope with allocator-domain
    accounting before bypass and attribution coverage is established. Cross-lane charging or
    premature release would regress P1/DP-20 and is not recommended.
- **Lead.** The project-structure size models have no in-use derive-based sizer (DA-022). A
  derive-based sizer would be a new library, out of scope.

**Alternatives.** The baseline is truthful at the refusal level, since refusals are typed before
work, but the published capability is not. The simplest viable step is correcting the Uno record
row and the mapping alone, which leaves DA-016's locality defect. A PETSc fixed-point route
(`SNESANDERSON`) through the new representation is a lead only: no benefit has been shown, and it
would add a second fixed-point path.

**Preserved.** Explicit selection, typed outcomes, original-space qualification, PS-09 library
iteration, and every P4 hook including the KLU observers and fixed-pattern application and the
IDAS checkpoint and replay budgets.

**Consumers stop:** naming `Backend::Pounce`/`Kinsol`; re-deriving JVP demand; reading
capabilities the record overstates.

**Reverse check.**
- **P1–P3.** Unaffected.
- **P4.** Respected: no hook is removed, and only the four numerics hunks are judged numerically.

### 4.7 C7. Boundary vocabularies, identities and documents are hand-paired

**Diagnosis.** Explains [DA-004](../audit/semantic-model-and-authority.md#da-004),
[DA-005](../audit/semantic-model-and-authority.md#da-005),
[DA-011](../audit/structure-and-invariants.md#da-011),
[DA-010](../audit/structure-and-invariants.md#da-010) (provenance) and
[DA-019](../audit/boundaries-and-composition.md#da-019).

- **Vocabularies.** `EnumValue<T>` implements only `IntoPyObject`, with an `OUTPUT_TYPE` hint
  (`crates/pse-py/src/enums.rs:15-26`). Inputs are parsed by hand or through serde over a
  `Value::String`.
- **Documents.**
  - `documents.rs` carries 73 hand `NAME` constants.
  - About 37 methods return bytes that Python decodes into a hand-chosen type.
  - The import check compares the package version and registry fingerprint only
    (`python/pse/_build.py:110-120`).
- **Identities.** They cross as hex `str` in documents.
- **Physical inputs.** Side-route inputs are bare floats or positional vectors.
- **Covariance.** It is admitted on its face; `run_id` is never resolved.
- **Python operator tooling.** It restates conformance defaults, vocabularies and the diagnostics
  launch protocol, and `scripts/surreal_server.py` writes canonical tables with f-string SQL.

**Correction.**

*R17, typed vocabularies and identities at the PyO3 boundary.*
- **Vocabulary input.** Add `impl FromPyObject for EnumValue<T>` with
  `INPUT_TYPE = type_hint_identifier!("pse.contracts.enums", T::NAME)`. The pyo3 0.29.2
  `FromPyObject::INPUT_TYPE` machinery is used by the crate itself (`conversion.rs:520`). The input
  is strict: an `isinstance` check against the generated `StrEnum`, then strum `FromStr`.
- **Missing vocabularies.** Declare scientific artifact kind, artifact owner kind and terminal class
  as registry enumerations. These are the same declarations R1 emits into the store.
- **Live events.** Live-simulation events use `EnumValue` for `kind`/`termination`, or a pyo3
  complex enum.
- **Identity roles.** Identity schemas carry their role through a schemars custom keyword
  (`x-pse-identity`), following the existing `x-pse-order` precedent. The document generator maps
  that keyword to the generated `identities.<Role>` NewTypes.
- **Codec and binding.** The codec registers msgspec `dec_hook`/`enc_hook` for `SemanticId`, and an
  `Identity<R>` `FromPyObject` accepts the Python `SemanticId` with a width check.
- **Representation.** Retain the existing bytes-subclass `SemanticId` for this mechanism. The
  recommendation-audit codec probe on msgspec 0.21.1 invoked both hooks and round-tripped hex
  for it and a non-bytes alternative. That removes the proposed need for a replacement value
  class; assembled generated-document decoding still needs its own integration check.
  `NewType` aliases provide static role distinctions only: they return the same runtime object.
  A width check establishes 128-bit identity shape, not runtime proof of its semantic role.
- **Clear win.** Cache the type lookups through pyo3 `PyOnceLock::import` (`sync/once_lock.rs:158`). A `static` inside the generic `EnumValue<T>` impl would be shared by every `T`, so each `RegistryEnum` impl gets its own cell, emitted by the `vocabularies!` macro (`crates/pse-py/src/enums.rs:27`).

*R18, physical meaning on bulk inputs and outputs through Arrow.*
- **Inputs.** Accept `pyo3_arrow::AnyRecordBatch` (`input.rs:24`; any `__arrow_c_stream__`/`__arrow_c_array__`
  producer) for starts, initialization stages, sample diagnosis and per-state tolerances, keyed by
  state identity. Validate against a registry relation contract whose value columns are
  `pse.quantity_value` and whose keys are `pse.semantic_id`. Validate expected-key membership,
  uniqueness and required coverage for each operation; transport metadata alone does not establish
  units or completeness. Establish an owned immutable numeric snapshot before asynchronous/native
  use of externally supplied Arrow buffers. The full C metadata journey remains adoption evidence.
- **Outputs.** Return native attempt primals as an exported `PyRecordBatch` instead of
  `list[tuple[str, float]]`.
- **Time semantics.** Distinguish positive wall-clock budgets from physical time coordinates
  and integration steps. `std::time::Duration` ↔ `datetime.timedelta` is a candidate only for
  budgets whose range and precision meet that contract. Pinned PyO3's conversion rejects
  negative timedeltas and exports microsecond precision (`pyo3-0.29.2/src/conversions/std/time.rs`).
  It is not a faithful blanket replacement for Float64 physical times: signed time origins and
  sub-microsecond increments are legitimate. Keep those values in the declared quantity/time
  contract with their existing precision, admitting any representation change explicitly.
- **Stubs.** pyo3-arrow types declare no `INPUT_TYPE`, so a thin local wrapper supplies the stub
  hint.
- **Remedy open, partly.** A Python-side unit and quantity resolver depends on whether units can be
  registry identity domains, which is unresolved.

*R19, one boundary document catalogue with schema-set identity.*
- **Catalogue.** A library-crate catalogue of `(type, slug, direction)` is consumed by both `xtask`
  and `pse-py`, with a `Version<N>` bound at every root, so `Completion` gains a version.
- **Schema generation.** One catalogue/aggregation owns the set, with separate schemars
  generator contexts for input and output contracts. `SchemaSettings::for_serialize()` for
  outputs and `for_deserialize()` for inputs are selected before `into_generator()`;
  schemars 1.2.2 does not expose a mutable per-root contract switch on a constructed generator
  (`src/generate.rs`, `SchemaSettings`, `SchemaGenerator`). Direction participates in canonical
  schema-set identity, so a type used both ways cannot collide or lose required/default behavior.
- **Import check.** The set's fingerprint is computed through a `pse_ids` frame (blake3 is confined
  to `pse-ids` by governance) and generated into Python. The extension computes the same
  fingerprint from its compiled types, and the import check compares the two.
- **Typed returns.** Bytes-returning methods become `DocumentValue<T>`.
- **Compiled-catalogue identity.** Compute the extension's fingerprint from its actual compiled
  catalogue, including declared direction and any verified feature-dependent membership/shape.
  The observed feature-gated Shooting `RunRequest`/`RunReport` variants are not themselves
  `JsonSchema` roots; their presence alone does not prove a serialized schema difference.
  Trace or generate the real catalogue under relevant features before selecting P6's exact
  fingerprint policy. Do not treat an unresolved feature variation as an established cause.

*R20, covariance content, provenance and mathematical validity (project-owned; F10).*
- Hydrate the covariance and its availability/local-validity conditions from the actual
  in-process or canonical fit result named by `ParameterCovariance.run_id`, or compare the
  caller-supplied ordered parameters and complete matrix against that record with its
  dependency/unit/interpretation conditions. Resolving a real run id alone does not certify
  altered values bearing that id.
- Legitimate external covariances remain supported, but their origin is explicitly external and
  uncertified (§19.8, PS-12); a caller-provided id cannot impersonate a certified fit record.
  Mathematical admission is separate from provenance: finite symmetry and a nonnegative
  diagonal do not establish positive semidefiniteness. Use an in-use linear algebra factor or
  eigensolver to reject/withhold an invalid covariance under a declared numerical tolerance,
  preserving legitimate singular positive-semidefinite covariance. A positive-definite-only
  Cholesky gate would reject valid inputs and is not the recommendation.
- Derived validity states which upstream statistical/local assumptions remain unverified; it
  never promotes an external input to certified merely because propagation succeeded.
- No new library, stamp, or bespoke numerical factorization is needed. The in-use faer family
  supplies `self_adjoint_eigenvalues(Side)` in pinned faer 0.24.4
  (`src/linalg/solvers.rs`), returning sorted eigenvalues or `EvdError` without eigenvectors.
  After square/symmetry admission, a declared triangle and scale-aware tolerance can establish
  PSD while accepting zero eigenvalues. Scratch/working allocation, parallelism, finite
  results and decomposition failure remain under existing admission and truthful withholding.
  Exact tolerance and integration remain Proposed numerical design, not an implemented pass.

*R21, operator and developer tooling consume Rust owners.*
- A versioned Rust `ConformanceManifest` document is parsed with `toml` in Rust and with
  `msgspec.toml` in Python, replacing `conformance.py`'s restated schema and defaults.
- The diagnostics tooling calls the Rust launch owner rather than re-implementing its protocol.
- `scripts/surreal_server.py` keeps its operator role but stops writing canonical tables directly.
  - Its mutations (authority rotation, storage rehoming, admission freezing, protection checks;
    `:5905`, `:6565-6600`, `:7266-7300`, `:7924-7929`) move into generated, guarded
    `fn::pse_*` maintenance functions. This is the same in-use mechanism the runtime uses, and the
    script becomes a caller.
  - Preserve privileged maintenance authentication, bounded/replayable pages, frozen-placement
    gates and historical restore/bootstrap activation (`canonical_maintenance.rs`, connection,
    installation and activation owners). Ordinary current-writer guards cannot simply be
    required before an old schema is restored or a new definition installed.
  - Bound the single-writer claim honestly: runtime lifecycle and the migrated supervisor
    meanings have generated guarded mutation owners; schema installation/restore/activation
    remains an explicitly scoped maintenance owner. R1's table checks are a backstop wherever
    installed, not proof that every maintenance path uses the ordinary writer.
  - The script reads generated contracts rather than literals.

**Alternatives.** The baseline fails closed through strict decoding and version-first checks, but
drifts silently in membership and pairing. The simplest viable step is R17's vocabulary input
alone. Using `uuid.UUID` everywhere for identities is natively supported by msgspec, pyo3 and
pyarrow, but uuid semantics are meaningless for hashed identities, so it is not recommended.

**Preserved.**
- The Python boundary performs no science.
- Strict decoding stays, and version-first refusal stays for inputs and recorded documents.
- Arrow streams for tables.

**Consumers stop:** choosing decode types by hand; parsing vocabularies; knowing solver state
order.

**Reverse check.** P1–P4 are unaffected. Ingress re-validation at the runtime entry stays (DP-03).

### 4.8 Findings outside the seven causes

- **[DA-020](../audit/boundaries-and-composition.md#da-020), placement of the performance
  toolkit.** *R22.*
  - **Placement (project-owned direction).** Separate capture orchestration, review/export and
    inspection-comparison tooling from the runtime's orchestration responsibility. A tooling
    crate is the selected organizational candidate, requiring an ADR ([RC09](#rc09)), but the
    dependency split must come first. Product workflows already consume `capture::DetailSession`
    (`worker.rs`, `study_diagnostics.rs`), and canonical result APIs consume
    `review_pack::PackResultSelection` (`connected_results.rs`, `pack_selection`/selection restore).
    Moving these whole modules into a crate that depends on runtime would create a cycle or
    make tooling a product foundation. Keep product-owned observation/detail and canonical
    result-selection contracts with their semantic owners, and let tooling consume those
    contracts. Export-specific packing/analysis then moves without a runtime→tooling dependency.
    A module boundary is a viable simpler first step if crate extraction adds no further benefit.
    - The observation hub is **not** moved there. Eleven product crates use it, so a tooling crate
      holding it would just become a new foundation dependency.
    - The hub's disposition is the substrate half below.
  - **Substrate (candidate pending a cost probe; the hub stays until then).** Make `tracing` the observation substrate,
    with the hub's aggregation as a `tracing_subscriber::Layer` holding per-span state in
    `extensions_mut()`. Production installs no subscriber today, and native jobs and DataFusion's
    `JoinSetTracer` already propagate *both* systems side by side (`math/jobs.rs:1422-1440`;
    `pse-engine/src/session/assurance.rs:46-84`). The probe compares the hub, a tracing Layer and
    no layer on a steady solve and a fit.
- **[DA-023](../audit/local-reasoning-and-assurance.md#da-023), architecture text lags code.**
  - The diagnosis is retained.
  - The correction belongs to the documentation owner and follows the adopted remedies above,
    for example the §0.5/§18.9 backend omissions and §13.5's nonexistent
    `PreparedSimulation::rebind`.
  - No library contributes. **Outside focus.**
- **[DA-024](../audit/local-reasoning-and-assurance.md#da-024), bespoke machinery without a
  production consumer.**
  - The diagnosis is retained: the native child lane and `pse.canon.v2` have only test or binary
    consumers.
  - Whether either has a bounded exploration purpose is a **maintainer decision**, and no in-use
    library owns either protocol.
  - Related: F06 (the DataFusion cache wrapper, inert in every session).

## 5. Additional findings

These were established by the evidence and are not in the audit; the review assigns them `F` IDs.
Each was re-read by the reviewer.

| ID | Finding | Principles / gates / scenario | Evidence | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | Horizon runs ignore caller cancellation after admission (lineage: split from [DA-012](../audit/boundaries-and-composition.md#da-012) as an independently fixable defect) | DP-19, DP-20; G5; [S03](#s03) | *Tested*: probe C, `probe_c_caller_cancel_after_admission_stops_horizon` (20-step antiwindup NMPC). Before the fix: 20/20 steps, usable result. After `driver.rs:142` → `cancel.child_with_deadline(None)`: cancelled after 1 step, join +0.5 ms; two existing horizon controls pass | A cancelled local study keeps solving horizon points to completion | The one-line child token now; R10 structurally | The probe test kept as a regression control. It must still show release only after the native join, which needs a gated native stage not shown by probe C. Add a deadline-expiry horizon: a typed TimeLimit and a trajectory marked partial |
| <a id="f02"></a>F02 | Uno's small-step stall while infeasible is published as a local infeasibility | PS-10; PS-G3 | *Tested* (synthetic): probe D, `probe_d_bridge_latch_and_native_status_mapping`: `(0,5)` → `Infeasible`. *Interface-checked*: `Uno_C_API.h:56-57`; `quality.rs:558-570` labels Infeasible-with-candidate as least-infeasible | Recovery and users read "locally infeasible" for a numerical stall | R15: map 5 to `Numerical`; named constants with `static_assert` | Mapping unit test over every `UNO_*` pair |
| <a id="f03"></a>F03 | Durable run starts require commit-order serialization; broad local pacing and untyped exhausted conflicts remain concerns | AP-07, H22, H24; DP-21; [S05](#s05) | *Implemented*: `begin_run` touches `canonical_guards:'execution-sequence:'+problem` (surql `:947`) under an exclusive in-process `ProblemTurns` write turn (`canonical.rs:722-786`, call `canonical_execution.rs:3346`), once per durable point (`workflow/study.rs:983`). Retry loops return the last raw `TransactionConflict` (the post-loop `Configuration("… retries exhausted")` is unreachable) | One shared sequence commit is necessary for no-lost-page visible-run paging. The local problem-wide mutation turn also queues independent operations and selection reads; its avoidable effect on 16 lanes is unresolved. Exhaustion is indistinguishable from other infrastructure faults | Typed exhaustion via R2. Keep necessary sequence serialization. Whether the broader local mutation turn can narrow without regressing guards, cancellation fairness or paging is unresolved; `DEFINE SEQUENCE` is not a commit-order-preserving drop-in | The 16-lane study probe: conflict retries and turn wait per point |
| <a id="f04"></a>F04 | The retained-surrogate rayon pool is dropped without joining its workers; the global rayon pool is never pinned | DP-20; H21 | *Interface-checked*: rayon-core 1.13.0 `ThreadPool::drop` only calls `registry.terminate()` (`thread_pool/mod.rs:399-402`). *Implemented*: `math/surrogate/library.rs:807-815`, `:830-835`; no `build_global` anywhere (the child process uses `RAYON_NUM_THREADS=1`) | Worker threads and stacks charged as `(threads+1) × stack` can outlive the charge; any library reaching the global pool would run uncounted threads (reach unresolved) | `ThreadPoolBuilder::spawn_handler` with retained `JoinHandle`s joined after pool drop (or `build_scoped`), with the handles retained for the pool's full charged lifetime. Global-pool reach stays unresolved: establish a real consumer before configuring one process-owned global pool or routing work into the admitted pool. Embedded hosts may have initialized the global pool already; record a typed incompatibility rather than claiming a per-deployment budget controls it | Thread census before and after drop; the global-pool reach probe |
| <a id="f05"></a>F05 | The LM fitting route disables the library's stationarity test | PS-07 (tolerance contract), DP-11 | *Interface-checked*: levenberg-marquardt `e67066c` `with_tol` sets `ftol = xtol = tol, gtol = 0` (`lm.rs:200-216`); `residual.rs:356-360` calls `.with_tol(p.accuracy.stationarity)` | Termination is driven by relative reduction and step tests standing in for a stationarity request. The independent original-space re-check (`residual.rs:380-407`) and covariance withholding keep outcomes truthful, so the cost is spurious non-convergence or extra iterations, unmeasured | **Candidate.** The library's `gtol` is MINPACK's scale-free cosine test, \|(Jᵀr)ᵢ\| / (‖Jeᵢ‖‖r‖) (`lm.rs:177-182`, `:461-468`), not the repository's scaled absolute gradient, so the request cannot be passed through. Use a separately declared, small `gtol` plus explicit `with_ftol`/`with_xtol`, and keep the independent check | The LM probe over existing `lm_inquiry.rs` cases must show no newly unqualified fits or withheld covariances |
| <a id="f06"></a>F06 | DataFusion's file caches are wrapped and then disabled for every session | DP-16, H27 (related to [DA-024](../audit/local-reasoning-and-assurance.md#da-024)) | *Implemented*: `NativeCacheService::unbound_config()` sets metadata, statistics and listing limits to 0 (`pse-engine/src/cache_service/mod.rs:356-361`), installed at `resources.rs:242`; `bind_state*` has no caller outside `cache_service/`. Counterevidence: the unbound default is deliberate ("Unqualified stores cannot safely use native Path-keyed caches", `cache_service/mod.rs:355`), and `bind_state*` exists to give each actual registered store a stable namespace (`:362`). The gap is the missing binding, not the safe default. | The native file-cache adapters have no reached binding in the inspected production paths. Other NativeCacheService preparation/analysis consumers remain active. A reached file-source reuse benefit is not established; protected canonical IPC decoding bypasses these DataFusion file caches | Bind bounded scan state at the actual source/session operation using `bind_state_with_generation` and the source's immutable identity or actual lifecycle generation, or remove only the unused native-file-cache adapters. A stable `ObjectStore` Arc/registration does not establish immutable content, and the session factory is not the registration owner. Keep unqualified stores unbound; library eligibility is unaffected | Reached file-source inspection shows generation-correct limits/hits under the same path across source generations, or only the unused file-cache adapters are removed |
| <a id="f07"></a>F07 | Every canonical table carries a redundant `key` UNIQUE index, while the server never asserts key–id agreement | DP-03, H14, H27; P1 | *Implemented*: 62 `<t>_key … UNIQUE` (`codegen/surreal.rs:135`); `key` is a plain `TYPE string`; agreement is checked only client-side (`check_record_identity`, `:148`); `canonical_study_dependencies` lacks `ENFORCED` | One extra index entry per canonical row in every guarded transaction; an invariant held by convention | R3 (candidate). Whole-table `key` paging in the supervisor script and the `WHERE key =` reads move to record-id access first | Generated DDL diff; `EXPLAIN` of the rewritten paging on RocksDB; a probe of the UPSERT and `INSERT … ON DUPLICATE KEY` assertion forms |
| <a id="f08"></a>F08 | statrs default features are unified on through egobox-moe, so `nalgebra 0.33.3` resolves beside `0.35.0`, and the pin note says the opposite | DP-24 | *Implemented*: `Cargo.lock` (`nalgebra` 0.33.3 and 0.35.0); pin note at `Cargo.toml:287-288` ("default features off: its nalgebra 0.33 and rand would split the pinned graph") | A stale pin reason; no type crosses between the two nalgebra majors in repository code (statrs distributions only) | Correct the pin note; no in-family fix short of an egobox feature change | `cargo tree -i nalgebra@0.33` matches the note |
| <a id="f09"></a>F09 | Default compiler retention limits are below the reference package's retained footprint, so every publish rotates the workspace | AP-07, DP-09, PS-11; H12, H13; [S01](#s01), [S02](#s02) | *Measured*: probe B2 (reference `ThreeUnits`; defaults 16,384 entries / 256 MiB / `query_values` 64). With limits lifted, the estimator reports ≈54.5k entries / ≈606 MB after publish. At defaults the generation rotates on publish, 0/143 bodies are reused after an unrelated edit, and 79 bodies re-run on a no-edit re-prepare. *Implemented*: `workspace.rs:1737-1752` | Edit → re-solve repeats whole-package checking and whole-root preparation at defaults | R8 | Rerun probe B2 at the selected new defaults for 143/143 live-body reuse/backdating, plus bounded churn and explicit oversized-live-set outcome |
| <a id="f10"></a>F10 | Covariance admission accepts an indefinite matrix as a covariance | PS-12, DP-03, DP-21; G3 and PS-G3 for certified derived results; lineage: DA-010, R20 | *Implemented*, source-derived: `ParameterCovariance::admit` checks square/symmetric/nonnegative diagonal only (`pse-runtime/src/math/settings.rs`); `uncertainty::propagate_checked` computes `JΣJᵀ` and rejects only non-finite outputs. The symmetric matrix `[[1,2],[2,1]]` passes admission, yet `J=[1,-1]` yields variance `-2`. This counterexample was not executed through Rust | Invalid external or altered covariance can produce finite negative variance under a result described as certified | R20: hydrate or fully reconcile qualified content; separately admit external covariance as positive semidefinite under a declared tolerance, with honest external validity | A negative-eigenvalue covariance is rejected/withheld, singular PSD covariance remains accepted, and a valid fit id with altered matrix values cannot obtain certified provenance |

## 6. Under-used capabilities (clear wins)

These are bounded improvements to existing functionality using in-use libraries. Their stated
conditions preserve P1–P4 and the scientific/lifecycle strengths; live-set retention still consumes
its explicit finite memory allowance. Larger candidates are in §4 with their conditions.

| Capability already linked | Improves | Why it is a clear win | Evidence |
|---|---|---|---|
| tokio-util `CancellationToken::child_token` through `CancelSource::child_with_deadline` | Horizon caller cancellation (F01) | One line; fixes a correctness defect; existing controls pass | *Tested* (probe C) |
| Uno C API `UNO_*` constants | Status mapping (F02) | Removes literal drift; corrects iterate 5 | *Interface-checked*; mapping *Tested* |
| SUNDIALS `SUNContext_ClearErrHandlers` then `SUNContext_PushErrHandler` (vendored `sundials_context.h:38-44`) | KINSOL/IDAS diagnostics: `NativeTermination.message` is `None` today (`kinsol.rs:2198`; `dynamics/idas.rs:519`) | Captures native error text. Handlers chain: every handler on the stack runs, and the default stderr logger is installed at context creation (`src/sundials/sundials_context.c:90`; `priv/sundials_errors_impl.h:156-161`). Clearing first is what stops stderr logging. No ABI change; diagnostics only | *Interface-checked* |
| Salsa `semantic_body::set_lru_capacity` at least the live root's distinct bodies | No-edit re-prepare re-runs 79 bodies (F09) | Configuration of an existing mechanism within a finite admitted live-root memory allowance. It does not by itself stop whole-generation rotation, which needs R8's policy/default correction | *Measured* (probe B2) |
| SurrealDB `ENFORCED` on `canonical_study_dependencies` | Relation endpoint existence, as on the other five relation tables | One keyword; existing writers relate existing points | *Interface-checked* |
| rayon `ThreadPoolBuilder::spawn_handler` | Surrogate pool drain (F04) | Joins workers before the charge ends | *Interface-checked* |
| pyo3 `PyOnceLock::import`, one cell per `RegistryEnum` and document type | Per-value `py.import(…).getattr(NAME)` in `EnumValue`/`DocumentValue` | Removes repeated module lookups; no semantic change | *Interface-checked*; speed *Proposed* |

**Candidates, not clear wins.** Each has a stated condition:
- Salsa input `heap_size` callbacks as a replacement aggregation path (R8): preserve shared-Arc
  accounting and generation-owned roots before removing the existing shared census. Callback
  availability alone does not establish footprint equivalence.
- SurrealDB `record::id` key assertion with the `<t>_key` index dropped (F07, R3). It needs the
  script paging and `WHERE key =` reads moved to record-id access, plus the UPSERT probe.
- FERAL `work_estimate().peak_bytes` as pre-numeric admission on the direct KKT path (R16). It is
  an a-priori figure, it may refuse cases admitted today, and §18.8 needs reconciling.
- levenberg-marquardt `with_gtol` (F05). It needs a declared conversion, since `gtol` is a cosine
  test, and the LM probe.
- `datafusion-execution/arrow_buffer_pool` **and** `arrow/pool` (`RecordBatch::claim` needs
  `arrow-array/pool`; `ArrowMemoryPool` adapts the DataFusion pool). This is library-measured
  Arrow accounting, but through unification every `Bytes` in the process carries a mutex-guarded
  reservation slot (`arrow-buffer` `bytes.rs:59-60`); cost unmeasured.
- KINSOL neutral reset across independent applications. It needs proof of bit-identity against a
  fresh session.
- SurrealDB `LIVE SELECT` wake-ups with polling kept as backstop. The integration patch's LIVE
  verifier accepts only CREATE, UPDATE and DELETE (`vendor/surrealdb/src/engine/remote/ws/metadata.rs:93`),
  although upstream types parse `KILLED`.
- Symbolica `hot_start` on evaluator rebuilds. It needs the previous scheme, is refused on the
  direct-translation path (`evaluate/tree.rs:327-329`), so it conflicts with R7's alias candidate,
  and the scheme must enter the evaluator identity (DP-11).

## 7. Numerical stages touched (slot 5, profile columns)

| Stage | Formulation policy | Derivative source and order | Scaling | Class · capability | Status → outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Body evaluation (Symbolica) | Guards, applicability and branches as host stages; arithmetic regions fused only when independent | Today: Taylor `Dualizer` plus existing symbolic routes (`SelectedFirstRegion`, direct `WeightedHessian` and action products) limited to flat bodies. R7: those routes extended per arithmetic region of the stage program, with joint evaluators over demanded outputs per `ProductKind`. Order = min(provider, smoothness, branch) | Not examined | n/a | n/a | Pre-differentiation admission budgets; operation counts *Measured* only for BT_PR flash, guard-free variants |
| Uno SQP over HiGHS QP | Smooth NLP; patched feasibility-LP seed | First-order + L-BFGS, dense-materialized | Uno's own | `SmoothNlp`; record overstates Hessian modes | Literal integers; iterate 5 → Infeasible (F02) | Original-space qualification independent of status |
| KINSOL declared fixed point | Picard / fixed point; declared restart schedule (authorized starts only) | No derivative demand; the Picard operator is never advertised as a derivative | One shared scaling for map and variables (R14) | `DeclaredFixedPoint` never routed; runtime-built fresh session | Complete status map (`kinsol.rs:2169-2201`); message `None` | Runtime calls `quality::qualify` on `original_residual`. R14 moves this into a map-specific seam route |
| LM fitting | Unbounded residual least squares | Exact Jacobian | `with_scale_diag` | Fitting-owned method (ADR-0209) | `TerminationReason` typed | `gtol = 0` (F05); the library's `gtol` is a cosine test, not the repository's scaled gradient; independent stationarity re-check; covariance withheld unless stationary |
| Horizon dynamics | IDAS/Diffsol per step | Integrator-owned sensitivities | Not examined | Dynamics | Typed; cancellation returns `Cancelled` after the drain (`jobs.rs:1461-1466`) | Caller cancellation not linked (F01); deadline-expiry partiality to be checked |

## 8. Library fit and ownership cost (slot 8)

| Capability / contract | Integration owner | Mechanism (selected unless marked) | Fit and limits | Cost and lifecycle | Removed |
|---|---|---|---|---|---|
| Typed lifecycle columns | `pse-codegen` SurrealQL generator; `pse-schema` registry | SurrealQL literal/tagged-union `TYPE`, `READONLY` on verified-immutable columns only, `$before` transition `ASSERT`s for fence, recovery and compaction rewrites (R1) | Enforced on all write paths (*Tested*, memory engine); refusals are `Internal` prose; UPSERT names the wrong field | Generator change; one migration of stored rows; RocksDB re-check | Hand enums, template literals, string sets, 14 `IN [...]` asserts |
| Typed refusals | `pse-operations` | `THROW 'pse.refusal:<code>'` + `is_thrown()` (R2) | Byte-stable (*Tested*); no detail channel | Registry vocabulary; one decoder | Prose matching; `RuntimeInfrastructure` collapse |
| Result pruning | `pse-runtime` result provider | `PruningPredicateBuilder`, `PrunableStatistics` over exact per-record-batch statistics, `FileReader::set_index`, full surviving-batch admission then output projection, conditional projected-decode design, `Inexact` pushdown (R4) | Historical standalone pruning soundness *Tested*; complete production-mode equivalence remains adoption evidence. Key pruning effective in original probe, value pruning not; decode saved only per whole record batch | Provider rewrite; exact statistics in the append contract; per-relation sort keys | `canonical_result_rows`, `scalar_predicate`, locator replay |
| Joint evaluation | `pse-math`, `pse-compiler` | Existing symbolic product routes extended per stage-program region, with `AtomCore::evaluator_multiple` over the demanded closure (R7); candidate `FunctionMap::add_aliases` + `EvaluatorBuilder::direct_translation` | Upper bound only (guard-free *Measured* variants on BT_PR); expression swell unprobed elsewhere | Admit-before-differentiation; route choice in the identity; per-instance validity and premises in the shared-program identity | `bound_bodies` re-expansion, per-output re-normalization |
| Bounded retention | `pse-compiler` workspace | Aggregate known-byte plus metadata/input accounting; existing Salsa count-based `lru` + per-query `set_lru_capacity`; finite resident live-root/firewall allowance; tracked accessors; input `heap_size` is a conditional census replacement (R8) | Eviction keeps edges but evicted memos still count as entries and cannot backdate (*Interface-checked*) | Explicit metadata/value policy and defaults plus configuration and two accessors | Routine rotation for ordinary supported work; observation-only specialization invalidation |
| Cancellation tree | `pse-runtime` supervisor | tokio-util `child_token`, `drop_guard` + one flat-leaf bridge (R10) | `is_cancelled` takes a mutex, so the flat leaf stays | One skeleton | Four lifecycle copies and forwarding closures |
| Capability truth | `pse-backend-native` | `capability_for(settings)`; `Problem::FixedPoint`; POUNCE `second_opinion_rungs` (R14) | Library iterates; seam owns admission | Registry and codegen for the representation | Runtime `Backend::` gates |
| Native memory | adapters | Candidates: FERAL `work_estimate().peak_bytes` pre-admission; PETSc `PetscMallocSet`; SuiteSparse allocator hooks (R16) | A-priori figures; process-global hooks need retained allocation-owner provenance under P1; allocator-domain coverage excludes MKL/BLAS bypasses | Bridge-level | Some shape models |
| Boundary types | `pse-py`, generators | pyo3 `INPUT_TYPE`, pyo3-arrow `AnyRecordBatch`, schemars keyword, msgspec hooks, `PyOnceLock` (R17–R19) | pyo3-arrow lacks stub hints; msgspec bytes-subclass hooks *Tested* by selective audit probe, NewType role aliases remain static | Local wrappers | Hand parsing, hand pairing, `settings::named` |

## 9. Foundations and gates (slot 6)

| Foundation | Scenario and evidence | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | The host composes runtime operations and owns defaults ([DA-015](../audit/boundaries-and-composition.md#da-015)); the toolkit lives in the orchestration root and the foundation crate ([DA-020](../audit/boundaries-and-composition.md#da-020)); domain values carry the whole composition root ([DA-021](../audit/local-reasoning-and-assurance.md#da-021)) | **Violated** | R11, R22 |
| AP-02 Stable contracts | The runtime names backends and re-derives adapter facts ([DA-016](../audit/boundaries-and-composition.md#da-016)); capability records overstate ([DA-017](../audit/boundaries-and-composition.md#da-017)); document pairing by hand ([DA-011](../audit/structure-and-invariants.md#da-011)) | **Violated** | R14, R19 |
| AP-03 Composition | Lifecycle copied five times and diverged (F01 *Tested*); start policy twice; annotation lowering per mode ([S07](#s07), [S09](#s09)) | **Violated** | R10–R13 |
| AP-04 Domain model and semantic authority | Model adequacy: the specialized model omits resolved-reference distinctions ([DA-001](../audit/semantic-model-and-authority.md#da-001)). Authority: store vocabularies not governed by the registry ([DA-007](../audit/structure-and-invariants.md#da-007)); orchestration restates compiler rules ([DA-031](../audit/semantic-model-and-authority.md#da-031)) | **Violated** (both halves) | R1, R5, R13 |
| AP-05 Explicit structure | Lifecycle as flag bags; refusals as prose; request/report pairing not structural | **Violated** | R1, R2, R11 |
| AP-06 Local reasoning and testability | Policy tests need a runtime, admission and compilation ([S10](#s10)); binding logic in a `test = false` crate | **Violated** | R11 |
| AP-07 Execution fits the workload | Whole-workspace rotation on every publish at defaults (F09 *Measured*); per-row transactional locators ([DA-026](../audit/execution-fit.md#da-026)); nested whole-model scans ([DA-029](../audit/execution-fit.md#da-029)); per-problem run-start serialization (F03; magnitude unresolved) | **Violated** | R4, R7–R9 |

| Gate | Result | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **Fail** | Store value sets and projections are independently hand-spelled rather than derived from their scoped declarations ([DA-007](../audit/structure-and-invariants.md#da-007)); observational and physical phase vocabularies are distinct, so differing spellings alone are not disagreement; vocabularies spelled five ways ([DA-004](../audit/semantic-model-and-authority.md#da-004)); two start-policy copies ([DA-013](../audit/boundaries-and-composition.md#da-013)); Python second writer ([DA-019](../audit/boundaries-and-composition.md#da-019)) | R1, R13, R17, R21 |
| G2 Semantic fidelity | **Fail** | Identity kinds and units lost on side routes ([DA-005](../audit/semantic-model-and-authority.md#da-005)); refusal cause lost to prose ([DA-008](../audit/structure-and-invariants.md#da-008)); string relation names skip coverage silently ([DA-032](../audit/semantic-model-and-authority.md#da-032)) | R2, R6, R17, R18 |
| G3 Validity | **Fail** | The `analysis` artifact owner can be released unguarded (source-established; [DA-007](../audit/structure-and-invariants.md#da-007)); invariants enforced only in an adapter ([DA-015](../audit/boundaries-and-composition.md#da-015)); covariance provenance never reconciled ([DA-010](../audit/structure-and-invariants.md#da-010)); indefinite covariance accepted (F10) | R1, R11, R20 |
| G4 Hidden behaviour | **Unresolved** | Effects are reachable from every domain value by convention ([DA-021](../audit/local-reasoning-and-assurance.md#da-021)), but no undeclared effect or meaning change was demonstrated | Settled by R11's explicit inputs |
| G5 Consistency and recovery | **Fail** | Retry permission rests on prose and incidental fields, and worker loss is recorded as an error ([DA-008](../audit/structure-and-invariants.md#da-008)); caller cancellation of a running horizon ignored (F01 *Tested*) | R2, R10 |
| G6 Transformation and reuse | **Unresolved** | Hand-keyed cross-lane keys, some with process-pointer addresses ([DA-030](../audit/execution-fit.md#da-030)); no stale reuse demonstrated | R8; key audit in plan |
| G7 Truthful capability claims | **Fail** | Uno exact-Hessian claim; KINSOL automatic fixed point never routed; PETSc methods published but refused ([DA-017](../audit/boundaries-and-composition.md#da-017)); `PreparedSimulation::rebind` in §13.5 ([DA-014](../audit/boundaries-and-composition.md#da-014)) | R14, R12 |
| G8 Library leverage | **Fail** | A bespoke row index and predicate kernels beside DataFusion's filter and pruning, with the predicate shown filter-equivalent (*Tested*; [DA-026](../audit/execution-fit.md#da-026)); parallel context propagation beside `tracing` at every boundary ([DA-020](../audit/boundaries-and-composition.md#da-020)) | R4, R22 |
| G9 Architectural fitness | **Fail** | Follows AP-01–AP-07, all violated | All |
| PS-G1 Physical consistency | **Unresolved** | Property envelopes (PS-02) were not examined. On the examined paths, physical typing and closure hold, and DA-005's side-route unit convention does not reach published results unqualified. It is a PS-01 deviation carried under G2 | R18; examine PS-02 separately |
| PS-G2 Well-posedness | **Unresolved** (scope) | Structural analysis is library-owned (pounce-presolve); diagnostic naming not examined; the runtime-built KINSOL path's admission coverage unresolved ([§2.3](#23-well-posedness-statement-slot-3-profile)) | R14 closes the path structurally |
| PS-G3 Numerical integrity | **Fail** | A solver status is misclassified: Uno iterate 5 → `Infeasible` (F02 *Tested*). F10 also admits mathematically invalid covariance to a finite derived result. Other inspected solver mappings retain independent qualification | R15, R20 |

## 10. Disposition index (DA-001–DA-032)

**Labels.**
- **Addressed** — a recommendation owns the correction; "in part" names what is left.
- **Remedy open** — used only where the brief defines it: every available in-use route would
  regress another part of the system.
- **Lead** — no in-use library offers the capability.
- **Maintainer decision** — a judgment outside review authority.
- **Outside focus** — outside the library-targeted focus.

| Audit finding | Disposition | Notes |
|---|---|---|
| [DA-001](../audit/semantic-model-and-authority.md#da-001) | Addressed by **R5** | Project-owned representation; [RC01](#rc01) |
| [DA-002](../audit/semantic-model-and-authority.md#da-002) | Addressed by **R5** | Catalogued derivation frames; relationships recorded in the model |
| [DA-003](../audit/semantic-model-and-authority.md#da-003) | Addressed by **R5** | `pse.expr_dsl` field extension metadata drives generated inventories |
| [DA-004](../audit/semantic-model-and-authority.md#da-004) | Addressed by **R1** (store) and **R17** (boundary) | One registry declaration per vocabulary |
| [DA-005](../audit/semantic-model-and-authority.md#da-005) | Addressed by **R17** and **R18**, in part | The unit resolver is **unresolved**: it depends on whether units can be registry identity domains, which is a project question |
| [DA-006](../audit/semantic-model-and-authority.md#da-006) | Addressed by **R13** | Compiler-published lowering per mode; [RC16](#rc16) |
| [DA-007](../audit/structure-and-invariants.md#da-007) | Addressed by **R1** | Includes the unguarded `analysis` release; guard rows kept; `READONLY` only on immutable columns |
| [DA-008](../audit/structure-and-invariants.md#da-008) | Addressed by **R2** | Covers `worker.rs:1697`, `durable.rs:2062-2099` and `study.rs:1694-1700`. Corrected detail: exhausted retries surface as raw conflicts, and the second prose match is test code |
| [DA-009](../audit/structure-and-invariants.md#da-009) | Addressed by **R11** | Paired outcome enum; a discriminant-owned operation set (strum + enum-map) |
| [DA-010](../audit/structure-and-invariants.md#da-010) | Addressed by **R20** | Full content/validity reconciliation; external provenance explicit; F10 adds PSD admission without rejecting singular valid covariance |
| [DA-011](../audit/structure-and-invariants.md#da-011) | Addressed by **R19** | Catalogue and schema-set fingerprint |
| [DA-012](../audit/boundaries-and-composition.md#da-012) | Addressed by **R10** and **R11** | Supervisor and cancellation tree (R10); one declared-procedure dispatch (R11). The explicit-strategy family is declared as a stated exception ([RC13](#rc13)); running strategies as supervised runs is a **maintainer decision**. Divergence now *Tested*; split defect F01 |
| [DA-013](../audit/boundaries-and-composition.md#da-013) | Addressed by **R13** | Project-owned |
| [DA-014](../audit/boundaries-and-composition.md#da-014) | Addressed by **R12**, in part | Parameter-valued Fit and Simulation points. Binding initial states, schedules and fit data is a **lead**. Shooting as a study operation is a **maintainer decision** ([RC12](#rc12)) |
| [DA-015](../audit/boundaries-and-composition.md#da-015) | Addressed by **R11** | Request-driven host, a runtime admission entry, one deployment owner (keeping ADR-0164's per-host assertion), validated constructors |
| [DA-016](../audit/boundaries-and-composition.md#da-016) | Addressed by **R14** | Map-specific fixed-point route; recovery profiles with the start permission passed in; only arclength backend suitability moves into the adapter |
| [DA-017](../audit/boundaries-and-composition.md#da-017) | Addressed by **R14** | Its architecture-text half is carried by [RC07](#rc07) |
| [DA-018](../audit/boundaries-and-composition.md#da-018) | **Diagnosis challenged in part** | Most patch hunks are P4 integration. The four numerics hunks are retained and declared by R15. Relocating them is **remedy open**: no in-use route keeps SQP-LBFGS, and Uno-IPM would be a major addition |
| [DA-019](../audit/boundaries-and-composition.md#da-019) | Addressed by **R21** and **R1** | Operator mutations move into generated guarded maintenance functions; table types back them up |
| [DA-020](../audit/boundaries-and-composition.md#da-020) | Addressed by **R22**, in part | Separate product observation/detail and result-selection contracts before extracting tooling; no runtime→tooling cycle. Tracing substrate and crate extraction remain conditional candidates |
| [DA-021](../audit/local-reasoning-and-assurance.md#da-021) | Addressed by **R11** | Explicit policy inputs; composition leaves values |
| [DA-022](../audit/local-reasoning-and-assurance.md#da-022) | Addressed by **R16** and **R8**, in part | Native accounting candidates; Salsa input callbacks replace the shared census only after accounting ownership is settled. Size models for the project's own structures are a **lead**: no in-use derive sizer |
| [DA-023](../audit/local-reasoning-and-assurance.md#da-023) | **Diagnosis retained — outside focus** | Documentation owner; follows adopted remedies and RCs |
| [DA-024](../audit/local-reasoning-and-assurance.md#da-024) | **Diagnosis retained — maintainer decision** | Exploration purpose of the native child lane and `pse.canon.v2`; no in-use library owns either |
| [DA-025](../audit/execution-fit.md#da-025) | **Diagnosis challenged in part** | Challenged: the participants are real (P1, P2); the SDK patch is P4 integration; owner-identity binding is sized to P2's multi-process workspaces and already uses in-use Linux primitives (rustix `pidfd`/`waitid`), so no simplification was found. Retained: result write volume (→ **R4**), redundant indexes (→ **R3**). Retained as a **maintainer decision**: the unconsumed online collector and possible unbounded payload growth. A collector could run on an existing trigger, such as `forget_*`/`reclaim_*` or on-demand primary activation, without a daemon (P2) |
| [DA-026](../audit/execution-fit.md#da-026) | Addressed by **R4** | Benefit is locator removal plus key pruning at record-batch granularity, not value pruning |
| [DA-027](../audit/execution-fit.md#da-027) | Addressed by **R5** and **R7** | No re-expansion. The body identity is re-framed over the resolved form plus validity and premises; it is not merged into the group hash |
| [DA-028](../audit/execution-fit.md#da-028) | Addressed by **R7** | Diagnosis retained; magnitude unresolved. It was not observed on BT_PR, but the audit's PR mixing-rule example lies in single-output non-substitutable instances the probe could not group. The alias candidate targets those |
| [DA-029](../audit/execution-fit.md#da-029) | Addressed by **R9** | — |
| [DA-030](../audit/execution-fit.md#da-030) | Addressed by **R8**, in part | Re-diagnosed: retention vs footprint dominates at defaults (F09). Cross-lane pointer-address and field-listed keys remain a G6 audit item |
| [DA-031](../audit/semantic-model-and-authority.md#da-031) | Addressed by **R13** | [RC16](#rc16) |
| [DA-032](../audit/semantic-model-and-authority.md#da-032) | Addressed by **R6** (and **R4** for result-side sites) | — |

**Counts.**
- **Addressed: 28.** Five of them are partial: DA-005, DA-014, DA-020, DA-022 and DA-030. DA-012
  also carries a maintainer decision for its strategy family.
- **Diagnosis challenged in part: 2** (DA-018, DA-025).
- **Diagnosis retained without a review remedy: 2.** DA-023 is outside focus; DA-024 is a
  maintainer decision.
- Of the addressed findings, DA-028's magnitude stays unresolved.

## 11. Priority and prerequisite order (slot 12)

| Priority | Change | Findings / scenarios | Acceptance evidence | Proposed owner |
|---|---|---|---|---|
| 1 | F01 child token; R15 mapping and constants | F01, F02, [S03](#s03) | Probe C and D tests as controls | Future plan |
| 2 | R8 bounded live-set/default policy, metadata/value accounting and catalog firewall | F09, DA-030, [S01](#s01), [S02](#s02) | Probe B2 at the redefined defaults: no rotation on publish, 143/143 bodies reused, `specialized` backdates | Future plan |
| 3 | R1, R2 typed store lifecycle and refusals | DA-007, DA-008, F03 (typed half), [S04](#s04) | Generated DDL; refusal decoding tests; RocksDB re-run of probe-surreal Q1/Q2 | Future plan |
| 4 | R10 single supervisor | DA-012 | Horizon and existing lifecycle controls through one skeleton | Future plan |
| 5 | R4 pruned result selection (locator removal, statistics, lowered predicate); R3 candidate | DA-026, F07, [S06](#s06) | Grid equivalence kept; locator count and append-transaction count before and after on one large solve. The declared-field step waits for R6 (row 10) | Future plan |
| 6 | R14, R16 seam capability and native memory | DA-016, DA-017, DA-022, [S07](#s07) | Capability-equals-routing test extended to per-settings rows and fixed-point class | Future plan |
| 7 | R11, R12, R13: operation request, parameter-valued binding, single owners | DA-009, DA-013–DA-015, DA-021, DA-006, DA-031, [S09](#s09), [S10](#s10) | Policy tests without a runtime; a fit sweep that binds parameter values with recorded starts | Future plan |
| 8 | R17–R21 boundary | DA-004, DA-005, DA-010, DA-011, DA-019, [S08](#s08) | Stubs typed; import check covers the schema set | Future plan |
| 9 | R7 evaluation granularity over the stage program (re-expansion removal and joint regional products). Its identity re-framing waits for R5 (row 10) | DA-027, DA-028 | Probe A re-run with obligations, per `ProductKind`; evaluation timing on wider fixtures; derivative-consistency conformance | Future plan |
| 10 | R5, R6, R9 resolved representation, handles, scans | DA-001–DA-003, DA-029, DA-032 | Body identity unchanged by a renderer change; projection-harness timings at 500, 2,000 and 8,000 cells | Future plan; R5 and R9's resolved-identity stage follow RC01. R6 and current-identity lookup indexes can precede it |
| — | R22 tooling placement; DA-023, DA-024 | — | ADR for the crate; maintainer decisions | Future plan |

**Prerequisite order** (distinct from priority). Each dependency is stated as the meaning it needs:
- **R1 before R17.** The registry enumerations R1/R17 share for artifact kind, owner kind and
  terminal class are the same declarations R17 projects into Python.
- **R1/R2 share registry ownership, not a mandatory lifecycle migration order.** R2 can
  introduce refusal codes and typed decoding independently; it does not need tagged lifecycle
  migration to land first.
- **R6 before R4's declared-field step.** R4's declared indexable fields need R6's generated
  relation handles.
- **R10 before R11's paired outcome.** The paired request/report enum lives in the supervisor's
  result.
- **R11's operation request before R12.** Value binding for dynamics and fits is carried by the
  request.
- **R14's fixed-point representation before removing `pse-py`'s KINSOL route** (part of R11).
- **R5 before R7's identity step and R9's final resolved-identity step.** Body identity over the resolved form and index-based
  scans can use existing indexes before that representation is complete. R7's regional joint
  products can be investigated over today's stage programs, retaining existing chain propagation
  until a replacement is qualified.
- **R8 is independent and should land first.** It is also the condition for measuring the other
  preparation changes at defaults.

## 12. Verification and settling evidence

Settling evidence is discretionary. Each item names the decision it affects.

| Claim / risk | Label now | Settling evidence | Affects |
|---|---|---|---|
| R1/R2 behave the same on the repository's RocksDB server | *Tested* on the memory engine only | Re-run probe-surreal Q1/Q2 against `rocksdb://` at v3.3.0 | R1, R2 adoption |
| `$this.id` is available to a field `ASSERT` on every write form | *Interface-checked* (production DDL already uses `$this.…`; core `doc/field.rs:613-619`, `:655`) | Probe the UPSERT and `INSERT … ON DUPLICATE KEY` forms; `EXPLAIN` the record-id rewrite of the script's key paging on RocksDB | R3 key assertion |
| F03 throttles 16 lanes | *Implemented*; magnitude unknown | 16-lane study with `QueryStatisticsObserver`: conflict retries and `ProblemTurns` wait per point (existing `canonical_joined_publication_progress_and_ordinary_cost_probe`, `workflow/durable.rs:4309`) | Priority of F03; whether its remedy matters |
| R4 write reduction | Structural only | Locator inserts and append transactions for one large solve, before and after | R4 priority |
| R4 inside the single decoder | *Proposed* | Decoded bytes with `FileReader::set_index` over pruned record batches and full surviving-batch admission; all four scalar keys, original coordinates, unfiltered null/NaN/∞, residual-before-limit and predicate-column retention. Projected decoding requires a separate sound admission boundary | R4 design |
| R7 on the stage program | Upper bound *Measured* on guard-free variants of one fixture; revised composition *Proposed* | Provider→arithmetic and branch→arithmetic first/mixed-second derivatives, local-action/adjoint composition, deterministic preflight and cancellation with no retained truncated program; re-run probe A with obligations counted and `probe_stage_kinds()` logged. Measure `SelectedPartials`, `WeightedHessian` and the action products separately against today's routes. Add evaluation timing, wider and deeper fixtures, nested implicit bodies and the frequency of the up-front Taylor choice | R7 adoption; [RC02](#rc02) |
| Alias-fused let chains on non-substitutable bodies | Unprobed | Inventory Q2: tree path vs `direct_translation(true)` vs today; `export_cpp` acceptance | R7 candidate |
| R8 fits finite admitted live set while bounding all actual retained state | *Proposed* policy; aggregate APIs and existing LRU *Interface-checked* | Probe B2 at the chosen defaults plus edit/deletion/root-switch churn; separate metadata/input and value bytes, retain resident backdating memos, exercise explicit oversized-live-set outcome | R8; [RC03](#rc03) |
| Tracing Layer cost | Unresolved | Hub vs tracing Layer vs no layer on a steady solve and a fit | R22 substrate half |
| Global rayon pool reach | Unresolved | Counting `start_handler` under `build_global` across journeys | F04 second half |
| LM `gtol` mapping | *Interface-checked* (cosine test, not a gradient tolerance) | `lm_inquiry.rs` cases with a declared `gtol`: `TerminationReason`, evaluations, and no newly unqualified fits or withheld covariances | F05 |
| PETSc / KLU allocator hooks | *Interface-checked* | Inventory P4/P5 | R16 candidates |
| msgspec `bytes`-subclass hooks | *Tested*, selective audit probe on msgspec 0.21.1, zero failures | Product generated nested/version-first codec integration remains | R17 representation retained |
| Feature-dependent schema set | Unresolved | Inventory P6 | R19 fingerprint form |
| pyarrow extension metadata through `PyRecordBatch` | Unresolved | Inventory P8 | R18 |
| F01 under an inherited deadline | Unresolved | Horizon whose caller deadline expires mid-run: typed TimeLimit, partial trajectory, release after the native join | F01 acceptance |
| PETSc or KLU hook with several lanes in one process | Unresolved | Retained allocation owner/size across allocate/free/realloc, process-init lifetime and bypasses; current-thread attribution alone is insufficient | R16, P1 |

**Conformance-suite status (PS-13).** No unit or property model is changed by these
recommendations. R5 and R7 change body construction for every model, so the plan must rerun the
derivative-consistency and closure conformance checks after R7. Their current status was not
examined.

Adoption evidence also needs R20's altered-content/genuine-id case, negative-eigenvalue
rejection, singular PSD acceptance and explicit external validity, plus R22's dependency-direction
check. These are future implementing checks, not current qualification requirements for publishing
this review.

## 13. Rule impacts (slot 11)

These are the current rules the recommendations would change. They change only after the operator
confirms them in plan creation; this review edits none of them.

| ID | Rule and location | Proposed change | Depends on it | If the rule is kept |
|---|---|---|---|---|
| <a id="rc01"></a>review#RC01 | Blueprint §7.1 *Design constraints*: "no intermediate IR is produced or stored" | Permit a resolved, identity-typed specialization representation between authored syntax and library algebra, with canonical local-slot body framing; extend expression metadata and version/domain-separate changed body/derived identity contracts under §5.3 | R5 (DA-001, DA-002, DA-003, DA-027 identity), R9 | R5 becomes remedy open; identity stays framed over rendered text, and re-derivation remains |
| <a id="rc02"></a>review#RC02 | Blueprint §7.1 *Specialization-local bodies*: "A body covers one definition and its consumed finite membership" | State the evaluation unit: per arithmetic region of the guarded stage program, a joint evaluator over the demanded output closure of each `ProductKind`, with eligible derivatives built symbolically and existing Taylor/reverse propagation retained or correctly composed across intermediate/provider/branch barriers. Host stages keep splitting regions. The route choice is made from deterministic preflight before differentiation and carried in identity, with the existing Taylor route for deterministically ineligible cases; transient capacity/cancellation remains a refusal, never an arithmetic fallback. A program shared across instances carries their validity and premises. No whole-group Dualizer | R7 | The text stays ambiguous; R7 can still land as an implementation choice under §7.1's current wording |
| <a id="rc03"></a>review#RC03 | Blueprint §14.3.2 limits table (`WorkspaceLimits`: 16,384 entries, 256 MiB, LRU 64) and §14.4 "rebuilds a fresh generation" | Separate actual metadata/input/interned growth from disposable values; retain aggregate known-byte accounting and finite live-root/backdating allowances; configure per-query count capacities and defaults for admitted workloads. No unavailable value-holding counter or library byte-weighted LRU. Rotation/refusal remains explicit when actual bounded state cannot fit | R8, F09 | F09 persists at defaults; edit → re-solve repeats whole-package work |
| <a id="rc04"></a>review#RC04 | Blueprint §19.2 "Scalar indexing currently covers supported fields…" and the §20 append protocol (`row_indexes`; generated `fn::pse_execution_v1::append`) | Exact per-record-batch key statistics (min/max, row and null counts) committed in the append on the existing block row, under a per-relation declared sort key; DataFusion pruning and a lowered canonical predicate; per-row locators retired. The §20 single protected decoder (ADR-0180/0186) stays unchanged | R4 | DA-026 remains; only the predicate-lowering partial step is possible |
| <a id="rc05"></a>review#RC05 | Substrate catalog conventions (`crates/pse-schema/src/catalog/substrate.rs` `text()`/`flag()` lifecycle columns; the generator's Arrow-only SurrealQL type mapping; the `key` + `<t>_key UNIQUE` convention) and §20.1/§20.6's canonical relation description | Registry enumerations and lifecycle unions generated into SurrealQL types and transition assertions. `READONLY` only on verified-immutable columns; fence, recovery and compaction rewrites governed by transition rules. Candidate: `key` asserted equal to the record id, with the redundant index dropped once key-paging readers move to record ids | R1, R3, F07 | DA-007 remains; R2 can still land on its own |
| <a id="rc06"></a>review#RC06 | Generated SurrealQL refusal convention (`THROW '<prose>'` in `pse-codegen` templates). §23.2 itself is unchanged; the code then meets it | Refusals name generated registry codes | R2 | DA-008 remains; retry stays prose-dependent |
| <a id="rc07"></a>review#RC07 | Blueprint §18.1 (problem representations), §18.7 (one static capability record per backend; "no workflow edits" claim) and §18.9 matrix | Settings-dependent capability rows; `FixedPoint`/`Picard` representations; recovery profiles and corrector suitability as adapter capability; Uno and PETSc entered in §0.5/§18.9 | R14 | DA-016/DA-017 remain; the seam claim stays untrue |
| <a id="rc08"></a>review#RC08 | ADR-0157 (proposed) "scoped binding" for Uno | Record the patch as integration hunks plus four named numerics hunks; declare the dense SQP size scope and per-profile curvature | R15 | The binding's numerical content stays undeclared |
| <a id="rc09"></a>review#RC09 | §23.1 placement of capture, review-pack, review-query and inspection-comparison tooling in `pse-runtime`; AGENTS.md crate-addition rule. §1.1's placement of the observation hub in `pse-diagnostics` changes only if the tracing-substrate lead is adopted | A product/tooling dependency split with canonical selection/detail contracts below tooling; a separate tooling crate is conditional and requires an ADR | R22 | DA-020's placement half remains |
| <a id="rc10"></a>review#RC10 | Blueprint §13.5 table row "`PreparedSimulation::rebind`…" | State the value-binding route R12 actually provides for simulation and fit studies | R12 | G7 text claim stays untrue |
| <a id="rc11"></a>review#RC11 | Pin note for `statrs` (`Cargo.toml:287-288`) | State that egobox-moe unifies statrs defaults and that nalgebra 0.33 resolves | F08 | A stale pin reason |
| <a id="rc12"></a>review#RC12 | §19.3 study operation set and the persisted `StudyOperation` (`Version<7>`) | Hosts share common request structs/preparation semantics; host-only variants use an explicit superset and typed study projection. `StudyOperation` stays the study-owned record. *If* the operator makes Shooting a study operation, that is a version bump with a stated replay or regeneration path; parameter-valued Fit and Simulation points become admitted study bindings | R11, R12 | Hosts keep separate operation enums; dynamic and fit sweeps stay structural |
| <a id="rc13"></a>review#RC13 | §17/§19.2 run lifecycle: the explicit strategies (conic, recycle, block initialization) return through `SolveHandle` without run, attempt, completion or replay, an exception neither section states | State the exception, or (maintainer decision) bring strategies under the supervisor | R10 | The unstated exception persists; G7 text gap |
| <a id="rc14"></a>review#RC14 | §21 public side-route API (`with_primal_start(Mapping[SemanticId, float])`, block-initialization stages, `diagnose_samples`, positional `SimulationSettings` tolerances, seconds as float) | Keyed Arrow inputs validated against registry contracts; per-state tolerances keyed by identity; precision-preserving physical times; `timedelta` only where budget range/precision fit | R18 | DA-005's side routes stay convention-typed |
| <a id="rc15"></a>review#RC15 | §18.8 "source-owned bounded FERAL profile" | For the direct KKT path, pre-numeric admission from FERAL `work_estimate().peak_bytes` beside post-factor accounting; the profile keeps covering POUNCE-internal FERAL | R16 | The direct path keeps its after-the-fact accounting |
| <a id="rc16"></a>review#RC16 | §16.2 sources and precedence (authored annotations as `ModelHint`) and the steady view-choice rule as realized in orchestration | The compiler publishes one numerical-annotation lowering per analysis mode, with deliberate per-mode differences recorded, and one view choice, consumed by steady, implicit and dynamic orchestration | R13 | DA-006 and DA-031 remain |
| <a id="rc17"></a>review#RC17 | The operator-supervisor protocol (`scripts/surreal_server.py` performing rotation, rehoming, freezing and protection checks by direct table writes), wherever §20.6 or ADR-0164 assign it | Operator mutations become generated guarded maintenance functions called by the script, retaining privileged/version-specific bootstrap, restore and activation ownership | R21 | A second canonical writer remains (DA-019) |
| <a id="rc18"></a>review#RC18 | §19.8 propagation input/validity and `ParameterCovariance::admit`'s accepted external-input contract | Distinguish certified fit content from external covariance; require complete content/validity reconciliation for the former and PSD admission plus honest upstream assumptions for the latter, preserving singular covariance | R20, F10 | Naming a run continues to imply provenance without reconciliation; finite indefinite inputs can produce negative variance |

**No change is needed to:**
- §23.2's failure taxonomy, whose stated contract R2 meets;
- §21/§21.5's Python contracts, whose stated NewType rule R17 meets;
- §1.1's `pse-py` role, which R11 restores;
- ADR-0164's remote server, which P1 and P2 keep;
- the vendored-hook notes.

On the last point: the vendor provenance note's sentence that every packaged file matches the
archive "except the local edits described here" does not enumerate the KLU and IDAS integration
hooks that §18 and ADR-0200 record. Under P4 this review does not raise that as a finding.

## 14. Decision

**Behavioural / semantic adequacy: Revise.** G1, G2, G3, G5, G7 and PS-G3 fail.
Original probe C tests the horizon caller-cancellation divergence (F01), and original probe D
tests Uno's status misclassification (F02). Source independently establishes the unguarded
`analysis` release precondition, retry-by-prose/class collapse, and the added covariance-validity
counterexample F10. These claims keep their different evidence scopes. G4, G6, PS-G1 and
PS-G2 remain unresolved; no strength or proposed remedy closes them.

**Architectural fitness: Revise.** G9 fails through all seven foundations in §9; G8 also fails.
The strongest executed architectural evidence remains the original default workspace rotation
and no-edit/unrelated-edit reruns (F09), together with the tested lifecycle divergence. Source
shows avoidable locator writes, reconstructed names/identities, policy copies and exposed
implementation knowledge. New remedy corrections do not make the current implementation pass.

**Overall: Revise.** The target remains **Proposed**, with **Interface-checked** enablers and
the bounded original **Tested/Measured** receipts plus the selective audit codec probe.
No production repair, rule confirmation, implementation-plan adoption or product qualification
is implied by this recommendation audit.

The coherent target keeps the existing physical/scientific owners, canonical fenced state,
durable lane independence, cancellation drain, original-space qualification, library-owned
derivatives and value rebinding:
- R1 models actual physical persistence phases, separately projecting observational states,
  retaining u64 generation extent and legitimate close-before-manifest/recovery/cancel transitions.
- R2 owns code decoding and diagnostics without inferring effects or replay permission from a
  class alone. R21 retains privileged restore/bootstrap/activation meaning.
- R4 lowers the complete existing selection contract, including unfiltered exceptional values,
  four relation key shapes, stable row coordinates and residual filtering before limits. It
  prunes record batches and fully admits surviving batches; projected decode is conditional.
- R5/R6/R9 remove name reconstruction and repeated scans while preserving positional sharing,
  physical/refinement/guard contracts, and explicit durable identity interpretation.
- R7 fuses only eligible demanded arithmetic and preserves chain terms through provider/branch
  barriers. Deterministic preflight selects a route; transient refusal does not select arithmetic.
- R8 bounds metadata, inputs and values with APIs that pinned Salsa actually exposes, keeping
  the admitted live set and backdating values resident. New defaults are not a claim of general
  byte-weighted LRU or measured resident memory.
- R10/R11 share lifecycle and preparation semantics while keeping durable work on the worker's
  cancellation tree and host-only operations outside the unchanged persisted study set.
- R14/R15 preserve authorized starts, fresh map sessions, typed outcomes and every legitimate
  vendored hook. R16 cannot trade allocator-domain counters for complete foreign admission.
- R17 retains the bytes identity representation supported by the selective hook probe; R18
  preserves signed/precision-bearing physical time rather than replacing it unconditionally.
- R20 reconciles actual covariance content and admits semidefinite mathematical validity, with
  external assumptions explicit. R22 separates product contracts before moving toolkit code.

**Remedy-open conflicts.** Uno numerical patch removal would lose SQP-LBFGS; asynchronous
readiness events can lose work; one mutable Salsa head shared with pinned durable lanes would
cancel valid work. Those routes are not selected. Necessary commit-order sequencing is retained;
the breadth of local pacing remains an investigation, not an excuse to remove that guarantee.

**Leads and decisions remain distinct.** Non-parameter study bindings, deeper structural
specialization and project-structure sizing remain leads/candidates. Collector triggers,
supervising explicit strategies, Shooting as a study operation and DA-024 purpose remain
maintainer decisions. R3, native allocator hooks, LM tolerance changes, alias fusion,
projected IPC decode and tracing substrate each retain their settling conditions.

**Material uncertainties.** Guarded regional product feasibility/benefit, revised bounded
retention under repeated edits, production RocksDB assertions/plans, 16-lane pacing magnitude,
native allocator bypass/ownership, schema-set features and tracing cost remain unresolved.
They constrain remedy adoption; none changes the current Revise verdict.

**Next consequential decision.** In plan creation, confirm the desired resolved-representation
rule impact RC01 and bounded retention/default policy RC03. R2's refusal vocabulary and the
independent F01/F02 correctness repairs do not need to wait for either redesign. R20/F10 now
needs an explicit covariance validity/provenance contract (RC18) before its implementation.

<a id="15-challenge-dispositions"></a>

## 15. Historical first-review challenge dispositions

This section preserves the first review's dated challenge account and receipts. Its judgments
belong to the original reviewer at input revision `e08b1f1dd9b89484ab7471f32b66f8e9c2a59a05`;
they are not endorsements of the revised remedies or new tests. The independent recommendation
auditor's net changes are in the linked companion; §§1–14 above carry the integrated judgment.

Three independent challenges of the first draft were assessed:
- **C1:** omission, preservation and enablers.
- **C2:** numerical and scientific guarantees.
- **C3:** library-claim verification.

The original reviewer reported re-reading the decisive source for each high item before
accepting it. The first draft was preserved separately by the coordinator. The exact audited
principal input is recoverable at the revision recorded in the companion.

| Challenge item | Severity | Disposition | Reason |
|---|---|---|---|
| C1 · R1 blanket `READONLY` would refuse guard touches, study cancel, retention guard, file-intent recovery and operation compaction | High | **Accepted** | Verified at surql `:799`, `:1437`, `:2013`, `:2142`, `:2807`; R1 now limits `READONLY` to immutable columns and uses transition rules |
| C1/C3 · R8's LRU cannot stop rotation: evicted memos still count as entries, and evicted memos cannot backdate | High | **Accepted** | Verified at salsa `function/memo.rs:354-367`, `database.rs:446`, `function/backdate.rs:22-25`, and probe B2's `query_values` 2²⁰ row; R8 now changes the criterion and keeps backdating memos resident |
| C2 · R7's joint evaluator over flattened "substitutable" expressions drops Domain, Applicability and order-qualified Require | High | **Accepted** | Verified at `execution.rs:1983-1991`, `:6357-6420`, `:6553`, `guarded.rs:96` and `probe_math_probe.rs:209-215`; R7 is redefined over the stage program and probe A relabelled an upper bound |
| C2 · Per-instance validity and premises must enter the shared-program identity | High | **Accepted** | Verified at `executable.rs:427-452` and `group.rs:123-200`; the second identity is retained and re-framed |
| C1 · R3 is not a clear win: the script's whole-table key paging uses the index | Medium | **Accepted** | Verified at `surreal_server.py:5749`, `:7275`; R3 is now a candidate |
| C1 · R21 leaves the script as a direct writer, contradicting "guarded functions only" | Medium | **Accepted** | Operator mutations move into generated guarded maintenance functions; RC17 added |
| C1 · `StudyOperation` is the persisted study record; Shooting would be new study capability | Medium | **Accepted** | Verified at `study_operations.rs:336-352`; R11 shares `OperationRequest` + `PreparationSettings`; Shooting is a maintainer decision (RC12) |
| C1 · R12 overstated: Simulation builds no `IntegratedExperiment`; parameters only | Medium | **Accepted** | R12 is limited to parameter-valued points with a binding builder for Simulation; the rest is a lead |
| C1 · DA-012 strategy family and dispatch subsets left without a disposition | Medium | **Accepted** | Dispatch folded into R11; strategies get a stated exception (RC13) or a maintainer decision |
| C1 · DA-015 admission composition and duplicate deployment sequence uncovered | Medium | **Accepted** | R11 extended; ADR-0164's per-host assertion kept |
| C1 · R2 omits the DA-008 retry-by-shape and lost-writer inference sites | Medium | **Accepted** | Both sites are named as consumers of the typed class and cause |
| C1 · R22 would make the observation hub a new foundation dependency | Medium | **Accepted** | Placement limited to capture and review tooling; the hub goes to the tracing lead |
| C1 · DA-025 collector and payload growth undispositioned; "no liveness capability" unsupported | Medium | **Accepted in part** | Collector is now a maintainer decision with daemonless triggers. The rustix `pidfd` point is accepted, but it confirms the existing owner-identity design rather than offering a simplification |
| C1 · R7 could fold observation-only outputs into solver iterations | Medium | **Accepted** | The joint unit is the demanded closure per product, with observation partitioned out |
| C1 · DA-028 "challenged (magnitude)" overreads one fixture | Medium | **Accepted** | Relabelled: retained, magnitude unresolved, addressed by R7 |
| C1 · "Remedy open" used beyond the brief | Medium | **Accepted** | Relabelled as lead, maintainer decision or outside focus; remedy open kept only for regression conflicts |
| C2 · Extend existing symbolic product routes; measure per `ProductKind`; admit before differentiating | Medium | **Accepted** | Verified `SelectedFirstRegion` (`execution.rs:4486`) and direct products (`products.rs:186-330`); R7 restated |
| C2 · Joint unit per demanded closure; deterministic route choice in identity | Medium | **Accepted** | Added to R7 |
| C2 · Fixed-point maps need a map-specific route, no derivative demand, a restart schedule, `original_residual` qualification and fresh sessions | Medium | **Accepted** | R14 and the §7 row corrected |
| C2/C3 · LM `gtol` is a cosine test, not the repository's stationarity | Medium | **Accepted** | Verified at `lm.rs:177-182`; F05 stays a candidate with a declared conversion and a no-new-unqualified-fits acceptance |
| C2 · R4 sort key must be declared per relation (trajectories are symbol-major) | Medium | **Accepted** | R4 step 1 rewritten |
| C2 · R12 start and IC source | Medium | **Accepted** | Declared or recorded seed; consistent ICs per point; IDAS retention gated |
| C2 · R10 bridge must return `Cancelled`; partial horizons; inherited deadline | Low–Medium | **Accepted** | Verified at `math/jobs.rs:1461-1466`; R10 and F01's acceptance updated |
| C2 · Recovery-profile permission passed in; arclength preconditions stay with the path owner | Low–Medium | **Accepted** | R14 updated |
| C3 · R4: IPC decodes whole record batches; prefer `FileReader::set_index`; exact statistics needed | Medium | **Accepted** | Verified at `result_snapshot.rs:89`, `:390` and `file_batch_index`; statistics are per record batch on the existing block row |
| C3 · `Indeterminate::Function` "as Dualizer does" unverifiable | Low | **Rejected** | `evaluate/dual.rs:558-570` builds parameters as `function!(dual_arg, index)` and converts them with `Indeterminate::try_from`. A term-order condition was added (C2 low, **accepted**) |
| C2 · Uno `NoRelaxation` override could be re-applied | Low | **Rejected** | Verified at Uno `805a27ec`: overrides flow into `Result` and are read only by the C API (`Uno_C_API.cpp:1187`). The serial-controls determinism note is **accepted** |
| C1 low · `SUNContext_ClearErrHandlers` needed to stop stderr; heap_size callbacks stay project-written; block statistics granularity; §18.8 FERAL RC; missing RCs; priority ordering; DA-030 in part; reverse checks for the PETSc limit and root-level closure; enum-map attribution | Low | **Accepted** (all) | Corrected in §6, R8, R4, R16/RC15, RC12–RC17, §11, §10, R16/R8, R11 |
| C2 low · hot_start identity, truncated-evaluator retention, `peak_bytes` a-priori only | Low | **Accepted** | §6 candidates, R7 and R16 updated |
| C3 low · strum `EnumDiscriminants` not derived; POUNCE ladder is public; `RecordBatch::claim` needs `arrow/pool`; hot_start refused on direct translation; `$this` in ASSERT upgradeable; `PyOnceLock` per-`T` cell; citation fixes (`error.rs:449`, `core.rs:1042`, `codegen/surreal.rs:135`/`:148`, `EvaluatorBuilder::direct_translation`, LIVE verifier wording) | Low | **Accepted** (all) | Corrected in place |

**Net effect.**
- **Decision.** Unchanged: Revise.
- **Verdicts (at that historical challenge boundary).** No foundation or gate verdict changed.
- **R1, R7, R8.** Corrected so they no longer carry a latent regression or rest on a mistaken
  library premise.
- **R3 and the FERAL pre-admission.** Downgraded to candidates.
- **New rule impacts.** RC12–RC17 added.
- **Disposition counts.** Changed from 27/3/2 to 28 addressed (5 in part), 2 challenged in part and
  2 retained without a review remedy.
