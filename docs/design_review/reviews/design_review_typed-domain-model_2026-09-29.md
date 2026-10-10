# Design review: typed domain model (ADR-0123–ADR-0126, Plan 23)

## 1. Scope, drivers and coverage

| Field | Content |
|---|---|
| Subject and boundary | Proposed records [ADR-0123](../../adr/0123-typed-package-schema.md) (typed package schema, structured IR (the relation version after H1's v7), entity records, relation constraints, provenance roles, envelopes, naming once, `ModelingSourceRevisionV2`) and [ADR-0124](../../adr/0124-unit-algebra-and-derived-kinds.md) (canonical unit products, derived quantity kinds). For coherence: [ADR-0125](../../adr/0125-package-data-documents.md) (Parquet data documents) and [ADR-0126](../../adr/0126-thermodynamic-domain-schema.md) (the domain schema in packages). The target and packets are in [Plan 23](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md) (*Architectural drivers and scenarios*, *Plan*). **Boundary:** the kernel/package knowledge boundary, IR and admission contracts, identity and hashing, and physical typing of coefficients. **Neighbours read:** blueprint §3.2, §5.1–§5.3, §6 (§6.1–§6.5, §6.13–§6.15), §8.1–§8.4, §9.1–§9.10, §20.5, §22. |
| Standard | Core **3.1** (AP-01–AP-06, DP-01–DP-24, G1–G9). Process-simulator profile **1.1** (PS-01–PS-13, PS-G1–PS-G3). Binding `pse-arrow` ([standard.toml](../design_principles/standard.toml)). |
| Tier / purpose | **Design tier, target purpose** (the binding default), bounded to the four records. Authority text that has to change is listed in slot 11. |
| Reviewer / date | Agent reviewer commissioned by the Plan 23 coordinator, 2026-09-29. It did not author the records, but it runs in the same agent system, so this is **not an independent human review**. |
| Decisions | **After the [resolution check](#resolution-check)**, at the Proposed level: behavioural adequacy **adequate**, with one scoped condition (N1). Architectural fitness **passes** (G9), with one condition before SM0 (N2). **ADR-0123: Accept**, after one editorial correction (E1). **ADR-0124: Accept-scoped with conditions** (N1). **ADR-0125: Accept.** **ADR-0126: Accept-scoped with conditions** (N2). *Initial decision the same day, before the amendments:* both adequacy and fitness unresolved; ADR-0123 and ADR-0124 Revise. See [slot 12](#decision). |
| Disposition owner | [Plan 23 *Finding dispositions*](https://github.com/paul-heyse/pse-arrow/blob/4c24721e691187e1a5b28398b29722fbde671da8/docs/plans/23-thermodynamic-domain-and-campaign.md#finding-dispositions). This review names proposed owners only (slot 11). |

**Functional target.** Before data banks and method families are ported (Plan 23 T2), the domain
is declared once, typed and relational, in packages. The target has four parts:

- Every library conforms to that declaration.
- The kernel enforces types, keys, references, completeness, roles and envelopes generically.
- The kernel defines no scientific concept (Plan 23 T1; the Plan 21 knowledge-boundary test; blueprint §22.3).
- Analysis modes are unchanged: the records alter admission, typing and identity, not solving.

**Drivers and variation axes.**

- **Scenarios** (not re-authored here; see slot 4):
  - CT-S01, CT-S03 and CT-S10 ([capability review](design_review_idaes-capability-target_2026-09-26.md#s01));
  - DM1–DM6 (Plan 23).
- **Credible variation axes:**
  - new banks, meaning new rows, including banks of hundreds of species in Parquet;
  - new correlation forms, meaning new refinements with new coefficient dimensions;
  - new mixture models (EoS and gᴱ families, with pair parameters);
  - new provenance categories;
  - package-level method selection.
- **Qualities:** one authority per concept (AP-04), no meaning in names or digits (DP-02), and admission-time enforcement (AP-05, DP-03). Identity and reuse must stay complete (DP-04, DP-09), and the semantic crates stay Arrow-free (§3.2).

**Baseline (Interface-checked, source read).**

- **IR.** `authored.modeling_declarations` v6 (`pse-schema/src/catalog/modeling.rs`) stores type names, missing policies, dataset sources, dataset cells, imports and annotation types as `Utf8`. That is the state at HEAD. In the working tree on 2026-09-29, Plan 23 H1 has already bumped this relation to **version 7** for the fixture execution policy (see [F01](#f01)).
- **Type parsing.** `pse-modeling/src/types.rs` parses types with a hand-written string grammar (`Fn(`, `Set<`, `Tuple<`, `^`, `Delta<`).
- **Table admission.** `data.rs` admits tables by a fixed-point rescan. It compares `missing_policy == "optional"` and re-evaluates cell text.
- **Entity attributes.** These are re-parsed on each access (`specialize/value.rs`, entity attribute traversal).
- **Kind bases.** `check.rs` refuses a kind base that is not an interface.
- **Annotations.** `annotation.rs` dispatches on `annotation_type.as_str()`.
- **Units.** Unit literals resolve by exact symbol (`QuantityRegistry::unit_by_symbol`).
- **Source identity.** `pse-runtime/src/math/modeling.rs` frames the source revision (`ModelingSourceRevisionV1`) over declaration rows and the alias-name map. The publication descriptor (`RunSourceV1`) and causal-map keys consume that identity.

**Inspected.**

- **Documents:** the four ADRs and Plan 23.
- **Source:**
  - `pse-schema` catalog `modeling.rs`, `operations.rs` (source bundles);
  - `pse-modeling`: `types.rs`, `data.rs`, `check.rs` (inheritance graph), `annotation.rs`, `analysis.rs`, `specialize/{value,group,rewrite,functions}.rs`;
  - `pse-quantity`: `registry.rs`, `literal.rs`, `infer.rs` (broadcast and reduction resolve shaped keys);
  - `pse-compiler` `workspace/modeling.rs` and its tests;
  - `pse-runtime`: `math/modeling.rs`, `workflow/publication.rs`, `authoring_driver/document/load.rs`.
- **Packages:** `packages/reference/physical` (`physical.yaml`: 466 quantity types, 39 kinds, 36 units, 86 operations) and the seed `package.toml` aliases and `.pse` datasets.
- **Library references:** the `rust-graphs` and `datafusion` skills (petgraph 0.8.3; parquet 59.3.0 `ParquetRecordBatchReaderBuilder`).

**Not examined.**

- No build, test or benchmark was run (instructed).
- The not-examined behaviour includes:
  - the numerical behaviour of any form;
  - fitting parameter-set coefficients as estimation parameters (DM6 beyond lineage);
  - the H, C and S tracks' process scenarios;
  - the Python boundary.

**Analysis modes in scope:** none changes. Every mode consumes the admitted knowledge. Journeys
considered: add a property model, out-of-envelope evaluation, boundary round trip, local testing
and edit → re-solve for a knowledge edit.

## 2. Decomposition, ownership and dependencies

| Component / responsibility | Decision or invariant hidden | Contract consumed / exposed | Dependency direction and reason | State/effect owner | Local test setup |
|---|---|---|---|---|---|
| `pse-quantity`: unit products, derived kinds, literal typing | Canonical factor order, rational exponents, monomial index, eligibility of leaves | Consumes physical document rows; exposes `compose`, `by_monomial`, inference with evidence | Foundation, Arrow-free | Immutable registry | Pure units (KR1, KR2) |
| `pse-authoring`: structured-IR syntax, `UnitProduct` literals | Grammar and spans | Source text → typed IR values | Below `pse-modeling` | None | Parser round trip. The `arbitrary` proptest generators already exist for the DSL |
| `pse-schema` registry | Structured IR shape (the version after H1's v7); kernel-acted enums (missing policy, annotation kind, fact namespace, data role, version operator, key symmetry); `DocumentKind::Data`; `reference.units` v2, `quantity_types` v2; `runtime.modeling_checks` v3, `modeling_conformance` v2 | Generated `pse-model` values | Generator → consumers | — | `just codegen-check` (not run) |
| `pse-modeling`: checking and specialization | Refinement, keys, references, completeness, symmetry, derived columns, taint, envelopes, `Ref` dispatch | Consumes typed IR, an Arrow-free row set and the physical context; exposes `CheckedPackage`, admitted tables | Arrow-free (§3.2) | None (pure) | Pure units; D0 refusal corpus |
| `pse-compiler`: Salsa workspace | Revision reuse; per ADR-0125, admitted-table reuse | Checked revision → specialized bodies | Arrow-free | Salsa database | Workspace tests |
| `pse-runtime`: composition root | Document I/O, Parquet decoding by Arrow type, source-revision framing, source bundles | Package documents → declarations and row sets; revision identity → publication | The only crate joining columnar and semantic code (§3.2) | Owns all effects | Runtime units: heavier build, no solver or store start needed |
| `pse-operations`: source bundles | Durable job sources | `operational_source_documents.content` is text today | Beneath runtime | PostgreSQL | Store tests |
| `pse.physical` (package) | Kinds, types, units, reference states, derived kinds, quantity names; per ADR-0126 also species, element, reaction and phase | Physical inventory; modeling kinds | Root package | Data | Admission of the package alone |
| `pse.domain` → `methods` → `data/<bank>` → property packages | Provenance, property, `parameter_set`, pair and group relations, `selection`, `component_role`, constants; forms; rows; bindings | Package imports by identity | Each layer depends downward only | Data | D0 corpus; H1 whole-seed conformance |

The composition root is still `pse-runtime`. Changes to kernel mechanisms land in the four
semantic crates plus the registry. Knowledge changes land in packages. That separation is the
decisive strength of the selected option in ADR-0123 (see [F13](#f13) for dispatch).

## 3. Contracts, authority and constraints

| Meaning / contract | Authoritative owner and update path | Consumer obligations / invariant | Enforcement and failure | Derived representations / evolution |
|---|---|---|---|---|
| Structured declaration IR | Registry relation, the version after H1's v7 (ADR-0123) | Names only as path segments, resolved once | Admission; artifacts under earlier versions are `MigrationRequired` (§20.5) | Rendered source, generated values. **Frames that render units or types change bytes ([F01](#f01))** |
| Source revision identity | `ModelingSourceRevisionV2` in `pse-runtime` | Distinguishes every revision that changes meaning | Keyed frame (§5.3) | Run source identity, causal-map keys. **Contents not enumerated ([F01](#f01))** |
| Keyed entity identity | ADR-0123: `(kind, key identities)` | Stable across banks and documents | Derived identity (§5.1) | `Ref` values, `selection` rows, lineage. **Scope and canonicalization open ([F03](#f03))** |
| Identifier schemes | Package declaration; values opaque | Unique within the package closure | Checked at closure admission, so a consumer's closure can fail where each package alone passed | Parquet references by scheme value |
| Relation constraints | Relation declaration | PK/FK, `complete_over`, symmetry, uniqueness, ranges, derived columns, `require` | Admission in reference order. **Granularity ([F05](#f05))** | Admitted tables (cached, ADR-0125, [F10](#f10)) |
| Provenance role and test-only taint | Registry enum `ModelingDataRole`; dataset/constant/test `source` | Oracle and synthetic data never reach a production root | Taint over the checked reference graph. **Granularity ([F04](#f04))**; vocabulary ([F12](#f12)) | `oracle_source_id` in conformance v2 |
| Validity layers | Form `valid`, data envelope with `ExtrapolationPolicy`, closure `annotation valid` | Intersected; the form layer never extrapolates | Guards via the domain-predicate path; `modeling_checks` v3 `layer` | **Policy owner and multi-argument binding ([F11](#f11))** |
| Units and derived kinds | Physical document (ADR-0124) | Chains resolve by monomial against declared kinds, never synthesized | Refusal names the factors | **Basis, scale and reference policy of a monomial ([F02](#f02)); unit-product identity ([F14](#f14))** |
| Quantity names and reference states | Physical document (KR8), replacing `[[quantity_aliases]]` | A name requires a manifest dependency; ambiguity refuses | Admission | **Datum also stated on `parameter_set.reference` ([F08](#f08))** |
| Method selection | `selection[k, j, p, t] → Ref<parameter_set>` in `pse.domain` | A generic function dispatches on the concrete form; a missing row is a typed refusal | Row `require` and completeness | **No seat for mixture or pair selection ([F06](#f06))** |
| Data documents | `data/*.parquet`, decoded in `pse-runtime` (ADR-0125) | Column names and types equal the declaration | Typed refusal naming the column | **Text-only loaders and bundles ([F09](#f09))** |

**Physical semantics (profile slot 3).**

| Quantity or model element | Dimension and unit | Basis | Reference state / convention | Validity envelope | Authority |
|---|---|---|---|---|---|
| Form argument `T` | Temperature, canonical K | — | Point; origin-sensitive kind | Form `valid` ∩ row envelope | Physical document; form |
| `dippr100` coefficients `c1…c5` | `MolarCp·Temperature^(1−n)`, literals such as `J/(kmol*K^2)` canonicalized | Molar, inherited from `MolarCp` | None. **The basis and scale policy of a monomial chain is unresolved ([F02](#f02))** | Row `T in a..b` (data layer) | Derived kinds in the physical document; the form in `methods` |
| Increments `dh(T0,T)`, `ds(T0,T)` | `DeltaH`, `DeltaS` | Molar | Difference; the datum cancels | Envelope of `cp` over the whole interval [T0, T]: **unbound ([F11](#f11))** | Form |
| Enthalpy with a datum | `MolarEnthalpy` | Molar | Datum in the type **and** in `parameter_set.reference` ([F08](#f08)) | — | Named reference states in the physical document |
| Gas constant | Sketch: `MolarCp`. Registry: a distinct `gas_constant` kind | Molar | — | — | `pse.domain` constant. **Mismatch ([F15](#f15))** |
| `species.molar_mass` | `MolarMass` | Molar | Derived: `sum(formula × standard_atomic_weight)` | — | Chemistry in `pse.physical`; `ciaaw` rows |
| Antoine pressure unit and °C offset | Pressure (absolute); temperature point | — | Gauge refused by the datum; an affine unit only as sole factor | Row envelope | Form attributes (CT-S01) |
| Composite unit literal | Canonical product with rational exponents | — | Affine or datum-restricted units only alone, exponent one | — | `pse-quantity`. **Identity ([F14](#f14))** |

**Well-posedness statement.** The records do not change variable roles, degree-of-freedom analysis
or structural analysis (PS-04, PS-05). What they add is rejection earlier than today:

- Missing required rows, completeness gaps, unit mismatches and selection mismatches are refused at package admission, before specialization.
- A lookup outside a completeness set is refused before evaluation (KR5).
- Diagnostics name the declaration, row and clause.

## 4. Change scenarios and composition

| Scenario / stimulus and conditions | Expected response and change boundary | Edit/composition path under the proposal | Observed or predicted impact | Acceptance and evidence |
|---|---|---|---|---|
| [CT-S01](design_review_idaes-capability-target_2026-09-26.md#s01) Add RPP5 cp and Antoine | Diff only in `packages/` | A refined `caloric_set` form in `methods`; rows in `data/poling2000`; a derived-kind row per new coefficient monomial | Local **if** chains resolve by monomial. As written, basis-carrying and origin-sensitive leaves fall back to stepwise rules, so each coefficient needs registered rules or intermediate kinds ([F02](#f02)) | IDAES `test_RPP5` values; package-only diff. Proposed |
| [CT-S03](design_review_idaes-capability-target_2026-09-26.md#s03) Add an EoS family | A form plus parameter data | A Helmholtz definition in `thermodynamics`; species rows; pair rows | Pure-component parameters fit `parameter_set`. Pair-source and phase-model selection have no declared seat ([F06](#f06)) | FeOS oracle bank. Proposed |
| [CT-S10](design_review_idaes-capability-target_2026-09-26.md#s10) Iterate leaves a correlation range | Typed rejection, or a selected extrapolation recorded with its layer | Envelopes become guards; `modeling_checks` v3 `layer` | Mechanism sound. The policy owner and interval binding are open ([F11](#f11)) | H5 lineage match. Proposed |
| DM1 Parquet bank | Rows only, zero Rust | `data/<bank>` with a document and a dataset declaration | Local in the package. Identity and durable execution need [F01](#f01) and [F09](#f09) | KR9 tests; 10⁵-row bench. Proposed |
| DM2 One species in two banks | Both admit; the package selects one | Two sets differing in the `source` key; one `selection` row | Local for pure-component sets. Detecting a same-key conflict depends on the identity scope ([F03](#f03)); pairs have no selection ([F06](#f06)) | D0 corpus. Proposed |
| DM3 Wrong coefficient units | Refusal naming the attribute and the expected and actual quantity | Typed cells checked against the attribute type | Literal units work. Typed coefficient kinds depend on [F02](#f02) | KR2, D0. Proposed |
| DM4 Oracle data in production | Refused | `admits` `require` on `selection` plus kernel taint | Two expressions of one policy; taint granularity open ([F04](#f04)) | KR6. Proposed |
| DM5 A method family as data (NRTL) | Forms plus pair rows | Form library and `pair` relation | Selecting the pair source and the gᴱ model has no seat ([F06](#f06)) | Closed form, Gibbs–Duhem, IDAES oracle. Proposed |
| DM6 Publish fitted parameters | Lineage required | Role `fitted` with lineage | Acyclicity per row versus the declaration-level DAG ([F05](#f05)) | KR6. Proposed |
| <a id="s01"></a>S01 Edit one coefficient in a Parquet bank, then re-solve | The source identity changes; unaffected admitted tables and specializations are reused; a knowledge edit legitimately re-specializes the forms it reaches (it is a definition revision, not a case value) | Document bytes → document hash → revision identity → Salsa inputs | Identity is incomplete as written ([F01](#f01)); the reuse owner is ambiguous ([F10](#f10)) | Incremental equals clean. Proposed |
| <a id="s02"></a>S02 A bank needs a new provenance category, such as DIPPR "predicted" | A package edit (knowledge-boundary test) | Closed registry enum | Registry edit plus codegen for a category the kernel does not act on ([F12](#f12)) | AUD. Proposed |
| <a id="s03"></a>S03 Test admission, taint or envelope policy locally (PSE-S05, CT-S11) | Pure `pse-modeling` tests from declarations | D0 corpus; KR4–KR7 units | Satisfied: no runtime, store or solver is needed. Only KR9's decoder needs `pse-runtime` | KR tests. Proposed |

## 5. Mechanisms and execution, where material

| Stage / owner | Contract and mechanism | Inputs / dependencies | Effects and lifecycle | Reuse / equivalence / limits | Evidence or uncertainty |
|---|---|---|---|---|---|
| Decode (`pse-runtime`) | Parquet → Arrow → Arrow-free row set, typed by the declared column | Document bytes; the dataset declaration | Pure after I/O; rows charged to workspace limits | Keyed on the document hash | Interface-checked: parquet 59.3.0 is pinned, but `pse-runtime` does not yet depend on it directly |
| Admit relations (`pse-modeling`) | Keys → identities; FKs; completeness; symmetry; derived columns; `require` | Typed IR and row sets | Pure; all-or-nothing admission (§5.2) | The DAG order is declared per declaration ([F05](#f05)) | Proposed |
| Resolve dispatch (`pse-modeling` specialization) | `selection` → `Ref` → concrete kind → bound function body | Static keys (package, species, property, phase type) | Pure | One specialization per resolved function | Interface-checked seam ([F13](#f13)) |
| Type coefficients (`pse-quantity`) | Monomial over kinds against declared derived kinds | Physical document | Pure | Eligibility rule open ([F02](#f02)) | Proposed |

**Numerical stage columns** (form evaluation through `selection`):

| Formulation policy | Derivative source and order | Scaling | Problem class · solver capability used | Status → outcome mapping | Tolerances · post-solve check |
|---|---|---|---|---|---|
| Envelope guards per layer, through the domain-predicate path; the form layer never extrapolates | Symbolic, from Symbolica over the selected form bodies; unchanged | Nominals from quantity types. A derived kind's canonical unit comes from its declaration; a nominal is optional | Unchanged | Unchanged | `modeling_checks` v3 records the layer. Validity and extrapolation stay separate observations (§9.10) |

## 6. Architectural assessment and gates

| Foundation | Scenario and evidence / scope reason | Verdict | Required action |
|---|---|---|---|
| AP-01 Separation of concerns | The kernel/package split holds. Every domain concept is a package declaration (sketch; ADR-0126), and the kernel gains only generic mechanisms. Parquet stays in `pse-runtime`. The package split (chemistry inside `pse.physical`) rests on a premise that is misstated ([F07](#f07)) | Satisfied at the kernel boundary. Package placement unresolved | F07 |
| AP-02 Stable contracts | Identity frame contents ([F01](#f01)), unit-product identity ([F14](#f14)), and binary documents crossing text-only contracts ([F09](#f09)) | Unresolved | F01, F09, F14 |
| AP-03 Composition | CT-S01, DM1 and DM2 compose from rows, forms and bindings. Mixture and pair selection lacks a composition seat (CT-S03, DM5) | Unresolved | F06 |
| AP-04 Authoritative meaning | Scope of key identity ([F03](#f03)), a datum stated twice ([F08](#f08)), extrapolation policy on data ([F11](#f11)), and taint versus `admits` ([F04](#f04)) | Unresolved | F03, F04, F08, F11 |
| AP-05 Explicit structure | Typed IR and kernel-acted enums replace string dispatch (a strength). The monomial eligibility and result policy are implicit ([F02](#f02)), as are DAG granularity ([F05](#f05)) and key values supplied by a dataset ([F03](#f03)) | Unresolved | F02, F03, F05 |
| AP-06 Local reasoning/testability | Admission, taint and envelopes can be tested purely with declarations (S03). The H1 regression net and the D0 refusal corpus are declared | Satisfied (Proposed) | — |

| Gate | Pass / fail / unresolved / not applicable | Evidence or scope reason | Required action |
|---|---|---|---|
| G1 Authority | Unresolved | F03, F08, F11; F04 (two expressions of one policy) | Amend ADR-0123; D0 |
| G2 Semantic fidelity | Unresolved | The monomial policy (F02); enum-member names in key identity (F03) | ADR-0124, ADR-0123 |
| G3 Validity | Unresolved | Taint granularity (F04); refusing legitimate self-references (F05) | KR5, KR6 text |
| G4 Hidden behaviour | Pass (Proposed) | Admission and specialization are pure; derived columns are evaluated once | — |
| G5 Consistency and recovery | Pass (existing contract) | Admission is all-or-nothing, and a failure leaves prior revisions intact (§5.2) | — |
| G6 Transformation and reuse | Unresolved | Source identity incomplete (F01); a second cache (F10) | ADR-0123, ADR-0125 |
| G7 Truthful capability claims | Pass for the ADRs, which are labelled *Proposed*. The plan's premise "177 types … as subjects" is inaccurate (F07) | Correct the plan text | F07 |
| G8 Library leverage | Pass | petgraph for the refinement and reference graphs (already used in `check.rs`); the parquet crate; typed IR replacing hand-parsed grammar; proptest generators reused | — |
| G9 Architectural fitness | **Unresolved** | No foundation is violated; AP-02, AP-04 and AP-05 are unresolved | Findings above |
| PS-G1 Physical consistency | Unresolved | F02 (basis and scale of a monomial), F08 (datum), F15 (typing of R) | ADR-0124; SM4 |
| PS-G2 Well-posedness | Not applicable | DoF and structural analysis are unchanged; admission refusals only move earlier | — |
| PS-G3 Numerical integrity | Pass (Proposed) | The form layer never extrapolates; the layer is recorded; extrapolation is observed separately from membership | F11 settles who selects |

**Knowledge boundary.** No scientific concept enters Rust or the registry in the proposal. The
following are generic: species, phase, property, parameter set, selection and component role are
all declared in `pse.physical` or `pse.domain`. Key symmetry, identifier schemes, refinement,
facets, `QuantityType`, `IndexShape`, envelopes, layers and version operators are generic. The
closed `ModelingDataRole` taxonomy is the one borderline case: it is epistemic, not scientific,
but part of it is vocabulary the kernel does not act on ([F12](#f12)).

## 7. Findings

| ID | Sev. | Finding | Principles / gate / scenario | Evidence or gap | Consequence | Correction | Verification |
|---|---|---|---|---|---|---|---|
| <a id="f01"></a>F01 | High | ADR-0123 changes the hashing contract but does not state what `ModelingSourceRevisionV2` frames. ADR-0125 puts data-document hashes into package identity, not into the source revision. Once aliases are deleted, V1's alias map has no stated successor. | DP-04, DP-09, DP-24 · G6 · DM1, S01 | V1 frames the declaration rows and the alias `names` (`pse-runtime/src/math/modeling.rs`). `RunSourceV1` (`workflow/publication.rs`) and `CausalMapV2` hash that identity. Data-document rows are not declarations. KR1 changes rendered unit literals, which enter `ModelingDispatchBodyV1` and function-specialization preimages (`specialize/group.rs`, `functions.rs`). Proposed | Two runs over different bank bytes, or different physical name bindings, can publish the same source identity, so lineage and reuse keys are incomplete. Digests under unchanged frame spellings move silently. | Enumerate V2's preimage: the structured declaration rows; each data document's identity and content hash, stating byte-level equivalence; and the physical-inventory identity in place of the alias map. List every frame whose preimage changes and give each a new catalog variant (DP-24). Because H1 already uses relation version 7, the structured IR will be a later version. The records should name that version by what it adds rather than as "v7". | Tests: changing one Parquet byte changes the revision and descriptor source identity; unchanged inputs reproduce it; the golden-vector list gains the new variants. |
| <a id="f02"></a>F02 | High | ADR-0124's compensating control ("origin-sensitive or basis-carrying leaves keep the stepwise rules") excludes the leaves of its own flagship. `MolarCp` carries a molar basis and `Temperature` is origin-sensitive, so `MolarCp/Temperature^2` and the terms `c3*T^2` would not resolve by monomial. The ADR also gives no basis, scale, reference or shape policy for a monomial result. | PS-01, DP-02, AP-05 · PS-G1, G2 · CT-S01, DM3 | Checked in `physical.yaml`: `MolarCp` has basis molar; the `temperature` kind is `origin_sensitive`. Stepwise rules carry non-trivial policies: `Mul[molar_heat_capacity, temperature] → molar_enthalpy` has scale *difference* and a declared reference, while `Mul[gas_constant, temperature] → molar_energy` gives a point. The expected KR2 test `polynomial_terms_type_without_intermediate_kinds` contradicts the control. Proposed | The typed-coefficient driver fails for every temperature polynomial, or the chain rule silently decides basis and scale outside PS-01. Each form then needs intermediate kinds or rules: change amplification into the physical document. | Define the eligibility rules and the result policy. Basis is require-equal across basis-carrying factors, or is declared on the derived kind. Points of ratio-scale kinds whose canonical unit has a true zero (absolute temperature) may be multiplicative factors. Points with a nonzero datum (gauge pressure, enthalpy with a datum) are refused. Scale, reference and subject of the result come from the derived-kind declaration. A chain with an ineligible leaf is refused with the factor named, not partly resolved. Increments such as `c3*T^3/3` resolve to the declared `DeltaH` kind. | The KR2 tests over the actual SM3 forms (dippr100, Shomate, RPP4 `cp`, `dh`, `ds`) type without intermediate kinds; `origin_sensitive_leaf_uses_registered_rules` pins the exact eligible set; a gauge-pressure leaf is refused. |
| <a id="f03"></a>F03 | Med–High | The keyed identity `(kind, key identities)` does not say which kind: the key-declaring abstract kind or the concrete refinement. Nor does it say where key uniqueness is enforced across refinements, how enum-member key values (for example `phase_type`) are encoded, or how defaults (`variant = 1`) and keys supplied by the dataset (`source` from `provenance(...)`) enter the identity. | DP-04, AP-04, AP-05 · G1, G2 · DM2 | In the sketch, `parameter_set` declares five keys but rows supply three; `source` comes from the dataset. v6 enum members (and H1's v7) are names without IDs (`strings("members")`), and `Value::Enum` frames the member name. The approach matches §5.1 "derived entities" otherwise. Proposed | If identity uses the concrete kind, the same key admitted under two forms is not a conflict, a `Ref` written by key is ambiguous, and DM2's same-key refusal cannot be enforced. Renaming an enum member re-keys every parameter set. | Identity = a catalog frame over (the key-declaring kind, the ordered typed key values including defaults). Uniqueness is at the key-declaring kind across all refinements. The concrete kind is content, not identity. Enum members get identities, or are declared name-contracted as in `named` policy. Keys supplied by the dataset are an explicit declared binding. | Tests: the same key in `dippr100` and `shomate` is refused; moving a row between forms keeps its identity; changing a default is visible in identity. |
| <a id="f04"></a>F04 | Medium | Test-only taint ("every reader") has no stated granularity. `selection` is one relation holding rows contributed by many packages. Taint at the relation level makes every production reader test-only; taint at the row level needs the rows resolved at specialization. DM4 is also expressed a second time by the package `require value.role in k.admits`. | DP-03, AP-04 · G3, G1 · DM4 | ADR-0123 *Provenance*; KR6 "taint over the checked reference graph"; the sketch's `selection` `require`. Rows carry their dataset (`Table.origins`), but `value.role` is not a declared attribute. Proposed | Either the whole seed turns test-only once an oracle package is in the closure, or a production root reads oracle rows while the taint pass reports clean. | Taint propagates along resolved row and entity references: statically where keys are literal, otherwise at specialization. The kernel taint is the authority for "production reads no oracle data"; `admits` is the package's selection filter. Define the row-origin role as a kernel-provided property. | A production package and an oracle package share `selection`: the production root stays admissible, and a root reading an oracle row is refused outside fixtures. |
| <a id="f05"></a>F05 | Medium | "Admission in reference-DAG order" at the granularity of declarations refuses legitimate self-references. Examples: `fitted` or `derived` sets whose lineage points into their own kind, and `oracle_test.release: Ref<software_release>` inside the `source` hierarchy. Identity derived from keys makes declaration order unnecessary for FK resolution. | DP-03, DP-07 · G3 · DM6 | KR5 `admission_follows_reference_dag_and_refuses_cycles`, `lineage_cycle_refused`; the sketch's kinds. Today's `data.rs` loops to a fixed point. Proposed | Valid banks and lineage are refused, or authors split kinds only to satisfy the order. | Phase 1: row identities from keys. Phase 2: FK, completeness and symmetry checks, which need no order. Phase 3: derived columns and `require` clauses, in the DAG of value dependencies. Row-level cycle detection for lineage uses petgraph `toposort` and `kosaraju_scc`, as `check.rs` already does. | Acyclic self-referential lineage admits; a cycle is refused with its rows named. |
| <a id="f06"></a>F06 | Medium | The domain schema gives method selection a seat only for pure-component parameter sets keyed per species (`selection[k,j,p,t]`). There is no declared selection of the pair-parameter source (for example two banks' NRTL τᵢⱼ or kᵢⱼ for one pair) and no statement of who selects the phase model (EoS or gᴱ family). | AP-03, AP-04, PS-02 · CT-S03, DM2, DM5 | `pair[s,p,i,j]` is keyed by source; `parameter_set.subject` is a single species, so pairs cannot be selected. ADR-0126 *What libraries do* is silent. Proposed | Mixture ports choose sources in function bodies or by naming convention: the implicit, library-specific behaviour Plan 23 exists to remove. | In `pse.domain`, declare a package-level selection for pair and group data (a keyed `pair_set` kind, or `pair_selection[k,p] → Ref<source>`). State that the phase formulation is a property-package definition binding (CT-S07 `formulation`). No kernel change. | DM2 extended to pairs; a D0 refusal for a missing pair selection. |
| <a id="f07"></a>F07 | Medium | ADR-0126 keeps species, element, reaction and phase in `pse.physical` "because physical quantity types name them as subjects". Plan 23 says 177 types. That premise is inaccurate, and whether it is necessary is unresolved. | AP-01, DP-22 · G7 · SM0 | `physical.yaml`: `subject_kind` is null on all 466 types. The kinds appear as **shape axes**: species or element in 221 types (species 203, element 42), reaction in none. No package alias or `.pse` document names any of the 221 (repository search). Modeling finite reductions expand to scalar prototypes (`specialize/rewrite.rs`). An existing test indexes variables over a package-declared kind (`kernel_table_function_references_dispatch_and_share_specializations`). Inference broadcast and reduction resolve shaped keys (`infer.rs`). Interface-checked | Chemistry schema (identifier schemes, formula, charge, phase type) is fixed in the physical package for a reason that may not hold. For `reaction` it does not hold. The choice is costly to reverse after SM0–SM6. | Before SM0, establish whether the modeling path needs registry-shaped types for chemistry axes. If not, delete the unreferenced shaped types and place the chemical core in `pse.domain`. If so, state the constraint as "shape axes" and move `reaction` up. Correct the plan's count. | A package-declared axis indexes a sum in a modeling fixture without a registered shaped type. |
| <a id="f08"></a>F08 | Medium | The datum of a parameter set is stated twice: `parameter_set.reference: Ref<reference_state>?` (data), and the reference state inside the form's result quantity types (type; PS-01's authority). `Ref<reference_state>` also needs a kernel type naming a physical-registry row, because reference states are not modeling entities. | AP-04, PS-01 · G1, PS-G1 | Sketch in Plan 23; KR8 "named, addressable reference states"; §8.4. Reference states are `reference.reference_states` rows (`physical.yaml`: 4 rows). Proposed | A set can declare datum X while its `h` returns a type with datum Y. Consumers then read whichever copy they meet. | Derive the datum from the form's result types, or keep the attribute with a kernel check that it equals the datum of each datum-carrying result. Specify physical-reference language types (`QuantityType`, `ReferenceState`) as one generic mechanism. | D0 refusal: "set datum disagrees with the form result type". |
| <a id="f09"></a>F09 | Medium | ADR-0125's binary documents cross contracts that are text-only today, and its *Consequences* list only the registry and the checksum. | AP-02, DP-15 · DM1 | `load_package_texts(BTreeMap<String,String>)` and `package_checksum` hash path and text pairs (`authoring_driver/document/load.rs`). `runtime.operational_source_documents.content` is text (`catalog/operations.rs`). Worker bundle verification works on texts. "Declared storage unit": the ADR does not say where the unit is declared. Interface-checked | Durable jobs cannot carry a package that has a bank, or they encode bytes ad hoc. A unit stated in Parquet metadata would be a second authority. | Documents become bytes with a declared kind. Source bundles get a binary content column (regenerated store, ADR-0114 reset). Units are declared only by the dataset declaration, and Parquet metadata that disagrees is refused. List these in ADR-0125. | A durable job over a package with a data document round-trips through the operational store. |
| <a id="f10"></a>F10 | Low–Med | ADR-0125's "admitted tables cached by table declaration and dataset hashes" is not placed in the Salsa workspace, which §22.2 makes the single reuse owner. Checking today is one whole-revision `pse_modeling::check`, outside tracked queries. | DP-09 · G6 · S01 | `pse-compiler/src/workspace/modeling.rs` `publish_modeling` (equality reuse, then `check`). Proposed | A second, hash-keyed reuse mechanism beside Salsa, with its own invalidation. | Make the admitted table a tracked query over the table declaration and the document input. | `unchanged_dataset_reuses_admitted_table`, plus incremental equals clean after editing one document. |
| <a id="f11"></a>F11 | Medium | Two gaps in the envelope contract. (a) The data layer carries its own `ExtrapolationPolicy`, so a bank or schema decides for every consumer. (b) A named envelope axis (`envelope T`) has no rule for forms with several arguments of that axis: `dh(T0,T)` integrates `cp` over [T0, T], and T0 is often the stock datum 298.15 K. | PS-02, DP-05, AP-04 · CT-S10 | ADR-0123 *Validity*; KR7; the sketch's `caloric_set`. ADR-0115 Outcome 3 put the policy on the consumer's `annotation valid`. Proposed | (a) The policy is selected by the wrong owner. (b) An increment can silently evaluate `cp` below its envelope, the silent extrapolation PS-02 forbids. | Data declares envelopes only; the property package or the analysis selects extrapolation per layer, and the selection is recorded. Forms declare which arguments, or which integration interval, each envelope guards. | CT-S10 with one bank row under two packages that choose different policies; `dh(T0,T)` with T0 outside the envelope is refused. |
| <a id="f12"></a>F12 | Low | `ModelingDataRole` fixes six provenance roles in the registry. The kernel acts on only two properties: test-only taint (`oracle_input`, `synthetic`) and required lineage (`fitted`, `derived`). `published` and `measured` have no kernel behaviour. | Plan 23 T1, ADR-0115, DP-16 · S02 | ADR-0123 *Vocabulary*; KR6. Proposed | Each new category a bank needs becomes a registry and codegen change. | Kernel facets (`test_only`, `requires_lineage`), with roles as package enum members that declare their facets. Alternatively, record why the taxonomy is closed. | A new role is a package-only edit. |
| <a id="f13"></a>F13 | Low | The Plan 23 open item places `Ref` dispatch in "dispatch grouping" and names "function-typed slot tables" as a fallback. The actual seam is function specialization keyed by the resolved call. The fallback already exists; the real precondition is that a `Ref` is static at specialization. | AP-05 · CT-S01, SM5 | `Value::Entity{id, kind}`, and attribute traversal by kind (`specialize/value.rs`). `DispatchGroup` groups instances by body hash, including called function identities (`group.rs`). An `Fn`-typed table column specializes per selected function in `kernel_table_function_references_dispatch_and_share_specializations` (a named test that exists; not run here). Interface-checked | The risk is misdirected: effort goes to grouping, while the static-key precondition goes unenforced. | Restate the open item: a `Ref` must be static; a `Ref` that depends on a runtime value is refused. Refinement makes `kind` the most-derived kind. | `ref_calls_dispatch_one_body_per_concrete_kind`, plus a refusal test for a non-static `Ref`. |
| <a id="f14"></a>F14 | Low | ADR-0124 rejects synthesized kinds because their identity would depend on usage, but gives no identity for composite unit products. `QuantityType.canonical_unit`, `modeling_reports.unit_id` and the physical-inventory identity all need a unit ID. | DP-04, AP-02 · KR1 | §8.2: a spelling must name an admitted unit. Today composite units are registered whole (`J/(mol*K)`, `J/(kmol*K)`, `m^3/(mol*s)`). Proposed | Report and inventory identities for composite units are undefined or spelling-dependent. | A unit product's identity is a catalog-frame derivation over its canonical factors, independent of spelling. The canonical units of derived kinds are declared. | `unit_product_is_order_independent` asserts identity equality; a report in a composite unit carries it. |
| <a id="f15"></a>F15 | Low | The sketch types `gas_constant` as `MolarCp`. The physical document has a distinct `gas_constant` kind, so the inferred type of `R*T` changes from `molar_energy` (point) to a `molar_enthalpy` difference. This is behavioural (scientific) adequacy only. | PS-01 · PS-G1 · SM4 | The two registered rules, from `physical.yaml`. Proposed | Energy terms built from R acquire a difference scale and a declared datum. | Type R with the existing kind in SM4. | The eos and homogeneous fixtures are unchanged (H1). |

**Strengths that carry the argument.**

- The selected option in ADR-0123 is the least-machinery design that satisfies T1. Knowledge becomes rows, forms and bindings; the alternative of registry families is correctly rejected.
- The typed IR deletes the bespoke string type grammar, string policy dispatch, prefix parsing and fixed-point rescans (DP-02, DP-16).
- ADR-0124's refusal to synthesize kinds keeps one owner for physical types.
- Arrow stays in `pse-runtime` (§3.2).
- Every packet names what it deletes.

## 8. Library fit and ownership cost

| Capability / contract | Integration owner / exposed types | Candidate or current mechanism | Fit and limits | Coupling, lifecycle, test, upgrade/replacement cost | Bespoke code removed / recommendation |
|---|---|---|---|---|---|
| Refinement and reference graphs | `pse-modeling`, private | petgraph 0.8.3 `DiGraph`, `toposort`, `kosaraju_scc`, `has_path_connecting` (already in `check.rs`) | Fits. `kosaraju_scc` returns reverse topological order (rust-graphs note) | None new | Replaces the fixed-point loop. Adopt (F05) |
| Bulk data decoding | `pse-runtime`; exposes an Arrow-free row set | parquet 59.3.0 `ParquetRecordBatchReaderBuilder` over bytes; the family is pinned | Fits: typed columns, compression, statistics. DataFusion `ListingTable` is unnecessary until the reserved projection | One more direct dependency inside the pinned family | Adopt, as ADR-0125 does |
| Incremental reuse of admitted tables | `pse-compiler` | Salsa (the workspace owner) | Fits as a tracked query | None new | Use Salsa, not a side cache (F10) |
| Unit and kind algebra | `pse-quantity` | Existing exact `Ratio` algebra; static libraries such as `uom` | A static-type units crate cannot express registry-declared runtime kinds, bases and datums | — | Keep in `pse-quantity` (§8) |
| Round-trip property tests | `pse-authoring` | proptest 1.11 `arbitrary` generators (exist) | Fits | — | Reuse for the type arena |
| Recursive IR in Arrow | Registry | Post-order arena with child indices | Arrow has no recursive types, so an arena is the standard encoding | — | Adopt |

## 9. Alternatives and tradeoffs

| Alternative | Scenarios served / change locality | Contracts, composition and test isolation | Meaning or machinery carried | Correctness / operational cost | Selection and revisit condition |
|---|---|---|---|---|---|
| Current baseline (v6 text IR) | CT-S01 and DM1 not served; meaning lives in names and digits | Re-parsing everywhere | Implicit conventions per library | Unchecked coefficient units; lookup-time failures | Rejected (ADR-0123) |
| Proposed design | All scenarios, subject to F01–F12 | Pure admission tests | Generic kernel mechanisms; domain in packages | Wide IR change; seed rewrite | Selected. Revisit on a kernel gap under the knowledge-boundary test |
| Registry-declared scientific families | Strong typing | Every new concept goes through Rust and codegen | Science in the registry | Violates T1 and §6 | Rejected (maintainer, 2026-09-29) |
| Simplest viable for ADR-0124: typed coefficients with explicit per-form kinds | No algebra | One kind row per coefficient, including intermediates | More rows | No new inference rule | The monomial design is preferable **once F02 defines its policy**; otherwise this alternative is safer |

**Reference practice** (behaviour only): established simulators keep pure-component correlations
as typed equation forms with unit-bearing coefficient sets. Binary-interaction data is held in
separate banks, selected per package and per pair. That separation is what F06 asks for.

## 10. Verification

| Claim / scenario / risk | Evidence label | Reasoning, test or measurement | Conditions and expected result | Result or gap |
|---|---|---|---|---|
| Baseline carries meaning in text | Interface-checked | Source read (slot 1) | — | Confirmed |
| The chemistry kinds are shape axes, not subjects | Interface-checked | `python3` summary of `physical.yaml`; search for the 221 IDs across the repository (outside `target/`, `external/`, `build/`) | 0 subjects; 221 species or element; 0 reaction; only `physical.yaml` and the generated projection | F07 |
| Stepwise rules carry scale and reference policies | Interface-checked | Rules listed from `physical.yaml` | Cp·T gives a difference with a declared reference | F02, F15 |
| The fallback for `Ref` dispatch exists | Implemented; covered by a named test (not run) | `kernel_table_function_references_dispatch_and_share_specializations` | 4 function specializations for 3 members | F13 |
| The proposed design meets DM1–DM6, CT-S01, CT-S03, CT-S10 | Proposed | The KR, D0 and S acceptances in Plan 23 | Per packet | Pending |

**Commands.** All were read-only:

- `rg` over `crates/` and `packages/reference`;
- `python3` JSON summaries of `packages/reference/physical/materials/physical.yaml`;
- `rg -l -F -f <221 shaped-type IDs>` over the repository.

**Checks.** No build, test, lint, codegen or benchmark ran, so there is no failure count to
report against the zero baseline. The design is accepted at no execution level.

## 11. Authority changes, exceptions and disposition

**Required changes to authority text, each through the wave-end `design:` route in Plan 23:**

- **§8.2.** "A spelling must name an admitted unit" becomes a unit product with a derived identity (F14).
- **§8.3.** Add the monomial rule. It is a declared rule family (the definition of a derived kind), not dimension inference. State its basis, scale and reference policy (F02).
- **§6.2.** Remove `authored.package_quantity_aliases`.
- **§6.4.** Record where the chemistry core lives (F07).
- **§5.3.** Add the new frames (F01).
- **§20.5.** Artifacts carrying an earlier source frame or IR version are `MigrationRequired`.
- **§22.1.** Add the `data/*.parquet` document kind and binary bundles (F09).

There are no SHOULD deviations and no exception records.

**Proposed dispositions**, for Plan 23's table:

| Finding | Scenario | Proposed disposition | Proposed owner |
|---|---|---|---|
| F01 | DM1, S01 | open | ADR-0123 and ADR-0125 text; KR3, KR9 |
| F02 | CT-S01, DM3 | open | ADR-0124 text; KR2 |
| F03 | DM2 | open | ADR-0123 text; KR4 |
| F04 | DM4 | open | ADR-0123 text; KR6 |
| F05 | DM6 | open | KR5 |
| F06 | CT-S03, DM5, DM2 | open | ADR-0126 text; D0 |
| F07 | — | open | ADR-0126 text; before SM0 |
| F08 | CT-S01 | open | KR8, D0 |
| F09 | DM1 | open | ADR-0125 text; KR9 |
| F10 | S01 | open | KR9 |
| F11 | CT-S10 | open | ADR-0123 text; KR7 |
| F12 | S02 | open | KR6 |
| F13 | SM5 | open | Plan 23 *Open items*; KR4 |
| F14 | CT-S01 | open | ADR-0124 text; KR1 |
| F15 | — | open | SM4 |

## 12. Decision
<a id="decision"></a>

### Updated decision after the resolution check

This decision follows the [resolution check](#resolution-check) of the amended texts, dated
2026-09-29.

**Behavioural adequacy: adequate at the Proposed level, with one scoped condition (N1).**
- F02 settles the PS-01 semantics of the monomial rule.
- F08 settles the datum authority.
- F04 settles taint granularity.
- F01 settles identity completeness.
- G1, G3, G6 and PS-G1 pass at the Proposed level.
- G2 passes within the scope that N1 sets.

**Architectural fitness: G9 passes at the Proposed level.**
- AP-02 is now satisfied (F01, F09, F14).
- AP-03 is now satisfied (F06).
- AP-04 is now satisfied (F03, F04, F08, F11).
- AP-05 is now satisfied (F02, F03, F05).
- AP-06 was already satisfied.
- AP-01 is satisfied on the condition that SM0 deletes every shaped type that names a moved or deleted kind (N2).

**Per record:**

- **ADR-0123: Accept** at the Proposed level. Before `status: accepted` (after which the record is immutable), correct E1:
  - the `verification:` field cites "the revision-identity tests of Outcome 9", but no Outcome 9 exists;
  - the name-binding identity case has no named packet test.
- **ADR-0124: Accept-scoped with conditions.** Chain resolution is accepted where no registered stepwise rule types a step. Where both apply, their results must agree, or the expression is refused (N1). This holds until the record states which route takes precedence. Also, say that a derived-kind declaration names its result *type* (for example `DeltaH`), not a kind.
- **ADR-0125: Accept** at the Proposed level. The F01, F09 and F10 resolutions are all in the text.
- **ADR-0126: Accept-scoped with conditions.** Placing the chemical core in `pse.domain` depends on SM0's control. In addition, SM0 must delete all 354 quantity types whose shape names a kind that moves (species, element, phase) or is deleted (`phase_species`) (N2). The alternative is to keep `phase` in `pse.physical`. The per-pair source limit of `pair_selection` is a declared scope, not a defect.

Acceptance covers the design only. Nothing here is implemented, and no packet closes because a
record is accepted. The named KR, SM and D0 tests carry the executed evidence.

| Priority | Change | Findings / scenarios | Acceptance evidence | Disposition owner |
|---|---|---|---|---|
| 1 | Delete every shaped type that names a moved or deleted kind (354), or keep `phase` in `pse.physical` | N2 (F07) · SM0 | `package_declared_axis_indexes_a_sum_without_a_registered_shaped_type`; physical admission; H1 unchanged | Plan 23 SM0 |
| 2 | State route precedence, or refuse disagreement between routes | N1 (F02) · CT-S01, DM3 | A KR2 test in which a registered rule and a chain both apply | ADR-0124 text, KR2 |
| 3 | Fix the dangling Outcome 9 reference; name the name-binding identity test; frame the quantity `name` into the physical inventory | E1 (F01) | KR3 or KR8 identity tests | ADR-0123 text, KR3, KR8 |
| 4 | Link this review's findings from Plan 23's disposition table | All | Plan table rows | Plan 23 |

### Initial decision, superseded by the check above

As first issued, both behavioural adequacy and architectural fitness were unresolved, with no
failed gate:
- ADR-0123: Revise (F01, F03, F04, F05);
- ADR-0124: Revise (F02, F14);
- ADR-0125: amend before acceptance (F01, F09, F10);
- ADR-0126: settle F06 and F07 before D0 and SM0.

The slot 1–11 observations record that state.

## 13. Resolution check (2026-09-29)
<a id="resolution-check"></a>

**Scope.** The amended ADR-0123–0126 and Plan 23 were reread in full on 2026-09-29. Only the
texts were checked; no build or test ran.

**Meaning of the dispositions.**
- **Resolved** means the Proposed design text now carries the correction, and a named packet test is assigned to show it.
- Implementation evidence stays with those tests (template slot 11). Accepting an ADR establishes no implementation.
- None of the dispositions is deferred.

**Counts** were checked again with `python3` over
`packages/reference/physical/materials/physical.yaml`, and with repository-wide `rg -l -F -f`
over the listed type IDs.

| Finding | Disposition | Amended text establishing it | Residual |
|---|---|---|---|
| [F01](#f01) | resolved | ADR-0123 Outcome 8 gives the three-part `ModelingSourceRevisionV2` preimage: structured rows; each data document's identity and byte-level content hash; and the physical-inventory identity replacing the alias map. It also gives a new catalog variant for every changed frame, dispatch-body and function-specialization frames included, with golden vectors. ADR-0123 Outcome 1 puts the structured IR in the relation version after H1's v7. ADR-0125 *Admission* adds the document hash to the source revision. KR3 carries these; KR9 has `data_bytes_enter_package_checksum_and_source_revision` | **E1 (editorial):** `verification:` cites a nonexistent "Outcome 9". The name-binding case of the compensating control has no named test. KR8's new quantity `name` must enter the physical-inventory preimage, a new variant under Outcome 8's own rule |
| [F02](#f02) | resolved | ADR-0124 *Derived kinds* makes eligible leaves differences, dimensionless factors and points of a true-zero ratio scale. A nonzero-datum point is refused with its factor named, and a chain is never partly resolved. Basis must be equal or declared. The result's scale, reference, subject and shape come from the declaration. KR2 has `seed_forms_type_without_intermediate_kinds`, `eligible_leaves_…`, `nonzero_datum_leaf_is_refused_with_its_factor` and `basis_must_agree_or_be_declared`. Checked against `physical.yaml`: `MolarCp` (additive, no datum) and a `Temperature` point (K, no datum) are eligible; `MolarEnthalpy` (with a datum) and gauge pressure are refused; `DeltaH` (a difference) is eligible | **N1 (Low):** the record does not say which route types an expression that a registered rule also matches, for example `Mul[molar_heat_capacity, temperature]`. Nor does it say whether disagreement refuses. §8.3 refuses when several rules match. Scoped in slot 12 |
| [F03](#f03) | resolved | ADR-0123 Outcome 2 frames identity over the key-declaring kind and typed keys, including defaults. Uniqueness holds across refinements, the concrete kind is content, enum members have identities, and dataset-supplied keys are declared bindings. KR4 has `same_key_in_two_forms_is_refused` and `moving_a_row_between_forms_keeps_its_identity` | — |
| [F04](#f04) | resolved | ADR-0123 Outcome 5: taint follows resolved row references, statically or at specialization, and is the one authority for this policy. The row-origin role is provided by the kernel. The sketch drops `admits`, and the `selection` `require` no longer restates the role. KR6 has `shared_relation_with_oracle_rows_keeps_production_roots_admissible`; DM4 is restated | — |
| [F05](#f05) | resolved | ADR-0123 Outcome 3 admits in three phases, with row-level cycles refused. KR5 has `self_referential_lineage_admits_and_cycles_are_refused` | — |
| [F06](#f06) | resolved | ADR-0126 adds `pair_selection[k,p] → Ref<source>` for pair and group data and makes the phase formulation a definition binding. The D0 corpus refuses a missing pair selection, and DM2 extends to pairs | A scope limit, not a defect: sources are chosen per package and property, so mixing banks within one property's pairs is not expressible. Revisit when a package needs it |
| [F07](#f07) | resolved in direction; N2 open | ADR-0126 *Chemical core in `pse.domain`* corrects the premise (221 types, shape axes, none of them subjects). SM0 confirms it with a control and deletes the unreferenced shaped types. A needed shaped type becomes a kernel-gap row. SM0's deletion list covers the 221 types | **N2 (Medium):** SM0 also moves `phase` to `pse.domain` and deletes `phase_species`. Yet 133 further types carry a `phase` or `phase_species` axis without a species or element axis, 354 in all. They are likewise referenced only by `physical.yaml` and its generated projection. After SM0 as written, these types name a kind declared above `pse.physical` or a deleted kind, and physical admission refuses them. The correction is to delete all 354, or keep `phase` in `pse.physical` |
| [F08](#f08) | resolved | ADR-0123 Outcome 6 reads the datum from the type and adds generic physical-reference types. ADR-0126 declares no reference attribute, the sketch drops `reference`, and KR8 adds `QuantityType` and `ReferenceState` | — |
| [F09](#f09) | resolved | ADR-0125 *Document kind*, *Units* and *Consequences*: documents are bytes; loaders, `package_checksum`, a binary store column and worker verification follow; units come only from the declaration. KR9 has `parquet_metadata_unit_disagreement_is_refused` and `durable_job_round_trips_a_package_with_a_data_document` | — |
| [F10](#f10) | resolved | ADR-0125 *Admission* makes admitted tables Salsa-tracked queries. KR9's test requires incremental to equal clean | — |
| [F11](#f11) | resolved | ADR-0123 Outcome 4: data declares envelopes only, the consumer selects extrapolation per layer, and forms declare the arguments or interval an envelope guards. The sketch uses `guards(cp.T, dh.[T0,T], ds.[T0,T])`. KR7 has `increment_guards_its_integration_interval` and `consumer_selects_extrapolation_per_layer` | — |
| [F12](#f12) | resolved | ADR-0123 Outcomes 1 and 5 make the registry enum hold facets only, with roles as package enum members declaring their facets. The sketch has `enum ProvenanceRole … facets(…)`; KR6 has `new_role_is_a_package_only_edit` | — |
| [F13](#f13) | resolved | ADR-0123 Outcome 2 requires a static `Ref`, uses the function-specialization path and refuses a non-static `Ref`. Plan 23 *Open items* is restated; KR4 has `non_static_ref_is_refused` | — |
| [F14](#f14) | resolved | ADR-0124 *Units*: a product's identity is a catalog-frame derivation over its canonical factors. KR1 has `report_in_a_composite_unit_carries_its_identity` | — |
| [F15](#f15) | resolved | The Plan 23 sketch declares `constant gas_constant: GasConstant`, and SM4 says "R typed with the existing gas-constant kind" | No alias `GasConstant` exists today. KR8 must name the existing `gas_constant` type in the physical document |

**Traceability.**
- Plan 23's *Finding dispositions* table still lists only the capability review's F01–F13.
- Its `review_sources` do not name this review.
- Under the binding, the plan's table is the current owner of these dispositions and of N1, N2 and E1. It should link them.

This is bookkeeping, not a design finding.

### 13.1 Correction from execution evidence (2026-09-29)

F07's observation that `subject_kind` is null on all 466 quantity types is wrong.
Plan 23 SM0 counted 177 types whose subject is species (146), element (29) or reaction (2),
six operations whose result subject is species or element, and two finite reductions over
species. After the 354 shaped chemistry-axis types were deleted, 34 subject-bearing types
remain, and they back live quantity names. The physical inventory is admitted from
`pse.physical` alone, so ADR-0126's placement of the chemical core in `pse.domain` was
inadmissible. ADR-0127 supersedes ADR-0126: the chemical core, including `phase`, lives in
a `chemistry` module of `pse.physical`. Register R-51 defers composing physical inventories
across packages. The other F07 conclusion stands, and SM0 applied it: the shaped
chemistry-axis types were unreferenced and are deleted.
