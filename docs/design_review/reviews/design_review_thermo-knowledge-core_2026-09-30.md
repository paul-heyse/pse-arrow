# Design review: thermodynamic knowledge base, core model v0 (Plan 24, TK2)

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | The standalone tree `thermo-knowledge/` at the "core model v0" checkpoint of [Plan 24](../../plans/24-thermodynamic-knowledge-base.md) (packet TK2): the contracts under `thermo-knowledge/docs/`, the declaration in `model/*.toml` and `forms/*.toml`, the implementation in `src/thermo_knowledge/`, `sql/generated/schema.sql`, `sql/physical.sql`, `sql/verify/`, the survey records in `survey/*.toml`, the CoolProp slice (`mappings/coolprop/`, `qualification/`, `oracles/coolprop.py`) and the built database `pse_thermo`. **Boundary:** what is known (identity, systems and phases, conventions, observables, forms, parameters, reactions, evidence, derivations, provenance and rights) and the pipeline that consolidates it. **Neighbours read, not reviewed:** production `packages/reference/domain/models/*.pse`, blueprint §6.15, §8.1, §8.4, §9.1–§9.10, register rows R-49 and R-50, and the handoff semantic dictionary (sections 1 and 6, sheets IC-15 and IC-21). |
| Standard | Core **3.3** (AP-01–AP-06, DP-01–DP-24, G1–G9). Process-simulator profile **1.3** (PS-01–PS-13, PS-G1–PS-G3). Binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Design tier, target purpose** (the binding default), bounded to the workstream. The production thermodynamic schema is not under review except where the target is meant to supersede it (slot 9 and [F21](#f21)). |
| Reviewer / date | Agent reviewer applying the `design-review` and `design-review-process-simulator` skills, 2026-09-30. It did not author the subject, but it runs in the same agent system, so this is **not an independent human review**. |
| Decisions | Behavioural and semantic adequacy: **not adequate** (G1, G2, G3, G6, G7 and PS-G1 fail, each on a narrow, named cause; G8 unresolved). Architectural fitness: **G9 fails** (AP-04 and AP-05 violated; AP-01, AP-02, AP-03 and AP-06 satisfied). Overall: **Revise**. See [slot 12](#decision). |
| Disposition owner | Proposed: [Plan 24 *Finding dispositions*](../../plans/24-thermodynamic-knowledge-base.md#finding-dispositions). This review names proposed owners only; nothing here is scheduled. |

**Evidence labels used below.** *Interface-checked*: a contract, declaration or source file was
read. *Implemented*: code for the stated path exists and was read; its tests were not run by this
review. *Measured*: a read-only query of `pse_thermo` on 2026-09-30. *Proposed*: a correction or
a behaviour that does not exist yet. "Reported by the plan" marks a claim this review did not
re-execute.

**Functional target.** One neutral declaration of thermodynamic knowledge, rich enough to hold
what about 36 libraries and datasets represent in very different ways, from which the PostgreSQL
schema, loader types and (later) a `.pse` rendering are generated; an intake pipeline that loads
every source without loss of declared meaning; and a closed-world export (the resolved snapshot)
that production packages can adopt. The second product is early warning: every place the
production language cannot express the target is a recorded gap.

**Analysis modes and workloads (profile).** No square simulation, optimization, dynamics or
estimation is in scope. The simulator workloads this review considers are *extension* (adding a
property or reaction model and its data) and the *boundary round trip* (units, basis, conventions
and identity from source bytes to canonical records and out to a package). The reference
evaluator exists only to qualify forms.

**Drivers and variation axes.** New sources in known format families; new forms under existing
contracts; new concepts (kinds, relations, value shapes); new invariants; new oracle libraries;
competing assertions and identity decisions that change over time; and the eventual projection
to production. The qualities that matter are one authority per meaning (AP-04, DP-01), no meaning
in names or sentinels (DP-02), enforceable constraints (AP-05, DP-03), reuse that cannot go stale
(DP-09) and claims no stronger than their evidence (DP-22).

**Baseline.**

- *Measured.* `pse_thermo` holds one source (`coolprop@ae81610e7d23`): 123 parameter sets and 712
  term rows of `vapor_pressure_exp_series_tau.pure`, 123 envelopes, 123 `fit` derivations, 136
  source entities (119 `unique`/`structural`, 7 `ambiguous`, 10 `unresolved`), 150 species (17
  provisional), one `qualification_run` (`passed`, basis `source_library`, 3,075 points, relative
  tolerance 1e-10, worst deviation 4.48e-12) and one row in `qual.form_qualification`. There are no
  rows in `species_form`, `composition`, `defined_mixture`, `auxiliary_of`, `attribution` or any
  `ev` table, and no `api` schema.
- *Interface-checked.* Forms declared: NASA-7, NASA-9, Shomate and one saturation-pressure curve.
  `tk project` is a stub. The fourteen hard-case fixtures do not exist yet.
- *Reported by the plan, not re-executed.* Verify has no failing check; every stage has tests.

**Inspected.** All six contract pages; every module of `model/` and `forms/`; the plan; the
generator plan and fingerprint, the loader and the framework and slot resolution in
`declaration/resolve.py`; the canonical writer, `invariants.py`, `transposition.py` and
`identity.py`; the mapping runner, spec validation and the CoolProp mapping; resolution command
and engine header; build inputs; all seventeen verify checks and `sql/physical.sql`; the qualify
run, source and case modules; the expression checker (slot typing, dimension rules), the implicit
solver and the compile cache; the survey records for CoolProp, AGA8, pycalphad, PHREEQC, RMG
database and the ThermoML schema in detail, and the `validity` and `precision` fields of all 37.

**Not examined.** Acquisition code beyond its contract page; the staging framework and the
CoolProp reader internals; side environments; most of `declaration/resolve.py` and
`expression/evaluate.py` line by line; the tests (read for shape only); 31 of the 37 survey
records beyond the two fields above; the production kernel beyond the cited sections. No build,
test, generator or pipeline stage was run (instructed), so **no failure count was measured
against the zero baseline** (slot 10 lists the commands that were run).

### Answers to the scoped questions

1. **Model adequacy.** The model holds the realised slice well and most of the concepts the six
   test cases need. It is not yet adequate for its declared scope: five concrete structures that
   the surveys document cannot be said ([F02](#f02)–[F05](#f05), [F09](#f09), [F13](#f13)), and
   one rule refuses a valid state ([F07](#f07)). Slot 2 gives the case-by-case result.
2. **Authority and derivation.** The derivation chain from declaration to DDL, `meta` rows,
   Arrow schemas and the fingerprint is single-sourced (strength). Second definitions exist
   inside the model itself and between the declaration and pipeline code ([F06](#f06),
   [F11](#f11), [F15](#f15), [F16](#f16)). One transformation lacks a contract ([F03](#f03)) and
   reuse keys are incomplete in two places ([F01](#f01), [F08](#f08)).
3. **Separation and composition.** Stage boundaries are real and ordinary extensions are local
   (slot 4). The forms mechanism is proportionate to the surveyed variation; it is incomplete
   rather than speculative ([F02](#f02), [F20](#f20)).
4. **Constraints, validity and truthful claims.** Value states and unit assumptions are enforced
   well. Conventions and envelopes are recorded but no path requires or consults them
   ([F06](#f06), [F09](#f09)); relations cannot carry invariants ([F10](#f10)). Qualification
   claims only what it establishes, except through one stale-reuse path ([F01](#f01)).
5. **Library leverage.** Parsing, units, symbolic mathematics, root finding, structure keys,
   Arrow and bulk loading are library-owned. Three bespoke generic components have no recorded
   consideration ([F19](#f19)).
6. **Fitness for production.** The shape maps onto production concepts for kinds, relations,
   absence policies and datasets. Adoption is not yet a projection: five declared kernel gaps
   remain, plus identity, physical typing, envelope binding and role obligations ([F12](#f12),
   [F15](#f15), [F21](#f21)).

## 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State / effect owner | Local test setup |
|---|---|---|---|---|---|
| Declaration and loader (`model/`, `forms/`, `declaration/`) | What the model is; which declarations are refused | Exposes a resolved `Declaration`; consumes TOML | Depends on nothing of the pipeline, except a late import of the expression checker and of the generator's identifier-length diagnostics | None; pure | Fixture directories; no database |
| Generator (`generate/`) | Projection rules to PostgreSQL; the `meta` reification; the fingerprint | Consumes the declaration; exposes a deterministic file tree, table descriptions and the fingerprint | Mechanism depends on meaning | None; pure, `--check` compares trees | No database for generation; a disposable database for DDL tests |
| Expression engine (`expression/`) | Grammar, name resolution, dimensional closure, evaluation, implicit solving | Consumes the declaration and a `ParameterSource`; exposes `bind` and the evaluation hash | Depends on the declaration; SymPy, NumPy, SciPy and pint inside | A compile cache owned by the caller | `InMemorySource`; no database |
| Acquisition (`acquire/`, `sources/`, `sources.lock`) | What was acquired and that it is untouched | Exposes the raw store and lock | The only networked stage | Raw store, lock | Local fixture repositories and files |
| Staging and readers (`staging/`, `readers/<id>`) | Source format knowledge | Consumes raw store and lock; exposes staged Parquet with a manifest and field metadata | Reader depends on the staging contract only | Staged store | Fixture payloads |
| Resolution (`resolve/`, `identity/decisions.toml`) | The identity rules and their order | Consumes phase-1 claims and decisions; exposes `_resolution/` Parquet, report and manifest | Reads the mapping's claim layout (`mapping.claims`) | `_resolution/` directory | Pure function over claim records |
| Mapping framework and mappings (`mapping/`, `mappings/<id>/`) | Source-to-canonical meaning, unit assumptions, loss | Consumes staged tables, the declaration and the resolution; exposes canonical Parquet and coverage | Depends on the writer and staging manifests | `canonical/<id>/` directory | Staged fixtures; no database |
| Canonical writer (`canonical/`) | Identifiers, provenance rows, unit conversion, orientation, validation | The only way rows are emitted | Depends on the declaration, the generator's table descriptions, and the acquisition and staging manifests for carrier facts | In-memory tables until written | No database |
| Build (`build/`) | The union across sources and the single transaction | Consumes canonical directories by manifest; exposes the database | Depends on the generator and the canonical store | The canonical schemas of `pse_thermo` | Disposable database |
| Verify (`verify/`, `sql/verify/`) | The definition of each verify invariant | Consumes the database and the declaration's `requires` | One check file per declared invariant, both directions enforced | None; every check is rolled back | Disposable database |
| Qualify (`qualify/`, `qualification/`, `oracles/`) | Subject and grid selection, comparison, run records | Consumes the database, the evaluator and the harness protocol | Writes run Parquet through the writer and loads it into the live database with the build's loader | `_qualification/<case>/`, and the `qual` rows of that case | Disposable database and a stub harness |

The composition root is the CLI (`cli.py`), which only wires stage commands. Stages communicate
through directories with manifests, not through each other's objects; the cross-stage imports
that exist are of layout constants and manifest types (*Interface-checked*: import survey with
`rg`). Two edges are worth naming: the loader refuses a declaration whose projected PostgreSQL
identifiers exceed 63 bytes, so a store limit is part of declaration validity; and qualification
writes into the database that `tk build` otherwise owns (declared in `pipeline.md` section 5.4).

### Model adequacy against representative cases (AP-04, first half)

| Case | Concepts that hold it | Identity | Invariants and where enforced | Lost or cannot be said |
|---|---|---|---|---|
| **C0** CoolProp saturation ancillary (realised) | `source_entity`, `identity_assertion`, `species`, `parameterization`, slot group `pure` with family `term`, `envelope`, `fit` | Set: (parameterization, slot group, subject key, occurrence); species: InChIKey or provisional key | Units stated or assumed with recorded loss (spec validation); finite values and family index minimum (DDL); contiguous indices, record origin, producing derivation (verify) | Declared in `mapping.toml`: no `auxiliary_of` and no fit input until the equation of state is mapped. Undeclared: pseudo-pure blends and Air are provisional *species* ([F04](#f04)) |
| **C1** GERG-2008 multifluid mixture (CoolProp, teqp, AGA8 surveys) | Pure residual forms with an additive term slot; a reducing-function pair group with `reciprocal` over the two beta slots; a departure pair group; `model_assembly` | Pair set keyed on the canonical-ordered pair | Role order and forbidden diagonal (DDL); nested-set contract (verify) | One departure function shared by eight pairs, and exponent layouts shared by several fluids, cannot be stored once and referenced ([F02](#f02)). The per-pair choice of departure form has no relation. Stored beta values are inverted according to identifier order, with no record of which sets were inverted ([F03](#f03)). Composition ranges cannot be stated ([F09](#f09)) |
| **C2** CALPHAD sublattice phase (pycalphad, Thermochimica) | `chemical_system`, `phase_definition`, `site_class`, `site_occupant`, `constituent_array` with positioned members, sets with `piece` families, `energy_reference` and `element_reference` | Array: (phase, canonical key with species ascending by identifier) | Piece non-overlap (DDL exclusion, writer, evaluator); site ratio (verify) | A TDB parameter is an expression in temperature that references named functions; there is no value shape for a combination of shared functions ([F02](#f02)). Reordering an array changes the sign of odd orders and the meaning of a ternary index, which the role-swap transposition cannot apply ([F03](#f03)). Order–disorder pairing of two phases, the per-formula-unit against per-atom basis and the consumer's extrapolation policy have no home ([F14](#f14)) |
| **C3** PHREEQC aqueous database | `system_member` roles, `reaction` with participants and standard states, `system_reaction.defines`, reaction-subject forms, `selection_policy` recorded for the carrier, `site_class` for exchangers and surfaces | Reaction: canonical key over participants and coefficients | Conservation over quantities that have composition entries (verify) | Solvent and solute standard states of one liquid phase cannot be held in one convention set ([F05](#f05)). Valence-state totals and alkalinity have no counterpart. The rule that refuses an equilibrium-constant set for a participant without a standard state is prose ([F10](#f10)). The canonical encoding of fractional coefficients is undefined ([F16](#f16)) |
| **C4** RMG group trees | `group_scheme` with `dataless_rule`, `group` with `parent` and `position`, per-group sets, `redirect` state, `slot_uncertainty`, `fit` with covariance | Group: (scheme, label) | Parent acyclic and in scheme (verify); redirect restricted to the same slot group (DDL) | A pointer is per node; `redirect` is per slot and unavailable to family slots. The reader follows at most 8 redirects where RMG follows 100. Union nodes, radical and pair roles are absent. The covariance is typed dimensionless ([F12](#f12)). Patterns are opaque by declared scope |
| **C5** ThermoML dataset with typed uncertainty | `dataset`, `dataset_component` with `sample`, `dataset_phase`, `dataset_column`, `data_point`, `datum` | Dataset: (carrier, local key); datum: (point, column) | `constraint_has_constant` (load); nothing on `datum` | One uncertainty per value where ThermoML carries several assessments, combined, repeatability and device entries; no asymmetric uncertainty; no side of a censored limit; no reference-state presentation on a column ([F13](#f13)). State, value and uncertainty combinations are unchecked ([F10](#f10)) |
| **C6** One published table in three carriers | Three parameterizations, `import_record`, `record_origin`, `attribution`, `equivalence_assessment` with a `conversion` | Parameterization keyed per carrier and pin; publication keyed by citation key | Record origin and carrier rights (verify) | Nothing at record level. Unrealised: no stage emits attributions or assessments (*Measured*: zero rows; *Interface-checked*: no emitter). Publications have no identity resolution across carriers ([F16](#f16)) |

**Authoritative realisation (AP-04, second half).** DDL, `meta` rows, Arrow schemas and the
fingerprint derive from the declaration through one generator; the writer validates against the
same declaration; `transposition.py` is shared by the loader, writer and evaluator; verify
refuses a declared invariant without a check file and a check file without a declared invariant.
These are real strengths. The exceptions are in slot 7: duplicated facts inside the model
([F06](#f06), [F16](#f16)), the undeclared shape contract between the declaration and pipeline
code ([F11](#f11)) and rules that live in prose and SQL ([F10](#f10), [F15](#f15)).

## 3. Contracts, authority and constraints

| Phenomenon, concept or operation | Semantic scope, owner and update path | Consumer obligations / invariant | Enforcement and failure | Derived representations / evolution |
|---|---|---|---|---|
| The model (kinds, relations, forms) | `model/*.toml`, `forms/*.toml`; edit and regenerate | Names unique; every invariant names an enforcement point | Loader refuses with coded, located diagnostics | DDL, `meta`, Arrow schemas, fingerprint; rebuilt, never migrated |
| Identity of records | `identity.py`: version-5 UUID over declared identity values | Identity values are never floats | Writer computes; a second row with one identifier and other content is a `CompetingAssertion` | `canonical_key` text keys are computed by stage code from prose rules ([F16](#f16)) |
| Value presence | `value_state` and relation `absence` | Missing is the absence of a set; a stored zero is known | Stateful slot check (DDL); required slot missing holds the block | A known zero of an `absolute` type is refused ([F07](#f07)) |
| Transposition | Declared per slot group or relation | One canonical orientation is stored | Role-order check (DDL); writer reorients and rewrites values | No statement of precision or of which sets were rewritten ([F03](#f03)) |
| Unit conversion at mapping | `mapping.toml` field rules; pint registry | Source unit stated, or the assumption recorded as loss | Spec validation refuses; factor recorded in `qual.mapping_rule` | pint version is not a reuse input ([F08](#f08)) |
| Identity resolution | `pipeline.md` section 1; `resolve/engine.py`; `identity/decisions.toml` | First applicable rule decides; ambiguity is preserved | Pure function; ambiguous subjects hold their rows | Targets are species only ([F04](#f04)) |
| Build | `pipeline.md` section 3 | Same key and same content is one row; same key and other content refuses | One transaction; commit validates deferred constraints | Qualification outputs are admitted on a weaker key than they were made with ([F01](#f01)) |
| Verify | Declared `requires` plus structural checks | One named query per invariant returning violators | Non-zero exit on violations, missing checks or orphan checks | Relation invariants cannot be declared ([F10](#f10)) |
| Qualification | Case files, harness protocol, `qual.form_qualification` | Comparison basis recorded; status derived from runs | Blocked runs are recorded and never reused | Basis ranking is stated; the first case is source-library only |
| Conventions | `convention_set` on a parameterization | None declared | None: no invariant and no consumer ([F06](#f06)) | — |

**Physical-semantics table (profile).**

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| Slot value | Quantity type; coherent SI storage unit; conversion at mapping | By dimension only (molar against mass) | Optional `convention_set` on the parameterization; never checked | `envelope` rows per set; not enforced | Form declaration and `mapping.toml` |
| `Real` value (envelope bound, datum, tabulated point) | Unit taken through `unit_from` | Observable facet `basis` | Observable facet `relation`; column `standard_state` | — | Declaration; writer refuses an unresolved unit path |
| Composition argument of a contract | `Fraction`, dimensionless | `basis` text on the argument, unchecked ([F12](#f12)) | — | None expressible per component ([F09](#f09)) | Contract declaration |
| Standard-state outputs (`h`, `s`, `cp`) | `MolarEnergy`, `MolarEntropy` | Molar | "Relative to the energy reference of its convention set", which may be absent ([F06](#f06)) | Piece intervals only | Form and parameterization |
| Gas constant | `GasConstant` | — | Stated twice: slot `constants.R` and `convention_set.gas_constant` ([F06](#f06)) | — | Two declarations |
| Fit covariance | Declared dimensionless; documented as dimensioned ([F12](#f12)) | — | — | — | `provenance.toml` |
| Qualification tolerance | Relative, dimensionless; absolute in the observable's unit | — | Comparison basis recorded | Points taken inside the set's envelope | Case file |

**Well-posedness statement (profile).** No flowsheet problem is formulated, so variable roles,
degrees of freedom and structural analysis do not apply. The one place equations are solved is
an implicit block of a form: the loader checks that expanded residuals number the unknown
elements and refuses otherwise, naming the form and block (*Interface-checked*, `expressions.md`
section 4). That is a count check, not a structural rank check, which is proportionate here.

## 4. Change scenarios and composition

| Scenario / stimulus, kind and conditions | Expected response and boundary | Edit / composition path | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|
| <a id="s01"></a>**S01** New source in an existing format family (instance; teqp fluids in CoolProp format) | One manifest, one mapping | `sources/teqp.toml`, `mappings/teqp/mapping.toml` and `mapping.py`; the reader is named by the manifest | Local. No framework or declaration edit | *Interface-checked*; the plan's row for this change holds |
| <a id="s02"></a>**S02** New form under an existing contract (instance; Antoine under `pure_vapor_pressure`) | One form declaration and its rows | `forms/*.toml`; regenerate | Local, provided its slots are numeric ([F20](#f20)) | *Interface-checked*; exercised by four declared forms |
| <a id="s03"></a>**S03** New kind or relation in the model (domain concept; an order–disorder pairing of phases) | One declaration, one emitter call in a mapping | `model/systems.toml`; regenerate; `emit.relation` | Local when the pipeline does not know the kind. A change to any of about two dozen pipeline-known kinds also edits stage code and fails at run time, not at load ([F11](#f11)) | *Interface-checked* |
| <a id="s04"></a>**S04** New verify invariant (policy) | One `requires` and one check file | Declaration plus `sql/verify/<kind>.<name>.sql` | Local for kinds. Not expressible for a relation ([F10](#f10)) | *Interface-checked* |
| <a id="s05"></a>**S05** New oracle library (mechanism; teqp) | One harness script, one case | `oracles/teqp.py`, `qualification/*.toml`, an environment | Local; protocol is versioned and refuses unsupported calls and units | *Interface-checked*; one harness exists |
| <a id="s06"></a>**S06** A departure function shared by eight pairs (domain concept; C1) | One stored set, eight references | None available | Not expressible; needs a new value shape across loader, generator, writer, source and evaluator ([F02](#f02)). A wide change is legitimate here: it is a new core concept | *Interface-checked*; three surveys name the need |
| <a id="s07"></a>**S07** A curated identity decision lands for one fluid (binding) | The fluid's records attach to the canonical species | `identity/decisions.toml`; resolve, map, build | The species identifier changes, so the stored orientation and therefore the stored numbers of its reciprocal, parity and linear pair sets may change ([F03](#f03)) | *Interface-checked*; predicted |
| <a id="s08"></a>**S08** A mapping fix changes a stored value under the same declaration and pin (failure journey) | Earlier qualification of those sets is no longer presented as current | `tk map`, `tk build` | The earlier passing run is loaded and listed by `qual.form_qualification` ([F01](#f01)) | *Interface-checked*; predicted, not executed |
| <a id="s09"></a>**S09** Upgrade pint, RDKit, SymPy or an oracle script (mechanism upgrade; PSE-S06) | Affected stages report stale | Lock change or script edit | Staging notices reader source changes; mapping, resolution and qualification report `current` ([F08](#f08)) | *Interface-checked* |
| <a id="s10"></a>**S10** Replace the reference evaluator by a production-language backend (mechanism substitution; PSE-S02) | Typed expression tree and `ParameterSource` stay; evaluator changes | New backend behind the same tree | Feasible: the evaluation hash is computed from the canonical tree, not from SymPy. The "SymPy canonical order" determinism class is evaluator-specific and must be restated | *Proposed* |
| <a id="s11"></a>**S11** Change the store (representation; PSE-S04) | Generator back end and check queries change; meaning does not | `generate/`, `sql/verify/` | Seventeen checks are PostgreSQL SQL; the loader's identifier-length refusal would need to move to the projection | *Proposed* |

**Simulator journeys (profile).** *Add a property model*: S02. *Replace an implementation*: S05,
S10. *Test admission or policy locally*: the loader, writer, resolver, transposition and
evaluator are all exercised without a database (slot 2). *Out-of-envelope property evaluation*:
the evaluator has no knowledge of envelopes ([F09](#f09)). *Boundary round trip*: units are
carried faithfully; basis, conventions and identity class are not yet ([F04](#f04), [F06](#f06),
[F12](#f12)). The remaining journeys (edit and re-solve, studies, recycles, dynamics, infeasible
problems) do not apply to this scope.

**Is the forms mechanism the least machinery?** Each construct was checked against survey
evidence. Contracts, slot groups with role-tuple subjects, families, sub-form slots, nested sets
and implicit blocks each have several surveyed cases and are exercised by tests or the slice.
Transposition rules `symmetric`, `ordered`, `reciprocal`, `parity` and `linear` each have a
surveyed source; `permutation_group` has credible cases (ternary interaction parameters,
equivalent sublattices) but no declared use yet. `observable_from_set` serves one surveyed case
and is small. Nothing was found that is speculative. Two things are incomplete: `per = "subject"`
sub-form slots have no stored relation for the per-subject choice, and tabulated, enum and
reference slots cannot be read by an expression ([F02](#f02), [F20](#f20)). Whether any surveyed
form needs `integral` was not established.

## 5. Mechanisms and execution, where material

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|
| Read | Reader to staged Parquet with a manifest | Lock tree hash, reader name, version and source files, environment name, format number | Writes a staged directory | Key omits the side environment's lock | *Interface-checked* |
| Map, phase 1 and 2 | Declared rules plus structural code, through the writer | Staged key, hash of `mapping.toml` and `mapping.py`, format number, fingerprint; phase 2 adds identity and resolution hashes | Directory written beside and renamed | Key omits writer, identity, transposition and conversion code and the pint version ([F08](#f08)) | *Interface-checked* |
| Resolve | Pure function over claims and decisions | Phase-1 manifest hashes, decisions hash, fingerprint, format number | Byte-identical output for equal inputs (reported) | Key omits the RDKit version that computes structure keys ([F08](#f08)) | *Interface-checked* |
| Build | Union, then one transaction | Canonical directories by manifest | All or nothing; previous schemas kept on failure | Qualification outputs admitted on fingerprint and set identifiers only ([F01](#f01)) | *Interface-checked* |
| Verify | Named queries | Database with matching fingerprint | Read-only; each check rolled back | Tolerance `1e-9` is written inside two checks, not declared | *Interface-checked* |
| Qualify | Evaluate, ask the harness, compare, record | Fingerprint, evaluation hash, case hash, library version, subjects, digest of stored values, format number | Parquet, then a scoped live load | Key omits the harness script, the evaluator's libraries and the side environment lock ([F08](#f08)) | *Measured* for the one run |

**Numerical stage columns (profile), reference evaluator.**

| Formulation policy | Derivative source and order | Scaling | Problem class · solver capability | Status to outcome | Tolerances · post-solve check |
|---|---|---|---|---|---|
| Conditions on stored values decided at bind; conditions on arguments kept as piecewise; `at` selects half-open pieces and refuses outside them; no smoothing | Symbolic (SymPy), any order; through implicit blocks by the implicit-function theorem | Residuals compared with the sum of the magnitudes of their own terms | One bounded unknown: polynomial roots or a scan with bracketing; otherwise a square system from a start, with bounds | Any failure is a `SolveFailure` naming form, block and point; never a NaN. SciPy's own status is not read; an independent residual and step test decides | Residual 1e-8 relative to term size and Newton step 1e-10, fixed in `implicit.py` and not stated in `expressions.md`; the case tolerance is declared with its reasoning. Conforming runs agree to rounding, not bitwise (stated) |

The solver adds its own Newton polishing after the SciPy call. Its purpose is evident
(working-accuracy roots for comparisons at 1e-10) but not stated where the next reader will look
([F19](#f19)).

## 6. Architectural assessment and gates

| Foundation | Scenario and evidence | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | S01–S05 stay with their owners; stages exchange manifests and Parquet. The loader's dependence on a store limit and the qualification write into the built database are declared and bounded | **Satisfied** | None required; see [F11](#f11) for the undeclared coupling, judged under AP-05 |
| AP-02 Stable contracts | `ParameterSource` has two implementations; the harness protocol is versioned; the evaluation hash is independent of the evaluator (S10). Capability limits are explicit: a non-numeric slot in an expression is refused at load | **Satisfied** | None |
| AP-03 Composition | S01–S05: an ordinary extension is a declaration, a mapping or a script; no workflow is copied. The composition root is the CLI | **Satisfied** | None |
| AP-04 Domain model and semantic authority | Adequacy: C1–C5 each contain something the declared model cannot say, within the plan's own hard cases ([F02](#f02)–[F05](#f05), [F09](#f09), [F13](#f13)), and one valid state is refused ([F07](#f07)). Authority: duplicated facts and an undeclared shape contract ([F06](#f06), [F11](#f11), [F16](#f16)) | **Violated** | Revise the named concepts before the hard-case fixtures are written; no deferral is available for supported scope |
| AP-05 Explicit structure | Strong where used: every kind invariant has an enforcement point and verify checks both directions. But relations cannot declare invariants, several stated rules are prose, conventions and envelopes bind nothing, and pipeline code relies on unvalidated kind shapes ([F06](#f06), [F09](#f09)–[F11](#f11)) | **Violated** | [F10](#f10) and [F11](#f11) first; they are small |
| AP-06 Local reasoning and testability | Loader, writer, resolver, transposition and evaluator run without a database; database tests use disposable databases (slot 2) | **Satisfied** (*Interface-checked*; tests not run) | None |

| Gate | Verdict | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | **Fail** (narrow) | The gas constant of a parameterization has two writable homes with no invariant; standard pressure three; a set's origin role two ([F06](#f06), [F16](#f16)) | One home each, or a declared tying invariant |
| G2 Semantic fidelity | **Fail** (narrow) | Composition `basis` is unchecked text; the fit covariance is typed dimensionless; mixtures are stored as species ([F04](#f04), [F12](#f12)). No numeric sentinel was found | [F04](#f04), [F12](#f12) |
| G3 Validity | **Fail** (narrow) | `datum` admits a known state without a value and an uncertainty kind without a magnitude; stated rules without a declared requirement ([F10](#f10)) | [F10](#f10) |
| G4 Hidden behaviour | **Pass** | Loading, generation and resolution are pure; verify rolls back; the one extra write path is declared | — |
| G5 Consistency and recovery | **Pass** at *Interface-checked* | Build is one transaction; canonical directories are installed by rename; blocked runs are recorded and never reused. Acquisition crash paths were not examined | — |
| G6 Transformation and reuse | **Fail** | A stale qualification pass can be loaded ([F01](#f01)); reuse keys omit providers and framework code ([F08](#f08)); the orientation rewrite has no stated equivalence ([F03](#f03)) | [F01](#f01), [F03](#f03), [F08](#f08) |
| G7 Truthful capability claims | **Fail** (narrow) | The plan states that reuse is keyed on complete inputs, and that the changes the surveys require are applied to `model/` ([F08](#f08), [F14](#f14)). Qualification itself is honestly scoped: the basis is recorded and ranked, and status is derived | Correct the two statements or make them true |
| G8 Library leverage | **Unresolved** | Libraries own the generic work in most places (slot 8). For the declaration language and generator, stage orchestration and reuse, and acquisition, no consideration is recorded; this review judges the first and third defensible and the second open ([F19](#f19)) | Record the three decisions; decide orchestration together with [F08](#f08) |
| G9 Architectural fitness | **Fail** | AP-04 and AP-05 are violated | As above |
| PS-G1 Physical consistency | **Fail** (narrow, largely prospective) | A composition in the wrong basis can reach a sub-form; convention sets are never required or compared; no evaluation path consults an envelope ([F06](#f06), [F09](#f09), [F12](#f12)). Dimensions and units are sound | [F06](#f06), [F09](#f09), [F12](#f12) |
| PS-G2 Well-posedness | **Not applicable** | No flowsheet or case is formulated; implicit blocks have a count check at load | — |
| PS-G3 Numerical integrity | **Pass**, scoped to the reference evaluator | Typed refusals, an independent post-solve test, symbolic derivatives, stated determinism class (slot 5). *Interface-checked*; one explicit form *Measured* | State the fixed solver tolerances in `expressions.md` |

## 7. Findings

Severity: **High** is a MUST gap with a concrete wrong or unrepresentable outcome in declared
scope. **Medium** is a MUST gap with a bounded consequence, or amplified change. **Low** is a
SHOULD deviation or a local defect. Class: *design defect* (the declared design is wrong or
insufficient), *implementation gap* (the design is right and the code does not yet realise it),
*open question* (a decision is missing).

| ID | Severity · class | Finding | Principles / gate / scenario |
|---|---|---|---|
| [F01](#f01) | High · implementation gap | `tk build` loads a stored qualification pass without checking the values it was made against | DP-09, DP-22 · G6, G7 · S08 |
| [F02](#f02) | High · design defect | A parameter set shared by several parents cannot be stored once, referenced or evaluated | AP-04, DP-06 · G9 · S06, C1, C2 |
| [F03](#f03) | High · design defect | Canonical orientation rewrites asserted values according to identifier order, without record or stated precision, and cannot reach order inside a subject | DP-05, DP-08, DP-11, AP-04 · G6 · S07, C1, C2 |
| [F04](#f04) | High · design defect | Identity resolution can only produce species; mixtures, materials and pseudo-components become provisional species | AP-04, DP-02 · G2 · C0 |
| [F05](#f05) | High · design defect | Standard states are keyed by aggregation, so one convention set cannot hold the solvent and solute states of one liquid | AP-04, PS-01 · PS-G1 · C3 |
| [F06](#f06) | Medium · design defect | Conventions bind nothing, and the gas constant and standard pressure have several homes | DP-01, DP-03, PS-01, PS-02 · G1, PS-G1 |
| [F07](#f07) | Medium · design defect | A known zero of an `absolute` quantity is refused | DP-02, DP-03 · G2 |
| [F08](#f08) | Medium · implementation gap | Reuse keys omit provider versions, framework code, the harness script and environment locks | DP-09, DP-15 · G6, G7 · S09 |
| [F09](#f09) | Medium · design defect | The envelope is one interval per set and observable, binds to no argument and is consulted by no evaluation | AP-04, PS-02, DP-02 · PS-G1 |
| [F10](#f10) | Medium · design defect | Relations cannot declare invariants, and several stated rules have no declared requirement | DP-03, AP-05 · G3 · S04 |
| [F11](#f11) | Medium · design defect | Pipeline code depends on about two dozen declared kinds by literal name and attribute; the framework binding declares five | AP-05, AP-02, DP-01 · S03 |
| [F12](#f12) | Medium · design defect | Typed holes: covariance declared dimensionless, composition basis unchecked, expression checking by dimension only | DP-02, PS-01 · G2, PS-G1 |
| [F13](#f13) | Medium · design defect | The evidence module holds one uncertainty per value and no side for a censored limit | AP-04, DP-02 · C5 |
| [F14](#f14) | Medium · implementation gap | Survey losses are not dispositioned, and the checkpoint claims they are applied | DP-22, DP-23 · G7 |
| [F15](#f15) | Low · design defect | Role obligations live in member prose and a hard-coded SQL list | DP-01, DP-03 |
| [F16](#f16) | Low · design defect | Identity text that restates typed references or follows prose rules has no tying invariant; publications have no resolution | DP-01, DP-04 · G1 · C3, C6 |
| [F17](#f17) | Low · implementation gap | Held-row and blocked-run reasons are prose | DP-21 |
| [F18](#f18) | Low · implementation gap | Declared field rules are recorded whether or not the mapping applied them | DP-23 · G7 |
| [F19](#f19) | Low · open question | No recorded library consideration for three bespoke generic components | DP-13, DP-16 · G8 |
| [F20](#f20) | Low · open question | Tabulated, enum and reference slots cannot be read by an expression | AP-04, DP-22 · S02 |
| [F21](#f21) | Low · open question | Adoption is not yet a projection: construct identity, physical typing and selection keys differ from production | DP-04, DP-24 |

<a id="f01"></a>
### F01 — A stale qualification pass can be loaded by the build

- **Evidence** (*Interface-checked*). `build/inputs.py` `discover_qualification` keeps an output
  when its manifest records the current declaration fingerprint and every evaluated parameter-set
  identifier is present in the build. `qualify/run.py` `stored_digest` exists because "an
  identifier does not change with a value", and the digest is part of the qualification reuse key,
  but the build does not compare it. `qual.form_qualification` filters on outcome and expression
  hash only.
- **Consequence.** Correct a unit or a structure rule in a mapping, run `tk map` and `tk build`,
  and the earlier run is loaded as `passed` and listed as current for sets whose values were never
  evaluated. With basis `source_library`, mapping fidelity is exactly what the run claims.
- **Correction.** Make substitutability decidable before load: record in the qualification
  manifest a content hash of the canonical rows of each evaluated set and envelope, computed from
  the same Parquet the build reads, and skip an output whose hashes differ, with the reason. Or
  recompute the digest inside the build transaction and delete non-matching runs before commit.
- **Verification.** A test that changes one mapped value, rebuilds without `tk qualify`, and
  finds the run skipped and absent from `qual.form_qualification`.

<a id="f02"></a>
### F02 — Shared parameter sets cannot be represented

- **Evidence** (*Interface-checked*). A nested set's identity is derived from its holder
  (`CanonicalWriter._nested`: `subject_key` is the encoding of the parent set, slot and index), so
  it has exactly one parent. A reference-typed slot may name kind `parameter_set`, but it is not
  contract-checked and the expression checker refuses it ("holds a reference, which an expression
  cannot use as a number"). `redirect` points only at the same slot of another set of the same
  group. `assembly_choice` is keyed (assembly, path, ordinal) and `entity_model` by one entity, so
  a per-pair choice of sub-form is stored nowhere; `DatabaseSource` takes sub-form choices from the
  case file. The surveys state the need three times: CoolProp ("sharing one function among pairs
  is a many-to-one reference that the model must express"), AGA8 ("model 10 as one shared
  parameter_set referenced from 8 pairs"; exponent layouts shared by up to fourteen fluids) and
  pycalphad ("the model has no free-standing named function"). None is in `alignment-notes.md`.
- **Consequence.** GERG-2008 and every TDB either duplicate the shared coefficients per parent,
  which loses the fact that they are one published object, or cannot be loaded. A TDB parameter,
  which is a sum of literal terms and multiples of named functions, has no value shape at all.
- **Correction.** Add a slot shape "set reference by contract": the value is an independently
  identified top-level set of a form implementing the named contract, checked like a nested set
  in the writer and in verify, and callable in expressions like one. Add a kind for a named model
  component (a function name scoped to a parameterization or chemical system) to serve as its
  subject. Add a relation for the per-subject choice of a sub-form slot. For TDB, declare a form
  whose families are literal terms and (multiplier, reference) rows; an expression outside that
  basis holds its row with a typed reason.
- **Verification.** Fixtures for cases (a) and (e): one generalised departure set referenced by
  eight pair sets and evaluated through them; one `GHSER` function referenced by two parameters.

<a id="f03"></a>
### F03 — Canonical orientation rewrites asserted values

- **Evidence** (*Implemented*). `transposition.canonical_orientation` orders two subjects by
  their identifiers. When the asserted order differs, `CanonicalWriter._swap` stores the
  reciprocal of `reciprocal` slots and the matrix image of `linear` slots, and `_family_rows`
  negates odd `parity` rows. A reciprocal slot holding zero is refused. No column records that a
  set was rewritten (*Interface-checked*: no orientation column in `schema.sql`). Identifiers
  derive from `canonical_key`, which changes when resolution status changes. Transposition is
  declared over two roles of one slot group; a `constituent_array` is one subject whose key
  orders species by identifier, and its documentation defers to "the contribution form's
  transposition rule", which cannot act inside a subject. The plan places the rule in `api`
  views; `meta-model.md` section 4.3 places it in the writer and evaluator.
- **Consequence.** (1) A published beta of 1.004 is stored as its double-precision reciprocal
  for roughly half of all pairs; the published number is recoverable only from the `src_` row,
  and the plan's round-trip check cannot hold bitwise, since inverting twice is not exact.
  (2) A curated identity decision for one fluid can change the stored numbers of its pair sets
  (S07). (3) CALPHAD odd-order signs and ternary index meaning must be re-derived by mapping
  code, against the rule that a mapping holds no arithmetic. (4) No contract states what the
  rewrite preserves.
- **Correction.** Store values as asserted. Keep canonical-ordered subject columns for
  uniqueness and add a declared orientation state on the set ("as stored" or "asserted for the
  swapped order"); the evaluator and the read views apply the rule once, on reading. For
  constituent arrays, keep the asserted order as data and declare reordering as a permutation
  acting on the order index of the contribution form. If canonical rewriting is kept instead,
  declare it as a transformation under DP-08 with its precision and record it per set.
- **Verification.** A swapped reciprocal pair returns the source number bitwise on reading in
  the source orientation; changing an identity decision leaves every stored slot value unchanged.

<a id="f04"></a>
### F04 — Resolution yields only species

- **Evidence.** *Measured*: `Air`, `R404A`, `R407C`, `R410A`, `R507A` and `SES36` are rows of
  `tk.species` with `provisional = true`; `tk.defined_mixture` is empty. *Interface-checked*:
  `pipeline.md` section 1 rule 5 speaks of a provisional `material_entity`, section 1.2 of "its
  species"; an `identify` decision resolves "to the species with that canonical key"; rules 2 to
  4 are structural, registry and formula rules for chemical species.
- **Consequence.** Five of six refinements of `material_entity` have no resolution route. The
  154 predefined mixtures, ISODB adsorbents, petroleum cuts and polymers either load as species,
  with a default charge of zero asserted for a blend, or need paths outside the framework, and
  cannot be joined across carriers.
- **Correction.** A scope declares the class of its entities; each class has its own rules (a
  defined mixture by its resolved components and definition, a material by registry key, a
  pseudo-component by derivation and index). A provisional entity of undetermined class gets an
  explicit refinement, not `species`. Decisions gain a class.
- **Verification.** CoolProp `Air` resolves to a `defined_mixture` whose components are resolved
  species; no provisional species remains for a pseudo-pure fluid.

<a id="f05"></a>
### F05 — Standard states keyed by aggregation

- **Evidence** (*Interface-checked*). `convention_standard_state` is keyed (convention set,
  aggregation) and says a set "that needs more than one per aggregation declares separate
  convention sets"; `aggregation` deliberately has no aqueous member; a parameterization has one
  convention set. The PHREEQC survey records solutes on the molal infinite-dilution scale and
  water as an activity in one database.
- **Consequence.** A PHREEQC, SUPCRT or ThermoFun parameterization cannot state the standard
  states its species-subject sets assume. The per-participant standard state on a reaction
  covers equilibrium constants only.
- **Correction.** Key the relation by the role a species form plays in the chemical system
  (or attach the standard state to `system_member`), not by aggregation alone.
- **Verification.** A fixture for case (f) states pure-liquid water and one-molal solutes in one
  convention set and both are read for one reaction.

<a id="f06"></a>
### F06 — Conventions bind nothing and repeat facts

- **Evidence** (*Interface-checked*). `parameterization.convention_set` is optional;
  `mapping/spec.py` requires only that a mapping state a set or a reason for none. The NASA forms
  carry `constants.R` while `convention_set.gas_constant` holds the same fact; a standard
  pressure can be stated on `convention_set`, `standard_state` and `energy_reference`. The
  standard-state contract says its outputs are relative to the convention set's energy reference,
  and nothing requires one. `DatabaseSource` reads an ordered list of parameterizations and
  compares no conventions.
- **Consequence.** A mapping can write two gas constants for one parameterization. An enthalpy
  can be stored and evaluated against an unstated datum. Sets built on different conventions can
  be combined in one evaluation.
- **Correction.** One home for each convention fact (the form reads the gas constant from the
  convention set through a declared convention input, or the convention set does not carry it).
  A form or contract declares the conventions it needs; a verify invariant requires them on any
  parameterization holding its sets; the evaluator and the snapshot refuse to combine
  parameterizations whose convention sets differ without a declared conversion.
- **Verification.** A NASA parameterization without an energy reference fails verify; two
  parameterizations with different gas constants are refused in one evaluation.

<a id="f07"></a>
### F07 — A known zero of an absolute quantity is refused

- **Evidence** (*Implemented*). `generate/plan.py` emits `VALUE > 0` for `scale = "absolute"`;
  `canonical/values.py` and the loader refuse `number <= 0`. `physical.toml` defines absolute as
  "never negative", and the value-state rule says a known zero is known.
- **Consequence.** A zero dipole moment, a piece starting at 0 K, the zero mass of a vacancy or
  electron (pycalphad survey) and a zero fraction are held as invalid.
- **Correction.** `absolute` means non-negative. Where strict positivity is physical, declare it
  on the slot or as a separate scale.
- **Verification.** A set with a zero dipole moment loads; a negative one is refused.

<a id="f08"></a>
### F08 — Reuse keys are incomplete

- **Evidence** (*Interface-checked*). Mapping keys hash the staged key, `mapping.toml`,
  `mapping.py`, the fingerprint and a hand-maintained format number (`mapping/runner.py`).
  Resolution hashes claims, decisions, fingerprint and a format number (`resolve/command.py`);
  `resolve/structure.py` computes InChIKeys with RDKit. Qualification hashes the case file and
  library version but not `oracles/<library>.py`, the evaluator's libraries or the environment
  lock. Staging hashes reader source files but only the name of a side environment.
- **Consequence.** Upgrading pint, RDKit or SymPy, editing the writer's conversion or orientation
  code, or fixing an oracle script leaves stages reporting `current` (S09). The plan's statement
  that reuse is keyed on complete inputs is not true as written.
- **Correction.** One key builder shared by all stages that always includes a digest of the
  stage's framework modules, the resolved versions of its declared providers, the harness script
  and the environment lock. Format numbers then cover only deliberate contract changes.
- **Verification.** Changing a provider pin or an oracle script marks the dependent stages stale.

<a id="f09"></a>
### F09 — The envelope is narrower than the sources and binds nothing

- **Evidence** (*Interface-checked*). `envelope` is identified by (parameter set, observable,
  kind) with optional bounds. It cannot name a component or phase, join axes, express a union, or
  attach to a parameterization or assembly. Contract arguments carry a type but no observable, so
  the binding of an envelope axis to an argument is made in the qualification case file. The
  evaluator does not know envelopes. The dictionary sheet the kind traces (IC-15) requires
  domains that "need not be rectangular", composition and phase coordinates, an explicit unknown
  coverage and an extrapolation treatment. Surveys show coupled limits (IAPWS transport), species
  scope (teqp quantum-corrected cubics) and class scope (Joback).
- **Consequence.** Stated validity is dropped or flattened at mapping; "no envelope row" means
  both unknown and not yet mapped; a snapshot cannot render a production `envelope` and `guards`
  without guessing which argument an axis constrains; an evaluation outside the range is not
  diagnosed.
- **Correction.** An envelope is a region attached to any record, made of clauses (observable,
  optional component and phase qualifier, interval); several regions of one kind are a union; a
  coverage state distinguishes stated from not stated. Contract arguments name their observable.
  The evaluator reports, per point, whether it lies inside, and a run records it.
- **Verification.** A two-axis region and a composition clause load; a case with explicit points
  outside the range records them as outside.

<a id="f10"></a>
### F10 — Relations cannot declare invariants

- **Evidence** (*Interface-checked*). `RelationDecl` has no `requires`. `ev.datum` has no check
  tying `state` to `value` or `uncertainty_kind` to `uncertainty`, while the same rule is a
  verify invariant on the kind `slot_uncertainty`. Stated but undeclared: "a record has at most
  one producing derivation"; fractions of a defined mixture close to one; supersession acyclic;
  equivalent site classes share a host; a group count names a group of the assignment's scheme; a
  datum's column belongs to the point's dataset; an equilibrium-constant set is refused for a
  participant without a standard state.
- **Consequence.** Invalid evidence rows load and pass verify. The rule that every invariant has
  an enforcement point holds only for what was declared as one.
- **Correction.** Allow `requires` on relations with the same three enforcement points. Turn each
  stated rule into a declared requirement or remove the statement.
- **Verification.** A datum with a known state and no value is refused at load; each listed rule
  has a check or no prose.

<a id="f11"></a>
### F11 — The declaration-to-code shape contract is undeclared

- **Evidence** (*Interface-checked*). `meta-model.md` section 5 says the domain concepts the
  generator relies on are bound in `[framework]`, which names five roles; the loader validates one
  attribute (`slot_group`). Stage code names about two dozen kinds and their attributes as
  literals: `carrier`, `artifact`, `import_record`, `licence`, `rights_determination`,
  `record_origin`, `tabulated_axis`, `tabulated_series`, the attributes of `parameter_set` and
  `parameterization`, `derivation`, `fit`, `derivation_output`, `convention_set`, `envelope`,
  `source_entity`, `identity_assertion`, `resolution_candidate`, `species`, `species_form`,
  `naming_scheme`, `mapping_rule`, `mapping_coverage`, `held_row`, `qualification_run`,
  `run_parameter_set`, `software_release`.
- **Consequence.** Renaming or reshaping one of these in the declaration loads and generates
  cleanly and fails inside a stage. A reader of the declaration cannot tell which kinds are data
  and which are machinery.
- **Correction.** Either move these kinds into the generator's own machinery beside
  `prov.record` and `meta`, or declare a pipeline contract (kind, attributes and types the code
  requires) that the loader checks with located diagnostics.
- **Verification.** Renaming `carrier.tree_hash` is refused at load.

<a id="f12"></a>
### F12 — Typed holes in physical semantics

- **Evidence** (*Interface-checked*). `fit.covariance` is `Array<Real>` with
  `unit_from = "dimensionless"` and a description saying each entry has the product of two
  parameter units; the writer therefore converts nothing, while the RMG survey gives covariances
  in squared kilojoules per mole. A contract argument's `basis` is validated as snake-case text
  only (`declaration/resolve.py`) and is never compared at a sub-form call, where
  `expressions.md` shows a remapped composition being passed. The expression checker compares
  dimensions only, so two quantity types of one dimension are interchangeable inside expressions.
- **Consequence.** A covariance can disagree in units with the values it qualifies. A composition
  on another basis reaches a callee that assumes mole fractions.
- **Correction.** Store the covariance as a relation over pairs of free parameters whose unit
  follows from their slots, or store correlations with typed standard uncertainties. Make `basis`
  a reference to a declared `composition_basis` and require equality, or a declared conversion, at
  calls. Record the dimension-only rule as a stated limit and a kernel-gap input, since production
  types carry basis and reference.
- **Verification.** A covariance in source units is converted; a call passing a volume-fraction
  vector to a mole-fraction argument is refused.

<a id="f13"></a>
### F13 — Evidence uncertainty is single and one-sided

- **Evidence** (*Interface-checked*). `datum` carries one `uncertainty_kind`, one magnitude and a
  coverage factor; `datum_state.censored` does not say which side; `dataset_column` has no
  reference-state presentation. The ThermoML survey lists as losses: several assessments per
  value, combined, repeatability, device-specification and curve-deviation entries, asymmetric
  magnitudes, the confidence level, the uncertainty and digits of a constraint, and
  `ePresentation` with `eRefStateType`.
- **Consequence.** The plan's case (n) loads with undeclared loss of the information that
  distinguishes measured data from evaluated data.
- **Correction.** A child relation of `datum` keyed by assessment, with lower and upper
  magnitudes and an extended kind vocabulary; an assessment record per column (evaluator, method,
  coverage, confidence); censored-above and censored-below states; column attributes for the
  reference state.
- **Verification.** A ThermoML value with a standard and an expanded uncertainty and a
  repeatability loads as three typed rows.

<a id="f14"></a>
### F14 — Survey losses are not dispositioned

- **Evidence** (*Measured* by parsing the survey files). Of 956 constructs, 59 are marked
  `broader`, 87 `unmapped` and 643 `close`; `alignment-notes.md` has 16 required changes and 15
  held rows. Losses with no row include those behind [F02](#f02) and [F13](#f13), the
  order–disorder pairing of phases, RMG union nodes and PHREEQC valence-state totals. Items 8
  (parameterless site), 10 (a size for an abstract component) and 15 (an extrapolation policy
  recorded with a case) are not reflected in `model/` or the case schema. The plan's TK2 row says
  the required changes are "applied to `model/`".
- **Consequence.** Model adequacy at this checkpoint rests on an alignment that cannot be traced
  from evidence to disposition.
- **Correction.** Give every construct with a non-empty `loss` one disposition (model change,
  declared mapping loss, out of scope with reason), generated into the residue report the plan
  already owns. Correct the checkpoint statement.
- **Verification.** The residue report lists no construct without a disposition.

<a id="f15"></a>
### F15 — Role obligations are prose and a SQL list

- **Evidence** (*Interface-checked*). `origin_role` members say in their descriptions which need
  a derivation; `producing_derivation.sql` hard-codes three member names; nothing checks that a
  fitted record's derivation is a `fit`, or that `synthetic` and `oracle_input` records stay out
  of a snapshot. Production's `provenance.Role` declares these as facets on the members.
- **Correction.** Enum members carry declared facets; the check reads them from `meta`.
- **Verification.** Adding a member with a lineage facet needs no SQL edit.

<a id="f16"></a>
### F16 — Identity text without a tying invariant

- **Evidence** (*Interface-checked*). `site_class.host_key`, `association_site.carrier_key` and
  `parameter_set.subject_key` restate typed references as text so that identity is one tuple;
  nothing declares that they agree. `parameter_set.role` and `record_origin.role` state one fact
  twice. `canonical_key` rules for species, reactions and arrays are prose implemented in stage
  code; a reaction's key encodes fractional coefficients as text with no stated canonical form,
  which passes floats into identity against the meta-model's own rule. `occurrence` is a source
  position. `publication.key` is a carrier's citation key while `doi` is unique, so one paper
  under two keys either splits or refuses the build at commit.
- **Correction.** Let identity name "exactly one of" two references, or declare the agreement as
  a requirement. Declare the canonical encoding of each `canonical_key`, with exact rational
  coefficients. Resolve publications as source entities are resolved.
- **Verification.** Two carriers citing one DOI under different keys yield one publication.

<a id="f17"></a>
### F17 — Reasons are prose

- **Evidence** (*Measured*). `qual.held_row.reason` holds sentences ("the source entity ... is
  ambiguous; candidates: ..."); a blocked run's cause is `note`.
- **Correction.** A reason enum (ambiguous subject, unknown subject, validation, scheme mismatch,
  unit not parseable) with the detail beside it; the same for blocked runs.
- **Verification.** The residue report groups held rows without parsing text.

<a id="f18"></a>
### F18 — Field rules are not proven applied

- **Evidence** (*Interface-checked*). Every column of a mapped table needs a rule, and every rule
  is written to `qual.mapping_rule`; nothing records whether `mapping.py` consumed it. Coverage
  is per row.
- **Correction.** The context counts rule use and the run reports, or refuses, value rules never
  consumed for mapped rows.
- **Verification.** A mapping that stops emitting the envelope is reported.

<a id="f19"></a>
### F19 — Library consideration is not recorded

See slot 8. No document in the tree or the plan mentions a schema language, a workflow engine or
a download library. **Correction.** Record the three decisions briefly where a reader will look
(`meta-model.md`, `pipeline.md`, `acquisition.md`), and state in `implicit.py` why Newton
polishing follows the SciPy solve. Decide orchestration together with [F08](#f08).

<a id="f20"></a>
### F20 — Non-numeric slots cannot be evaluated

- **Evidence** (*Implemented*). The checker refuses a slot that holds a reference, an enum member
  or a tabulated function; the grammar has no interpolation; `ParameterSource` returns numbers.
- **Question.** Is a form over a tabulated function (sigma profiles, collision-integral tables,
  a tabulated heat capacity with linear interpolation) meant to become `expressed`, or to remain
  `structure_declared_equation_external`? Either answer is acceptable; it should be stated, and
  if the former, the grammar needs an interpolation construct whose rule is the table's declared
  `interpolation`.

<a id="f21"></a>
### F21 — Adoption is not yet a projection

- **Evidence** (*Interface-checked*; production read, not reviewed). (1) Production identifies
  declarations by authored `@id` values that survive renaming; the target derives construct
  identifiers from names (`meta_identifier`), and entity identifiers from keys. (2) A production
  quantity type carries basis, reference state and point or difference scale (§8.1, §8.4); the
  target's `production =` mark is a name with no check, and datum information sits on observables
  and convention sets. (3) Production's `parameter_set` is keyed by one species, a property, a
  phase type, a source and a variant, and `pair` is one scalar table keyed by property; the
  target's slot groups add subject roles per refinement and have no property or phase-type key.
  (4) No construct in `model/` or `forms/` carries a `pse = "gap:..."` mark yet, so the kernel-gap
  register has no input beyond the tables in the contract pages.
- **Consequence.** Adoption would force production changes (keys added by refinement or a root
  per subject shape; value-changing transpositions; stateful slots; nested and referenced sets;
  rights; lineage; piece selection) and a loss in the model only for rename stability.
- **Correction** (*Proposed*). Treat the production packages as a carrier, so that production
  entities resolve to target entities by the ordinary rules and a snapshot can emit production
  identifiers. Let a declared construct carry an optional authored identifier. State in the
  snapshot contract which envelope kind becomes a production guard and how a convention set
  becomes a named reference state. Apply the gap marks.
- **Verification.** The plan's seed-slice export equals the shipped seed values with production
  identifiers.

## 8. Library fit and ownership cost

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade cost | Recommendation |
|---|---|---|---|---|---|
| Declaration decoding | `declaration/schema.py` | **Current:** `tomllib`, msgspec structs | Fits; unknown keys refused | Low | Keep |
| Schema language and DDL generation | `declaration/`, `generate/` | **Current:** bespoke meta-model, resolver and generator. **Candidates:** LinkML with its SQL generators; SQLAlchemy Core for DDL rendering | LinkML offers classes, inheritance, enums, identifiers and generators. It lacks keyed n-ary relations with absence semantics, transposition, forms, enforcement points, deterministic identity and the reified `meta`; its SQL generator would be replaced. SQLAlchemy could render DDL but removes little | Adoption would couple the model's meaning to an external metamodel and keep most of the resolver | Keep bespoke; record the reason. Note that one meta-model field is written in five modules |
| Expression parsing | `expression/parser.py` | **Current:** standard `ast`, then a grammar check | Fits; nothing is executed | Low | Keep |
| Units and dimensions | `declaration/types.py`, `expression/units.py`, `canonical/values.py` | **Current:** pint registry as the unit authority | Fits; offset units refused in literals | pint's version and registry are result-affecting and not in reuse keys ([F08](#f08)) | Keep; key it |
| Symbolic mathematics and compilation | `expression/evaluate.py` | **Current:** SymPy with `lambdify` | Fits a reference evaluator; ordering is SymPy's, stated | Evaluator-specific determinism class | Keep |
| Root finding and quadrature | `expression/implicit.py` | **Current:** `numpy.roots`, `brentq`, `scipy.optimize.root`, `least_squares`, `quad`, plus own Newton polishing and own acceptance test | The acceptance test is an independent post-solve check and is right to keep. The polishing is own iteration | Small | Keep; state the reason for polishing (PS-09) |
| Structure keys | `resolve/structure.py` | **Current:** RDKit | Fits | Version not in the resolution key ([F08](#f08)) | Keep; key it |
| Columnar files and bulk load | `canonical/store.py`, `build/database.py`, `staging/load.py` | **Current:** pyarrow, Parquet, ADBC `adbc_ingest` | Fits; ranges have no ingestible form, so `Range<Q>` is refused for now (stated) | Low | Keep |
| Stage orchestration and reuse | Each stage's command | **Current:** per-stage key and manifest, stages refuse stale upstreams. **Candidates:** Snakemake, DVC, doit | An engine tracks code, parameters, inputs and per-rule environments, which is what [F08](#f08) misses. Costs: database outputs need marker files; the engine's metadata would duplicate the manifests; a second cache layout | Moderate to adopt; small to complete the keys | Decide explicitly. This review leans to one shared key builder, with the reason recorded |
| Acquisition | `acquire/` | **Current:** bespoke git, archive, file and rate-limited page kinds with a lock. **Candidates:** pooch for hashed files and archives; DVC imports | A library covers two or three of five kinds; page retrieval with resumption and rights metadata stays bespoke | A second store layout for part of the sources | Keep bespoke; record the reason |
| Named SQL assertions | `verify/` | **Current:** one query per check returning violators. **Candidates:** pgTAP, dbt tests | Equivalent in shape; the bijection with declared invariants is the valuable part and is specific | Low | Keep |

## 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline: grow production `.pse` packages source by source | S02 only inside today's kernel | One closed world; no competing assertions | No new machinery; the five declared kernel gaps block most hard cases | Every gap becomes a production change discovered late | Not selected by the plan, rightly |
| Proposed design: neutral declaration, generated PostgreSQL, staged pipeline, snapshot export | S01–S05 local; open world; early warning | Stage contracts are manifests; pure components test without a database | One declaration, generator, writer and evaluator | Revisions in slot 12; no performance claim is made or needed yet | **Selected, with revisions.** Revisit if the snapshot cannot be rendered to `.pse` without more than the declared gaps |
| Library-owned alternative: LinkML for structure, a workflow engine for stages | S03 by schema edit; S09 solved by the engine | External metamodel and engine own lifecycle | Removes the DDL emitter and key code; keeps resolver, forms, writer | Couples meaning to an external metamodel | Not selected for structure; open for orchestration ([F19](#f19)) |
| Simplest viable: per-source loaders into a hand-written SQL schema | S01 by new code each time | None shared | Meaning repeated per loader | Cheap first slice; no projection to production; no single authority | Rejected: it fails the purpose |

**Reference practice.** The surveyed libraries store a parameter under a formula-specific name
and resolve sharing by name lookup at load (CoolProp departure functions, TDB functions). The
target is right to make sharing and orientation explicit; [F02](#f02) and [F03](#f03) are about
finishing that, not about reverting it.

## 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| The slice is built, verified and qualified | *Measured* (stored rows); run itself reported by the plan | `psql` queries of `meta.build_source`, `qual.qualification_run`, `qual.form_qualification`, `qual.mapping_coverage` | One source, one passing run | As reported: 3,075 points, worst 4.48e-12 against 1e-10 |
| Pseudo-pure fluids are stored as species | *Measured* | Join of `tk.source_entity` and `tk.material_entity` for non-unique statuses | — | Six blends and four spin isomers are provisional species |
| A stale pass can be loaded (S08) | *Interface-checked* | Source read of `discover_qualification`, `stored_digest`, the view | — | Predicted; not executed. The test in [F01](#f01) settles it |
| Orientation rewrite (S07) | *Implemented* | Source read of `canonical_orientation`, `_swap`, `_family_rows` | — | No reciprocal pair exists in the database yet |
| Known zero refused | *Implemented* | Source read; generated domain checks | — | `tests/test_canonical_writer.py` asserts the refusal text (not run) |
| Survey disposition counts | *Measured* | `tomllib` parse of `survey/*.toml`, counting `precision` | 37 files | 956 constructs: 643 close, 106 exact, 87 unmapped, 61 narrower, 59 broader |
| Model adequacy for the fourteen hard cases | *Proposed* | This review reasoned from the declaration for six cases | Fixtures do not exist | The fixtures are the evidence; write them after the revisions |

**Commands run by this review.** File reads; `rg` searches over `thermo-knowledge/` and `docs/`;
`python3` with `tomllib` to parse survey records; read-only `SELECT` statements through
`psql -X -h /var/run/postgresql -d pse_thermo`. **Not run:** any `just` recipe, test, generator,
pipeline stage or write. Mode and failure counts against the zero baseline are therefore not
reported; the plan's own statements about tests and verify are unverified here.

## 11. Authority changes, exceptions and disposition

No blueprint section, accepted ADR or register row needs to change for this workstream, which is
standalone tooling under an active plan. Adoption by production will need an ADR; the inputs
are ADR-0127's revisit trigger and register rows R-49 and R-50, and the gaps in [F21](#f21).

Text that should be corrected through the active plan (route: sequencing and scope, the plan in
`docs/plans/`):

- Plan 24, *Database and pipeline*: "Reuse is keyed on the complete inputs of each stage"
  ([F08](#f08)).
- Plan 24, packet TK2 status: the required changes are "applied to `model/`" ([F14](#f14)).
- Plan 24 projection rules against `meta-model.md` section 4.3 on where parity and reciprocal
  rules are applied ([F03](#f03)).
- `pipeline.md` section 1 rule 5 against section 1.2 on what a provisional entity is
  ([F04](#f04)); `meta-model.md` section 5 on what the framework binding covers ([F11](#f11)).

No SHOULD exception is recorded: [F19](#f19) asks for the consideration, not an exception. The
MUST gaps remain explicit in slot 6.

Proposed disposition rows for the plan's table, all **open**, scenario and owner as proposed:

| Finding | Scenario | Proposed work owner | Evidence or revisit trigger |
|---|---|---|---|
| F01 | S08 | TK3c or TK3d follow-up | The test named in F01 |
| F02, F03, F05, F09, F13 | S06, S07, C1–C5 | TK2a, before the hard-case fixtures | Fixtures (a), (e), (f), (h), (n) load and evaluate |
| F04 | C0 | TK3b follow-up | No provisional species for a pseudo-pure fluid |
| F06, F07, F10, F12, F15, F16 | — | TK2a | The checks named in each finding |
| F08, F19 | S09 | TK3a and TK3b follow-up | A provider or script change marks stages stale |
| F11 | S03 | TK0b follow-up | A reshaped pipeline kind is refused at load |
| F14 | — | TK2 | Residue report without undispositioned constructs |
| F17, F18 | — | TK3b follow-up | Typed reasons; unused rules reported |
| F20, F21 | S02 | TK2 (decision), TK9 (reports) | The decision is recorded; gap marks applied |

<a id="decision"></a>
## 12. Decision

**Behavioural and semantic adequacy: not adequate.** G1, G2, G3, G6, G7 and PS-G1 fail on
narrow, named causes; G8 is unresolved for want of three recorded decisions; G4, G5 and PS-G3
pass at the evidence level stated; PS-G2 does not apply.

**Architectural fitness: G9 fails.** AP-04 is violated on model adequacy within the plan's own
hard cases and on a small number of duplicated facts; AP-05 is violated because important
constraints (relation invariants, conventions, envelopes, the pipeline's shape contract) have no
enforceable boundary. AP-01, AP-02, AP-03 and AP-06 are satisfied, and those verdicts matter:
they are why the revisions below are local.

**Overall: Revise.** The strongest evidence for the design is the realised slice (*Measured*)
together with the single derivation chain from the declaration (*Interface-checked*): stage
boundaries are real, ordinary extensions are local, losses at mapping are declared and
qualification is honestly scoped. What should be kept without change is the architecture: the
declaration as the one authority, the universal record root for provenance and rights, the
open-world store with a closed-world export, forms as data, and qualification derived from
recorded runs. What must change before the hard-case fixtures and Wave 1 are written is a set of
model concepts and four pipeline contracts. The main uncertainty is that model adequacy was
reasoned for six cases from the declaration; the fixtures remain the test, and the surveys not
read in detail may hold further cases. This decision accepts nothing as implemented or tested
beyond what slot 10 labels.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| 1 | Close the stale-qualification path and complete the reuse keys | F01, F08 · S08, S09 | The two tests named in the findings | Plan 24 (proposed) |
| 2 | Revise the model where the hard cases cannot be said: shared sets, orientation as asserted, entity classes in resolution, standard states by role | F02–F05 · S06, S07, C0–C3 | Fixtures (a), (e), (f) load, verify and evaluate | Plan 24, TK2a (proposed) |
| 3 | Give constraints a boundary: relation invariants, convention requirements and one home per convention fact, non-negative absolute quantities, typed covariance and basis | F06, F07, F10, F12 | Refusals and checks named in each finding | Plan 24, TK2a (proposed) |
| 4 | Widen the envelope and the evidence uncertainty model; bind envelopes to arguments | F09, F13 · C5 | Fixtures (h), (n) | Plan 24, TK2a (proposed) |
| 5 | Declare the pipeline's shape contract; disposition the survey losses; correct the plan statements | F11, F14 | Load-time refusal; residue report | Plan 24, TK2 (proposed) |
| 6 | Smaller items and recorded decisions | F15–F21 | As named | Plan 24 (proposed) |
