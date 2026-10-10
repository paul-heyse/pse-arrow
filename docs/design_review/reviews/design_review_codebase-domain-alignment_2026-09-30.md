# Design review: whole-codebase domain-model alignment and library leverage

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | The whole production codebase at commit `550f35cb` ("plan 23 close out"). Production code is unchanged at `0c55c78a`: `git diff 550f35cb..0c55c78a` is empty over `crates/`, `packages/`, `python/`, `tests/`, `xtask/`, the architecture sections and the ADRs. The review covers all 27 `pse-*` crates, hand-written `python/pse`, the authored knowledge in `packages/reference` (after Plan 23 this *is* the domain model) and generators judged as authorities. xtask, testkit and benches are covered only where they hold domain decisions. **Excluded:** Plan 24 / `thermo-knowledge/` (by maintainer direction, not used as a reference model), `external/`, build output, CI and governance tooling, and new scientific qualification runs. |
| Standard | Core **3.3**, process-simulator profile **1.3**, binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Design tier, target purpose** (the binding default). The *focus* is the §1 semantic-model-first MUST, assessed through AP-04 and G9 (model adequacy **and** authoritative realization), and then G8 library leverage. Every foundation and gate still gets a verdict. Authority text that blocks the target is listed in [slot 11](#11-authority-changes-exceptions-and-disposition). |
| Reviewer / date | Maintainer-requested review, 2026-09-30. It was coordinated by the main agent session and carried out by twelve `design-reviewer` agents (Opus) working in four waves: ten concept, subsystem and library reviewers, then two adversarial verifiers who had authored none of the findings. This is **not an independent human review**. All reviewers ran in the same agent system. |
| Decisions | Behavioural and semantic adequacy: **not adequate** (G1, G2, G7, PS-G1 and PS-G3 fail). Architectural fitness: **G9 fails** (AP-02, AP-03 and AP-04 are violated; AP-01, AP-05 and AP-06 are violated within bounds). **Overall: Revise.** See [slot 12](#12-decision). |
| Disposition owner | None yet. Reviews are evidence, not authority. This review names *proposed* owners only (slot 11). Findings take status when the maintainer adopts them into a plan, an ADR or a register row. |

**Functional target.** The profile's target is a simulator that lets an engineer author physically
typed, reusable unit and property models once, compose them into flowsheets and studies, and solve
them in several modes (square, optimization including discrete, dynamics, estimation). Results must
be physical, diagnosable and fast enough for edit→re-solve loops, sweeps and recycles. The
adequacy reference is:

- that target;
- the [Plan 20 capability target](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/20-idaes-capability-target.md);
- IDAES 2.13.0 behaviour, read clean-room.

The repository's own commitments apply as well:

- D1–D14 ([§2](../../authoritative_design/sections/architecture-overview.md));
- the Plan 21 knowledge-boundary test ([placement guide](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/21-knowledge-placement.md));
- Plan 23's typed package model.

**Drivers and credible variation axes.**

- New property-method families (activity, cubic and multiparameter EOS variants) and data banks.
- New unit operations and flowsheets, including recycles.
- New analysis modes and study kinds.
- New solver backends and outcome categories.
- Storage and transport representation changes.
- DataFusion and Arrow majors.
- Vocabulary growth (terminations, backends, diagnostic codes).

**Baseline evidence cited, not re-run** (Plan 23 Outcome, local Linux, `force_validate`,
memory-capped):

| Run | Result |
|---|---|
| `just seed-conformance` | 115/115 seed fixtures (4,046 checks), 6/6 domain fixtures |
| `just test --profile ci` | 1,991/1,991 |
| `just native-test --profile ci` | 2,306/2,306 |
| native acceptance | 17/17 |
| `just native-python` | 173/173 |
| `just publication-test` | 9/9 |
| value-only study (*Measured*) | 1,000 points, one structural view |

This review ran **no builds, tests or probes**. Its claims are *Implemented*, meaning source-traced
at the baseline, or *Interface-checked*, meaning declarations or pinned library source inspected.

**Method.** The review used three axes:

1. **Concept dossiers C1–C9**, which follow a domain meaning across crates:
   - C1 physical quantity and conventions;
   - C2 material system;
   - C3 process structure;
   - C4 mathematical problem;
   - C5 analysis mode;
   - C6 cases, studies and reuse;
   - C7 outcomes and diagnostics;
   - C8 results and publication;
   - C9 identity.
2. **Subsystem units U1–U8**, which follow responsibility owners:
   - U1 registry and derivation;
   - U2 authoring and modeling;
   - U3 preparation and mathematics;
   - U4 native execution;
   - U5 runtime orchestration;
   - U6 relational data;
   - U7 persistence and publication;
   - U8 the Python boundary.
3. **A library sweep**, checked against the pinned versions using the library skills and pinned
   crate sources. Each candidate got a §F record.

Every candidate finding was then re-read and challenged by a verifier who had not produced it.
Findings were merged by structural cause. None was disproved outright, but 3 were weakened and
many amended, including five reframed as target gaps against accepted decisions. The working
notes were not committed; this review is the one artifact.

**Not examined in depth:**

- electrolytes and reaction packages beyond `reactors.pse`;
- the pse-compiler lowering of `partial` over `Fn` parameters;
- the implicit and cubic-root accelerators;
- the authoring driver's document hydration;
- pse-operations-queries SQL beyond the study and schema paths;
- benchmarks;
- runtime numerical accuracy, which this review makes no claims about.

## 2. Decomposition, ownership and dependencies

Dependency edges, derived from the manifests (`pse-*` only):

```
foundations  diagnostics ← ids, vocabulary ← quantity, columnar
declaration  schema → {columnar, ids, quantity, vocabulary}; model → {ids, quantity, vocabulary}
             relations → {columnar, model, schema}
data         engine → relations; catalog → engine; rules → engine; operations → model (+ queries)
mathematics  kernels → model, quantity; structural → kernels; math → kernels, model
             authoring → model, quantity; modeling → authoring; compiler → modeling, math, structural
             backend-native → math, structural, kernels (not compiler/modeling)
orchestration runtime → every crate except rules, py, codegen
boundary     py → runtime, engine, catalog, rules, relations, schema, modeling, … (14 crates)
```

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State / effect owner | Local test setup |
|---|---|---|---|---|---|
| `pse-quantity` with `packages/reference/physical` | Kind, dimension, basis, datum, scale, shape and subject algebra; unit products; derived kinds (ADR-0124) | `QuantityRegistry`, `infer_*`, `convert_spec_for_type` | Foundation; data authored in `physical.yaml`, a registry-declared document (ADR-0064) | Pure | Generated fixture registry; no runtime |
| Registry and codegen (`pse-vocabulary`, `pse-schema`, `pse-model`, `pse-codegen`, `pse-relations`) | One declaration per durable shape (D1); semantic witness and exact compatibility; generated Rust, Python, Postgres | Immutable `Registry`; generated rows and enums; `ValidationContext` | Arrow-native by design (§4.1); postgres value mapping is an optional feature | Registry memo; ambient first-wins validation context ([f31](#f31)) | Pure, except SQL row checks need an engine binding |
| Authoring and modeling (`pse-authoring`, `pse-modeling`) | Grammar; checking; keyed identity; provenance and taint; envelopes; specialization | `check`/`check_with` → `CheckedPackage` (fields `pub(crate)`) → `SpecializedModel` | Pure; Arrow-free | Stateless | Kernel `#[cfg(test)]` fixtures (about 5.7K lines, not compiled into production) |
| Preparation (`pse-compiler`) | Salsa workspace; body admission; case views; `view_key` | `PreparedModeling`, `PreparedCase` | Mechanism over meaning | Salsa database; views held on the package ([f15](#f15)) | `CompilerWorkspace::new` plus package rows |
| Mathematics (`pse-math`, `pse-kernels`, `pse-structural`) | Symbolica bodies behind barriers; jets; assembly (faer); facts; factorable DAG; DCP and Gram certificates; provider contracts; matching, DM and BTF via pounce-presolve; topology | `CasePlan`, `ProblemFacts`, `StructuralAnalysis`, `FlowGraph` | Library-owned mathematics (D6 holds; [f25](#f25) wording) | Attempt-owned workers | Pure unit tests |
| Native execution (`pse-backend-native`) | One adapter and one capability record per backend; routing; structural admission; original-space qualification | `BackendExecution`, `Requirements` → `Route`; `Qualification` | Consumes math contracts; defines no domain meaning | Native session on the owning worker | `STUB_TABLE`: routing without solver startup |
| Runtime orchestration (`pse-runtime`) | Case binding; `Staged`; the single usability decision (§16.6); run supervision; durability; studies; publication composition; failure translation | `ModelingPackage` (about 78 public functions), `RunHandle`/`RunResult`, `StudyPlan` | The only crate joining data and mathematics (by design) | Budgets, sessions, attempts | Pure numerics; everything else needs a native session or Postgres ([f05](#f05)) |
| Relational data (`pse-columnar`, `pse-engine`, `pse-rules`) | Buffer ownership; metadata purposes; DataFusion session assembly; plan admission; caches; requirement planner | `EngineSession`, `NativeCacheService`, `RequirementPlanner` | Mechanism crates; `pse-rules` has no production composition ([f20](#f20)) | Memory pool, caches | In-memory DataFusion |
| Persistence (`pse-catalog`, `pse-operations`) | Delta member I/O and contracts; the PostgreSQL store and queue; the publication CAS (D10); study conclusion policy ([f05](#f05)) | `Catalog::commit`, repositories, `DeclaredCheck` | Catalog does not depend on Postgres and operations does not depend on Delta; the runtime composes them | All mutable operational state | Live Postgres for store logic |
| Python boundary (`pse-py`, `python/pse`) | Request composition and route pins ([f04](#f04)); generated and hand-written documents ([f18](#f18)) | Native classes; msgspec and attrs contracts; Arrow C streams | A second composition root | Process-global runtime | Needs the built extension |

**Composition roots.** There are three:

- `Runtime::from_shared` plus `with_durability`, used by `pse-worker`;
- `pse-py`, which also binds `RegistryRequirementPlanner`, a capability `Runtime` lacks;
- xtask codegen, which runs the registry invariants.

At workflow level, every case solve (conformance, diagnostics and `solve_case` included) goes through
`launch` → `Staged` → `numerics::complete` → `RunResult` → one publication commit. That is a real
strength.

## 3. Contracts, authority and constraints

### 3.1 Authority map (concept dossiers)

| Concept | Authoritative owner | Adequacy | Authority over behaviour | Findings |
|---|---|---|---|---|
| C1 Physical quantity: kind, dimension, basis, datum, point/difference, shape, subject, unit product | `physical.yaml` → `pse-quantity`; derived kinds (ADR-0124) | **Strong model.** Gaps: no datum-free transfer kind, no direction, no reduced-coordinate type; dimensionless `Scalar` absorbs ratios and stripped dimensions | Boundaries compare the complete contract. **Constitutive bodies bypass it** by stripping to `Scalar` | [f01](#f01), [f03](#f03), [f08](#f08) |
| C2 Material system: species, phases, VLE formulation, properties, parameter sets, selection, provenance, envelopes | `pse.physical`, `pse.domain`, `methods`, `thermodynamics` packages, interpreted generically by pse-modeling | **Mostly strong.** Selection and provenance are data; coefficients are fully dimensioned. Gaps: phase key forced on phase-independent sets; envelopes optional | No production Rust branches on species or phase; the knowledge boundary holds for SRK | [f11](#f11), [f12](#f12) |
| C3 Process structure: units, ports, connections, topology, accumulators | `packages/reference/process`; pse-modeling `specialize`; `pse-structural::flowsheet` | **Inadequate:** no stream or aggregate-port concept; the control volume is single-inlet | Balances come from contributions (sound); topology is unused by shipped flowsheets | [f02](#f02), [f16](#f16), [f33](#f33) |
| C4 Mathematical problem: roles, equations, DOF/structure, guards, derivatives, nominals | pse-modeling, compiler, pse-math, pse-structural, backend-native `structural` | **Adequate.** Roles explicit; one structural owner; guards become barriers before simplification; no finite-difference fallback; nominals are model attributes | One admission policy; the DOF *number* has four rules | [f16](#f16), [f29](#f29) |
| C5 Analysis mode and problem class | Registry route, intent and class enums; `routing::problem_classes`; capability records | **Adequate.** One model across modes (CT-S05 holds). Gaps: sensitivity for square systems; terminal-event completion | Class derived in one function with no fallback (PS-09). **The case's declared execution is ignored by most entry points** | [f04](#f04), [f10](#f10), [f22](#f22) |
| C6 Cases, studies and reuse | Compiler (Salsa, `view_key`); runtime `study` and in-process engines; pse-operations `studies` | Case projections sound. **"Study" has two definitions** | Keys are complete; in-process reuse is *Measured*. Durable path re-prepares | [f05](#f05), [f15](#f15) |
| C7 Outcomes and diagnostics | Registry `NativeTermination`, `TrajectoryTermination`, `NativeRunState`, `CandidateUse`, `NativeQualification`, `ClosureAssessment` (the §0.5 facts); `pse-diagnostics` codes; runtime classes (§23.2) | **Outcome facts adequate and separate.** Diagnostic taxonomy unreconciled; rules are open strings | One usability owner for case solves; shooting bypasses it | [f06](#f06), [f07](#f07), [f09](#f09), [f17](#f17) |
| C8 Results, provenance, publication | `RunResult` (contains per-step `ModelingResult`); registry `runtime.*`; `pse-operations` catalog CAS (D10) | **Adequate.** Lineage complete; results typed by quantity and unit id | One commit boundary; definitions never mutated (DP-05, D13) | [f13](#f13), [f14](#f14) |
| C9 Identity | `pse-ids`; member identity `pse.modeling.member.v1`; frame catalog with golden vectors | Layers distinct; library handles private | Canonicalization is a call-site convention; hash roles untyped; bindings stored as paths | [f08](#f08), [f30](#f30) |

**Specification against implementation.** The review found these divergences:

- **§14.4** describes per-definition reuse that the production path does not take ([f15](#f15)).
- **§4.6** says stage 3 runs `pse_rules::invariants` at admission; no production path does
  ([f20](#f20)).
- **§5.1** says case targets are stored by identity; they are path strings ([f08](#f08)).
- **§16.2, §15.3 and D6/§7.5** have smaller drifts ([f35](#f35)).
- **§19.3** specifies both divergent study rules ([f05](#f05)).
- **§25 lists shooting "authored or in Rust"**, but the Rust route has no usability decision
  ([f09](#f09)).

### 3.2 Physical-semantics table (profile slot 3)

| Quantity or element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| T, ΔT | Θ, K (point vs difference) | — | absolute; no datum | data: `temperature_correlation.T`; closure per binding | `Temperature`, `DeltaTemperature` |
| p (absolute / gauge) | Pa | — | gauge = `package_datum` 101325 Pa, stated three times ([f35](#f35)) | closure: bt-ideal, bt-pr | `Pressure` |
| F, Fⱼ, xⱼ | mol/s; 1 | molar only (no mass-basis state or flows) | xⱼ is a species-subject ratio | bounds | `Flow`, `ComponentFlow`, `MoleFraction` |
| h, s, Δh | J/mol, J/(mol·K) | molar | `stock` datum at process interfaces | guards over [T0, T] | `MolarEnthalpy`, `DeltaH` |
| Heat, work | W | — | **typed `Power` = energy_flow@stock; the sign lives only in the contribution role** | — | `control-volumes.pse` ([f03](#f03)) |
| EOS coordinates t, r, n | **reduced by 1 K, 1 mol/m³, 1 mol; typed `Scalar`** | — | convention only | form `valid` guards only | `helmholtz.pse` ([f01](#f01)) |
| PC-SAFT m, σ, ε/k; T_c, P_c, ω | 1, m, K … | — | filed under `vaporPhase` ([f11](#f11)) | positivity only ([f12](#f12)) | `pcsaft-parameters.pse`, `cubic.pse` |
| Reactor extent; costing area | typed `Scalar`; the unit is in a label string | — | — | — | `reactors.pse`, `costing.pse` ([f01](#f01)) |
| Study and initialization overlays | **bare f64 in an implicit canonical unit, keyed by path** | implicit | implicit | — | `PointOverlay` ([f08](#f08)) |

### 3.3 Well-posedness statement

- **Roles.** Roles come from declarations (`var` free, `param` input) plus a typed case overlay
  (`fix`/`free`). A value on a free variable is a start with a `StartSource`. Nothing infers a role
  from the presence of a value.
- **Structural analysis.** It runs on the bound case view (`structural_plan`, via pounce-presolve
  HK/DM/BTF). Under- and over-determined structure is refused before the native call for the Roots,
  Initialize and NLP representations, and dynamic partitions are admitted through the same path.
- **What is not admitted.**
  - The coefficient and cone routes skip matching legitimately; §15.2 does not say so.
  - The factorable (SCIP) route skips it without a class reason ([f29](#f29)).
- **Diagnostics.**
  - Refusals carry row and column `SemanticId`s.
  - Model paths are added only by `diagnose_case` ([f35](#f35)).
  - The DOF *count* shown by conformance follows three mode-specific rules, plus a fourth
    squareness rule in fitting ([f16](#f16)).

## 4. Change scenarios and composition

The seeds are PSE-S01–S06 from the [binding](../design_principles/binding/pse-arrow.md#architecture-scenarios) and the profile journeys.

| ID | Scenario / stimulus | Expected response | Observed or predicted impact | Evidence |
|---|---|---|---|---|
| <a id="s01"></a>S01 | Add a property-method family (activity or cubic variant) under the knowledge-boundary test (PSE-S01) | One package declaration plus data | **Holds for Rust locality:** SRK was added by data (`cubic.pse srk`, `srk_kappa`). A PR alpha variant or UNIQUAC is predicted package-only. **Fails for adequacy:** a new EOS is authored over reduced `Scalar`s, and typing it lawfully needs an edit to `physical.yaml` in another package, so authors strip units ([f01](#f01)). A τ/δ-reduced Helmholtz law binds silently | Implemented / Interface-checked |
| <a id="s02"></a>S02 | Add a unit operation using existing physics (PSE-S01) | Package-only; balances from contributions | **Holds for a single unit:** HeatExchanger composes two Heaters; shared conformance checks attach automatically. **Fails for flowsheets:** no stream concept, so a flowsheet wires 4 to 5 scalar equations per stream ([f02](#f02)); multi-port units re-declare balances ([f33](#f33)) | Implemented |
| <a id="s03"></a>S03 | Add or replace a solver for an existing class (PSE-S02) | One capability record plus an adapter | **Mostly holds:** routing and admission are record-driven with no fallback. Leaks: `objectives.rs` re-derives lexicographic eligibility ([f21](#f21)); a new backend adds a `NativeBackend` member, which forces a store reset ([f13](#f13)) | Implemented |
| <a id="s04"></a>S04 | Compose a new analysis workflow: sensitivity or estimation over a sweep (PSE-S03) | Reuse prepare, bind, solve, qualify and publish | **Partly holds:** every case solve composes `Staged`. Sensitivity of a square flowsheet needs a dummy objective ([f22](#f22)). Study points are case solves only. Study rules exist twice ([f05](#f05)). The route is chosen per entry point ([f04](#f04)) | Implemented |
| <a id="s05"></a>S05 | Change a result storage or transport representation (PSE-S04) | Persistence owners absorb it | **Fails:** a store-table or enum change resets the store, losing the publication catalog and orphaning members ([f13](#f13)). Declared Delta migrations have no reader ([f13](#f13)) | Implemented |
| <a id="s06"></a>S06 | Test admission or routing policy locally (PSE-S05) | Pure inputs, with no store or solver | **Holds** for routing (`STUB_TABLE`), usability (`stop_use`, `native_use`, `complete`), the compiler, and lifecycle tables. **Fails** for study policy, which needs Postgres or a native session ([f05](#f05)) | Implemented |
| <a id="s07"></a>S07 | Edit→re-solve; value-only study | A value change never rebuilds structure; warm start recorded | **Holds in-process** (*Measured*: 1,000 points, one view). Warm starts are framed into lineage. **Fails durably** (each job builds a fresh package and Salsa workspace). One equation edit re-admits every body ([f15](#f15)) | Implemented; Measured (in-process only) |
| <a id="s08"></a>S08 | Out-of-envelope evaluation; ill-posed problem | Rejection or recorded extrapolation; diagnostics in model terms | **Holds where declared:** typed Validity plus recorded extrapolation with its layer. EOS, pair and activity data declare no envelope ([f12](#f12)). Structural refusal names IDs, and paths appear only in `diagnose_case`. Durable failures keep prose only ([f07](#f07)) | Implemented |
| <a id="s09"></a>S09 | Round trip of units, basis and identity through authoring → store → Python | Types preserved at every leg | Authoring, results and Arrow C streams are typed. **Fails** for study and case overlays: bare floats keyed by path, then hashed into durable identity ([f08](#f08)) | Implemented |
| <a id="s10"></a>S10 | Add an outcome or termination class | One authority; compile-forced consumers | The registry is one authority and `stop_use` is compile-forced. Diagnostic class, retry and "is a result" partitions default silently ([f06](#f06), [f17](#f17)). The store must be reset ([f13](#f13)) | Implemented |
| <a id="s11"></a>S11 | Upgrade an integration library major (DataFusion) | Integration owners absorb it | Authoring and modeling are Arrow-free (good isolation). **Every published member table with an adapted field becomes unreadable**, because DataFusion-proto predicate bytes are persisted ([f14](#f14)) | Implemented / Interface-checked |

**Classification.** The most significant changes are **new core concepts**, and they legitimately
change several owners:

- aggregate ports and stream connections ([f02](#f02));
- reduced-coordinate or anonymous intermediate physical types ([f01](#f01));
- a datum-free, directed energy-transfer kind ([f03](#f03)).

The *failure* the review identifies is different: in their absence, ordinary *instances* such as a
new EOS or a new flowsheet are forced to re-express meaning locally.

## 5. Mechanisms and execution, where material

Numerical stage columns (profile slot 5):

| Stage / owner | Formulation policy | Derivative source and order | Scaling | Problem class · capability | Status → outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|---|
| Formulate: specialize → compiler `projection`/`grouped::admit` → `typed_math` → `BodyBuilder` | Obligations become barriers before Symbolica normalization; complementarity, disjunction and smoothing are named, recorded realizations | Symbolica symbolic, order ≤ 2; providers declare source and order; above-order is refused; no finite-difference source | Authored nominal and scale hints (`DerivedNominal`, `ModelHint`) | Facts from the admitted program | Typed `MathError`, `CompileError` | Accumulator tolerances declared by packages |
| Prepare view: `bound_structure`, `prepare_view`, `structural_plan` | Case states applied; factorable projection with per-row `Fidelity` | faer patterns fixed once | — | `ProblemFacts` (coefficients, DCP or exact Gram) | `ProjectionError` | — |
| Evaluate: pse-math workers; implicit nested solve | Guards enforced per evaluation | Taylor jets; implicit via IFT (faer LU), accuracy follows the inner tolerance | Original coordinates | — | `MathError::Domain` attributed to its occurrence | Nested budgets from policy |
| Admit and route: runtime `solves.rs` + backend-native `structural`, `routing` | Structural matching for Roots/NLP only ([f29](#f29)) | Capability requires an order | — | `problem_classes`; one `admit` rule; explicit selection never falls back | `ProblemError::{Structural, Unsupported, …}` | `ResolvedAccuracy` from policy |
| Solve: adapters | No bound relaxation (`bound_relax_factor = 0`) | Exact Hessian, L-BFGS, Jacobian products, coefficients | `x = Sx z`, row scaling | One adapter per class | Each adapter maps once; raw code kept; unknown → inconclusive | Scaled native budgets |
| Qualify: backend `quality` + runtime closure | Independent original-model re-evaluation | — | Unscaled, per physical unit | Qualification by evidence, never by backend | `Qualification`; `CandidateUse` | Physical tolerances; closure from contributions (sound). Authored closures are misreported ([f16](#f16)); shooting bypasses this stage ([f09](#f09)) |

**Reuse (DP-09/10):**
- **Salsa keys are complete** for what the modeling queries read. Failures and cancellations are
  not memoized.
- **Granularity on the production path is per root selection:** all bodies are admitted inside one
  tracked query. The value-only path backdates at `Projection`.
- **Views are prepared outside Salsa** and held per package object ([f15](#f15)).
- **Native artifacts are reused** through DataFusion `DefaultCache` across edits.

**Publication (DP-19).** Publication is one PostgreSQL transaction with an intent lock, a
finished-attempt check and a CAS on the head. Retry applies only to the infrastructure class, and
settlement is typed and never rolled back.

## 6. Architectural assessment and gates

| Foundation | Scenario and evidence / scope reason | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | **Strengths:** layering holds (the mathematics branch has no Arrow, DataFusion or Delta; catalog and operations are independent; backend-native consumes meaning rather than defining it; compiler and modeling are pure). **Violations:** durable study policy is SQL transaction script in the persistence crate ([f05](#f05)); conformance is the only interpreter of a case's declared execution ([f04](#f04)); one store fingerprint couples queue lifetime to catalog lifetime ([f13](#f13)) | **violated (bounded)** | f04, f05, f13 |
| AP-02 Stable contracts | **Strengths:** the execution seam (capability records, `Representation`), the `Durability` seam, and DataFusion types as an intended shared contract. **Violations:** swapping the study executor changes semantics ([f05](#f05)); contract evolution has no compatibility classes ([f13](#f13)); a DataFusion major makes published data unreadable ([f14](#f14)); public shooting has no outcome contract ([f09](#f09)); Python documents bypass generation ([f18](#f18)) | **violated** | f05, f09, f13, f14, f18 |
| AP-03 Composition | **Strengths:** every case solve composes `Staged` and one usability owner; methods compose by selection; unit composition is package-only (S02). **Violations:** flowsheets compose by equations with hand-authored tears ([f02](#f02)); the square-sensitivity primitive is private to fitting ([f22](#f22)); lexicographic policy is repeated ([f21](#f21)) | **violated** | f02, f21, f22 |
| AP-04 Domain model and semantic authority | **Adequacy gaps:** no stream or aggregate port ([f02](#f02)); no reduced-coordinate or intermediate typing, so constitutive laws strip units ([f01](#f01)); no datum-free or directed energy transfer ([f03](#f03)); a phase sentinel ([f11](#f11)); optional envelopes ([f12](#f12)); undecided terminal events ([f10](#f10)). **Authority gaps:** a case's execution is decided per entry point ([f04](#f04)); "study" has two definitions ([f05](#f05)); failure code, class and termination maps are unreconciled ([f06](#f06)); shooting bypasses §16.6 ([f09](#f09)). **Strengths:** the typed package model, selection as data, provenance and taint, the five separated §0.5 outcome facts, no science in Rust, balances from contributions | **violated** | f01–f06, f09–f12 |
| AP-05 Explicit structure | **Strengths:** `CheckedPackage` construction cannot be bypassed; pure lifecycle tables; capability records; typed `Start`/`StartSource`. **Violations:** conventions callers must remember (canonical units in overlays [f08](#f08), reduced-coordinate scales [f01](#f01), `vaporPhase` [f11](#f11)); `Contract(String)` hides failure semantics ([f06](#f06)); ambient validation installation ([f31](#f31)) | **violated (bounded)** | f01, f06, f08, f11 |
| AP-06 Local reasoning and testability | **Strengths:** routing with a stub table; pure compiler, `pse-quantity`, numerics and lifecycle; kernel fixtures that are test-only. **Violation:** study predecessor, failure and conclusion policy is testable only with Postgres plus a solver or a native session ([f05](#f05)) | **violated (bounded)** | f05 |

| Gate | Result | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **fail** | Two study definitions ([f05](#f05)); code and class maps ([f06](#f06)); lexicographic rule ([f21](#f21)); Python mirrors ([f18](#f18)); unexecuted SQL copies of Rust-owned invariants ([f20](#f20)) | f05, f06, f18, f20, f21 |
| G2 Semantic fidelity | **fail** | Meaning dropped into neutral `Scalar` ([f01](#f01)); the `vaporPhase` sentinel ([f11](#f11)); unitless overlays ([f08](#f08)); untyped transfer convention ([f03](#f03)); prose durable failures ([f07](#f07)) | f01, f03, f07, f08, f11 |
| G3 Validity | **pass**, with unresolved items | Declared invariants are enforced (checking, contract comparison, declared envelopes, Roots/NLP structural admission). Unresolved: factorable structural admission ([f29](#f29)); undeclared EOS envelopes ([f12](#f12)) | f12, f29 |
| G4 Hidden behaviour | **pass** | Tracked queries read only inputs; inspection does not mutate. Notes: first-wins ambient validation ([f31](#f31)); the profile pool's silent re-run of a pure computation ([f27](#f27)) | — |
| G5 Consistency and recovery | **pass** | One CAS commit; restoration of initialization by construction (overlays on a clone; CT-S09 confirmed); typed settlement; no partial output mistaken for complete. Note: orphaned members after a reset ([f13](#f13)) | — |
| G6 Transformation and reuse | **pass** (current) | Reuse keys are complete; no stale reuse was found. Latent: canonicalization regimes ([f30](#f30)); hand-listed profile fields in `view_key` ([f15](#f15)) | — |
| G7 Truthful capability claims | **fail (narrow)** | §14.4 per-definition reuse ([f15](#f15)); §4.6 stage 3 ([f20](#f20)); §25 shooting "in Rust" ([f09](#f09)); §7.5 exactness ([f25](#f25)); §9.8 "physical partial derivatives" over reduced scalars ([f01](#f01)) | f01, f09, f15, f20, f25 |
| G8 Library leverage | **fail (minor)** | Reuse outside Salsa and `DefaultCache` ([f15](#f15)); five enum mechanisms ([f24](#f24)); a second rational type ([f25](#f25)); small utilities ([f26](#f26)). **Strength:** solvers, structural analysis, derivatives, sparse algebra, collocation, caches, tracing and CLI are all library-owned; no hand-written Newton, line search, factorization, retry framework or graph algorithm was found | f15, f24–f26 |
| **G9 Architectural fitness** | **fail** | AP-02, AP-03 and AP-04 violated; AP-01, AP-05 and AP-06 violated within bounds. Not averaged with the strengths | slot 12 |
| PS-G1 Physical consistency | **fail** | Dimension, datum and convention errors can reach results through stripped constitutive bodies ([f01](#f01)), untyped transfer terms ([f03](#f03)) and unitless overlays ([f08](#f08)); EOS, pair and activity envelopes are unenforced ([f12](#f12)) | f01, f03, f08, f12 |
| PS-G2 Well-posedness | **pass** for Roots, NLP and dynamic partitions; **unresolved** for factorable ([f29](#f29)) | Rejection before the solver with row and column IDs; model paths only in `diagnose_case` | f29 |
| PS-G3 Numerical integrity | **fail (narrow)** | The shooting conformance verdict reads `Success` without qualification and counts `Acceptable` as failure ([f09](#f09)). The case-solve usability owner is exhaustive, qualification is independent, and no status is read as success | f09 |

## 7. Findings

Ordering follows the template: correctness and authority first (A), then change cost and testability
(B), then target gaps that need an authority change, then library and minor items. Severity and
labels are those assigned after verification.

### A: correctness and semantic authority

#### <a id="f01"></a>F01: Physical typing is bypassed inside constitutive and unit models

**Principles / gate / scenario.** §1 MUST, AP-04 (realization), AP-02, DP-02, PS-01; PS-G1, G2, G7;
S01, S09.

**Label.** Implemented.

**Finding.** Three mechanisms combine:

- **Neutral scaling passes any type through.** Multiplying by the neutral `Scalar` keeps the other
  operand's complete type (`pse-quantity/src/infer.rs:350–367`; the checker mirrors this in
  `pse-modeling/src/expression.rs:305–329`).
- **Unit constants exist to strip units.** `packages/reference/domain/models/constants.pse`
  declares constants of value 1 (`temperature_scale`, `pressure_unit`, `density_unit`,
  `amount_unit`, …). They carry `provenance(codata_2018, published)`, although 1 Pa is not a CODATA
  value.
- **The lawful route is expensive.** Derived kinds can be declared only in `physical.yaml`
  (ADR-0124: "kinds are never synthesized"). Chain resolution refuses any indexed factor
  (`infer.rs:1493–1498`). There is no reduced-coordinate type.

Constitutive laws are therefore written over reduced, dimensionless numbers:

- about 93 strip and about 109 re-dress sites in 17 files;
- 25 `Fn(…:Scalar…)` signatures;
- Helmholtz, PR, PC-SAFT and NRTL potentials;
- the reactor extent, reported as dimensionless with "mol/s" in a label (`reactors.pse:36–44`);
- Q = UA·ΔT (`heat-exchanger.pse:21`);
- the oracle-datum → `stock` enthalpy shift (`bt-ideal.pse:54`), which defeats the `DatumMismatch`
  refusal.

The Plan 21 knowledge-boundary test did not flag this, because its "done" step measures only whether
Rust changed. The guide's line 12 does call a construct that cannot be placed a kernel gap, but its
signatures are marked "illustrative" (process cause, formerly K03).

**Consequence.**

- A multiparameter Helmholtz EOS written in τ = T_c/T and δ = ρ/ρ_c is **accepted** by the same
  `Fn(Scalar, Scalar, Scalar[])` contract. Pressure, enthalpy, entropy and fugacities are then
  wrong without any refusal. Only an oracle fixture would catch this, and only at seeded conditions.
- Dimensional slips inside reduced algebra are invisible to typing.
- The strongest guarantee in the system, complete-contract typing, holds at boundaries but not where
  unit errors are most likely.

**Correction.** Owners: `pse-quantity` → `pse-modeling` checker → packages.

1. Type intermediates by dimension, anonymously, and require a declared complete type only where a
   value is named (`var`, `param`, `let`, a function result, a port, an `Fn` signature).
2. Add a `Reduced(kind, reference)` type that only the same reference can re-dress.
3. Limit neutral scaling to genuine neutrals.
4. Type the potential signatures physically (`T: Temperature, ρ: Density, n: Amount[species]`), so
   that `partial` yields derived types.
5. Route datum shifts through a typed operation.
6. Retire the unit constants.
7. Only then refuse re-dressing in the checker. A checker rule alone would relocate the workaround.

**Verification.**

- Binding a τ/δ-reduced law is refused at admission.
- `rg` finds no `/constants.*_unit` stripping in `methods`, `thermodynamics` or `process`.
- `extent` is typed as a flow and its report row carries mol/s in `unit_id`.
- Oracle fixtures pass unchanged.

#### <a id="f02"></a>F02: No stream or aggregate-port concept; flowsheets wire units by equations and hand-author their tears

**Principles / gate / scenario.** §1 MUST (adequacy), AP-04, AP-03, DP-06, DP-07, PS-05, PS-08; G9;
S02, S07, the recycle journey.

**Label.** Implemented.

**Finding.**

- `connect` parses one scalar `from -> to` with no ranged or grouped form
  (`pse-authoring/src/language/parser.rs:2552–2560`). Specialization creates one scalar equality
  per connection, and `pse-compiler` `flow.rs:~105–111` emits one binding.
- Mixer and the 1D control volume declare no ports. Flash, Separator and PFR ports can be connected
  only if every consuming flowsheet supplies their connectivity policy (`specialize.rs:645–660`).
- The only `connect`s in all packages are seven per-species lines in `unit-fixtures.pse:43–49`.
- `recycle-flash.pse` wires five streams with 20 `eq`s that reach into child state. Its tear is a
  `stage "tear"` that overrides four equations, because stages can override only equations
  (`check.rs:1503–1521`).
- The topology (`FlowGraph`), exact tear selection and KINSOL recycle machinery therefore have no
  shipped-flowsheet consumer, although §25 lists them as implemented.
- "Delegated and aggregate ports" is an undispositioned row of the Plan 20 target
  (`20-target-modeling-and-flowsheets.md`). Plan 23's F06 resolution, which covers scalar delegated
  ports, is not contradicted.

**Consequence.**

- PS-05's topology structure is absent for real flowsheets.
- Each recycle needs a hand-picked tear coupled to equation names.
- Adding a species edits every flowsheet.
- Connecting dependent port families over-specifies the stream, and that surfaces only as a DOF
  failure.
- Flowsheet-wide closure and stream tables cannot be derived ([f16](#f16)).

**Correction.** A new core concept, so a multi-owner change is legitimate:

1. Registry value arm: port groups with member roles and index domains; connections gain indices and
   group endpoints.
2. Grammar and render.
3. Group compatibility checking.
4. Expansion to members that keeps **one** `Connection` occurrence carrying all bindings (the shape
   `pse_structural::flowsheet::Connection` already has).
5. Stages or strategies tear a connection occurrence.
6. `flow.rs` emits every binding.
7. Packages expose `inlet`/`outlet` groups, and `RecycleFlash` is re-authored.

Short ADR plus §12 amendment.

**Verification.**

- `RecycleFlash` has one `connect` per stream.
- `prepare_flow` yields the five-unit cycle, and tear selection picks the recycle connection.
- Adding a component changes no flowsheet line.

#### <a id="f03"></a>F03: Energy-transfer terms have neither datum-free nor direction typing

**Principles / gate / scenario.** PS-01 (reference states and sign conventions are part of the
type), AP-04 (adequacy), DP-02; PS-G1, G2; S01, S02.

**Label.** Implemented.

**Finding.**

- Add/Sub dispatches only to the built-in `additive` rule, which requires equal kind, basis, datum
  and subject (`infer.rs:311–312, 574–628`). None of the 91 registered operations is an Add or Sub,
  and scalar conversions may not change kind (`registry.rs:990–998`).
- All four `energy_flow` types carry a datum. Heat and work are `Power` = energy_flow@`stock`
  (`control-volumes.pse:50–51`).
- The direction exists only as `contribute … role positive` inside the owning accumulator.
- `heat-exchanger.pse:20` (`hot.heat + cold.heat == 0`) relies on an undeclared convention.
- The datum-free `ReactionHeat` is laundered into the balance (`reactors.pse:38`).

**Consequence.**

- A duty's type asserts a datum it does not have.
- Packages with different native datums cannot share typed heat terms without stripping
  ([f01](#f01)).
- A unit whose `duty` means "heat removed" has the same type as a heater's `heat`, so a flowsheet
  equation binding them silently flips the energy balance.

**Correction.** Owners: `pse-quantity`, `physical.yaml`, process packages.

- Add a datum-free energy-transfer kind with an additive rule family: a point or difference at
  datum X, plus or minus a datum-free difference of the same dimension, yields datum X.
- Add a direction or convention facet (into or out of the owning control volume) that contribution
  roles check. Cross-unit equations then require equal conventions or explicit negation.
- Changes D5 and the §8.3 Add/Sub row (ADR).

**Verification.**

- Adding enthalpy flows at two different datums still refuses.
- `heat: HeatRate` composes without `power_unit()`.
- Binding an "out" duty to an "in" heat is refused at specialization.

#### <a id="f04"></a>F04: An authored case does not own its analysis execution

**Principles / gate / scenario.** AP-04 (authority and adequacy), AP-03, AP-01, DP-01, PS-08,
PS-11; G1, G2; S04, S06.

**Label.** Implemented.

**Finding.** A case declares `run <execution>` (`ModelingFixtureExecution` = pure | steady |
initialized | integrated | simultaneous | shooting). That enum fuses the time route with the
procedure. It also declares `intent`, a solver policy and an initialization strategy.

- `declared_analysis(root, route, …)` takes the route from its caller and never reads the execution
  (`pse-runtime/src/workflow/modeling.rs:502–537`).
- The only execution→route map is inside conformance (`conformance.rs:1094–1101`). Production
  `declared_analysis` depends on `conformance::fixture_solver`/`fixture_limits`, so the product
  depends on the checker.
- `pse-py` hard-codes `ModelingAnalysisRoute::Steady` in 12 entry points (diagnose, explain,
  initialize, the in-process and durable study, inspect, and the flow, recycle and block preparation
  functions). It also assembles `StudyPlan`s and duplicates runtime checks.
- Python `initialize` builds its stages and homotopy from its own defaults, which restate
  `engines.rs:62–74`, not from the case's authored strategy.
- `declared_shooting` has only a test caller.

**Consequence.**

- A `run simultaneous` case cannot be initialized, diagnosed, inspected or studied in its declared
  mode. Unguarded models are silently specialized steady.
- A `run initialized` case initialized from Python ignores its authored strategy (PS-08).
- A new route or procedure edits the conformance loop, `declared_*` and every pse-py entry point.

**Correction.** Owner: `pse-runtime` `workflow/modeling`.

- One `declared_execution(case)` returns a typed plan: route, procedure {check, solve,
  initialize(policy), integrate(profile), shoot(problem)}, intent, solver policy and limits.
- A caller override must agree with the plan or is refused.
- Conformance, pse-py and both study executors consume the plan.
- Split the registry enum into route plus procedure.
- pse-py only translates.

**Verification.**

- There is no `ModelingAnalysisRoute::` literal in pse-py.
- A `run simultaneous` case is studied and initialized from Python without passing a route.
- A conflicting route is refused.

#### <a id="f05"></a>F05: "Study" has two definitions with divergent semantics; the durable policy is SQL transaction script

**Principles / gate / scenario.** AP-04, AP-02, AP-01, AP-06, DP-01, DP-03; G1, G9; S04, S05, S06.

**Label.** Implemented.

**Finding.** In-process studies and durable studies are separate request and outcome models with no
mapping between them:

- in-process: `ModelingStudyPoint`/`ModelingStudyReport`, `engines.rs:~338–480`;
- durable: `StudyPlan`/`StudyDefinition`, `study.rs:~77–292`.

They diverge in several ways:

| Situation | In-process | Durable |
|---|---|---|
| Predecessor without a usable seed | Refuses the point | Runs it fresh, with a reason |
| Failed predecessor | Refuses the dependents | Cancels the dependents transitively |
| Route | Per point | One per study |
| Overlay composition | `staged.rs:67–88` | `worker.rs:537–546` |
| Outcome relation | `runtime.modeling_studies` | `runtime.study_outcomes` |

- The durable policy (`conclude`, `job_changed`, `release_dependents`, `cancel_dependents`) lives in
  `pse-operations/src/studies.rs:340–545`, interleaved with SQL under row locks.
- `predecessor_start` treats any non-`Operations` error as "the seed does not fit"
  (`study.rs:281–289`).
- Study points can only be case solves.
- §19.3 specifies both rules, so the specification itself holds two authorities.

**Consequence.**

- Moving a study from in-process to durable (a mechanism substitution) changes which points run and
  from what start.
- Consumers handle two outcome schemas.
- Every new study rule is written twice, in two crates.
- The durable rules can be tested only against live PostgreSQL (`study_tests.rs`).
- Sweeps of fits, simulations or horizons, which are in the functional target, have no primitive.

**Correction.**

- Owner: `pse-runtime` `workflow/study`.
- One study definition and outcome vocabulary.
- A **pure** point-graph policy (point outcomes → next action and conclusion) beside
  `pse_operations::lifecycle`, applied by both executors. The repository only applies it under its
  locks.
- Generalize points over run kinds.
- Amend §19.3.

**Verification.**

- Policy unit tests run with no database or solver.
- One definition runs under both executors with identical per-point states and start sources.
- One published outcome relation remains.

#### <a id="f06"></a>F06: Failure classification is not reconciled across its code, class and termination maps; rules are open strings

**Principles / gate / scenario.** DP-01, DP-02, DP-21, AP-04, AP-05; G1, G2; S08, S10.

**Label.** Implemented.

**Finding.** §23.2 sanctions separate `FailureClass`/`DiagnosticCode` and `BoundaryClass`
vocabularies, with runtime class derivation and a declared class→code map. The implementation does
not reconcile them with that map.

- **Code and class disagree.** `pse-math/src/error.rs:~168–176` maps `MathError::Evaluation`
  (among others) through `_ =>` to `compile.math`. The runtime's 17-arm downcast chain in
  `workflow/diagnostics.rs:72–125` classes the same error `trial_rejected`, whose declared code is
  `solve.evaluation_error`. Python sees both.
- **Wildcard class maps.** `results.rs:154–161` maps terminations to classes with a wildcard, so
  `Panic` and `Invalid` become `trial_rejected` (§23.2: internal) and limits become
  `trial_rejected` (§23.2: resource_limit).
- **Retry follows that map.** Initialization `retryable` (`engines.rs:173`) acts on this map.
- **Rules are compared as text.** Production compares
  `rule == "modeling.qualification.rejected"` (`engines.rs:161`), and authored `diagnose "…"`
  fixtures match by string (`conformance.rs:~704`).
- **`WorkflowError::Contract(String)` is a catch-all.** It has 456 sites; 39 of them stringify typed
  errors. It is always coded `compile.math` and classed `invalid_model`, including internal
  postconditions such as "lost public run supervisor". Its six real classes are then re-classified
  by call sites: durable `finish` retries internal encoding defects, and `predecessor_start` turns
  any defect into a silent fallback.
- **Subcodes are lost in pse-modeling (severity B).** About 1,024 `invalid(…)` refusals share one
  coarse code, and about 148 flatten typed `DslError` or `QuantityError` sources to text.

**Consequence.**

- The store, the Python exception and the published finding can disagree about whether a failure was
  a model error, an evaluation failure or an internal defect.
- Retry follows a class the failing operation never declared.
- A misspelled rule becomes `internal` or fails a fixture as though the model misbehaved.
- An agent or tool cannot distinguish a syntax error from a missing derived kind without parsing
  prose.

**Correction.** Owners: each crate's error projection, `pse-runtime` diagnostics, the registry.

- Each failing operation projects its own typed `BoundaryDiagnostic`.
- Codes are derived from classes through the one §23.2 map, as an exhaustive function or a registry
  attribute, with no wildcards.
- Numerical-diagnostic and qualification rules become a closed registry vocabulary carrying class
  and severity, so authoring can refuse an unknown `diagnose`.
- `Contract` splits into request, internal-invariant and typed-cause variants that wrap sources with
  `#[source]`.
- pse-modeling gets typed refusal families.
- miette's derive does not help here: codes are typed and value-dependent (§F record L1a).

**Verification.**

- A test over `DiagnosticCode::ALL` and the class map shows agreement.
- There is no `_ =>` in the class maps.
- There is no `contract(e.to_string())`.
- A misspelled `diagnose` rule is refused at authoring.

#### <a id="f07"></a>F07: Durable and study failures keep only prose

**Principles / gate / scenario.** PS-10 (failures name model elements), DP-21, DP-19; G2, G3; S08
(in a study), the "study over many cases" journey.

**Label.** Implemented.

**Finding.**

- A durable attempt records `TerminationCause::Error { message: String, rule: String }` beside a
  typed code (`durable.rs:687–707`). Class, sources and observations are dropped.
- A step refused before its solve records only `RunState(rejected)`.
- Members are written only for `Completed | Partial` (`durable.rs:487–490`), so a failed point
  publishes nothing, and `study_outcomes.error` is `Option<String>` (`study.rs:747`).
- The in-process `Completion.diagnostics`, with class, rule, sources and observations, exists but
  is never persisted for failed durable work.

**Consequence.** Suppose 30 points of a 1,000-point durable sweep leave a property envelope. The
study then reports 30 prose strings. Grouping by class, or finding the offending units or streams,
requires parsing text or re-running each point in process.

**Correction.** Owners: `pse-runtime` durable and study; the registry.

- Persist the run's `BoundaryDiagnostic`s for every terminal attempt: failed points write
  `modeling_findings` as members.
- Or version `TerminationDetail` to carry typed findings.
- `study_outcomes` gains typed class and code columns.

**Verification.** A failed durable point shows its class, code and source identities as typed
columns.

#### <a id="f08"></a>F08: Study and case overlays cross the Python and durable boundary as unitless numbers keyed by path strings

**Principles / gate / scenario.** PS-01 ("a number never crosses an interface without its
dimension"), DP-02, DP-04, AP-05; PS-G1, G2; S09, S04, S07.

**Label.** Implemented.

**Finding.**

- `PointOverlay { values: BTreeMap<String, f64>, parameters: BTreeMap<SemanticId, f64> }`
  (`worker.rs:152–164`; generated in Python as `dict[str, float]`) is composed into
  `ModelingCaseBindings.values`, documented "in canonical physical units"
  (`pse-compiler/…/executable/solve.rs:18–20`).
- It is inserted as the symbol's bare canonical scalar (`solve.rs:~119–128`).
- Values are keyed by modeling-language path, resolved by exact string lookup. §5.1 says case
  targets are stored by identity.
- The overlay is persisted in `StudyDefinition` and framed into `binding_hash` and the operational
  request identity.

**Consequence.**

- A sweep entered in °C, bar or kPa silently solves at the wrong conditions; the °C affine offset is
  lost entirely.
- Stored studies break on a child rename or a different coordinate spelling (`[0{s}]` vs
  `[0.0{s}]`).
- A unitless number becomes a durable identity input.
- This is the only programmatic boundary that drops units: literals, Parquet columns, fit
  observations, knowledge rows and reports all carry them.

**Correction.** Owners: the runtime study and worker contract, `ModelingCaseBindings`, pse-py, the
generated document.

- Overlay values become typed quantities (`QuantityValue`: magnitude plus unit), converted by
  `convert_spec_for_type` at the boundary. Bare numbers are refused.
- Paths are resolved to member identities before persisting or hashing; paths remain input sugar.
- Correct §5.1 or make it true.

**Verification.**

- `{"root.inlet.T": 80 °C}` solves at 353.15 K.
- An incompatible unit is refused and names the path.
- A stored study survives a child rename.

#### <a id="f09"></a>F09: Shooting bypasses the §16.6 usability decision and the admitted execution route

**Principles / gate / scenario.** PS-10, AP-04 (authority), AP-02, DP-20; PS-G3, G7; S03.

**Label.** Implemented.

**Finding.**

- `ShootingReport` (`shooting.rs:129–157`) carries a raw `SolveReport` and checks, but no
  `CandidateDecision`.
- Conformance decides "solved" as `termination.category == Success` plus `checks_complete`
  (`conformance.rs:1170–1181`). It ignores qualification and treats `Acceptable` as failure.
- `ShootingProblem::solve` is `pub` and synchronous (`shooting.rs:1029`), while the admitted
  submission is `pub(crate)` (`math/jobs.rs:125`). Conformance itself submits through admission;
  external Rust callers do not.
- ADR-0106 makes `CandidateUse` "the only acceptance rule every workflow consumes", and §25 claims
  shooting "authored or in Rust".

**Consequence.**

- An unqualified `Success` passes a fixture and a qualified `Acceptable` fails one.
- A new termination category needs a conformance edit.
- A Rust caller runs outside the memory and core budget: the failure class behind the 2026-09-29
  editor crash.

**Correction.** Owner: `pse-runtime` shooting and conformance.

- Route shooting through `native_use` → `complete`, with continuity and sample checks as checks.
- Make `solve` crate-private and expose an admitted `start() → RunHandle`.
- Conformance reads `accepted`.
- No authority change: ADR-0106 already requires this.

**Verification.**

- No `termination.category ==` remains in `workflow/`.
- An `Acceptable` qualified shooting fixture passes and an unqualified `Success` fails.

#### <a id="f10"></a>F10: The usability of a trajectory ended by an authored terminal event is undecided and defaults to unusable

**Principles / gate / scenario.** AP-04 (adequacy), DP-02, PS-11, PS-12; G2; S10, the "dynamic
start and event" journey.

**Label.** Implemented.

**Finding.**

- Terminal events are a supported, authored construct (`check.rs:~714–742`; §13 lists "terminal
  event").
- `trajectory_use` tests `== Completed` (`numerics.rs:182–190`), so `Event` falls through to
  `Unusable`. Samples after the event count as missing.
- `integrated.rs::completed` reports `Event` as an internal defect.
- §16.6 says only "usable only after a completed integration", and no decision covers the case.

**Consequence.**

- A batch-to-endpoint or time-to-trip simulation can never be a result, publish as usable or seed a
  later step.
- A new `TrajectoryTermination` member silently becomes unusable.

**Correction.** Owner: `pse-runtime` numerics; §16.6.

- Decide the rule once: a terminal event ends the horizon, later samples are *not applicable*, and
  the trajectory is usable if its checks pass up to the event. Or record the refusal as a deliberate
  policy with its own reason.
- Use an exhaustive match.

**Verification.** The `stopped` test states the decided outcome, and `Event` no longer maps to an
internal defect.

#### <a id="f11"></a>F11: Phase-independent parameter sets are forced into a phase-typed key, with `vaporPhase` as a sentinel

**Principles / gate / scenario.** DP-02 (no sentinels), AP-04 (adequacy), PS-02; G2; S01.

**Label.** Implemented.

**Finding.**

- `parameter_set` requires `key phase_type` and `phase_type in property.applies`
  (`domain/models/properties.pse:29–32`).
- Critical constants and PC-SAFT parameters have no phase. They are declared
  `applies = {vaporPhase}` and selected with `PhaseType.vaporPhase` (`cubic.pse`,
  `pcsaft.pse:~43–45`).
- The sentinel is baked into published bank data (`data/gross-sadowski-2001/models/parameters.pse:12`)
  and forms the `complete_over` axis of eight selection datasets (`properties.vapor_types`).
- Record-valued properties declare `quantity = Scalar`.

**Consequence.**

- Liquid-phase PR and PC-SAFT evaluations read a "vapor" parameterization.
- Every new EOS family or bank must learn the convention, and the migration cost grows with each
  bank.

**Correction.** Package-level, `pse.domain`. Split a species-level parameter set (no phase key) from
a phase-specific one, and type record-valued properties by the refined kind's attributes. No
authority change.

**Verification.** No `PhaseType.vaporPhase` literal remains in a phase-independent selector.

#### <a id="f12"></a>F12: Validity envelopes are optional at the data layer and declared only for temperature correlations

**Principles / gate / scenario.** PS-02 (MUST), AP-05, DP-03; PS-G1, G3; S08.

**Label.** Interface-checked.

**Finding.**

- The only data-layer envelope is `temperature_correlation.T` (`properties.pse:39`).
- PC-SAFT, NRTL, binary-interaction, `critical_point` and `cubic.kappa` sets declare none, and
  `parameter_set` requires none.
- Closure-layer T, P and z ranges exist in some seed bindings (bt-ideal, bt-pr, aqueous, PR
  density).
- Form `valid(t > 0 …)` guards are mathematical domains, not regression ranges.

**Consequence.**

- PC-SAFT, PR or NRTL evaluation outside its parameters' regression range reaches a result with no
  validity observation.
- The strong S08 machinery (typed rejection or recorded extrapolation with its layer) covers only
  temperature correlations.
- §26 names "declared envelopes" as the mitigation for scientific adequacy.

**Correction.** Package-level, `pse.domain` and the method packages.

- Declare envelope axes on the abstract set or family kinds, so every refinement binds bounds (an
  explicit "unbounded" is allowed).
- Give pair tables a T range.
- If "a refined kind must bind an envelope" cannot be expressed, that is a small kernel gap.

**Verification.**

- A family without envelope bounds is refused.
- An out-of-range PC-SAFT fixture yields a `layer=data` check row.

### B: change cost, reuse and testability

#### <a id="f13"></a>F13: Contract evolution has no compatibility classes, and one store fingerprint couples the publication catalog to every vocabulary

**Principles / gate / scenario.** DP-24, AP-02, AP-01, DP-19; S03, S05, S10.

**Label.** Implemented.

**Finding.**

- `compatibility::require` is exact equality over a witness that includes every reachable enum's
  member map (`pse-schema/src/compatibility.rs:83–99`, `fingerprint.rs:~135–147`).
- `MigrationStep` has only column steps, and `declare_migration` has no production caller.
- `Store::open` creates the schema or refuses it by one fingerprint covering 21 ENUMs, including the
  68-member `diagnostic_code`, and the catalog tables. Declared Delta migrations
  (`prepare_schema_transform`) have no reader caller.
- The no-migration stance is an **accepted decision** (ADR-0114 Outcome 23, deferred as R-35). This
  part is therefore a target gap. R-35's trigger ("contents must survive") ignores frequency: every
  new `DiagnosticCode`, `NativeBackend`, `NativeTermination` or `AttemptKind` forces a reset, and
  [f06](#f06)'s own correction adds codes.
- **An uncovered defect.** After a reset, `register_workspace` re-registers the root
  (`publication.rs:491–496`), while `collect` acts only on catalog marks and intents
  (`collect.rs:4–13`). Previous-generation member prefixes are therefore orphaned, never reclaimed,
  and absent from §20.6 *Limits*.

**Consequence.**

- Ordinary additive vocabulary growth invalidates every publication id and lineage reference.
- Storage leaks without bound.
- §20.6's "holds only regenerable state" is untrue for the catalog.

**Correction.** Owners: `pse-schema`, `pse-codegen` (postgres), `pse-operations`.

- Classify witness differences: an enum superset or a deprecation is reader-compatible; a removal or
  rename is refused.
- Give the catalog its own schema identity and migration line.
- Generate additive `ALTER TYPE … ADD VALUE` migrations from a registry diff, applied by refinery
  over the existing tokio-postgres driver. The schema lock must become a session lock, and
  refinery's history table must live inside `pse_ops`.
- Compose Delta schema transforms at open.
- Until then, document and collect orphaned prefixes.

**Authority.** Supersede ADR-0114 O23; §20.5 and §20.6; the R-35 trigger.

**Verification.**

- A fixture registry adds one member, and an existing store and older publications open.
- A queue-table change leaves the catalog identity unchanged.

#### <a id="f14"></a>F14: Published Delta tables persist DataFusion-version-bound predicate bytes that could be derived

**Principles / gate / scenario.** DP-24, DP-15, DP-01, AP-02; S11, S05.

**Label.** Implemented; library fact Interface-checked.

**Finding.**

- `pse-catalog/src/delta/contract.rs:28` sets `EXPRESSION_ENCODING = "datafusion-proto-55.1.0"`, a
  literal.
- Every adapter-bearing field (any collection, or a storage-type mismatch such as ids) stores its
  value predicate as proto bytes in a table property (`:95–105`).
- On reopen the bytes are decoded and *used*; any other encoding is refused `UnsupportedEncoding`
  (`:189–201`).
- datafusion-proto 55.1 disclaims cross-version compatibility (`lib.rs:47–51`).
- The predicate is derivable: `field_value` needs only enum members, which the recorded semantic
  contract already carries.
- The refusal is typed and designed (§20.5), so nothing is silent.

**Consequence.** Every DataFusion major, a pinned family that moves every few months, makes every
published table with an adapted field unreadable although its meaning is intact. No rewrite tool
exists.

**Correction.**

- Owner: `pse-catalog` `delta::contract`; §20.5 text.
- Persist only portable meaning, and re-derive adapter predicates at open from the recorded
  contract.
- Keep SQL via `sql::unparser` only for registry-free readers, and only with a round-trip test (a
  lambda-capable dialect is needed).

**Verification.** A table written under a different encoding label opens and validates, and `verify`
no longer compares bytes.

#### <a id="f15"></a>F15: Production reuse works per root selection, outside the adopted reuse libraries; §14.4 describes a path production does not take

**Principles / gate / scenario.** DP-09 (finest granularity), DP-10, PS-11, DP-13/14 (G8), DP-24;
G7, G8; S07, S04.

**Label.** Implemented; Salsa facts Interface-checked.

**Finding.**

- **Body admission.** All of a model's bodies are admitted inside one tracked `admitted(inventory,
  catalog, request)` query. Deduplication is by a call-local map (`grouped.rs:416–490`,
  `executable.rs:1085–1094`).
- **Bindings.** One `Request` input per `(root, instance)` is overwritten by `set_bindings`, so
  alternating binding sets recompute. The workspace refuses its 65th root unless memory retention
  rotated it first (`modeling.rs:569–587`, `workspace.rs:136`). The recorded seed-conformance
  pass depends on that rotation.
- **§14.4 describes a different path.** Its per-definition rows describe the raw `Inputs` path, which
  production never populates ([f20](#f20)).
- **Views.** Prepared views are held per package object in a hand-written `VecDeque` LRU
  (`views.rs:18–50`), while the same crate already uses DataFusion `DefaultCache` for artifacts.
  `view_key` includes spans and seven hand-listed profile fields.
- **Durable workers** build a fresh package and Salsa workspace per job (`worker.rs:503–545`).
- §14.4 itself specifies per-revision views and "no persistent Salsa store".

**Consequence.**

- Editing one equation re-lowers every body through Symbolica.
- Each nonlinear-explanation trial re-admits every body.
- A durable N-point value-only study re-parses, re-admits and re-prepares N times (unmeasured);
  the *Measured* 1,000-point reuse holds only in process.
- A comment-only edit changes `view_key`.

**Correction.**

- **Owner: pse-compiler.**
  - Add a Salsa `#[interned]` body key: the span-free `ModelingConsumerBodyV2` identity, completed
    with provider descriptors, referenced functions and limits.
  - Add a `#[tracked(lru, heap_size)]` body-admission query, following the existing
    `PlanKey`/`document_table` pattern.
  - Intern `(root, instance, bindings, limits)` using a framed identity, since `Bindings` is not
    `Hash`.
- **Owner: runtime `MathService`.**
  - Hold prepared views in a `DefaultCache` keyed by `view_key`, with their own byte budget.
  - Drop spans from the key, and hash `Profile` through one framed serializer.
  - Cache admitted packages per worker by source hash.
- **Amend §14.4.**

**Verification.**

- Salsa `WillExecute` events show one body admission after a one-equation edit.
- A five-point durable study on one worker reports one view preparation.
- `Bounded<T>` is gone.

#### <a id="f16"></a>F16: Shared model checks: judged closures are misreported, flowsheet closure has no verdict, and DOF is counted four ways

**Principles / gate / scenario.** PS-03, PS-13, PS-10, DP-01; G2; S02.

**Label.** Implemented.

**Finding.**

- **By design (§10.1),** accounting accumulators produce no closure verdict.
- **Authored closures are misreported.** CV1D, vessel and separator closures are authored as
  generic `annotation check`s that gate usability (`control-volume-1d.pse:47–50`, `vessels.pse:72–75`,
  `separator.pse:57–64`). Meanwhile the published §0.5 closure fact reports `NotRequired`
  (`numerics.rs:~208–235`), and `ClosurePolicy` does not govern them.
- **Flowsheet closure is computed but never judged.** `RecycleLoop` declares an accounting `overall`
  with no check (`recycle-flash.pse:60`). A PS-03 flowsheet-level verdict also depends on
  [f02](#f02).
- **DOF has four rules:**
  - pure: free vs `Eq` count;
  - steady: bound columns vs equal-bound rows;
  - dynamic: asserted from admission;
  - fitting's squareness test (`fitting/oracle.rs:1066–1073`).

  PS-04 well-posedness itself has one owner (sound).

**Consequence.**

- `runtime.physical_checks` misstates closure status for distributed units and vessels.
- Each new distributed or partition unit re-authors its closure instead of gaining it.
- The same fixture `dof N` means different counts per mode.

**Correction.**

- Give accounting subjects an optional kernel-assessed "must close" verdict with the accumulator
  tolerance, so authored closures become `CheckKind::Closure` (amend §10.1).
- Derive flowsheet boundary closure from stream ports once [f02](#f02) lands.
- Add one `degrees_of_freedom()` on the bound case view, consumed by conformance, `diagnose_case`
  and fitting.

**Verification.**

- These closures appear as `CheckKind::Closure` and obey `ClosurePolicy`.
- One DOF definition has three consumers.

#### <a id="f17"></a>F17: Outcome partitions are re-spelled by consumers with non-exhaustive matches

**Principles / gate / scenario.** AP-01, AP-03, DP-01, DP-08; S10.

**Label.** Implemented.

**Finding.**

- `stop_use` (`numerics.rs:98–130`) is the only compile-forced partition of `NativeTermination`.
- Initialization retry (`engines.rs:173`) uses `matches!`, and the qualification entry uses `==`.
- The "is a result" predicate over `CandidateUse` is written three times: `permits_use`
  (`numerics.rs:87–92`), `RunResult::usable` (`run.rs:183–191`) and `durable::classify`
  (`durable.rs:783–788`). All three are identical today.
- `NativeTermination` members carry no declared descriptions.
- §16.6 claims "none re-derives acceptance".
- The termination→class map is folded into [f06](#f06).

**Consequence.** A new stop category is compile-forced for usability but silently defaulted for retry
and qualification. A new `CandidateUse` member edits three predicate copies.

**Correction.**

- Put `CandidateUse::is_result()`, the retry projection and the class projection beside `stop_use`
  as exhaustive functions, or as registry attributes.
- Declare member descriptions.

**Verification.** Adding a variant fails compilation at every partition, and one predicate definition
remains.

#### <a id="f18"></a>F18: The Python boundary bypasses the registry and document generators

**Principles / gate / scenario.** D1 (§2, §4.2, §21.5; ADR-0116 "every Rust-owned boundary
document"), DP-01, DP-02, AP-02; G1, G2, G7; S05, S09.

**Label.** Implemented.

**Finding.**

- **Hand-written mirrors.** Six msgspec mirrors in `python/pse/_runs.py:90–427` (`Workspace`,
  `Published`, `ExportReceipt`, `StudyPointStatus`, `StudyStatus`, `StudyCancel`), plus
  `PublicationSettlement` tags that hand-spell the registry's `SettlementOutcome`.
- **Ad hoc JSON.** Fourteen `serde_json::json!` documents in three pse-py files decode to
  `dict[str, object]`.
- **Enums as strings.** Eight getters return registry enum spellings as `str`
  (`NativeAttempt.termination`, `backend`, `qualification`; `DiagnosticReport.code`,
  `boundary_class`, `severity`, …), while `RunCompletion` rows carry generated `StrEnum`s.
- **Policy in the adapter.** The `FlowSelectionDocument` wire type and its uniqueness rules live in
  pse-py.
- **Prose for a closed enum.** `CandidateReason`, a closed Rust enum, is published as prose text
  (`native_math.rs:213`).

**Consequence.**

- A Rust field or member change fails Python decoding only at runtime; `codegen-check` cannot see
  it.
- `== "sucess"` passes type checking.
- Decision-relevant structure (flow policies, tears, quantity ids) is untyped for Python consumers.

**Correction.** Owners: pse-py, the runtime types, the xtask `documents()` list.

- Derive `JsonSchema` and emit these types.
- Make a single `document<T: Serialize + JsonSchema>` helper the only JSON return.
- Return generated `StrEnum`s from getters.
- Move `FlowSelectionDocument` into runtime `math::flows`.
- Make `CandidateReason` a registry enum.

**Verification.**

- `_runs.py` imports generated classes.
- `rg 'json!\('` over pse-py is empty.
- The stubs type `termination` as `NativeTermination`.

#### <a id="f19"></a>F19: Expressions are parsed at use sites; `logic` propositions have a second grammar

**Principles / gate / scenario.** DP-02 ("parse at the boundary"), DP-16, DP-21; G2.

**Label.** Implemented.

**Finding.**

- `language::parse` captures expressions, predicates and domains as text after bracket balancing
  (`parser.rs:283–313`). Consumers re-parse them: about 98 DSL parse calls in 16 modeling and
  compiler files. `pse-compiler` even builds ASTs by formatting and parsing names
  (`executable.rs:~606–771`).
- `logic` bodies are re-tokenized by a second, independent parser (`pse-modeling/src/logic.rs`).
  That parser rejects quoted path segments that DSL paths allow, and admits `a-b` as one atom that
  is refused later.
- The generated declaration IR is text-bearing by design (§7.7).
- This is not a library gap: winnow 1.0.4's Pratt parser does not beat the stated reason at
  `dsl/parser.rs:4` (§F records F5 and F6).

**Consequence.**

- Syntax errors surface at check time as `Contract` prose without an expression span.
- A binary variable with a quoted path segment cannot appear in `logic`.
- Two precedence grammars must be kept aligned by hand.

**Correction.** Owner: pse-authoring and the pse-modeling check.

- Fold `xor`, `implies` and `exactly(k, …)` into the DSL predicate grammar, and delete `logic.rs`.
- Build a checked-AST side table once at `check` (the IR stays text), consumed by specialization
  and the compiler.

**Verification.**

- `logic.rs` is gone.
- A proposition over a quoted-segment path specializes.
- A malformed equation fails with a `Syntax` span.

#### <a id="f20"></a>F20: Mechanisms with no production composition survive, and one is described as an enforcement stage

**Principles / gate / scenario.** DP-16 (delete replaced code), DP-03, DP-01; G1, G7; S01, S11.

**Label.** Implemented.

**Finding.**

- **The raw `Inputs` front door** (`pse_compiler::workspace::Inputs.{flows, definitions, domains,
  groups, cases}`) and about 20 tracked queries have only test callers. The production
  `compiler_inputs` leaves them empty, and pse-py and Python `prepare_flow` go through the port
  projection.
- **The whole-model `PreparedModeling.structure` query** runs a matching on every preparation, and
  nothing in production reads it. Only the per-view `structural_plan` is consumed.
- **`pse_structural::projection::Projection`** (about 450 lines) has no consumer.
- **The registry closure and acyclicity invariants** run only in xtask codegen (`run_invariants`).
  Rust ingestion enforces the same rules (`pse-authoring/src/p0.rs:43–55`,
  `authoring_driver/document/hydrate.rs:72`). Drift is latent: SQL exact-version equality against
  Rust operator matching agree for the one operator, `exact`. §4.6 claims stage 3 runs
  `pse_rules::invariants`.
- **Composition roots disagree.** `RequirementPlanner` and `ProviderPolicy.requirements` are bound
  by pse-py but not by `Runtime`; both are inert because every production policy set is empty.

**Consequence.**

- Each DataFusion, petgraph or Salsa upgrade must requalify unused code.
- These mechanisms make reuse granularity ([f15](#f15)) and enforcement appear present.
- A second topology input survives beside the port projection.

**Correction.** For each mechanism, either compose it at a named production boundary, or delete it.
Tests move to package-authored cases or a test-support constructor.

- For the invariants, choose one owner: execute the registry invariants at physical admission and
  source publication, or delete the registry copies and correct §4.6.
- `pse.canon.v2` and `plan_codec` have declared roles (§5.3 *Limits*, ADR-0044), so they are
  recorded only as examined.

**Verification.** `rg` shows each mechanism has a production caller, a register row, or is gone.

#### <a id="f21"></a>F21: Lexicographic eligibility is re-derived beside the capability records

**Principles / gate / scenario.** AP-03, AP-04 (realization), DP-01, PS-09; G1 (narrow); S03.

**Label.** Implemented.

**Finding.**

- `native_levels` (`objectives.rs:230–256`) decides the native lexicographic route by
  `facts.coefficients && !facts.quadratic`, naming HiGHS.
- The owner of that rule is the capability record (`execution/highs.rs:35`
  `lexicographic: [Linear, MixedLinear]`), enforced by `routing::admit`.
- Under an explicit non-lexicographic selection, the local test sends the case down the native path
  and routing refuses it, where the staged route would have honoured it. ADR-0111 does not say which
  outcome is intended.

**Consequence.** A QP-lexicographic adapter needs an `objectives.rs` edit, contradicting the
execution seam's "no workflow edits" claim.

**Correction.** Routing exposes a `lexicographic_route(facts, intent, selection)` query, and
`objectives.rs` consumes it.

**Verification.**

- No `coefficients` or `quadratic` test remains in `objectives.rs`.
- A stub adapter declaring QP lexicographic routes natively.

### Target gaps that need an authority change

#### <a id="f22"></a>F22: Sensitivity is admitted only under `optimize`; the square-solution primitive is private to fitting

**Principles / gate / scenario.** AP-04 (adequacy, target), AP-03, PS-11, PS-12, DP-16; S04.

**Label.** Implemented.

**Finding.**

- `SensitivityRequest::admit` refuses unless the intent is `Optimize`
  (`math/settings.rs:144–150`).
- §25 explicitly refuses sensitivities on root routes, and ADR-0118 scopes the single KKT analysis
  to NLP/QP. This is therefore a deliberate limit, not a defect.
- `fitting/oracle.rs:~1063–1130` already computes the scaled implicit response dx/dp at a square
  steady solution, with a rank test.

**Consequence.** The most common simulator question, dx/dp of a square flowsheet, and sensitivity
over a sweep (S04, in the functional target) need re-authoring with a dummy objective. PS-11
forbids a mode switch that requires a model edit.

**Correction.**

- Lift the fitting primitive into `local_analysis` plus the backend `kkt` as the `root`-intent
  degeneration: J_x⁻¹J_p, valid when the square Jacobian is non-singular at the stated scaling.
- Fitting consumes it; squareness comes from [f16](#f16)'s DOF operation.

**Authority.** ADR-0118 scope, the §25 Refused column, §15.5.1.

**Verification.**

- Square-fixture sensitivities equal a finite-difference sweep.
- They are withheld when rank fails.

#### <a id="f23"></a>F23: No policy admits a limit-stopped, qualified incumbent as a result

**Principles / gate / scenario.** AP-04 (adequacy, target), PS-09, PS-12; S10.

**Label.** Implemented.

**Finding.**

- ADR-0106 item 11 decides that `seed_only` means "the native stop forbids use as a result".
- `ClosurePolicy` is the only usability opt-in, although §0.5 speaks of "explicit opt-ins".
- A SCIP or HiGHS time-, node- or solution-limited incumbent with an original-feasible candidate
  and a certified gap is therefore always `seed_only`.
- The factorization already represents this case without a new termination member: TimeLimit +
  FeasiblePoint + Feasible/GapQualified.

**Consequence.** Budgeted MILP and MINLP solves, a routine optimization workload, cannot feed a
study result, a staged commit or a publication.

**Correction.** Add a registry `IncumbentPolicy` {refuse, accept_feasible, accept_within_gap} on the
case or analysis, and a `CandidateUse` member stating the termination and gap, threaded into
`native_use` beside `ClosurePolicy`.

**Authority.** Supersede ADR-0106 item 11; §16.6.

**Verification.**

- A time-limited SCIP incumbent under `accept_within_gap` publishes with its gap stated.
- Under the default it stays `seed_only`.

### Library and minor findings

| ID | Finding | Principles · sev | Evidence (label) | Correction · owner | Verification |
|---|---|---|---|---|---|
| <a id="f24"></a>F24 | Five hand-written closed-vocabulary mechanisms (`vocabulary!` and `codes!` in pse-diagnostics, `vocabulary!` in pse-vocabulary, `closed_enum!` in pse-quantity, the codegen `declared()` emitter). They produce the same `ALL`/`as_str`/parse/`Display` surface with three different unknown-member errors | DP-14, DP-16; G8 · C-minor | strum 0.28 covers `VariantArray`, `IntoStaticStr` + `const_into_str`, `EnumString` with `parse_err_ty`, and `Display`. Serde, schemars, postgres and code→class stay bespoke; codegen emission is §C-sanctioned (Implemented; Interface-checked) | One mechanism at the lowest layer (pse-diagnostics) built on strum derives and emitted by codegen; one unknown-member error | One derive set and one error type across closed enums |
| <a id="f25"></a>F25 | A second exact-rational type (`factorable::Rational`, i64) next to Symbolica's `Rational` in the same crate. It falls back to binary64 on overflow and for `Large` coefficients, while §7.5 says constants are "exact rational when the library atom is rational". ADR-0121 item 2 certifies binary64 matrices as rationals by decision, so this is not a certificate defect | DP-13, DP-14, G7; G8 · C | `factorable.rs:57–105, 146–153, 2305–2308`; `curvature.rs:38, 1330–1335` (Implemented) | Use Symbolica `Rational` in `Constant::Rational` (no new dependency), or state the i64 reason and correct §7.5. Also rescope D6's "no custom expression IR" to permit the non-evaluating factorable export (ADR-0105) | One rational type in pse-math; §7.5 matches |
| <a id="f26"></a>F26 | Small bespoke utilities where a pinned crate or an adopted library fits: three hex codecs with two different case rules (`pse-ids/src/id.rs:17–61`, `pse-schema/src/literal/decode.rs:407–425`, `pse-columnar` `native_value.rs:526–531`; `hex` 0.4.3 is in the lock); the codegen Python emitter's recursive DFS topological sort with O(n²) `Vec::contains` (`codegen/python/mod.rs:155–193`; petgraph `toposort` over `DiGraphMap`); the generated Python `collection` validator re-implementing attrs `min_len`/`max_len`, with O(n²) uniqueness (`contracts/values.py`) | DP-14; G8 · D | (Implemented; Interface-checked) | Adopt `hex`, keeping the canonical-literal lowercase guard; use petgraph `toposort` in codegen; emit `min_len`/`max_len` and a set-based `unique` | Utilities removed; regenerated output reviewed |
| <a id="f27"></a>F27 | The profile-chain worker pool swallows worker panics (`let _ = handle.join()`) and spawn failures, then silently re-runs the chain inline. Keeping `std::thread::scope` is correct: the rayon `build_scoped` wrapper would panic on rayon's worker assertion for multi-threaded POUNCE scopes, and `par_iter` breaks core admission | DP-19, DP-20; G5 note · D | `fitting/profile.rs:406–451`; rayon-core `registry.rs:708–713` (Implemented; Interface-checked) | Record a panic as the chain's stop reason, or propagate it; report reduced parallelism; add a one-line reason for not using rayon | A panicking chain is visible in the fit report |
| <a id="f28"></a>F28 | The route decision is not a persisted result fact: `solve_runs` records `backend` but not intent, auto/explicit selection or derived classes; `PreparedSolve::eligibility` is in-process only | PS-09, DP-15, DP-21 · D | `native_math.rs:~549–590`; `results.rs:~262–330` (Implemented) | Add intent, selection and classes to `solve_runs`, or add a `solve_routes` relation with `NativeIneligibility` reasons | A published run explains its route |
| <a id="f29"></a>F29 | Structural admission by representation is an unstated runtime `match` (`solves.rs:917–923`). The linear and conic exemption is sound but not in §15.2. The factorable route skips admission with no class reason, so an over-determined nonlinear equality block (Optimize or Certify MINLP) reaches SCIP and returns as infeasible rather than as an attributable refusal | PS-04, AP-05, DP-15; PS-G2 unresolved · D | (Implemented) | Declare the structural mode per representation in the capability record; apply NLP-mode equality matching to factorable equality rows; state the exemptions in §15.2 | A factorable case with an over-determined block is refused before SCIP, with row ids |
| <a id="f30"></a>F30 | Identity canonicalization is a call-site convention: `FramedHasher` has no float part, there are about 69 raw `.u64(…to_bits())` sites, `identity::of` keeps NaN payloads, and two frames hash JSON text (`WarmStart::content_key`, dynamic `profile_identity`), contrary to §5.3 and ADR-0116. Content-hash roles are untyped: `latest_compatible(stamp, preparation)` takes two `ContentHash`es side by side, and "request identity" names both the operational job frame and the lineage frame | DP-04, DP-11, AP-05; G6 latent · D | `pse-ids/src/derive.rs:~467–520`; `solve.rs:950–957`; `dynamics.rs:611–616`; `solutions.rs:238–324` (Implemented) | Give pse-ids an `f64` part and move the serde `Framer` there (no library fits: §F F12); issue new frame versions for the JSON-text frames; add per-projection `hash_role!`s; rename the operational column | No raw float framing; golden vectors for NaN, ±inf and −0; `compile_fail` swap test |
| <a id="f31"></a>F31 | The validation context is installed into the static registry as public, first-wins ambient state, with the memo lock held during `build()` | AP-05, AP-06, DP-18; G4 note · D | `pse-relations/.../prepared.rs:71–83`; `pse-schema/src/implementation_cache.rs:26–43` (Implemented) | Composition roots pass `ValidationContext` explicitly, or install it from one engine constructor with a crate-private install; per-slot `OnceLock` | No `bind_defaults` in test setup |
| <a id="f32"></a>F32 | Orchestration re-spellings: the derivative-order rule is written three times; initialization defaults are restated in pse-py; `with_declarations` resets a selected accelerator inventory; `AppliedStart.requested` is a `&'static str` metric. The supervisor's spawn and join glue repeats four times, but the durable protocol itself is centralized (`DurableAttempt`), and §13 specifies the horizon as modeling steps | DP-01, DP-02, AP-03 · D | `modeling.rs:520–524`, `conformance.rs:1085–1088`, `fitting/modeling.rs:233`; `modeling.rs:650`; `worker.rs:626–660`; `run.rs:321–728` (Implemented) | `SolverProfile::derivative_order()`; Python takes defaults from Rust; preserve accelerators; a typed start enum; optionally one supervisor helper | One definition per rule |
| <a id="f33"></a>F33 | `ControlVolume0D` is single-inlet, so Mixer and Separator re-declare their own material and energy accumulators. Mixer supports only componentTotal and refuses dynamics, while IDAES's Mixer supports componentPhase, componentTotal and total bases | DP-06, PS-03 · D (adequacy note) | `control-volumes.pse`; `mixer.pse:13, 27–32`; `separator.pse:17–64` (Implemented) | Generalize the control volume over an indexed port set (builds on [f02](#f02)'s port groups) | Mixer and Separator have no `accumulate` of their own |
| <a id="f34"></a>F34 | The data layer classifies meaning in several places. Arrow field-meaning metadata ("which keys are value meaning") is defined at four or more sites with different key sets and literal spellings (`native_field.rs:92–120`, `admission.rs:781–798`, `checked_value.rs:143–170`, `preserve_field.rs`). Unit compatibility in the relational validator refuses every defined non-canonical unit, more narrowly than `pse-quantity` (it is conservative and applies only to reference-state T and P) | DP-01, AP-04; G1 · D | (Implemented) | pse-schema declares each facet's purpose, and every comparator builds on `native_field::project(purpose)` plus Arrow `==`; physical admission provides a unit-compatibility projection | No `"pse.domain.*"` literals outside pse-schema |
| <a id="f35"></a>F35 | Specification drift and small authority duplicates. §16.2 lists 6 numerical sources where the code has 8. §5.1 cites a missing owner (`workflow::composition`; the real owner is `specialize::member_id`). §15.3 cites the unused projection. The gauge datum (101325 Pa) is stated three times, with the affine offset unchecked and `conversions_between` unused (`costing.pse:29`, `physical.yaml`). Solve-path structural refusals carry IDs, and model paths are added only by `diagnose_case`. §9.8 "physical partial derivatives" is overstated ([f01](#f01)) | DP-24, DP-01; G7 · D | (Implemented; Interface-checked) | A `design:` PR corrects §5.1, §9.8, §15.3 and §16.2. Derive or check the affine offset against the datum. Enrich `native.structural` diagnostics with lineage paths at the modeling solve boundary | Sections match the code; mutating `package_datum` moves every declaration |

### Examined and found sound

These leads were investigated and rejected, and the strengths below carry the argument.

**Concept duplication settled as legitimate stage projections**

- `Port` in three crates: the pse-kernels type is a typed scalar coordinate, mapped explicitly in
  `flow.rs`.
- `Connection` in two crates: mapped 1:1 in `flow.rs`.
- `Variable` in two crates: via `assembled::contract`.
- Case types across stages.
- Studies' operational projections (`StudyRecord → StudyStatus`).

**Outcome and results**

- Separate facts, as §0.5 requires: the outcome enums (native stop, integrator stop, step kind,
  candidate kind, qualification, closure, usability), the two `cancelled` members, and
  `scip::Status`/`SolutionStatus` (adapter-private, mapped once).
- `RunResult` contains per-step `ModelingResult`s; the two `Completion` types only share a name.
- No production code parses status strings, and every backend maps its raw statuses once.
- Post-solve qualification is independent, in original coordinates with verified certificates.
- Seed-only and diagnostic-only candidates are never results.

**Publication and persistence**

- One PostgreSQL CAS commit boundary (D10).
- `pse-catalog` does not depend on Postgres.
- The two `cache_service`s are one owner with distinct scopes.
- DP-19 lifecycle, retry and cancellation are sound.

**Physical and material model**

- `pse-runtime/physical` is a derived projection.
- `physical.yaml` is an intended registry-declared source.
- The postgres dependency in foundations is an optional value-only feature.
- No production Rust branches on species or phase names.
- Plan 23's name-encoded carriers do not survive.
- Correlation coefficients are fully dimensioned.
- Selection, provenance and taint are data.
- S01 holds for SRK.

**Mathematical problem, modes and structure**

- Variable roles are explicit.
- Guards become barriers before simplification.
- Providers declare derivative order, with no finite-difference or zero fallback.
- Nominals are model attributes (F11 holds).
- One structural owner, and one admission policy with stage adapters.
- Balances come from contributions, and closure is computed from original contributions, not
  residuals.
- Problem-class derivation, capability admission and no fallback (PS-09).
- One model across modes (CT-S05; F08 holds).
- Warm starts are framed into lineage.
- Salsa and petgraph handles stay private.

**Construction, execution and data layer**

- `CheckedPackage` cannot be built without checking.
- The `kernel_*` fixtures are `#[cfg(test)]`.
- Conformance is a product (PS-13), not a test harness.
- Initialization restoration holds by construction (CT-S09).
- DP-20 budgets are coordinated.
- The engine traversal, invocation maps and relation memos are justified.
- UDF signatures refuse at planning.
- Filter pushdown is `Inexact`.
- Metadata is restamped after optimization.
- "Python does no math" holds.

## 8. Library fit and ownership cost

The full §F records are in the working notes. The material decisions:

| Capability | Integration owner | Current or candidate | Fit and limits (pinned) | Recommendation |
|---|---|---|---|---|
| Incremental body admission and view reuse | pse-compiler; runtime `MathService` | Salsa 0.28.4 interned key plus tracked, LRU'd, heap-sized query; DataFusion 55.1 `DefaultCache` | Exact precedent (`PlanKey`/`document_table`). Salsa does not fit views (value-bound, async, leased; `lru` counts memos) | **Adopt** ([f15](#f15)) |
| Store migrations | pse-codegen `postgres`; pse-operations | refinery over tokio-postgres; PG 18 `ALTER TYPE … ADD VALUE` | Needs a session advisory lock and history inside `pse_ops`. sqlx needs a second driver; Atlas diffs DDL but the compatibility class is a registry fact | **Adapt** ([f13](#f13)) |
| Persisted Delta predicates | pse-catalog | re-derivation from the recorded contract; `sql::unparser` | Proto is not stable across versions. The unparser round trip needs a lambda dialect and is not probed | **Adapt**: re-derive ([f14](#f14)) |
| Closed vocabularies | pse-diagnostics, codegen | strum 0.28 derives | Covers ALL, str, parse and Display. `EnumMessage`/`EnumProperty` are non-const, so the code→class map stays a match | **Adapt** ([f24](#f24)) |
| Exact rationals in the factorable export | pse-math | Symbolica `Rational` | Already a dependency and used in `curvature`; loses `Copy` | **Adopt** ([f25](#f25)) |
| Hex; codegen topological sort; Python cardinality validators | pse-ids/schema/columnar; pse-codegen | `hex` 0.4.3; petgraph `toposort`; attrs `min_len`/`max_len` | Fit; already compiled | **Adopt** ([f26](#f26)) |
| Error projection | pse-diagnostics | miette 7.6 derive | `code(...)` is a literal and cannot carry typed, value-dependent codes; forwarding is already covered | **Keep** (reason stated). [f06](#f06) is variant design, not a library gap |
| DSL and declaration parsers | pse-authoring | winnow 1.0.4 `expression`/`TokenSlice` | No depth guard; refusals would move into folds; the stated reason holds | **Keep**; [f19](#f19) is a parse-boundary issue |
| Arrow type comparison | pse-columnar, pse-schema | arrow-schema `equals_datatype`/`contains`; DataFusion logical/semantic equality | Ignores struct names, metadata or nullability direction | **Keep**; consolidate on `project(purpose)` ([f34](#f34)) |
| Keyed single flight | pse-columnar `flight` | moka `try_get_with`; tokio `OnceCell` | moka re-runs init after caller cancellation, which would duplicate a native solve | **Keep** |
| Postgres composite rows | pse-model | postgres-derive | No per-field converters; name-only type check | **Keep** |
| Durable job queue | pse-operations | apalis-postgres and similar | sqlx-based with its own schema; would break D1 and attempt lineage | **Keep** (add a one-line DP-13 note to ADR-0114's alternatives) |
| Identity framing | pse-ids (move from backend-native) | postcard, JCS, CBOR, serde_hashkey | Each lacks name framing, exact float bits or canonical order | **Keep bespoke**; move it to pse-ids ([f30](#f30)) |
| Profile worker pool | runtime fitting | rayon `build_scoped` | Unsound with per-scope POUNCE rayon pools | **Keep** `std::thread::scope` ([f27](#f27)) |
| Degree-2 polynomial; dimension `Ratio` | pse-math; pse-quantity | Symbolica polynomials; num-rational | Dense layout, no budgeted truncating multiply; num-rational has different `Ord`, panics and overflow behaviour | **Keep** (reasons stated in code) |
| Logic, fidelity and regime reasoning | pse-modeling, pse-math | biodivine BDD, z3, ascent, datafrog | No bespoke equivalent of their capabilities; a z3 exhaustiveness proof would be a new proposal | **Examined, no fit** |

**Overall G8 picture.** The heavy generic capabilities are library-owned, with integration owners
that absorb library types:

- solvers: Ipopt, POUNCE, KINSOL, IDAS, HiGHS, Clarabel, SCIP;
- structural analysis: pounce-presolve;
- derivatives and collocation: Symbolica;
- sparse algebra: faer, FERAL;
- graphs: petgraph, rustworkx;
- relational processing and caches: DataFusion, delta-rs;
- tracing: datafusion-tracing.

The remaining leverage gaps are the reuse path ([f15](#f15)) and small glue.

**A tooling note.** `docs/library-utilization.jsonl` records libraries only, with no capability
rows. It omits symbolica, faer, pounce and winnow, and marks deltalake "not-used" although it is a
direct dependency of pse-catalog and pse-runtime. The review used it only as a lead source.

## 9. Alternatives and tradeoffs

**Physical typing inside constitutive models** ([f01](#f01), [f03](#f03)):

| Alternative | Scenarios / locality | Contracts, composition, testability | Meaning or machinery carried | Cost | Selection and revisit |
|---|---|---|---|---|---|
| Current baseline | S01 is Rust-local, but typing is abandoned inside laws | Boundaries typed, bodies untyped | Unit constants; reduction conventions per author | Silent unit errors in new methods | Rejected: violates the §1 MUST |
| Checker rule only (refuse re-dressing) | Moves the workaround into central `physical.yaml` edits | — | Declared kinds for every partial | High authoring friction; likely new bypasses | Rejected alone |
| **Proposed:** anonymous dimensioned intermediates, `Reduced(kind, ref)`, a datum-free directed transfer kind; declared types only at named boundaries | S01 becomes package-only *and* typed | The language carries the meaning; `partial` types through | Revises ADR-0124's "never synthesized" for intermediates only | Kernel work in pse-quantity plus the checker; package migration | **Selected.** Revisit if intermediate types weaken kind distinctions (torque/energy) at named boundaries |
| Library-owned (`uom`, `dimensioned`) | — | Compile-time and closed; cannot express runtime-declared, datum-aware types | — | — | Not viable (§F) |

**Flowsheet structure** ([f02](#f02)):

| Alternative | Assessment |
|---|---|
| Current (scalar connects plus equations) | Topology, tear and recycle machinery unused; stream convention re-decided per flowsheet |
| **Proposed** (port groups plus indexed connect, one occurrence carrying member bindings) | Reuses the existing `flowsheet::Connection { bindings }`, `FlowGraph`, HiGHS tear MILP and KINSOL recycle; one new kernel concept. **Selected** |
| Simplest (a macro expanding scalar connects) | Removes the typing burden but keeps topology per-scalar and tears on equations; fails PS-05. Rejected |

**Execution ownership and studies** ([f04](#f04), [f05](#f05)): the proposed design is one
`declared_execution` plan plus one study definition with a pure point policy and two executors. It
removes code rather than adding it (conformance dispatch, pse-py pins, duplicated overlay and policy
logic). The simplest alternative, documenting the divergence, leaves G1 failing and is rejected.

## 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| Every finding's cited source exists and supports the claim | Implemented / Interface-checked | Wave 4 independent re-read of every cited path at `550f35cb` (production code equal at `0c55c78a`) | Cited paths re-read and counts recounted | 0 disproved; 3 weakened; many amended (counts, scope, severity, reframings); applied above |
| Findings against accepted decisions are target gaps, not defects | Interface-checked | Cross-check against ADR-0106, 0111, 0114, 0118, 0121, 0124, §4.6, §10.1, §14.4, §19.3, §23.2, §25 and register R-35/R-51 | Blocking text named in slot 11 | Applied to f13, f22, f23 and parts of f05, f06, f15, f16 |
| Library fit claims | Interface-checked | Pinned sources under `~/.cargo/registry`, the salsa, rust-graphs, datafusion and symbolica skills, Context7 (refinery, PG 18, moka, apalis, attrs) | Each §F decision states fit and gaps at the pin | No probe was needed; the unparser round trip is left unprobed and not relied on |
| Current functional behaviour | Tested (prior runs, cited) | Plan 23 Outcome runs (slot 1) | As recorded there | Not re-run; this review makes no new Tested or Measured claim |
| Durable-study re-preparation cost | Proposed | Source trace only | — | **Not measured.** It is a hypothesis until a durable study reports preparation counts |
| PS-13 conformance for touched models | Tested (prior) | `just seed-conformance` 115/115 | — | Unchanged by this review |

**Not examined:** runtime numerical accuracy, electrolyte and reaction breadth, the hydration
internals of the authoring driver, and the accelerators (slot 1). A document-only review changes no
code, so static and product checks are `not_run` under the AGENTS.md execution rhythm.

## 11. Authority changes, exceptions and disposition

**Authority text that blocks or contradicts the target** (route in parentheses):

| Authority | Required change | Findings | Route |
|---|---|---|---|
| D5, §8.3 neutral-scaling and Add/Sub rows; ADR-0124 "kinds are never synthesized"; §9.8 | Permit typed anonymous intermediates and reduced coordinates; add a datum-free, directed transfer kind; restrict neutral scaling | f01, f03 | ADR plus design review |
| Plan 21 placement guide, *Porting* step 5 and the "illustrative" disclaimer | A port is done only when it realizes the placement-map construct; typed signatures are normative; workarounds are kernel gaps | f01, f02 | Plan text in the successor plan |
| §12 connectivity; new kernel contract | Port groups and indexed connections; tearing a connection occurrence | f02, f33 | Short ADR plus `design:` PR |
| §19.3 | One study rule set | f05 | `design:` PR (ADR at discretion) |
| ADR-0114 Outcome 23; §20.5, §20.6; register R-35 trigger | Compatibility classes; catalog schema identity; additive migrations; orphaned-prefix limit | f13 | Supersede ADR-0114 O23 (ADR plus design review) |
| §20.5 encoding identity | Re-derive adapter predicates; stop persisting proto bytes | f14 | `design:` PR |
| §14.4 | Describe the modeling path's reuse, service-scoped views and interned bodies | f15 | `design:` PR |
| §10.1 accounting | Optional kernel-assessed "must close" verdict | f16 | `design:` PR |
| ADR-0118 scope; §25 Refused column; §15.5.1 | Root-intent sensitivity | f22 | ADR (supersede the ADR-0118 scope) |
| ADR-0106 item 11; §16.6 | Incumbent policy | f23 | Supersede item 11 (ADR) |
| §16.6 terminal events | Decide the terminal-event rule | f10 | `design:` PR |
| §4.6 stage 3; §5.1; §7.5 and D6; §15.2; §15.3; §16.2 | Correct to match the code | f20, f08, f25, f29, f35 | `design:` PR |
| Register R-51 trigger | Widen it: authors strip units rather than trigger it | f01 | Register row |

**Exceptions.** None. Several SHOULD-level items could take §H records if deferred (f24, f26, f27,
f31–f33). No MUST gap is excused.

**Proposed disposition owners.** These are not yet scheduled. The maintainer adopts them into a plan
created with `just plan`, into ADRs, or into register rows. A remediation plan could group the
findings by owner:

| Proposed workstream | Findings | Primary owners |
|---|---|---|
| W1 Physical typing integrity | f01, f03, f08, f11, f12, part of f35 | pse-quantity, pse-modeling, packages, runtime study boundary |
| W2 Flowsheet structure and shared checks | f02, f16, f33 | registry, pse-authoring, pse-modeling, pse-compiler, process packages |
| W3 Analysis execution and studies | f04, f05, f09, f10, f22, f23, f32 | pse-runtime workflow, pse-operations (pure policy), pse-py |
| W4 Failure semantics | f06, f07, f17, f18 (enum getters), f28 | pse-diagnostics, pse-runtime, registry, pse-modeling |
| W5 Evolution and persistence | f13, f14 | pse-schema, pse-codegen, pse-operations, pse-catalog |
| W6 Reuse and dead mechanisms | f15, f20, f21, f29 | pse-compiler, runtime math, backend-native routing |
| W7 Boundary and library hygiene | f18, f19, f24–f27, f30, f31, f34 | pse-py, pse-codegen, pse-authoring, pse-ids, pse-diagnostics |

## 12. Decision

**Behavioural and semantic adequacy: not adequate.**
- G1, G2, G7, PS-G1 and PS-G3 fail.
- G3 passes, with the factorable and envelope items unresolved.
- G4, G5 and G6 pass.
- G8 fails in minor ways.
- The strongest evidence of adequacy is real: the typed package model, complete-contract boundary
  typing, the five separated outcome facts, independent qualification, one commit boundary, and the
  prior Plan 23 test and measurement record.
- The failures sit where meaning is reconstructed locally rather than consumed:
  - inside constitutive laws ([f01](#f01));
  - at the study boundary ([f08](#f08));
  - in failure classification ([f06](#f06), [f07](#f07));
  - in shooting's acceptance ([f09](#f09)).

**Architectural fitness: G9 fails.**
- AP-02, AP-03 and AP-04 are violated; AP-01, AP-05 and AP-06 are violated within bounds.
- The model is adequate for authoring units, methods and data, and for outcome facts.
- It is **inadequate** for process structure (streams), transfer conventions and reduced
  coordinates.
- Its authority is **split** for case execution, studies and failure classification.

**Overall decision: Revise.** This decision:
- covers the declared scope at the stated evidence levels (Implemented / Interface-checked);
- accepts nothing about unexamined numerical accuracy;
- closes no implementation work.

The review makes the next representative changes understandable:
- a new EOS (S01);
- a new flowsheet with a recycle (S02, S07);
- a durable sensitivity sweep (S04, S09);
- a new backend or termination (S03, S10).

It also names what would falsify its main claims:
- a τ/δ-reduced law refused at admission today;
- a shipped flowsheet using `FlowGraph` tears;
- a Python route honoring a case's declared execution;
- a durable study whose failed points publish typed findings.

| Priority | Change | Findings / scenarios | Acceptance evidence | Proposed owner |
|---|---|---|---|---|
| 1 | Physical typing inside laws and at the study boundary: typed intermediates, reduced coordinates and transfer kinds; typed, identity-keyed overlays | f01, f03, f08; S01, S09 | A reduced law is refused; unit constants are retired; an overlay in bar equals one in Pa; oracle fixtures unchanged | W1 (ADR first) |
| 2 | Streams and aggregate ports; connection tears; closure verdicts | f02, f16; S02, S07 | `RecycleFlash` authored with connects, with `FlowGraph` tears and closure verdicts | W2 (short ADR) |
| 3 | One owner for case execution and for studies (pure policy); shooting through §16.6 | f04, f05, f09; S04, S06 | No route literals in pse-py; one study definition under both executors; policy tests without a store | W3 |
| 4 | Failure classification reconciled to §23.2; typed durable failures | f06, f07, f17; S08, S10 | Code/class agreement test; typed `study_outcomes` failure columns | W4 |
| 5 | Evolution: compatibility classes, catalog identity, migrations; derivable Delta predicates | f13, f14; S05, S10, S11 | A registry member addition opens old publications; a DataFusion-label change opens tables | W5 (ADR) |
| 6 | Reuse on Salsa and `DefaultCache`; delete unused mechanisms | f15, f20, f21; S07 | One body admission per edit; one view per durable study; unused mechanisms gone or registered | W6 |
| 7 | Target gaps: root-intent sensitivity; incumbent policy; terminal events; material-model content (phase key, envelopes) | f22, f23, f10, f11, f12 | As in each finding | W3 and W1 (ADRs for f22 and f23) |
| 8 | Boundary and library hygiene | f18, f19, f24–f35 | As in each row | W7 |
