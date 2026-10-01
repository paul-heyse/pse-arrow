# Design review: thermodynamic knowledge base, the code's domain model and design (Plan 24, mid-W1)

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | The standalone tree `thermo-knowledge/` of [Plan 24](../../plans/24-thermodynamic-knowledge-base.md) at repository HEAD `721749df`. The tree's code was last changed in `ece085d7` (W1-M1) and `1e2ac5c5` (plan checkpoint). Contents: about 44k lines in `src/thermo_knowledge/`, 104 test files, 4 mappings (CoolProp, Cantera, NASA CEA, JANAF), 9 readers, 80 verify checks and one qualification case. **Boundary:** the Python code that realises the declaration and runs the pipeline, from staging through mapping, resolution, build, verify and qualify, plus the evaluator. **Neighbours read, not reviewed:** the declaration (`model/`, `forms/`); the contract pages under `thermo-knowledge/docs/`; the [TK2 review](design_review_thermo-knowledge-core_2026-09-30.md) and the plan's disposition table. |
| Standard | Core **3.3** (AP-01–AP-06, DP-01–DP-24, G1–G9). Process-simulator profile **1.3** (PS-01–PS-13, PS-G1–PS-G3). Binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Design tier, target purpose** (the binding default), bounded to the tree. Requested by the maintainer outside the plan's review cadence, which is TK2 and TK9. |
| Reviewer / date | Coordinated by the main agent session, 2026-09-30, and carried out by agents in three phases (details below). This is **not an independent human review**: every reviewer ran in the same agent system. |
| Decisions | Behavioural and semantic adequacy: **not adequate**. G1, G2, G3, G5, G6, G7 and PS-G3 fail; G4, G8 and PS-G1 pass; PS-G2 does not apply. Architectural fitness: **G9 fails**. AP-03, AP-04 and AP-05 are violated; AP-01 and AP-06 are violated within bounds; AP-02 is satisfied. Overall: **Revise**. See [slot 12](#12-decision). |
| Disposition owner | Proposed: [Plan 24 *Finding dispositions*](../../plans/24-thermodynamic-knowledge-base.md#finding-dispositions). This review names proposed owners only; nothing here is scheduled. |

**Who reviewed:**
- **Phase A, evidence.** Three read-only mapping passes, one `code-mapper` and one `library-research` agent gathered evidence. Two of their claims were wrong and were corrected before use:
  - the claim that `missing_convention` is never raised;
  - the claim that deferred foreign-key checks cause "no memory spike".
- **Phase B, review.** Three independent `design-reviewer` agents (Opus) each took one slice:
  - the meta-model in code;
  - source knowledge to canonical records;
  - execution, evaluation and persistence.
- **Phase C, verification.** A fourth `design-reviewer`, who wrote none of the findings, tried to disprove each one.

The coordinator re-read the decisive evidence for every High finding (slot 10).

**Functional target.** The tree must provide:
- one neutral declaration of thermodynamic knowledge, from which DDL, `meta` rows and loader types are generated;
- an intake pipeline that loads about 40 sources without undeclared loss;
- an open-world store with competing assertions, and a closed-world resolved snapshot for production;
- qualification derived from recorded runs against independent oracles;
- early-warning reports of gaps in the production language.

TK2 judged the declared model. This review judges **the domain model of the code that does the work**: does the code represent the model's concepts, and does one authority govern behaviour? It asks this in three domains:
1. the meta-model as realised in Python;
2. thermodynamic and source knowledge at the mapping and identity layer;
3. the pipeline and evaluation domain.

**Analysis modes and workloads (profile).** No flowsheet problem is in scope. The workloads considered are:
- *extension*: a new source, form, entity class, rule form or slot type;
- *boundary round trip*: units, conventions and identity, from source bytes to canonical records;
- *qualification of a form against an oracle library*.

The evaluator exists only for qualification.

**Drivers and variation axes.** W1-M2 and Waves 2–8 add about 30 sources: chemicals (about 76.5k compounds), equation-of-state carriers, CALPHAD, aqueous databases, and ThermoML (about 12k files, millions of data points). Re-resolution will happen routinely as identity grows. More oracles per form are coming: Cantera, `cea`, `chemicals` and `thermo`. The meta-model grows by new rule forms, slot types and entity classes. TK9 must export a snapshot. The qualities that matter:
- one behavioural owner per concept (AP-04, DP-01);
- semantic distinctions as types (DP-02);
- reuse that cannot go stale (DP-09);
- bounded resources (DP-20);
- structured diagnostics (DP-21);
- claims no stronger than their evidence (DP-22).

**Baseline.**
- *Measured.* `pse_thermo` holds four sources: 1,494,807 canonical rows in 271 tables with 525 foreign keys, and 3,669 held rows. `tk build --dry-run` peaks at 614,600 kB RSS in 1.97 s. Mapping JANAF's records phase peaks at 1,761,184 kB in 65 s (slot 10).
- *Reported by the plan, not re-executed.* The suite last ran whole at 2,665 passed; verify passes 80 of 80 checks; one case is qualified.

**Inspected.**
- All of `declaration/`, `generate/`, `canonical/`, `mapping/`, `resolve/`, `build/`, `verify/` and `qualify/`, plus `reuse.py`, `transposition.py`, `pipeline_contract.*` and `cli.py`. A few large modules were read in sections: `declaration/resolve.py`, `canonical/writer.py`, `expression/evaluate.py` and `qualify/source.py`.
- All four mappings; `identity/decisions.toml` with its generating test helper; samples of readers and `staging/`.
- `expression/{tree,check,scope,parser,implicit,compiled}.py`; `acquire/{runner,lock,runtime}.py`.
- A sample of `sql/verify/`; `conftest.py` and the test support modules.
- The contract pages `meta-model.md`, `pipeline.md`, `expressions.md` and `acquisition.md`.
- PostgreSQL 18 documentation (`populate.html`, §14.4) and source (`trigger.c`, `ri_triggers.c`).

**Not examined.**
- Most reader internals and side environments.
- `oracles/coolprop.py` in detail.
- `expression/evaluate.py` line by line.
- Survey records beyond the chemicals, thermo and ThermoML samples.
- The declared ontology, except where code or W1 evidence contradicts it.

Following the review plan, no test, generator or pipeline write was run. The only executions are the probes listed in slot 10, so no failure count against the zero baseline is measured here.

**Exclusions** (maintainer choices):
- The platform is fixed (Python and PostgreSQL); moving work to Rust or DataFusion is not assessed.
- Ontology adequacy was settled by TK2 and the 14 hard cases, and is re-opened only on evidence.

### Answers to the review's questions

1. **Does each meta-model concept have one behavioural owner?** No, for the closed sets that pipeline behaviour switches on:
   - check-rule forms ([C06](#c06));
   - the stored layout of a parameter set and the states of its values ([C05](#c05));
   - three binding channels that are not reconciled ([C17](#c17)).

   Transposition is the counter-example and the model to follow: it is typed as a `Literal`, has one runtime owner, and is expanded in one place.

   The least machinery that fixes the rest is:
   1. a type checker ([C14](#c14));
   2. closed types for the closed sets;
   3. expansion in the resolver of every derived column, following the `arrangement` precedent;
   4. exhaustive renderers.

   A registry or plugin layer is not needed. The single `Field` record is adequate for columns and overloaded for contract parameters ([C16](#c16)).
2. **Is a layer of domain operations missing at the mapping layer?** Not a layer. The split into declaration, `mapping.toml` and a short `mapping.py` is the right amount of machinery. What is missing is a handful of domain operations:
   - a piecewise parameter set whose region follows from its declared intervals;
   - a dataset;
   - a composition;
   - a subject lookup;
   - one statement of a source's unit convention.

   Without them, R1 and R2 fail locality ([C08](#c08)).
3. **Is the identity model fit for W1-M2a?** No. The formula-scope rule joins different substances across carriers and duplicates structural species ([C01](#c01)). A derived rule is stored as curated, positional decisions ([C03](#c03)). Nothing checks identity across carriers ([C04](#c04)). This must be fixed **before** the W1-M1 equivalence assessments, not after.
4. **Should stages share one lifecycle?** Yes, as a small bespoke module rather than a workflow library. The copies have already diverged: the build does not check the resolution edge ([C02](#c02)). TK2 F19 left this open; the shared key builder that now exists was half of the answer.
5. **Does it scale?** The build union does, by estimate (73 B per row in Arrow). Mapping memory does not: about 1.45 kB per canonical row, all held in memory. The deferred foreign-key queue does not either: it is linear, held in memory, and cannot spill ([C09](#c09)).
6. **Is there a typed diagnostic model?** Only in the declaration layer and for `Blocked`. Held reasons drift from their declared meanings for 65 % of held rows, writer and stage refusals are prose, and 360 tests match message text ([C07](#c07)).
7. **Is qualification ready for several oracles per form?** The case and harness machinery is. Three things are not:
   - the subject → library-key model ([C10](#c10));
   - the relative-bound grid ([C12](#c12));
   - quadrature status ([C13](#c13)).
8. **Is the code ready for TK9 (judged lightly)?** Not in three respects. The snapshot would become a fifth reader of the stored layout ([C05](#c05)). It would export two species for water ([C01](#c01)). And its rights refusal could act on stale rights ([C11](#c11)).

## 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State / effect owner | Local test setup |
|---|---|---|---|---|---|
| `declaration/` (strict msgspec decoding, one 3,225-line `Resolver`, `loader`) | Name and type resolution; validity of a declaration; expansion of `arrangement` | TOML → `model.Declaration`; located `Diagnostic` with `Code` | Depends on nothing of the pipeline, except late imports of the expression checker and of projection diagnostics, which are declared refusals | None; pure | Fixture directories |
| `generate/` (`plan`, `ir`, `meta_tables`, `meta_rows`) | Projection to PostgreSQL, reification, fingerprint | Declaration → `ir.Table` → SQL text | Mechanism depends on meaning | None; purity is tested | No database |
| `canonical/` (writer, values, invariants) | Identifiers, units, orientation, provenance, competing assertions, `load` invariants | `emit.*` → Arrow tables | Depends on the declaration and on `generate` | All rows held in memory until `tables()` | No database |
| `staging/`, `readers/` | Source formats | Module convention (`READER_VERSION`, `TABLES`, `read`) → schema-checked Parquet with a manifest | Readers depend only on the staging writer | Streamed in batches | Fixtures; real-store round trips |
| `mapping/` (spec, context, runner) and `mappings/<id>/` | Source-to-canonical meaning, block atomicity, held outcomes, rule use | `mapping.toml` validated against the declaration and staged schemas; `identities(ctx)`, `records(ctx)` | Mapping → context → writer and spec → declaration | Whole staged tables cached as dicts; one writer per run | Unit tests without a database; source logic only end to end |
| `resolve/` and `identity/decisions.toml` | Identity rules and their order | Phase-1 claims plus decisions → `_resolution/` | Pure function between the two mapping phases | `_resolution/` | Unit tests; the decisions/rule check needs staged data |
| `reuse.py` plus each stage's runner | What a result depends on; skip, install | `stage_key` → `Key`; discovery, currency and install are copied per stage | Every stage depends on `reuse` | Work directory, then rename | Filesystem |
| `build/` (plan, union, database, currency) | Union and conflicts; one-transaction replace; currency of qualification runs | Canonical manifests plus IR → `BuildPlan` → database | Depends on `generate` and on canonical manifests | The canonical schemas of `pse_thermo` | Dry run without a database; the real build needs PostgreSQL |
| `verify/` and `sql/verify/` | The definition of each verify invariant | 80 named queries, bound to declared invariants by a header line | Reads the built database | None; each check rolled back | One module-scoped database |
| `qualify/` (case, harness, source, run, persist) and `oracles/` | Subject selection, grids, comparison, run records | Case TOML; JSON harness protocol; `ParameterSource` over the database | Depends on build output, the evaluator and a subprocess | `_qualification/<case>/` plus the live `qual` rows of that case | Database plus subprocess, for policy too |
| `expression/` | Grammar, dimensional closure, evaluation, implicit solving, validity membership | `ParameterSource` → values | Depends on the declaration; SymPy, NumPy, SciPy and pint inside | `CompileCache` owned by the caller | `InMemorySource`; no database |

**Composition root.** The composition root is `cli.py`. The stages exchange directories with manifests rather than each other's objects. There are four context types: `acquire.Runtime`, `staging.StageContext`, `canonical.Environment` and `qualify.Context`. Each falls back to `config` through `default_factory`, and three library sites ignore an injected tree ([C22](#c22)).

**Model adequacy and authority in the code (AP-04).**
- **The meta-model domain** is realised as frozen records with almost no behaviour. Where a concept is typed and owned once, behaviour is coherent:
  - transposition (`model.py:136` `Literal`, `transposition.py`);
  - expression nodes (`expression/tree.py`, a closed union with visitors);
  - the `arrangement` expansion (`resolve.py:1354-1392`).

  Where a concept is a `str` with optional fields, each consumer re-derives its meaning:
  - check rules ([C06](#c06));
  - slot shape, value state and the stored layout ([C05](#c05)).
- **The source-knowledge domain** has a generic writer and a declarative mapping specification; these are strengths. Its domain operations are written per mapping ([C08](#c08)). Its identity model is inadequate across carriers ([C01](#c01), [C03](#c03), [C04](#c04)).
- **The pipeline domain** has explicit concepts for key, run, blocked outcome and conflict. Upstream currency, phase and cause do not have them ([C02](#c02), [C07](#c07)).

## 3. Contracts, authority and constraints

| Concept or operation | Semantic owner and update path | Consumer obligation / invariant | Enforcement and failure | Implementation and divergence |
|---|---|---|---|---|
| Species identity across carriers | `docs/pipeline.md` §1 rule 4: formula identity "for entity classes where a formula and charge identify the species by convention (elements, monatomic and simple inorganic ions, and the members of a chemical system…)"; each mapping's `[[formula_scope]]` | One canonical species per substance | The engine is pure and order-independent; no verify check covers identity agreement | The mappings declare whole thermochemical scopes; the engine keys `formula:<f>:<q>` globally (`resolve/engine.py:384-401`). Specification and implementation diverge ([C01](#c01)) |
| Curated identity decision | `identity/decisions.toml`; `reject` means "the source's identification is wrong" (`resolve/decisions.py:24`) | Human decisions only | A test compares the file with a rule in `tests/identity_support.py`; it is skipped without staged data | 1,478 rule-derived rejects keyed by Cantera list position ([C03](#c03)) |
| Upstream currency | `docs/pipeline.md` §6; each stage's runner | A stage refuses a stale upstream | Map checks identity and resolution; build checks identity and fingerprint only | The build omits resolution ([C02](#c02)); the map key omits the source manifest ([C11](#c11)) |
| Check-rule form | `CheckDecl` (`schema.py:107`), flattened to `Requirement.rule: str` | One meaning per form | SQL CHECK (`generate/plan.py:218-245`) plus a Python pre-check (`canonical/invariants.py:115-174`) | Two renderings, both with catch-all branches; no differential test ([C06](#c06)) |
| Stored set layout and value state | Generator projection (`plan.py:322-373`); `value_state` enum facets (`model/parameters.toml:47`: "a rule reads the facets, never the names of members") | Every reader sees the same set | `_put` column check; SQL errors on a rename | Re-derived by the writer, `DatabaseSource`, currency, `meta_rows` and verify SQL; currency already reads less than evaluation does ([C05](#c05)) |
| Held reason | `held_reason` enum (`model/qualification.toml:38-49`) | The reason states the cause | Checked against the enum at raise time | `missing_convention` carries five other causes, 65 % of held rows ([C07](#c07)) |
| Piecewise validity | The family's `interval` (`forms/standard_state.toml:45`); validity regions | A region states where values are valid | The writer refuses overlapping pieces (`writer.py:1668-1686`), not gaps | Each mapping builds the hull from first and last pieces ([C08](#c08)) |
| Qualification subject key | Case `subjects.library_key` (carrier, scope) | One library key per evaluated subject | `Blocked("subject_unidentified")` | Keyed by target, not by the evaluated set's origin ([C10](#c10)) |
| Validity of a grid point | `RecordValidity` in `DatabaseSource.validities` (`qualify/source.py:529-595`) | Points inside the stated region | Membership in the evaluator | `qualify/run.py:327-354` re-reads regions without relative bounds ([C12](#c12)) |
| Definite integral | `docs/expressions.md` §3 "Definite integrals" (rtol 1e-12) | Value to the stated tolerance | None | `quad(...)[0]`; the status is discarded ([C13](#c13)) |

**Physical semantics (profile), for what the code carries:**

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| Slot value | Quantity type, coherent SI storage; pint conversion at mapping | Declared composition basis, compared at calls | Convention set required at emit when a form reads it (`mapping/context.py:1057-1096`) | Validity regions with clauses; membership reported by the evaluator | Declaration plus `mapping.toml` |
| NASA and Shomate coefficients | Scale stated per coefficient in each `mapping.toml`; only the dimension is checked (`mapping/spec.py:908-913`) | Molar | Gas constant and energy reference from the carrier's convention set | Hull of the pieces, computed in mapping code; gaps unchecked ([C08](#c08)) | Split between the form and each mapping |
| Species identity | — | Formula plus charge, or InChIKey | — | — | Engine rules plus generated rejects ([C01](#c01), [C03](#c03)) |
| Datum (JANAF) | The observable's storage unit | — | Standard state decoded per segment | None stated | `evidence.toml` plus `mapping.toml` |
| Qualification tolerance | Relative or absolute, per case | — | Comparison basis recorded | Grid inside the region; relative bounds lost ([C12](#c12)) | Case file |

**Well-posedness statement (profile).** No flowsheet problem is formulated. The one solve in scope is an implicit block of a form. At load, its residual count must equal its unknown-element count, and a mismatch is refused with the form and block named. There is no structural rank analysis, which is proportionate.

## 4. Change scenarios and composition

The scenarios keep the labels R1–R10 from this review's plan. Plan 24's own representative
changes stay in its §E table; TK2's scenarios keep their IDs S01–S11.

| Scenario / stimulus, kind and conditions | Expected response and boundary | Edit / composition path | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|
| <a id="r1"></a>**R1** A fifth NASA/Shomate carrier, such as chemicals' WebBook Shomate (instance) | `mapping.toml` plus a short `mapping.py`; no unit rules restated | Restate 7 coefficient units plus T bounds with loss text (as `cantera/mapping.toml:665-705`). Copy piece sorting, occurrence counting, the first/last hull and the derivation (`cantera/mapping.py:169-196`, `nasa_cea/mapping.py:88-109`). Subjects keyed by CAS resolve to InChIKey species, while Cantera's and CEA's resolve to `formula:` species | **Fail.** The workflow is copied, and the new carrier never meets the existing ones ([C08](#c08), [C01](#c01)) | *Interface-checked* |
| <a id="r2"></a>**R2** ThermoML at millions of points, W6 (instance at scale) | One dataset operation; bounded memory | Copy `janaf/mapping.py:184-257` (raw `emit.kind` and `emit.relation`) | **Fail.** No dataset operation exists. Mapping memory is linear (1.45 kB per canonical row, *Measured* on JANAF), and the foreign-key queue is linear and unspillable ([C09](#c09)) | *Measured* for JANAF; extrapolated beyond |
| <a id="r3"></a>**R3** A `pseudo_component` resolution class, W8 (domain concept) | One class declaration plus its rules | `spec.py:498`, `decisions.py:88`, `engine.py:55-68`, `:254-260`, `:734-741`, a branch in `_others`, `output.py`, `pipeline.md` §1 | **Partial.** Rules written as code are legitimate; the class facets are scattered ([C19](#c19)) | *Interface-checked* |
| <a id="r4"></a>**R4** A new check-rule form (policy) | One owner | `CheckDecl`, `CHECK_FORMS`, `_check_rule`, `Requirement` fields, `plan._requirement_constraint`, `invariants.ddl_violation`, `meta_tables` vocabulary, docs: 7–9 edits in 6 modules. A missed SQL branch silently becomes `num_nonnulls(...) = 1`; a missed Python branch silently passes. A set-level form ("fractions sum to one") is not declarable at all | **Fail** ([C06](#c06), [C15](#c15)) | *Interface-checked* |
| <a id="r5"></a>**R5** An integer slot type, an open W2 decision (domain concept) | A few owners, all discoverable | `_slot` (`resolve.py:2352`), the checker's slot read (`check.py:446`), evaluator, parameter source; also `meta_tables.py:37` `SHAPES` and `qualify/source.py:313-318,453-454`, which silently skips shapes it does not list | **Partial.** The number of owners is fair for a new concept, but they cannot be discovered and one of them drops the new shape silently ([C05](#c05)) | *Interface-checked* |
| <a id="r6"></a>**R6** A streaming or engine-backed build (mechanism substitution, PSE-S04) | Contained in `build/` | `union_table(...) -> TableUnion` is a narrow seam; foreign keys are already separate `ir.ForeignKey` statements | **Pass** | *Interface-checked* |
| <a id="r7"></a>**R7** Upstream reorders a Cantera file, or an identity decision changes (binding) | No positional churn in curated data | Cantera source entities are keyed `_locator` (`mappings/cantera/mapping.toml:16-25`); 1,392 rejects and the provisional canonical keys embed list positions; a stale decision is "reported, not applied" | **Fail.** Decisions move silently onto other species. Records built before a re-resolve are not refused by the build ([C03](#c03), [C02](#c02)) | *Interface-checked*; (artifact, section, name) is unique across the 3,837 staged Cantera entries (*Measured* by the DR2 reviewer) |
| <a id="r8"></a>**R8** A Cantera oracle and a second case for one form (mechanism, PSE-S02) | Harness plus case file | The harness protocol and case files carry it. The subject key model blocks: Cantera has 2,661 entities on 2,246 targets, up to 24 per target | **Fail** ([C10](#c10)) | *Measured* (SQL) |
| <a id="r9"></a>**R9** Rename a kind attribute, or the parameter column convention (binding) | Refused at load, or one owner | A pipeline-contract attribute is refused at load. `<slot>__state`/`__redirect` is restated in `plan.py:327`, `writer.py:544,802-803`, `qualify/source.py:265-266` and `sql/verify/set_default_needs_policy_default.sql:29`. Facet and enum literals in verify SQL are caught only by `tests/test_verify.py` against a built database | **Partial** ([C05](#c05), [C21](#c21)) | *Interface-checked* |
| <a id="r10"></a>**R10** Test a policy without PostgreSQL (PSE-S05) | No database where the responsibility does not need one | Declaration, generator, writer, spec, context, resolver and evaluator: no database needed. Verify needs a relational engine by nature. Qualify's selection, grid, sampling and verdict policies need a database plus a subprocess (`tests/test_qualify_run.py:85-98,401-436`) | **Pass**, except qualify ([C12](#c12)) | *Interface-checked* |

**Simulator journeys (profile).**
- *Add a property model:* a new form under a contract is still local (TK2 S02). A new carrier of an existing form fails R1.
- *Replace an implementation:* R6 passes; R8 fails on the key model.
- *Test admission locally:* R10.
- *Out-of-envelope evaluation:* membership is reported, but a relative bound is mis-gridded ([C12](#c12)).
- *Boundary round trip:* units and conventions are carried. Identity is not ([C01](#c01)). A unit-scale slip on a coefficient (K against kK) passes load and verify, and only qualification catches it ([C08](#c08)).

The remaining journeys do not apply.

## 5. Mechanisms and execution, where material

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|
| Map, phase 2 | Staged tables cached as dict rows (`mapping/staged.py:99-117`); every emitted row held by the writer (`writer.py:271,316-356`); one writer per run | Staged key, mapping bytes, declaration, framework, identity hash, **global** resolution hash | Work directory, then rename | Any re-resolve re-maps every source (`runner.py:360-379`) | *Measured*: JANAF 1,218,228 canonical rows, 1.76 GB, 65 s |
| Build | Per-table in-memory union (`build/union.py:21-22,203-251`); one transaction; `SET CONSTRAINTS ALL DEFERRED` (`database.py:182-198`) | Canonical manifests; checks identity and fingerprint, **not resolution** (`build/inputs.py:62-144`) | Drop, DDL, ingest, commit; previous schemas kept on failure | Union memory about 73 B per row (*Measured* on `ev.datum`). Each inserted row with foreign keys queues an AFTER-trigger event in backend memory with no spill, and each is checked by a per-row RI query at commit (PostgreSQL 18 `trigger.c`, `ri_triggers.c`; `populate.html`) | Dry run *Measured*; database load not measured |
| Verify | Named queries; `LIMIT 0` describe, count, fetch | Built database with matching fingerprint | Read-only, each rolled back | Binding: invariant name (header) for 69 checks; a test list for 11 structural checks; vocabulary literals unchecked ([C21](#c21)) | *Interface-checked* |
| Qualify | Select subjects, grid, evaluate, call harness, compare, record | Fingerprint, evaluation hash, case hash, library version, stored-value digest | Parquet, then a scoped live load | Currency recomputed by `build/currency.py` from a hand-written read set ([C05](#c05)) | *Interface-checked* |

**Numerical stage columns (profile), reference evaluator.**

| Formulation policy | Derivative source and order | Scaling | Problem class · solver capability | Status to outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|
| Guards and piece policy are explicit; conditionals are lowered to their branch predicates; no smoothing | Symbolic (SymPy); implicit-function theorem through implicit blocks; the derivative of a quadrature is refused | Residuals compared with the sum of the magnitudes of their own terms | Polynomial roots, or a bracketing scan plus `brentq`; otherwise `root`/`least_squares` with stated Newton polishing; `quad` for definite integrals | Implicit blocks: failure is a `SolveFailure` naming form, block and point, never NaN. **Quadrature: `quad(...)[0]`, status and error estimate discarded** (`evaluate.py:161-163,1320-1327`) | Implicit: residual ≤ 1e-8 relative to term size plus a step test, independent of SciPy's status. Quadrature: none ([C13](#c13)). Near-double roots can be dropped by the imaginary-part test (`implicit.py:273`); this occurs only at exact tangency and is noted, not a finding |

## 6. Architectural assessment and gates

| Foundation | Scenario and evidence | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | Stage boundaries are real. The stored set layout is known to the generator, writer, `DatabaseSource`, currency and verify SQL (R9). `qualify/run.py` re-reads validity regions that `qualify/source.py` owns | **Violated** (narrow) | [C05](#c05), [C12](#c12) |
| AP-02 Stable contracts | `ParameterSource`, the harness protocol, `RowSource`, `union_table` and the emit API are explicit; R6 is local | **Satisfied** | None |
| AP-03 Composition | The stage lifecycle (currency check, discovery, work directory, install) is copied per stage and has diverged. The piecewise-set and composition workflows are copied per mapping, and a dataset operation is missing (R1, R2) | **Violated** | [C02](#c02), [C08](#c08) |
| AP-04 Domain model and semantic authority | *Adequacy:* cross-carrier species identity is wrong ([C01](#c01)); the qualification key model cannot express several entities per subject ([C10](#c10)); causes are not modelled ([C07](#c07)). *Authority:* check rules ([C06](#c06)), the stored set read model ([C05](#c05)) and rule-derived "curated" decisions ([C03](#c03)) each have several owners | **Violated** | [C01](#c01), [C03](#c03), [C05](#c05), [C06](#c06) first |
| AP-05 Explicit structure | No type checker runs, so exhaustiveness over closed sets is unenforced ([C14](#c14)). Held reasons drift from their declarations ([C07](#c07)). Resource bounds are undeclared ([C09](#c09)) | **Violated** | [C14](#c14), [C07](#c07) |
| AP-06 Local reasoning and testability | Declaration, generator, writer, mapping specification, resolver and evaluator are testable without a database (R10). Qualify policy is not; tests bind 360 message texts | **Violated** (narrow) | [C12](#c12), [C07](#c07) |

| Gate | Verdict | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **Fail** | Two renderings of each check form, which can diverge without failure ([C06](#c06)); the set read model derived per consumer, with currency already narrower than evaluation ([C05](#c05)); a rule stored as curated data ([C03](#c03)) | As cited |
| G2 Semantic fidelity | **Fail** | Ethylene oxide (JANAF) and acetaldehyde (Cantera gri30) are one species form; water is two species ([C01](#c01)). 65 % of held rows carry a reason false for them ([C07](#c07)). `withheld` and `not_applicable` both become "no value" at the parameter source ([C05](#c05)) | [C01](#c01), [C07](#c07) |
| G3 Validity | **Fail** (narrow) | Unknown check forms silently pass or are mis-rendered ([C06](#c06)); a hull is published across piece gaps ([C08](#c08)); relative bounds are mis-gridded rather than refused ([C12](#c12)) | As cited |
| G4 Hidden behaviour | **Pass** | Effects sit at stage boundaries; generator purity is tested. Ambient defaults are noted in [C22](#c22) | — |
| G5 Consistency and recovery | **Fail** | The build commits records mapped against a superseded resolution ([C02](#c02)). Publication is never torn: work directory and rename, flock'd lock, one-transaction build | [C02](#c02) |
| G6 Transformation and reuse | **Fail** | The map key omits the source manifest that supplies rights ([C11](#c11)); currency misses redirect targets, policy defaults and positions ([C05](#c05)) | [C11](#c11), [C05](#c05) |
| G7 Truthful capability claims | **Fail** (narrow) | The plan says `tk verify` covers "cross-source agreement" ([C04](#c04)); `expressions.md` states a quadrature tolerance that is never checked ([C13](#c13)); the root `pyproject.toml` says the tree "checks itself" for typing ([C14](#c14)); "no staged row is unexplained" rests on drifted reasons ([C07](#c07)); `pipeline.md` §3 step 5 says the commit validates check constraints, which PostgreSQL checks at insert | Correct the statements or make them true (slot 11) |
| G8 Library leverage | **Pass** | pyarrow, msgspec, pint, SymPy, SciPy, RDKit, httpx and ADBC own the generic work; the bespoke lifecycle and declaration language are reasoned in the contract pages. Gains are available inside pinned libraries (slot 8) | — |
| G9 Architectural fitness | **Fail** | AP-03, AP-04 and AP-05 violated; AP-01 and AP-06 violated within bounds | As above |
| PS-G1 Physical consistency | **Pass** (*Interface-checked*), with a latent envelope concern | Units, dimensions, bases and convention facts are carried and refused when missing. The hull across gaps ([C08](#c08)) is latent: no current set was shown to have a gap. Identity errors are judged under G2 | [C08](#c08) |
| PS-G2 Well-posedness | **Not applicable** | No flowsheet or case is formulated; implicit blocks have a count check at load | — |
| PS-G3 Numerical integrity | **Fail** (narrow; revises TK2's pass) | Quadrature status is discarded ([C13](#c13)); a relative bound produces a silent mis-grid ([C12](#c12), latent: 0 relative clauses today). The implicit solver's acceptance is sound | [C13](#c13), [C12](#c12) |

## 7. Findings

Severity:
- **High:** a MUST gap with a concrete wrong outcome in declared scope.
- **Medium:** a MUST gap with a bounded consequence, or amplified change.
- **Low:** a SHOULD deviation or a local defect.

Class:
- *design defect:* the design is wrong or insufficient.
- *implementation gap:* the design is right and the code does not yet realise it.
- *open question:* a decision is missing.

"TK2" marks an incomplete correction of a finding that Plan 24 records as resolved.

| ID | Severity · class | Finding | Principles / gate / scenario |
|---|---|---|---|
| [C01](#c01) | High · design defect | The formula-scope rule joins different substances across carriers and builds a second species universe beside the structural one | AP-04, DP-04 · G1, G2 · R1 |
| [C02](#c02) | High · design defect | Upstream currency is re-implemented by each stage, and the build omits the resolution edge | AP-03, AP-04, DP-19 · G5, G7 · R7 |
| [C03](#c03) | Medium · design defect | A derived identity rule is stored as 1,478 "curated" rejects keyed by list position | DP-01, DP-04 · G1 · R7 |
| [C04](#c04) | Medium · design defect | Nothing checks identity agreement across carriers | AP-04, DP-21 · G2, G7 |
| [C05](#c05) | Medium · design defect | The stored parameter-set read model (shape, value state, layout, redirects, defaults) has no owner, and currency has diverged from evaluation. TK2 F01 and F15 | AP-01, AP-04, DP-01, DP-02, DP-09 · G1, G6 · R5, R9 |
| [C06](#c06) | Medium · design defect | Check-rule forms have no single behavioural owner; both renderings fall through silently | AP-04, AP-05, DP-01, DP-03 · G1, G3 · R4 |
| [C07](#c07) | Medium · design defect | There is no typed cause model below the declaration: held reasons drift, refusals are prose, and tests bind message text. TK2 F17 | AP-05, AP-06, DP-02, DP-21 · G2, G7 |
| [C08](#c08) | Medium · design defect | Form-level operations are written per mapping: piecewise hull, composition, subject lookup, unit convention, datasets | AP-03, DP-06, DP-16 · G3 · R1, R2 |
| [C09](#c09) | Medium · implementation gap | Neither mapping nor the build's foreign-key validation is bounded in memory | DP-10, DP-20 · R2 |
| [C10](#c10) | Medium · design defect | The qualification library key assumes one source entity per subject per carrier | AP-04 · R8 |
| [C11](#c11) | Medium · implementation gap | The map reuse key omits the source manifest that supplies the carrier's title and rights. TK2 F08 | DP-09 · G6 |
| [C12](#c12) | Medium · implementation gap | Qualification grids drop relative validity bounds silently; `run.py` re-reads regions and mixes policy with SQL | AP-01, AP-06, PS-02 · G3, PS-G3 · R10 |
| [C13](#c13) | Medium · implementation gap | Quadrature status and error estimate are discarded | PS-07, PS-10, DP-11 · PS-G3, G7 |
| [C14](#c14) | Low · implementation gap | No type checker runs on the tree; exhaustiveness and 87 `type: ignore` markers go unchecked | AP-05, DP-02 · G7 |
| [C15](#c15) | Low · design defect | Recurring set-level invariants are copied SQL, not declarable forms | DP-06, DP-16 · R4 |
| [C16](#c16) | Low · design defect | `Field` conflates column parameters with contract parameters | DP-02 |
| [C17](#c17) | Low · design defect | Framework roles, the pipeline contract and resolver literals bind the same kinds without reconciliation. TK2 F11 | DP-01 · G1 |
| [C18](#c18) | Low · implementation gap | The DDL IR keeps constraint bodies as SQL text; primary keys are recovered by regex | DP-08, AP-02 |
| [C19](#c19) | Low · design defect | Per-class resolution facets are scattered over three modules | AP-04, DP-01 · R3 |
| [C20](#c20) | Low · open question | Any disagreement among one entity's structural identifiers makes it ambiguous; untested at 76.5k scale | DP-02 |
| [C21](#c21) | Low · design defect | Verify SQL binds enum and facet names as text literals | AP-05, DP-03 · R9 |
| [C22](#c22) | Low · implementation gap | Ambient `config` defaults ignore an injected tree; no database marker; the fingerprint is recomputed per case | AP-05, AP-06, DP-10 |

<a id="c01"></a>
### C01 — The formula-scope rule joins different substances across carriers

- **Evidence.**
  - *Implemented:* `resolve/engine.py:384-401` builds a global key `formula:<formula>:<charge>`. The Cantera, NASA CEA and JANAF mappings declare whole species scopes as formula-identified, with no discriminator (e.g. `mappings/janaf/mapping.toml:26-28`).
  - *Interface-checked:* the contract (`docs/pipeline.md` §1 rule 4) confines formula identity to "elements, monatomic and simple inorganic ions, and the members of a chemical system whose source declares its species by formula", and resolves it by formula, charge **and chemical system**.
  - *Measured* (SQL on `pse_thermo`, 2026-09-30): Cantera `data/gri30.yaml#/species/52` (acetaldehyde, CH3CHO), `gri30_highT` and `nDodecane_Reitz` entries, and JANAF `C-129#1` (oxirane) all resolve `unique` to the species form `["formula:C2H4O:+0","gas"]`. Water exists both as `formula:H2O:+0` and as `XLYOFNOQVPJJNP-UHFFFAOYSA-N`.
  - The DR2 reviewer's query finds that 902 of 1,690 formula species are reached from several carriers on formula alone.
- **Consequence.**
  - W1-M1's planned JANAF-against-NASA equivalence assessments would compare ethylene-oxide tables with acetaldehyde fits, and record the result as disagreement in the data.
  - CoolProp, chemicals and W2 data for water never meet the thermochemical data for water.
  - The plan's *Open items* records the risk but schedules the fix for W1-M2a, after the W1-M1 work that consumes these joins.
- **Correction** (*Proposed*).
  - Rule 4 joins only within its declared listing unit, or within the contract's conventional classes.
  - Cross-carrier joining goes through a declared formula → structure rule. RDKit computes the Hill formula of each structural species; a formula with exactly one structure in a pinned reference corpus resolves to that InChIKey species. Otherwise the entity stays provisional with a typed reason, and the link evidence is recorded in the resolution report.
  - Order this before the W1-M1 equivalence assessments.
  - Shares its correction with [C03](#c03).
- **Verification.** After re-resolution, the C2H4O entities are not joined, and water from Cantera, JANAF and CoolProp targets one InChIKey species. The report lists unlinked formulas with their candidate structures.

<a id="c02"></a>
### C02 — Upstream currency is re-implemented by each stage; the build omits the resolution edge

- **Evidence** (*Interface-checked*).
  - A records manifest stores its `resolution` hash (`mapping/runner.py:376-379`). The build checks only the phase-1 `identity` hash (`build/inputs.py:62-88`) and the declaration fingerprint (`:121-144`). Nothing compares a source's `inputs["resolution"]` with the current `_resolution/manifest.json`, and the build always loads the current resolution.
  - The lifecycle is copied per stage:
    - recorded-key comparison: `staging/stage.py:97-117`, `mapping/runner.py:236,382-388`, `resolve/command.py:90-97`, `qualify/run.py:862-870`;
    - install: `staging/stage.py:226-238` against `canonical/store.py:164-176`;
    - discovery: `resolve/command.py:48-60`, `build/inputs.py:91-121`, `mapping/runner.py:99-105`.
  - `docs/pipeline.md` §6 states that a stage is stale when the output it read was made again.
- **Consequence.** A source that is not re-mapped after `tk resolve` keeps its old subject identifiers. Wherever the old target still exists (for example a formula species another carrier still reaches), its records pass the foreign keys and attach to an entity its own source entity no longer resolves to. W1-M2a's re-resolution of about 76.5k compounds makes this the normal path. Verify does not check currency.
- **Correction** (*Proposed*, the least machinery). A shared stage module of a few hundred lines, not a workflow library:
  - typed per-phase manifests (a tagged union in place of `CanonicalManifest.phase: str`);
  - an `upstream: {artifact: manifest hash}` map kept apart from scalar inputs;
  - one `check_upstream()` that every consumer calls, the build included;
  - shared `current()` and `publish()`.

  Hash the resolution per carrier (the subjects this carrier's records use). That fixes both this edge and the global re-map in [C09](#c09). The build records every upstream hash, which `tk project` needs.
- **Verification.** After a decision change and `tk resolve`, `tk build` refuses and names the source to re-map. `rg` finds one install path and one currency comparison.

<a id="c03"></a>
### C03 — A derived identity rule is stored as curated, positional decisions

- **Evidence.**
  - *Measured:* `identity/decisions.toml` holds 1,478 `reject` decisions (1,392 Cantera, 80 NASA CEA, 6 JANAF) and no human `identify` or `distinct` decision.
  - *Implemented:*
    - The file's header (lines 33-47) states that one rule gives them all. The rule lives in `tests/identity_support.py:26-104`, and `tests/test_identity_decisions.py` compares the file with it; that test is skipped without staged data (`:29-32`). No command regenerates the file.
    - `reject` is declared as "the source's identification is wrong" (`resolve/decisions.py:24`). These entries mean only that a formula does not identify.
    - Cantera entities are keyed by `_locator` (`mappings/cantera/mapping.toml:16-25`), and provisional canonical keys embed that position.
- **Consequence.**
  - A Cantera file reorder (R7) moves rejects silently onto other species and churns provisional entities.
  - A human decision for Cantera has to name an opaque position.
  - The resolution report counts rule outcomes as curated.
- **Correction** (*Proposed*).
  - Make "the listing names several species under this formula" a declared condition of rule 4, parameterized on the formula scope (listing unit, name column, name normalisation). It yields `unresolved` with a typed reason, so records still load.
  - Key Cantera entities by (artifact, section, name).
  - Delete the 1,478 rejects, `identity_support.py` and its test, and test the rule in the engine.
- **Verification.** `decisions.toml` holds only human decisions. A fixture that reorders a Cantera file leaves keys and resolution byte-identical.

<a id="c04"></a>
### C04 — Nothing checks identity agreement across carriers

- **Evidence** (*Interface-checked*).
  - The plan's stage table gives `tk-verify` "cross-source agreement".
  - Of the 80 checks in `sql/verify/`, the identity-related ones are `source_entity.class_matches_target`, `source_entity.provisional_target_matches_status`, `species.charge_matches_composition` and two constituent and polymorph checks. None compares identity across carriers.
  - `tk verify` passes with the joins of [C01](#c01) in place (reported by the plan).
- **Consequence.** Re-resolution and every new formula scope can make or break joins with no stage noticing.
- **Correction** (*Proposed*).
  - Add a structural check: a formula-keyed species whose formula and charge equal those of a structure-keyed species.
  - Add a report: formula species reached from two or more carriers whose name assertions share no member.
- **Verification.** Against the current database, the check flags `formula:H2O:+0` and the report lists `formula:C2H4O:+0`.

<a id="c05"></a>
### C05 — The stored parameter-set read model has no owner

- **Evidence** (*Interface-checked*).
  - `Field.shape` is a `str | None`, whose vocabulary is set in `resolve.py:2344-2382` and restated in `meta_tables.py:37`.
  - "Holds a set" is restated at `meta_rows.py:590`, `writer.py:1323`, `qualify/source.py:313,454`, `expression/check.py:437` and `build/currency.py:49`.
  - Stateful slots are expanded separately by the generator (`plan.py:322-373`), the writer (`writer.py:534-562`), the reader (`qualify/source.py:265,309-318`) and verify SQL.
  - `value_state` member names are written as literals in `writer.py:549-562`, `qualify/source.py:309,317,319` and `resolve.py:2702`. That breaks the declared rule that "a rule … reads the facets, never the names of members" (`model/parameters.toml:47`), although `qualify/source.py:213` uses the checked binding `pc.UNASSERTED_POLICY.member`.
  - The currency read set (`build/currency.py:157-226`) follows only `nested_set` and `set_reference`. `DatabaseSource` also reads redirect targets (`source.py:426-444`), policy defaults (`:183-233`) and constituent positions (`:597-611`).
  - `withheld` and `not_applicable` both become "no value" at `source.py:316`.
- **Consequence.**
  - A changed redirect target or member position under the same identifiers does not retire a passing qualification run. This is an F01-class staleness, latent today: SQL finds no non-null `__redirect` value.
  - A new slot shape (R5) is dropped silently by the reader.
  - The TK9 snapshot would be a fifth reader of the same layout.
- **Correction** (*Proposed*).
  - A `SlotShape` and a `SlotState` closed type on the model, with predicates.
  - Stateful slots expanded once in the resolver into fields plus `present_iff` requirements, following the `arrangement` precedent.
  - One naming owner for derived columns and child tables.
  - One `SetReader` used by `DatabaseSource`. Currency, through a recording `RowSource`, then replays exactly what evaluation read, and the snapshot uses the same reader. Delete currency's own traversal.
- **Verification.** `rg '__state|"nested_set"'` finds only the owner. A run that read a redirected slot is skipped after the target value changes.

<a id="c06"></a>
### C06 — Check-rule forms have no single behavioural owner

- **Evidence** (*Interface-checked*; the fall-through branches were confirmed by the coordinator).
  - The typed `CheckDecl` (`schema.py:107-115`) is flattened into `Requirement.rule: str | None` plus flat parameters (`model.py:104-124`).
  - Forms are listed in `CHECK_FORMS` (`resolve.py:92`), the `CheckDecl` fields and the meta vocabulary (`meta_tables.py:489`), and the doc string at `:478` omits `ordered_same_reference`.
  - The semantics are rendered twice:
    - SQL in `generate/plan.py:218-245`, where an unknown rule becomes `num_nonnulls(...) = 1`;
    - Python in `canonical/invariants.py:115-174`, where an unknown one-attribute rule returns `None`.
  - The one equivalence test covers `present_iff` against hand-written expectations (`tests/test_requirements.py:267-286`), not PostgreSQL.
- **Consequence.** The next rule form produces wrong DDL and a Python pre-check that does nothing, unless the author writes the right targeted test. Where the two disagree, a row the database would accept is held as `validation_failed`.
- **Correction** (*Proposed*).
  - A closed `Check` union on `Requirement`, built by the resolver.
  - Exhaustive `match` with `assert_never` in both renderers under a type checker ([C14](#c14)).
  - `CHECK_FORMS` and the meta vocabulary derived from the union.
  - One shared case table that drives both the Python test and the PostgreSQL test.
- **Verification.** Adding a dummy variant fails type checking in both renderers, and the case table covers every variant.

<a id="c07"></a>
### C07 — No typed cause model below the declaration

- **Evidence.**
  - *Interface-checked:* `held_reason.missing_convention` is declared as "the convention facts a form reads are not stated" (`model/qualification.toml:44`). It is raised for other causes:
    - an unmapped column that states a value (`mapping/context.py:179-184`);
    - a decoding without a matching rule (`:473-479`);
    - "no phase includes the entry" and "phases of different aggregations" (`mappings/cantera/mapping.py:112-128`);
    - a JANAF designator that names no single aggregation (`mappings/janaf/mapping.py:94-99`).
  - Its declared condition raises `MappingError` instead (`context.py:1092`).
  - `unknown_subject` is also used for an undeclared vocabulary member (`context.py:576`, `mappings/janaf/mapping.py:73`).
  - Writer problems are strings (`canonical/values.py:89-99`, `writer.py:76-82`), all collapsed into `validation_failed` (`context.py:645-646`). `held_row` has no rule or cause reference (`mapping/coverage.py:157-161`).
  - There are 36 exception classes; only the declaration has a `Code` (`declaration/diagnostics.py:14`). `qualify/run.py:448-451` maps a `ReuseError` to `harness_failed`. Tests use 360 `match=` assertions.
  - *Measured:* 2,378 of 3,669 held rows (65 %) are `missing_convention`.
- **Consequence.**
  - The residue reports that close every wave can separate causes only by parsing prose.
  - The plan's "no staged row is unexplained" rests on that prose.
  - Rewording a message breaks tests.
  - Callers, including the future `tk project`, cannot tell "re-run upstream" from "invalid input" from an infrastructure failure.
- **Correction** (*Proposed*).
  - One coded, located diagnostic shared by `RowHeld`, the writer, stage errors and `Blocked`, reusing the shape of `declaration/diagnostics.Diagnostic`, with a `caused_by`.
  - Split `missing_convention` by cause (aggregation not stated, undecoded value, unmapped stated field, unit not stated), or re-document it.
  - Give `held_row` `rule`, `column` and `caused_by` fields.
  - Tests assert codes.
- **Verification.** A residue report grouped by (reason, rule) reproduces the plan's *Open items* categories without reading detail text.

<a id="c08"></a>
### C08 — Form-level operations are written per mapping

- **Evidence** (*Interface-checked*).
  - **Validity hull and composition, copied.** Cantera and NASA CEA each build a validity region from the `T_low` of the first piece and the `T_high` of the last (`cantera/mapping.py:169-196`, `nasa_cea/mapping.py:88-109`). The form declares the interval itself (`forms/standard_state.toml:45`). Both also copy the composition loop (`cantera:152-161`, `nasa_cea:60-70`).
  - **Gaps pass the writer.** It refuses overlapping pieces but not gaps (`writer.py:1668-1686`).
  - **Composition, subject key, occurrences.** Composition derived in phase 1 (`mapping/formula.py:57-74`) is emitted again per pair in phase 2. The scope key is restated in phase 2. There are three occurrence-counting idioms.
  - **Units.** Each coefficient's source unit is stated per slot, and only its dimension is checked (`spec.py:908-913`). So a K against kK slip on Shomate B–D passes.
  - **Datasets.** JANAF builds them from raw `emit.kind` and `emit.relation` calls (`janaf/mapping.py:184-257`).
  - CoolProp does not use the hull; it reads its stated region.
- **Consequence.**
  - Every NASA or Shomate carrier in W1-M2 and W2 copies 20–30 lines and about 10 unit rules (R1).
  - Every tabulated source copies the dataset assembly (R2).
  - A thousandfold coefficient error passes load and verify.
  - A validity gap between pieces is published as valid.
- **Correction** (*Proposed*, operations rather than a layer).
  - `emit.piecewise_set(...)` derives the region from the declared interval and refuses a gap, or declares it as a loss.
  - One `DatasetBuilder` (dataset, component, phase, column, point, datum).
  - `emit.composition(...)` reuses the phase-1 result.
  - `ctx.subject_of(scope, row)`.
  - Occurrences numbered by the writer.
  - One statement per mapping (or partition) of the source's unit convention, from which each coefficient's unit follows by its slot's dimension, with an explicit unit as override. The form fixes the dimension, not the scale.
- **Verification.** Cantera and CEA shrink with byte-identical canonical Parquet. A test Shomate mapping has no `T_low`/`T_high` literals and states its convention once.

<a id="c09"></a>
### C09 — Neither mapping nor the build's foreign-key validation is bounded in memory

- **Evidence.**
  - *Implemented:* `mapping/staged.py:99-117` caches whole staged tables as dict rows. The writer holds every row (`writer.py:271,316-356`), with one writer per run.
  - *Measured:* JANAF's records phase used 1.76 GB for 1,218,228 canonical rows, about 1.45 kB per row. That is twenty times the union's 73 B per row (`ev.datum`: 48.1 MB of Arrow for 656,793 rows).
  - *Implemented:* the build defers all foreign keys in one transaction (`build/database.py:182-198`). PostgreSQL 18 queues one AFTER-trigger event per inserted row and foreign key in backend memory without spilling. At commit it runs one RI query per row. Adding a foreign key afterwards validates it in one query (`trigger.c`, `ri_triggers.c`; `populate.html`).
  - Foreign keys are already separate `ir.ForeignKey` statements.
- **Consequence** (extrapolation; assumptions stated). At about 5–6 canonical rows per ThermoML data point, 2–5 M points means 15–45 GB for mapping alone, approaching the 120 GB scope cap at 10 M points. The foreign-key queue grows to tens of millions of events with per-row probes at commit. The union is not the constraint.
- **Correction** (*Proposed*).
  - Mapping reads staged rows per artifact (`pyarrow.dataset` filtered scans). The writer flushes block-scoped rows to per-table `ParquetWriter` row groups and keeps an identity index only for globally keyed kinds.
  - The build creates tables without foreign keys, loads, then runs `ALTER TABLE … ADD FOREIGN KEY` before commit, in the same transaction, so atomicity is unchanged.
- **Verification.** JANAF RSS falls and stays flat for a synthetic run at ten times JANAF, with byte-identical output. Time and backend RSS of both build variants measured on the current sources and a synthetic 30 M-row load.

<a id="c10"></a>
### C10 — The qualification library key assumes one source entity per subject per carrier

- **Evidence.**
  - *Implemented:* `qualify/run.py:241-285` looks up all source entities of the case's carrier and scope whose target is the subject, and blocks on any subject with other than one. It collects these problems before applying a `list` selection.
  - *Measured* (SQL): Cantera `species` has 2,661 entities on 2,246 targets, up to 24 per target. JANAF has up to 6. Cantera local keys are positional (`data/nasa_gas.yaml#/species/9`).
- **Consequence.**
  - A Cantera case blocks with `subject_unidentified`, even a `list` case whose own subjects are fine.
  - The plan's intended "key-prefix filter for per-file cases" would couple cases and oracles to reader locator syntax, so R7 churn reaches qualification.
  - `Blocked` is a truthful outcome, not a wrong pass, hence Medium.
- **Correction** (*Proposed*).
  - Take the library key from the evaluated set's own origin: the carrier's entity in that set's artifact.
  - Let the key be an asserted identifier (for Cantera, file and name).
  - Check only the selected subjects.
- **Verification.** A gri30 case selects H2O, and the harness receives (file, name).

<a id="c11"></a>
### C11 — The map reuse key omits the source manifest

- **Evidence** (*Interface-checked*). `provenance.from_source` (`canonical/provenance.py:84-110`) reads the carrier's title and rights from `sources/<id>.toml` (`mapping/runner.py:148`). Neither the map key (`runner.py:177-190`) nor the staged key (`staging/reader.py:200-223`) includes it. The key builder covers code and libraries, but which data a stage reads is chosen stage by stage. This is an incomplete correction of F08.
- **Consequence.** A corrected `[[rights]]` leaves `tk map` reporting `current`, and stale rights rows are built. Those are the rows the TK9 snapshot refuses on.
- **Correction** (*Proposed*). Hash the carrier's encoded source information into both phase keys. With [C02](#c02), declared data inputs become part of the shared stage contract.
- **Verification.** A rights edit re-runs phase 1.

<a id="c12"></a>
### C12 — Qualification grids drop relative validity bounds silently

- **Evidence** (*Interface-checked*).
  - `qualify/run.py:327-354` (`_regions`) selects no `lower_relative_to` or `upper_relative_to`, although `tk.region_clause` has them and `DatabaseSource.validities` reads them (`qualify/source.py:529-595`).
  - `_spread` (`run.py:405-423`) then grids offsets as absolute values.
  - *Measured:* 0 of 4,769 clauses are relative today. W1-M2a introduces them (chemicals' "valid up to Tc").
  - `run.py` mixes selection SQL, grid, sampling, comparison, verdict and reporting, so these policies can be tested only with a database and a subprocess.
- **Consequence.** The first relative region is gridded over its offsets (for example −50 to 0 K), where it should be resolved or refused with `no_grid`. Every policy change needs PostgreSQL to test.
- **Correction** (*Proposed*).
  - Selection returns typed facts, reusing `DatabaseSource.validities`.
  - A pure policy module covers grid, sample, compare and verdict, resolving or blocking relative clauses.
  - Recording stays separate.
- **Verification.** The policy tests take plain values, and one test covers a relative clause.

<a id="c13"></a>
### C13 — Quadrature status is discarded

- **Evidence** (*Implemented*).
  - `expression/evaluate.py:1320-1327` and the Debye function at `:161-163` take `scipy.integrate.quad(...)[0]`.
  - Nothing turns an `IntegrationWarning` into a refusal or records it.
  - `docs/expressions.md` §3 "Definite integrals" states the requested relative tolerance as if it were achieved.
- **Consequence.** An unconverged integral (IAST spreading pressure, Debye, any `integral`) is compared as if it met the tolerance.
- **Correction** (*Proposed*). Use `full_output`, and refuse with a typed `SolveFailure` when `ier != 0` or the error estimate exceeds the stated tolerance.
- **Verification.** An integrand that `quad` cannot resolve gives a refusal naming the form and point.

<a id="c14"></a>
### C14 — No type checker runs on the tree

- **Evidence** (*Interface-checked*).
  - `thermo-knowledge/pyproject.toml:50-56` configures Ruff only.
  - The root `pyproject.toml:358` excludes the tree from pyrefly with "it checks itself".
  - `src/` carries 87 `# type: ignore` markers that nothing reads.
- **Consequence.** The `Literal` and closed-union typing that [C05](#c05) and [C06](#c06) rely on cannot be enforced. Vocabulary drift shows up late or never. On its own this is a SHOULD deviation; it is listed because it enables the Medium corrections.
- **Correction** (*Proposed*). Run pyrefly, the repository's checker, over the tree with a zero baseline, and derive the `meta_tables` vocabularies from the model's types.
- **Verification.** A clean pyrefly run, and no hand-restated vocabulary.

<a id="c15"></a>
### C15 — Recurring set-level invariants are copied SQL

- **Evidence.** `*.ordinals_contiguous.sql`, `*.magnitude_matches_kind.sql` (`accuracy_statement` and `slot_uncertainty` differ only in the table name) and `*acyclic*.sql` are near-copies with the same `LATERAL` locator block.
- **Correction.** A few declared set-level forms (`contiguous_from_one`, `acyclic`, `sums_to`) whose verify SQL is generated. Hand-written checks remain for rules that are genuinely bespoke.

<a id="c16"></a>
### C16 — `Field` conflates column parameters with contract parameters

- **Evidence.** `declaration/model.py:71-94`: `over`, `basis` and `observable_from_set` apply only to contract arguments; `presence`, `shape`, `accepts` and `references` only to slots.
- **Judgement.** One record is adequate for attributes, keys, values, slots and indices, since each projects to a column.
- **Correction.** A separate `ContractParameter` record.

<a id="c17"></a>
### C17 — Three binding channels are not reconciled

- **Evidence.** The channels:
  - framework roles (`resolve.py:1848-1850`, consumed by `generate/plan.py:404` and `qualify/source.py:492,556`);
  - the pipeline contract (`pc.PARAMETER_SET`, consumed by `writer.py:1307`);
  - resolver literals (`resolve.py:2702`).

  This is what the F11 correction left in place. A rename is refused at load. Divergence is possible only if a role is bound to another existing kind.
- **Correction.** `contract_check` requires `framework[role]` to equal the contract's kind for that role. The `value_state` members become a contract enum entry.

<a id="c18"></a>
### C18 — The DDL IR keeps constraint bodies as text

- **Evidence.** `generate/ir.py:29-40` stores `Constraint.body: str`; `build/keys.py:20-33` recovers primary keys by regex over that body. A failure raises; it does not mislead.
- **Correction.** Typed bodies (`PrimaryKey`, `Unique`, `Check`, `Exclude`), rendered at the end. SQLAlchemy Core is not needed for this.

<a id="c19"></a>
### C19 — Per-class resolution facets are scattered

- **Evidence.** `mapping/spec.py:498`, `resolve/decisions.py:88`, `resolve/engine.py:55-68,254-260,734-741`. R3 touches three modules plus the declaration.
- **Correction.** One class-policy table in the engine (target kind, decision-identifiable, rules, identified through), which `spec` and `decisions` consume.

<a id="c20"></a>
### C20 — Disagreeing structural identifiers make an entity ambiguous

- **Evidence.** `resolve/engine.py:338-344`. *Measured:* 7 of 126 CoolProp structural entities are ambiguous.
- **Question.** At chemicals' scale, should a disagreement within one source be ambiguity, or a declared precedence among schemes with the conflict reported? Count it at M2a before deciding.

<a id="c21"></a>
### C21 — Verify SQL binds vocabulary through text literals

- **Evidence.** `slot_uncertainty.magnitude_matches_kind.sql:7-11` (`facet = 'unquantified'`), `producing_derivation_is_fit.sql:7`, `species.charge_matches_composition.sql` (`key = 'charge'`).
  - `tests/test_verify.py` inserts a violation for every check against the real declaration, so a rename fails a test (hence Low).
  - It is not refused at load, and catching it needs PostgreSQL.
- **Correction.** A generated `meta.member(enum, name)` function that raises on an undeclared name. The checks call it.

<a id="c22"></a>
### C22 — Ambient defaults, no database marker, repeated fingerprint

- **Evidence.**
  - Code that falls back to `config` and ignores an injected tree:
    - `qualify/harness.py:194` runs with `cwd=config.TREE_DIR` regardless of the `tree` it receives;
    - `reuse.py:147-149` reads the core lock from `config`;
    - `canonical/environment.py:31-32` defaults `sources_dir` and `lock_path` from `config` instead of from `tree`.
  - Pytest runs with `--strict-markers`, but no markers are declared. 57 test files need PostgreSQL, with no marker or skip.
  - The declaration fingerprint (a full DDL generation) is recomputed per case, in `qualify/run.py:798` and again in `qualify/source.py:638`.
- **Correction.**
  - Build one workspace value at the CLI root and remove the `default_factory` fallbacks from library code.
  - Add a `database` marker, set by the fixtures.
  - Memoize the fingerprint per declaration.

**Strengths to keep.** These are why most corrections stay local.
- **One declaration** drives DDL, Arrow schemas and `meta` rows.
- **Transposition:** values are stored as asserted, and the rule is typed and applied on read in one module (the precedent for [C05](#c05) and [C06](#c06)).
- **The expression tree** is a closed union with visitors, and `scope.py` is the one classifier of names. Operators are still strings.
- **Strict msgspec decoding** with located, coded diagnostics in the declaration layer.
- **The pipeline contract** is checked at load, and AST tests forbid unlisted names.
- **Verify** checks are bound to declared invariants in both directions (structural checks only by a test list).
- **`reuse.stage_key`** is one key builder.
- **Publication** is never torn: work directory and rename, a flock'd lock, a one-transaction build.
- **The build union** is columnar, with structured conflicts.
- **The harness protocol** is standalone and unit-stated; blocked reasons are typed.
- **Implicit solves** are accepted independently of the solver status.
- **The mapping specification** is validated before any row runs. Every staged column needs a rule, and an unused rule refuses the run.
- **Mapping blocks** are atomic.
- **Resolution** is pure and order-independent.
- **Staging and `load-src`** stream.

## 8. Library fit and ownership cost

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade cost | Recommendation |
|---|---|---|---|---|---|
| Static typing and exhaustiveness | Whole tree | **Candidate:** pyrefly (already the repository's checker) | Checks `Literal`, StrEnum and union `match` with `assert_never` | Low; 87 markers to resolve | **Adopt** ([C14](#c14)) |
| Closed meta-model sets | `declaration/model.py` | **Current:** `str` plus optional fields. **Candidates:** frozen dataclass unions built by the resolver; msgspec tagged unions | Tagged unions in TOML would need a tag in every authored check; converting in the resolver keeps the syntax | Low | Dataclass unions in the resolver; keep `CheckDecl` |
| Stage lifecycle | `reuse.py` plus runners | **Current:** a shared key, copied lifecycle. **Candidates:** Snakemake, DVC, doit, Dagster | The engines track files, not database outputs, and would add a second record of currency beside the manifests | Moderate to adopt, small to share | **Keep bespoke, shared** ([C02](#c02)); the `pipeline.md` §6 reasoning stands |
| Bounded mapping I/O | `mapping/staged.py`, `canonical/writer.py` | **Candidates:** `pyarrow.dataset` filtered scans; incremental `pq.ParquetWriter` (already used in staging) | Fits per-artifact processing | Low | **Adopt** ([C09](#c09)) |
| Union at scale | `build/union.py` | **Current:** pyarrow in memory. **Candidate:** DuckDB (spills; reads Parquet) | Not needed at 73 B per row | A new engine | **Keep**; revisit if one table's measured union exceeds memory |
| Foreign-key validation | `build/database.py` | **Current:** deferred triggers. **Candidate:** PostgreSQL's own `ADD FOREIGN KEY` after load | Native; one validating query per constraint | Small: the IR already separates foreign keys | **Adopt** ([C09](#c09)) |
| DDL IR | `generate/ir.py` | **Candidate:** SQLAlchemy Core | It has `ExcludeConstraint` and domains, but no construct for range types; churn in `schema.sql` | Moderate | **Not needed**; typed IR bodies suffice ([C18](#c18)) |
| Harness result validation | `qualify/harness.py:112-168` | **Current:** hand-written checks. **Candidate:** msgspec structs with `forbid_unknown_fields` | Fits; msgspec is already a dependency | Low | **Adopt** |
| Formula → structure link | `resolve/structure.py` | RDKit `CalcMolFormula` | Fits | Version already in the resolution key | **Use** ([C01](#c01)) |
| Quadrature | `expression/evaluate.py` | `scipy.integrate.quad` with `full_output` | Fits | None | Keep; read its status ([C13](#c13)) |

## 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline | R6, R10 (except qualify) local; R1, R2, R4, R7, R8 fail | Stage contracts are manifests; most policies are pure | A declaration-driven writer and spec; copied lifecycle and domain operations | Wrong identity joins; stale build inputs; unbounded memory at W6 | Not acceptable past W1-M1 |
| **Proposed:** closed types plus a type checker; one set read model; a shared stage module; a handful of emit operations; identity rules in the engine; foreign keys added after load | R1–R5, R7–R10 local; R6 unchanged | One owner per closed set; policies testable without a database | About a few hundred lines of shared code; deletes duplicated renderings, 1,478 decisions and a test-held rule | Correct joins; bounded memory; refusals named | **Selected.** Revisit if a closed set needs third-party extension, which would justify a registry |
| Library-owned: a workflow engine for stages, DuckDB for union and mapping, SQLAlchemy for DDL | Lifecycle and scale solved by the engines | External engines own lifecycle and cache | A second record of currency; engines learn nothing of the declaration | Adoption cost exceeds the gain at about 40 sources | Not selected; DuckDB stays the revisit candidate for the union |
| Simplest viable: keep the code, fix C01 and C02 only | C01 and C02 corrected; everything else as baseline | Unchanged | Copies keep growing per source | Correct now; amplification over W2–W8 | Rejected: the copies multiply by about 30 sources |

**Reference practice.** The surveyed libraries resolve species by name within one database and never join across databases by formula. Cantera, NASA CEA and JANAF each list several isomers under one formula. Cross-database compilations key species by structure or by registry identifiers; the NIST WebBook, for example, uses InChI and CAS, with formula only as a search filter. This was read for behaviour only and was not surveyed in depth.

## 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| Build union memory and time | *Measured* | `PSE_MEMORY_MAX=32G bash scripts/memory-cap.sh /usr/bin/time -v just -f thermo-knowledge/justfile tk-build --dry-run` | Four sources, no database touched | 1,494,807 rows; peak RSS 614,600 kB; 1.97 s wall, 5.05 s user |
| Mapping memory | *Measured* | `runner.run_records(Environment(canonical_dir=<scratch copy>), "janaf", force=True)` under the memory cap and `/usr/bin/time -v`; writes only to the scratch copy | JANAF records phase | Peak RSS 1,761,184 kB; 65 s; 1,218,228 canonical rows (from its manifest) |
| Union bytes per row | *Measured* (by the DR3 reviewer) | pyarrow read of `janaf/ev.datum.parquet` | — | 48.1 MB for 656,793 rows |
| C2H4O join and duplicate water ([C01](#c01)) | *Measured* | SQL joining `tk.source_entity`, `prov.carrier` and `tk.material_entity` | — | Five entities on `formula:C2H4O:+0` (three Cantera gas, JANAF `C-129#1` gas, CEA liquid); water as both `formula:H2O:+0` and an InChIKey species |
| Build ignores resolution ([C02](#c02)) | *Interface-checked* by the coordinator | Read `build/inputs.py:62-144` and `mapping/runner.py:370-379` | — | Only identity and fingerprint are compared |
| Entities per target ([C10](#c10)) | *Measured* | Grouped SQL on `tk.source_entity` | — | Cantera 2,661 on 2,246 (maximum 24); JANAF 2,073 on 1,500 (maximum 6); CEA and CoolProp one each |
| Held-reason drift ([C07](#c07)) | *Measured* plus *Interface-checked* | `select reason, count(*) from qual.held_row`; reading the raise sites | — | `missing_convention` 2,378 of 3,669 |
| Decisions file ([C03](#c03)) | *Measured* | `grep -c '^action = "reject"'` | — | 1,478 reject, 0 identify, 0 distinct |
| Check-rule fall-through ([C06](#c06)) | *Interface-checked* by the coordinator | `generate/plan.py:244-245`, `canonical/invariants.py:159-174` | — | Confirmed |
| Deferred foreign-key memory ([C09](#c09)) | *Interface-checked* | PostgreSQL 18 documentation and source, read by the DR3 reviewer | — | One queued event per row and foreign key, in memory; per-row RI checks at commit. The database load was not measured |
| ThermoML memory | *Proposed* (extrapolation) | Linear in canonical rows at JANAF's ratio | 5–6 canonical rows per point | 15–45 GB at 2–5 M points; not measured |

**Commands run.** File reads, `rg`, `grep`; read-only `psql -X -h /var/run/postgresql -d pse_thermo` `SELECT` queries; the two probes above. **Not run:** the test suite, generators, or any pipeline stage writing to the store or the database. The plan's test counts are reported, not re-executed.

**Phase C changes to the slice findings.**
- **Downgraded:** P02 (now [C10](#c10)) from High to Medium, because `Blocked` is truthful. S02 ([C03](#c03)) from High to Medium, because the divergence test runs on this machine. M3 ([C14](#c14)) to Low. P08 ([C21](#c21)) to Low, because `tests/test_verify.py` catches renames.
- **Merged:** M4 and P09 into S03 ([C07](#c07)); P03 into M2 ([C05](#c05)); P06 into S05 ([C09](#c09)).
- **Added:** N1 ([C04](#c04)).
- **Dropped:** the claim that currency fails to hash tabulated rows, because evaluation does not read them while TK2 F20 is deferred.
- **Corrected:** the held-row share (65 %, not 58 %); the decision counts; and the lead that deferred foreign-key checks cause "no memory spike".

## 11. Authority changes, exceptions and disposition

No blueprint section, accepted ADR or register row changes: the tree is standalone tooling under an active plan.

**Text to correct through Plan 24 (route: the plan in `docs/plans/`) or the tree's contract pages:**
- **Plan 24 *Open items*, formula-scope identity:** move the fix before the W1-M1 equivalence assessments ([C01](#c01)).
- **Plan 24 *Database and pipeline*:** `tk-verify` "cross-source agreement" ([C04](#c04)).
- **Plan 24 checkpoint:** "No staged row is unexplained (held rows carry typed reasons)" ([C07](#c07)).
- **Plan 24 *Finding dispositions*:** TK2 F01, F08, F11, F15 and F17 are recorded as resolved, but [C05](#c05), [C11](#c11), [C17](#c17), [C05](#c05) and [C07](#c07) respectively show their corrections are incomplete. Their rows should link here rather than being rewritten.
- **`docs/pipeline.md`:**
  - §1 rule 4 against the mappings' formula scopes ([C01](#c01));
  - §3 step 5: check constraints are validated at insert, not at commit;
  - §6 currency claim ([C02](#c02)).
- **`docs/expressions.md` §3 "Definite integrals":** quadrature tolerance ([C13](#c13)).
- **`model/qualification.toml`:** `held_reason` meanings ([C07](#c07)).
- **Root `pyproject.toml:358`:** the comment "it checks itself" ([C14](#c14)). An ordinary edit.

No SHOULD exception is recorded. The MUST gaps stay explicit in slot 6.

**Proposed disposition rows for the plan's table**, all **open**. Owners are proposed, not scheduled.

| Finding | Scenario | Proposed work owner | Evidence or revisit trigger |
|---|---|---|---|
| C01, C03, C04 | R1, R7 | A packet before W1-M1 completion (identity) | C2H4O not joined; one water species; decisions hold only human entries; the identity check flags duplicates |
| C02, C11 | R7 | The same packet, or one immediately after it (stage currency) | `tk build` refuses after a re-resolve and names the source; a rights edit re-runs phase 1 |
| C05, C06, C14, C17, C18 | R4, R5, R9 | A meta-model typing packet before W2 (the integer slot decision) | pyrefly clean; a dummy check variant fails both renderers; a redirect-target change retires a run |
| C07 | — | Before the W1 residue report | Residue grouped by (reason, rule) without detail text |
| C10, C12, C13 | R8, R10 | W1-M1 qualification | A gri30 case selects H2O; a relative-clause policy test; a `quad` refusal test |
| C08, C09 | R1, R2 | W1-M2a for the emit operations; before W6 for bounded mapping; the build change at any time | Cantera and CEA byte-identical after the change; JANAF RSS flat at ten times the size; foreign-key timing measured |
| C15, C16, C19, C21, C22 | R3, R4, R9 | When next touched | As named in each finding |
| C20 | — | W1-M2a (decision) | The ambiguous count, split into in-source and cross-source conflicts |

## 12. Decision

**Behavioural and semantic adequacy: not adequate.**
- **Fail:** G1, G2, G3, G5, G6, G7 and PS-G3, each on named causes.
- **Pass:** G4 and G8; PS-G1 at *Interface-checked*, with a latent envelope concern.
- **Not applicable:** PS-G2.

The decisive behavioural defects are two. Cross-carrier identity is wrong in the built database ([C01](#c01), *Measured*). And the build can commit records mapped against a superseded resolution ([C02](#c02)).

**Architectural fitness: G9 fails.**
- **Violated:** AP-03 (copied lifecycle and domain operations) and AP-04: model adequacy fails on identity, the qualification key and causes, and authority fails on check rules, the set read model and rule-derived decisions. AP-05 is violated too: closed sets go unchecked, held reasons drift, and resource bounds are undeclared.
- **Violated within bounds:** AP-01 and AP-06.
- **Satisfied:** AP-02, which is why the corrections stay local.

**Overall: Revise.**

**What should be kept:** the architecture TK2 accepted, which the code realises well where it is typed. One declaration drives the generated artifacts. Transposition and the expression tree have one typed owner each. The mapping specification is declarative, with rule-use accounting. Resolution is pure. Publication is never torn. The build union is columnar. The harness protocol is versioned. Implicit solves are accepted independently of the solver.

**What must change:** before the W1-M1 equivalence assessments and the W1-M2a re-resolution, identity and stage currency. Before W2, the meta-model's closed sets and the set read model. Before W6, bounded mapping and foreign keys added after load.

**The main uncertainty:** the scale figures beyond JANAF are extrapolations, and the database load was not measured.

This decision accepts nothing as implemented or tested beyond what slot 10 labels.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| 1 | Identity across carriers: rule 4 confined, formula → structure linking, the derived rule in the engine, an identity check | C01, C03, C04 · R1, R7 | C2H4O separated; one water; only human decisions; the check flags duplicates | Plan 24 (proposed) |
| 2 | One stage lifecycle with typed manifests and upstream hashes; carrier data in keys | C02, C11 · R7 | Build refuses stale records; a rights edit re-maps | Plan 24 (proposed) |
| 3 | Qualification ready for several oracles: key from the set's origin, a pure grid policy, quadrature status | C10, C12, C13 · R8, R10 | gri30 H2O case; relative-clause test; `quad` refusal | Plan 24, W1-M1 (proposed) |
| 4 | Closed types and a type checker; one set read model; one check-rule owner | C05, C06, C14, C17, C18 · R4, R5, R9 | pyrefly clean; differential case table; redirect currency | Plan 24 (proposed) |
| 5 | A typed cause model | C07 | Residue grouped by code | Plan 24 (proposed) |
| 6 | Emit operations and bounded resources | C08, C09 · R1, R2 | Byte-identical output; flat RSS; foreign-key timing | Plan 24, W1-M2a and pre-W6 (proposed) |
| 7 | Smaller items | C15, C16, C19, C20, C21, C22 | As named | Plan 24 (proposed) |
