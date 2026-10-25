# Semantic model and authority

These findings concern two things: whether domain meaning has an adequate model, and whether
behavior consumes that model or each consumer reinterprets it. The
[collection README](README.md) gives the baseline, the standard and the limits. The
[integrated account](system-architecture.md) explains how these findings interact with the
other pages. Citations are `path::symbol` at source commit `b5bdac9da`; line numbers are
approximate where given. The code of these four findings is unchanged since `cbf45ccc9`. The
2026-10-25 notes record premise checks against the new consumers and expectations.

**Evidence dates.** Earlier *refresh* paragraphs retain the input account's source and scope;
*followup* notes and the [current revalidation table](README.md#revalidation-outcomes) identify
this engagement's re-readings and corrections. Earlier exact counts and measurements were not
renewed by republication.

## <a id="da-006"></a>DA-006 — Runtime mode formulation re-derives or fills in what the compiler and modeling owners publish

**Principles:** AP-04, DP-01, DP-04, DP-08, PS-07, PS-11; G1, G2, G6.
**Refresh status:** reframed and widened. The original diagnosis was annotations lowered with
different rules per mode; that core is resolved. The residues of DA-001, DA-002, DA-029 and
DA-031 merge here because they share this cause.

**2026-10-25 refresh: unchanged.** The compiler, modeling, `cases.rs` and `dynamics.rs` are
untouched. Plan 55's new model consumers page and compare authored declarations:
`ModelingPackage::{declaration_page, comparison_observation}` and
`workflow/modeling/canonical.rs::declaration_window`. They consume neither `target_kind` nor a
per-mode inventory, so they neither add to this finding nor relieve it.

**Diagnosis.** Plan 49 gave the compiler and modeling owners authoritative products:
- a resolved, typed executable syntax with symbol, binder and node arenas
  (`pse-modeling/src/specialize.rs::SpecializedModel::resolved`);
- a catalogue of modeling derivations (`pse-modeling/src/derived.rs`);
- compiler-owned numerical annotation plans keyed by mode
  (`pse-compiler/.../executable/policy.rs::NumericalAnnotations`);
- a compiler view policy (`compiler_view(ViewPolicy{..})`).

The annotation plans are mode-keyed but carry one mode-invariant classification.
- `projection_annotations` computes `hints`, and each hint's `target_kind` (declared variable,
  equation output, else observable), once.
- It clones the same hints into the Steady, Implicit and Dynamic plans. Only start precedence and
  a few mode filters differ.
- The compiler has no notion of a dynamic state, an algebraic row or a closure target.

The runtime's per-mode formulation (`workflow/modeling/cases.rs` for steady,
`workflow/modeling/dynamics.rs` for dynamics) then supplies the mode semantics separately.

- **Target classification.**
  - No runtime code reads `target_kind`.
  - *Steady mode duplicates it.* `cases.rs::hints` drops the field, then reclassifies nominal
    targets by set membership (`variable_states`) and adds observable targets separately
    (`resolve_numerics`).
  - *Dynamic mode fills a gap the compiler leaves.* It builds its own inventory of states,
    algebraic rows and closures (`dynamics.rs` ≈2205–2300). It silently skips a nominal hint
    outside that inventory with `continue` (≈2342).
  - So a nominal annotation on an observable output is honoured in steady mode and dropped in
    dynamic mode. §16.2 promises "intentional per-mode differences", but this one is declared
    nowhere.
- **Symbol identity by name.**
  - `cases.rs::require_frozen_expression` decides whether an accuracy goal or engineering scale
    depends on a changing solve coordinate.
  - It walks the authored `dsl::Expr`, which the struct documents as inspection-only, and maps
    paths to symbols by scanning every symbol and comparing synthesized `s_<hex>` names.
  - An unmatched name is skipped, so this refusal check can fail open if naming and resolution
    diverge. It is also the remaining whole-model scan of former DA-029, run once per free path.
    The dynamic row lookup at `dynamics.rs` ≈2250 is quadratic in the number of DAE rows.
- **Derived identities re-minted.**
  - Requirement lowering mints `named_id(id, "tighter-endpoint-budget")` and
    `"inventory-closure-requirement"`, then mints them again in the same module to recognize
    producers (`cases.rs` ≈1610/2567, ≈1620/2639).
  - `reconstructed-original-bound` rows are minted independently in
    `pse-backend-native/src/derived.rs` and `pse-runtime/src/math/solves/derived.rs`.
- **Role admission inline.** Case resolution still filters inner-unknown requirements and admits
  sensitivity and covariance roles inline (`cases.rs` ≈1100–1210), beside the compiler view
  policy. This is the residue of DA-031.

**Consequence.**
- *PSE-S01/S03* (add an annotation or target kind; extend accuracy goals to fit or shooting):
  the change needs coordinated edits in the compiler and in each runtime mode's assembly.
  - Changing the compiler's published classification changes no behavior, so today's steady
    agreement holds by construction, not by contract.
  - Consuming `target_kind` would not suffice for dynamics, which needs a mode-aware inventory
    that no owner publishes.
- A mode can silently drop authored meaning (the dynamic nominal skip).
- A refusal can fail open (the name lookup).

**Footprint.**
- *Decisive:*
  - `pse-compiler/.../executable/policy.rs::{NumericalAnnotation::target_kind, projection_annotations}`;
  - `workflow/modeling/cases.rs::{hints, variable_states, resolve_numerics, require_frozen_expression, resolve_prepared_case_with_current_view}`;
  - `workflow/modeling/dynamics.rs` (≈2205–2460).
- *Suspected:* nested-implicit `TrialHints`. Fit and shooting inherit through the steady and
  dynamic paths.

**Counterevidence and limits.**
- Dynamic mode genuinely has different targets: states, algebraic rows, closure over time.
- Policy resolution needs case values the compiler does not hold.
- Each lowering rule has exactly one function.
- Precedence and duplicate handling are compiler-owned. That was the original DA-006 core, and
  it is resolved.
- A positive refusal test of `require_frozen_expression` exists (`cases.rs` tests). Its fail-open
  path for an unmatched name has not been exercised, and neither has the dynamic skip.
- The second `LexicalLookup::bound` in the runtime source index
  ([DA-003](#da-003)) is a further instance of runtime re-interpretation.

**Evidence:** *Implemented* for the unused field, the skip, the name lookup and the re-mints.

**Improvement direction.**
- One owner publishes, per mode, the target inventory, classification, lowering inputs and
  declared mode differences.
- Runtime formulation binds values and consumes resolved identities.
- An annotation a mode cannot honor is refused or diagnosed, never skipped.

**Related:** [DA-003](#da-003) (expression roles), [DA-021](local-reasoning-and-assurance.md#da-021)
(formulation is evaluation-coupled and non-local), [DA-038](#da-038).

## <a id="da-003"></a>DA-003 — Expression grammar and role are restated by the selected-source inventory

**Principles:** AP-04, DP-01, DP-02; G1.
**Refresh status:** narrowed. The registry now declares grammar and role per field
(`pse.expr_dsl`, `pse.domain.expression.role`). The remaining inventories are unchanged.
At 2026-10-25, `selected_source.rs` and both `LexicalLookup::bound` implementations are still
untouched. The only change in `workflow/modeling/canonical.rs` is declaration paging.

**Diagnosis.**
- `pse-modeling/src/selected_source.rs` is byte-identical to the previous baseline. It keeps:
  - a hand-written per-arm text inventory (`requirements`);
  - reconciliation of occurrences by text value, with a lossy fallback on parse failure.
- The two `LexicalLookup::bound` implementations still differ, although the trait promises one
  lexical rule:
  - the checker's version (`pse-modeling/src/check.rs`) covers function arguments and scope
    parameters;
  - the runtime source index (`workflow/modeling/canonical.rs`) also binds `coordinate_map`,
    `response`, `reconstruction` and `reference_translation` arguments.
- `realization.argument` is declared as expression DSL, yet `specialize/forms.rs` reads it as
  `f64` for `derived_big_m` and `hull`.
- About 95 production `expression_at(id, "role.field", n)` calls pass literal role strings.
  These fail closed.

**Consequence.**
- Adding a field or a binding form needs synchronized edits to the selector inventory and to
  both lexical rules.
- A disagreement between the rules can bind a name in one consumer and not in the other. This
  is unexercised.

**Counterevidence:** lookups now return typed results from one checked-occurrence map.
**Evidence:** *Implemented*.
**Improvement direction:**
- the selector inventory and binding rules are derived from the registry's field roles;
- there is one lexical binding rule.

## <a id="da-005"></a>DA-005 — Physical meaning checked in Rust is not directly projected to public result readers

**Principles:** PS-01, PS-12, DP-02, DP-04; G2 at the readable-result contract. Physical
admission still refuses mismatches; this does not establish a PS-G1 admission failure.
**Refresh status:** narrowed, reframed and split.
- Physical side inputs are now keyed, quantity-typed Arrow rows, checked before custody
  (`workflow/physical_values.rs`).
- What remains is the readable physical meaning of identities and outputs.
- The canonical-key half (bare `str` keys, `forget_study_results(study: str)`) moves to
  [DA-007](structure-and-invariants.md#da-007), which shares its cause.
- **2026-10-25: core unchanged; premise reinforced.**
  - What improved: inspection v2 gives typed cells (`ResultInspectionCell::{Float64{bits,class}, ParentNull, MissingElement, …}`),
    keeps the non-value context fields and makes `context_complete` explicit.
  - What did not: unit and quantity identities still render as `ResultInspectionCell::Binary{hex}`
    with no resolver, and usability is always `Unknown`.
  - Plan 55c set itself the target of inlining "unit/dimension, basis/convention, identity,
    quality/usability". Its outcome records usability unknown and completeness false, and §24.2
    says it "does not establish complete physical interpretation". A functional expectation now
    explicitly asks for what this finding says is missing.
  - The MCP description of `native_contracts.BINDINGS["result_inspect"]` says the view "retains
    original identity, units, basis and quality context". That is true only as ids (DP-22).

**Diagnosis.**
- **Physical identities have no kind.**
  - The registry identity catalog has eleven scientific domains.
  - Units, quantity types, bases and reference states are typed only inside `pse-quantity`.
  - Generated documents and rows therefore carry them as interchangeable `SemanticId`s. For
    example, `BindingQuantity { quantity: SemanticId, unit: SemanticId }` can be swapped, and
    only Rust admission would catch it.
- **No public resolver.**
  - `PhysicalContext` exposes only an identity.
  - Inspection members carry no quantity or unit.
  - The Arrow formatter renders `unit:id:<hex>`.
  - The native result-inspection view (`workflow/result_inspection.rs`, ADR-0224, now v2)
    returns physical context as hex identities.
  - The repository's own Python fixtures get unit ids by parsing package YAML
    (`python/pse/tests/physical_input_fixture.py`).
- **Outputs carry no unit.** Examples: `committed_values -> dict[SemanticId, float]`, and the
  result filters `minimum`/`maximum`, which are compared in the stored unit with no unit
  argument.

**Consequence.**
- Python/MCP result views do not directly resolve a result's unit, basis or energy reference
  state. An executable project bundle is different: its exact authored physical documents retain
  those definitions and can be parsed to recover meaning. The gap is the direct reader projection,
  not categorical loss of meaning from every export.
- An input cannot be supplied in °C or bar by name without obtaining ids out of band.
- PS-01's recoverability question requires following the result's physical closure. The typed
  internal owners resolve it, but the public readable view leaves source lookup and interpretation
  to the caller. Opaque ids are not a direct human-readable physical context.
- *PSE-S04:* executable exchange preserves the authored closure; a result-only view must be
  distinguished from that self-contained source bundle.

**Counterevidence.**
- `project_closure.rs::resolve_project_closure` copies physical source documents and records their
  member/path mapping in `ProjectSourceClosure::physical_documents`.
  `ProjectBundleReader::package_sources` returns those authored bytes and import reconstructs
  `PhysicalContext`. The Python fixture already resolves quantity/canonical-unit ids by parsing
  authored source. These refute the earlier claim of impossibility from any exported bundle.
- Ids are exact and stable; symbols are presentation.
- Admission refuses mismatches, so this is not a silent PS-G1 failure.
- Plan 49 decision I20 found that the generated physical owners `BindingQuantity` and
  `CanonicalConversionPlan` already resolve the complete registered meaning, and adopted no
  independent unit table or resolver. That stands. The gap is that this owned meaning is not
  projected to readers.

**Evidence:** *Interface-checked* (absence searched in `python/pse`, `_native.pyi`, `pse-py` and
`result_inspection.rs`). The followup coordinator and challenger also read executable project-closure physical-document
copying and bundle source access. No live `inspect_result` was run. Such a call can test direct
readability, but cannot justify a categorical claim about all export formats.

**Improvement direction.**
- Physical ids become declared identity domains with typed Python counterparts.
- The existing physical owners (`BindingQuantity`, `CanonicalConversionPlan`) are projected to
  readers, giving symbol, dimension, basis and reference state for Python, inspection and
  exchange. This is consistent with I20; it is not a second resolver.
- Inputs can be addressed by declared name or symbol.

**Related:** [DA-007](structure-and-invariants.md#da-007) shares a root: identity domains stop
at the scientific core.

## <a id="da-038"></a>DA-038 — Scientific permission does not govern every aggregate, derived product and downstream reuse

**Principles:** AP-04, DP-01, PS-10, PS-12, AP-06; G1, G7, PS-G3.

**Refresh status:** new. It partly continues DA-021's concern with driver locality. The
store-vocabulary projections the draft first placed here (the terminal-class wildcard,
generator-owned projections and `PackScientificState::Unknown`) belong to
[DA-007](structure-and-invariants.md#da-007).

**2026-10-25 refresh: code unchanged; premise reinforced.** `numerics.rs`, fitting, the
horizon driver and the study owners are untouched. Native result inspection is a further
consumer that cannot obtain the producer's aggregate permission:
`result_inspection.rs` calls `reader.pack_selection(PackScientificState::Unknown)`. Every new
reader of stored results either re-derives permission or, as here, reports it unknown.

**Diagnosis.**

`workflow/numerics.rs` is a genuine strength: a pure, typed completion owner (`native_use`,
`original_completion`, `complete`). Its contract stops at one candidate, and three kinds of
consumer re-derive permission around it.

1. **Aggregate usability is restated with different conjuncts.**

   | Site | Conjuncts |
   |---|---|
   | `run.rs::RunResult::usable` | requires `report().is_ok()` |
   | `horizon/driver.rs` (≈1387) | accepts `outcome.is_ok() \|\| publication_failure.is_some()` |
   | `study.rs::stored_scientific_facts` | omits the report conjunct |
   | `study_operations.rs::scientific_facts` and `study.rs` | each rewrite the single-assessment candidate rule |
   | pse-py `NativeRunHandle::usable` | combines a stored termination cause with the re-derived conjunction |

2. **A top-level fit route bypasses the completion owner.**
   - `FitOptimizer::Automatic`, the default, selects the residual Levenberg–Marquardt route
     whenever `fitting/oracle/residual.rs::eligible` holds, without considering the declared
     `IncumbentPolicy`.
   - On that route, `fitting.rs::FitReport::candidate_use` returns before `native_use`, and
     refuses any non-successful termination whatever the policy.
   - The NLP route consults the policy. An `IterationLimit` NLP fit becomes seed-only under
     `Refuse`, and a qualified result under `AcceptFeasible`.

3. **Composite analyses write their own inner acceptance.**
   - Profile-likelihood pins (`fitting/profile.rs::PinFailure::assess`, `Chains::solve`) use
     their own termination and qualification sets and an LM gradient rule. They skip the
     estimate's model checks, and they drop its applicability qualifiers (for example permitted
     extrapolation) from published interval ends.
   - Horizon move authorization (Solved/Held/Fallback/Predicted) is decided inline in the
     2.4k-line async `horizon/driver.rs::control`.

4. **Completed fit mathematics is promoted through a different scientific permission boundary.**
   - `fitting/oracle.rs::FitProblem::execute_profile_with_lm` computes covariance and intervals
     before `fitting/modeling.rs::PreparedFit::execute_original` performs original model checks.
     Retaining those computations as diagnostic evidence is legitimate.
   - `run.rs::RunResult::invocation_evidence` explicitly requires the governing original
     qualification as well as successful computation for covariance/Wald product availability.
     The existing fixture
     `fitting/oracle.rs::original_fit_failure_withholds_completed_statistical_products_in_invocation`
     constructs a numerically feasible fit with a failed authored check, retains covariance and
     Wald values, and expects those products to be `Unavailable`.
   - The result projection uses a weaker rule. `fitting/results.rs::uncertainty` certifies local
     validity from `covariance.values.is_ok()` and exports the covariance and intervals without
     the governing original permission. Its own-prediction projection similarly propagates
     retained covariance and marks successful propagation certified.
   - `uncertainty.rs::RunResult::parameter_covariance` wraps those values as
     `CovarianceOrigin::QualifiedFit`. The persisted `uncertainty/sources.rs::Runtime::fit_covariance`
     hydrates covariance, scientific context and certified local validity, but does not read the
     governing original completion. Both preserve content identity and protect the actual input;
     neither carries the original scientific refusal.
   - `uncertainty.rs::propagate_checked` checks content authentication, PSD and coordinate
     compatibility. `Propagated::into_row` publishes `QualifiedFit`, the label "qualified fit"
     and empty external assumptions. No field restores the lost original refusal.

**2026-10-25 followup at `682fc3830`.** Item 4 is a missed producer-to-consumer consequence,
not an implementation change since the preceding audit. The initial independent scientific pass
found it before reading this catalogue; the coordinator re-read the fixture, invocation rule,
local and persisted covariance constructors, result projection and propagation output. Item 1's
`study_operations.rs` site was also checked: it consumes `result.usable()` for aggregate usability
and re-projects candidate/seed facts, rather than independently deciding the whole aggregate.

**Consequence.**
- A fit can be declared scientifically unusable and its covariance unavailable by invocation
  evidence, yet supply the same retained covariance as a qualified input to another analysis.
  The downstream result retains neither the governing refusal nor an explicit weaker,
  conditional certification scope. Content authenticity and mathematical PSD do not settle
  this scientific permission question.
- *PSE-S02.*
  - The same authored fit and numerical policy can receive different result and seed
    permission depending on which route automatic selection picked.
  - §16.6 ("every workflow consumes this decision; none re-derives acceptance") and §19.4 state a
    uniformity that does not hold.
- *PSE-S03.*
  - A new composite analysis (multi-start, robust design) has no composable inner acceptance to
    reuse.
  - A changed aggregate policy (for example partial acceptance) must find four or more sites in
    three crates.
- *PS-12.*
  - Published profile intervals lose qualifiers that the producer's typed decision holds.
  - A horizon reader can disagree with a stored reader about usability after a publication
    failure (source-derived, unexercised).

**Counterevidence.**
- Blueprint §19.4 deliberately separates point-estimate eligibility from covariance qualification.
  A failed authored expectation does not make the covariance calculation mathematically wrong.
  The finding is the disagreement between declared product availability and reusable scientific
  provenance, not a demand to conflate these distinct qualifications or erase retained evidence.
- `ParameterCovariance::verified_fit` correctly refuses forged or changed certified content, and
  external covariance has explicit provenance and PSD admission. DA-010's bounded correction
  remains valid; those controls do not establish the missing original permission.
- Pins rightly demand stationarity, and model checks at every pin cost real work.
- Refusing a non-converged LM estimate is conservative, and the route reports its library
  termination honestly.
- Predicted horizon moves are labelled, conditioned on the active set, and fall back.
- None of these differences is declared, and the stricter rules do not justify losing qualifiers
  or making permission depend on the route.

**Evidence:**
- *Interface-checked* for the missed derived-product permission chain and the existing fixture's
  assertions. That fixture and a downstream propagation journey were not executed in this audit;
  source establishes inconsistent rules, not a new runtime or numerical qualification.
- *Implemented* for each site. The coordinator re-read `candidate_use` and both usability
  conjunctions.
- The divergences are *Interface-checked*. Two runs would settle them:
  - an evaluation-limited LM fit under `AcceptFeasible` against `FitOptimizer::NativeNlp`;
  - a horizon with an induced publication failure.

**Improvement direction.**
- The completion owner publishes aggregate permission as a typed fact recorded with the
  completion.
- Library routes produce the shared native-outcome evidence so that `native_use` applies.
- Derived products and downstream reuse consume the same governing permission, or carry an
  explicit weaker conditional qualification and its reasons consistently through invocation,
  result rows, local and persisted inputs, and propagation. Retained diagnostic computations
  remain distinguishable from scientifically authorized products. A regression scenario should
  follow the existing rejected-original-fit fixture through both covariance consumers.
- A composable inner-candidate acceptance lets a composite analysis declare which obligations it
  requires, waives or strengthens, and how qualifiers propagate.
- Horizon and profile decisions are pure functions of explicit facts.

## Resolved, merged and moved identities

| ID | Refresh outcome | Where its meaning now lives |
|---|---|---|
| <a id="da-001"></a>DA-001 | Resolved in the core | The resolved typed IR owns executable syntax. The runtime name-lookup residue is a manifestation of [DA-006](#da-006). |
| <a id="da-002"></a>DA-002 | Resolved for modeling relationships | `pse-modeling::derived` and the `ModelingDerivedRelationshipV1` frame. The runtime re-mints merge into [DA-006](#da-006); store-key derivations merge into [DA-007](structure-and-invariants.md#da-007). |
| <a id="da-004"></a>DA-004 | Resolved | Artifact and owner kinds cross as generated enums (`python/pse/_artifacts.py`; `EnumValue<ArtifactOwnerKind>`). |
| <a id="da-031"></a>DA-031 | Resolved for view choice | `compiler_view(ViewPolicy)`. The inline role admission merges into [DA-006](#da-006). |
| <a id="da-032"></a>DA-032 | Narrowed and merged | Relation names in result projection are typed. Artifact-profile membership by literal names (`pse-schema/src/catalog/products.rs`, `workflow/artifacts.rs` ≈897–984) is a manifestation of [DA-007](structure-and-invariants.md#da-007). |

The removal commit of this refresh preserves the 2026-10-21 text of these findings.
