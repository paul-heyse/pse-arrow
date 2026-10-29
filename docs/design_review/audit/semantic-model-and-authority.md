# Semantic model and authority

**Interface-checked. Refreshed 2026-10-29 at source `f83bac039` plus the working tree.** The
decisive files were clean and unchanged through `f3a05e0be`.

This page holds the primary causes in which scientific or modeling meaning is decided in more
than one place:
- the lookup rule ([DA-003](#da-003));
- formulation obligations interpreted in orchestration ([DA-006](#da-006));
- derived-product permission ([DA-048](#da-048));
- lossy permission summaries at aggregation and portability boundaries ([DA-049](#da-049)).

The [integrated account](system-architecture.md) relates them.

**What changed since `682fc3830`.** Plan56 corrected several of this page's earlier diagnoses:
- DA-005: physical context is projected to readers;
- DA-038: covariance reuse consumes the governing completion permission;
- compiler-owned mode targets.

Two identities, DA-003 and DA-006, are reopened. Two unanchored streams found that their causes
survive in other forms.

## <a id="da-048"></a>DA-048 — Only fit statistics have an owned product permission; other derived products are qualified by their consumers

**Principles:** AP-04, DP-01, DP-05, PS-12, PS-10; H19.

**Status:** new on 2026-10-29. It is the successor to the scope DA-038 left open: DA-038 is
resolved for covariance, and this finding covers the derived products outside it.

**Claim at risk.**
- §19.2: a product permission combines the run's original permission with the product's own
  mathematical validity, and "consumers never reconstruct its conjunction".
- Plan 56b B01 (recorded complete): local, durable and public consumers agree, with no
  duplicated conjunctions and no `Unknown` when the actual permission is available.

**Structural cause.**
- `RunResult::capture_product_permissions` fills `Completion.products` only for fits, and there
  only for parameter covariance, Wald intervals and profile intervals. For every other outcome it
  returns early (`workflow/completion.rs:200-207`, cases at `:225-268`).
- Modeling-step propagated covariance is a statistical product, yet it has no recorded product
  permission. `run.rs:667` hard-codes it `Unavailable`, and the lazy projection computes it later
  (`local_analysis.rs:257`).
- Modeling-step parametric sensitivities and reduced Hessians are permitted only through a
  consumer's conjunction (`run.rs:620-641`).
- `local_analysis::Rows::push` (`workflow/local_analysis.rs:147-288`) writes product-local
  `local_validity` from KKT/root evidence. Keeping that independent of original permission is
  §19.2's explicit design, and inspection exposes `governing_permission` beside it
  (`result_inspection.rs:138-143`).
- A fit's own-prediction propagation *is* gated on the governing covariance permission
  (`fitting/results.rs:1001-1016`), as §19.8 requires.
- No registry relation carries the combined permission. Only `candidate_assessments.permits_result`
  is per step, and the completion's coverage, report and no-candidate refusals exist only inside
  the Completion document (`completion.rs:66-110`).

**Consequence.** Consumers reconstruct the verdict, and they reconstruct it differently:
- `run.rs:620-641` combines `original_permission && produced` itself.
- `run.rs:667` reports propagated covariance as always `Unavailable`, even when the lazy
  projection later certifies it.
- `result_inspection.rs:1027-1037` keeps its own relation → quantity table:
  - modeling-derived relations come out `Unknown`;
  - mixed Wald/Profile `parameter_intervals` rows get a single verdict.
- `analyses.rs:811-850, 1043-1055` labels analysis edges "retained" from local validity alone.
- Relational (DataFusion) consumers cannot obtain the governing step or product permission
  without decoding the completion document. Pairing product-local validity with governing
  permission is therefore left to each consumer, and only the inspection API does it.

**Fitted-parameter publication is a related manifestation.** It is the one derived product that
re-enters model authority, and it is gated by a fit-local predicate rather than the governing
permission:
- Publication requires `FitReport::estimate_qualified()` (`workflow/fitting.rs:599-624`; checked at
  `fitting/results.rs:1196-1202, 1218-1222`).
- `research_publication.rs:140-240` reads cells and source receipts but never the stored completion.
- It writes the fitted value as authored DSL text and re-admits it through ordinary admission
  (`:300-316`), which is a strength. The authored parameter's applicability claims and evidence
  are left unchanged.
- Covariance reuse, by contrast, requires the product permission plus reconciliation
  (`uncertainty.rs:392-445`, `uncertainty/sources.rs:72-130`).
- Two predicates thus decide "qualified fit". They can plausibly diverge in both directions, for
  example "QualifiedUnclosed" under a closure-allowing policy, or a policy refusal. That
  divergence is a hypothesis that needs a probe.

**Footprint.**
- Known: modeling-step derived products (propagated covariance, sensitivities, reduced Hessians);
  fitted-parameter publication.
- Suspected: any SQL or cohort reader of `local_validity` or `propagated_covariances`.

**Counterevidence.**
- §19.2 limits recorded product permission to "statistical products", and §19.8 declares
  propagation lazy.
- `step_permissions` exist, so a careful consumer can combine them.
- The analysis graph disclaims usability.
- `estimate_qualified` adds rank/identifiability and is stricter in most respects.

Against these: propagated covariance is itself statistical, and B01 explicitly targeted
duplicated conjunctions.

## <a id="da-049"></a>DA-049 — Study, review and portability boundaries summarize permission in lossy, unenforced secondary vocabularies

**Principles:** DP-01, DP-02, DP-03, PS-12, AP-02.

**Status:** new on 2026-10-29.

**Claim at risk.** Qualifiers and refusals are carried through every derived or exported product
without strengthening. `PackScientificState` itself says "publication never upgrades it"
(`workflow/result_selection.rs:46-47`).

**Structural cause: studies.**
- `pse_model::study::ScientificFacts` (`crates/pse-model/src/study.rs:151-163`) reduces permission
  to `usable: bool`, an optional `candidate_use` and `seed_permission`.
- `candidate_use` is present only when exactly one assessment exists, so `None` conflates
  "several" with "absent".
- The projection is written twice in the runtime (`crates/pse-runtime/src/workflow/study.rs:502-519`
  `stored_scientific_facts`; `workflow/study_operations.rs:1313-1323`).
- Qualified-unclosed, extrapolation-allowed and accuracy-estimated outcomes all collapse into
  `usable = true`. The study summary and cohort membership consume that flag.

**Structural cause: review and portability.**
- `PackScientificState` (`result_selection.rs:46-64`) has five states and no qualifiers. Project
  bundles carry it as the "original result qualifiers" (`project_portability.rs:171-172`).
- The production review export always writes `Unknown` (`connected_results.rs:1283`).
- The public `pack_selection(usability)` accepts any value the caller supplies.
  `validate_pack_selection` compares every field except usability (`connected_results.rs:586-600`),
  and `tooling/review_pack.rs:1141-1149` relies on that validation.
- `durable.rs:2456-2472` derives Completed/Partial/Failed from permission and stores it as a
  `terminal_class` string.

**Consequence.**
- Study aggregates and exported bundles report usability without honest scope.
- The "never upgraded" promise is a doc comment, not an enforcement point.

**Counterevidence.**
- Plan 47d deliberately does not promote whole-attempt success to selected-product permission, so
  `Unknown` is intended.
- Study points refer back to retained runs whose completion is intact.
- The open `pack_selection` is Rust-only, and the Python route always passes `Unknown`.

**Interaction.** [DA-048](#da-048) is why these boundaries can say only `Unknown` for most
products. The missing relational projection of the governing permission is common to both.

## <a id="da-006"></a>DA-006 — Runtime orchestration interprets formulation obligations that modeling and compiler products already determine

**Principles:** AP-01, AP-03, DP-01, DP-02, DP-03, PS-03, PS-07; AP-02 (C5).

**Status:** reopened and re-scoped on 2026-10-29 against Plan56's resolved disposition (A02).
Plan56 resolved its original manifestations:
- the compiler owns complete mode targets (`executable/policy.rs`);
- absent nominal targets refuse (`workflow/modeling/dynamics.rs:2368-2385`);
- re-mints are centralized.

An unanchored preparation stream (U8) found that the cause, orchestration deciding what the
model contract determines, survives in obligation lowering, fixture policy and the breadth of
the specialized product.

**Claim at risk.** §14.3 and §16.2 (ADR-0215): the runtime "consumes compiler-owned
annotation/view products … instead of reconstructing target sets … or minting parallel derivation
recipes". PS-07: scaling and tolerances are model attributes.

**Structural cause.**
- **Obligation lowering.** `workflow/modeling/cases.rs` reads raw `SpecializedModel` fields
  (annotations, engineering rules, closures, state specifications, objectives) and itself:
  - lowers `AccuracyGoal` and `EngineeringScale` annotations into numerical-policy rows
    (`lower_authored_accuracy`, `:1857-2110`);
  - materializes engineering rules and detects conflicts;
  - checks objective membership and quantity agreement (`require_same_contract`,
    `:2271, 2315, 2327`);
  - converts closure, conservation and state-reconstruction tolerances into requirements
    (`lower_closure_requirements`, `:2372`; `conservation_row_requirements`, `:2663`;
    `state_reconstruction_requirements`, `:2708`; `closure_budgets`, `:2759`).
- **A duplicated invariant.** The positivity of closure tolerances is enforced again in
  `pse-modeling/src/specialize/process.rs:120-126`.
- **Two compositions.** Steady (`cases.rs:1630-1740`) and dynamic (`dynamics.rs:2420-2490`)
  assemble the sequence separately, and only the steady path strips and replaces earlier producer
  IDs.
- **Fixture policy parsed at use.** `workflow/modeling/declared.rs:9-110, 355-400` reads authored
  declaration row cells into backend, presolve, native options and limits, with its own refusals.
  An invalid fixture policy passes package checking and specialization and fails only when a
  declared run reads it.
- **An omnibus product.** `SpecializedModel` is one public struct of about 45 fields: topology,
  equations, closures, fixtures, objectives, regimes, meshes, requirements, annotations and a
  string `paths` map (`pse-modeling/src/specialize.rs:469-578`). It has no consumer contract.
  About 27 runtime files read `product.model.*` directly.

**Consequence.**
- A new authored numerical obligation needs coordinated edits in modeling, compiler policy and
  both runtime compositions.
- Model-contract invariants are first enforced when a case runs, not at preparation.
- Any internal representation change in pse-modeling reaches runtime orchestration directly.

**Counterevidence.**
- Accuracy and scale *values* need current case observations, which stay outside Salsa by design,
  so some binding must happen at runtime.
- The compiler does own the target inventory, and the runtime refuses targets it does not know
  ("no compiler numerical owner", `cases.rs:2211`).
- Solver backend and native options are runtime/native vocabulary that pse-modeling cannot depend
  on.

The judgment is that validation, conflict policy and requirement construction go beyond value
binding. They are pure functions of compiler and model products. The per-mode difference in
producer-ID replacement was not resolved from source.

## <a id="da-003"></a>DA-003 — The declaration lookup rule is authored in several places across modeling and runtime selection

**Principles:** DP-01, AP-04, DP-03, DP-04.

**Status:** reopened on 2026-10-29 against Plan56's resolved disposition (A01). Plan56
(`314a6b961`) made the checker and the
selected-source walk share `LexicalLookup` and `lexical_bindings`, and registry occurrences now
supply grammar-bearing fields. An unanchored stream found that the rule itself is still written
independently.

**Claim at risk.** A path in authored text resolves to one declaration for selection, dependency
extraction, checking, provenance and specialization. Selection completeness, and therefore reuse
soundness, depends on that agreement.

**Structural cause.** Only the recursion in `selected_source::resolve_segments` is shared. The
rule is authored three times, and a name-encoded identity is parsed back alongside it:
1. `CheckedPackage::resolve` / `resolve_with_members` walks imports with string alias-prefix
   tails, builds qualified names by `format!`, and checks lexical children and ancestors. It has
   no binder-shadowing step (`pse-modeling/src/check.rs:833-877, 908-1040`).
2. `CheckedLexical` / `declared_member` has its own recursive traversal of guarded `When`
   children (`check.rs:2329-2370`).
3. The runtime `SourceIndex` behind the same trait has its own member candidates, guard
   recursion, base recursion and import resolution
   (`workflow/modeling/canonical.rs:1521-1700`). It deliberately returns a conservative superset
   for selection before admission.
4. Lowered functions are addressed by a name-encoded `f_<hex>` identity, minted in
   `specialize/reductions.rs:30-36` and parsed back in `check.rs:948, 983` and
   `check/admission_view.rs:82-103`. Its identity aspect has its primary home in
   [DA-050](structure-and-invariants.md#da-050).

The per-arm structural requirement inventory remains hand-listed (`selected_source.rs:128-150`).

**Consequence.**
- A language change (inheritance, guards, imports, shadowing) must be made consistently in two to
  four owners across two crates.
- The runtime superset relationship is unenforced. A missed member yields "unknown member"
  refusals, or reuse gated on incomplete premises.
- The `f_` prefix gives identity meaning to names.

**Counterevidence.** Selection before admission must run over storage-backed objects with lazy
fetch and recorded premises. §14.4 and ADR-0183 describe that design, and over-approximation is
safe while it really is a superset. No behavioral divergence was established; the diagnosis
concerns where the rule's authority lives.

## Resolved, merged and moved identities

| ID | Outcome at `f83bac039` | Decisive evidence |
|---|---|---|
| <a id="da-001"></a>DA-001 | Resolved (2026-10-25) | The typed resolved IR owns executable syntax |
| <a id="da-002"></a>DA-002 | Resolved (2026-10-25) | `pse-modeling::derived`; runtime re-mints later centralized (`pse-ids/src/derive.rs`) |
| <a id="da-004"></a>DA-004 | Resolved (2026-10-25) | Generated artifact and owner enums |
| <a id="da-005"></a>DA-005 | Resolved | `ResultPhysicalContext` and `View::Physical` through the physical owner (`workflow/result_inspection.rs:138-171, 235-238`; `physical_inspection.rs:11-36`). Key-string residue is under [DA-007](structure-and-invariants.md#da-007) |
| <a id="da-031"></a>DA-031 | Resolved for view choice (2026-10-25) | `compiler_view(ViewPolicy)` |
| <a id="da-032"></a>DA-032 | Merged (2026-10-25) | Literal profile membership is under [DA-007](structure-and-invariants.md#da-007) |
| <a id="da-038"></a>DA-038 | Resolved for covariance | `completion.product_permission(ParameterCovariance)` withholds reuse (`workflow/uncertainty.rs:401-422`), consumed by sources, fit results, durable and inspection. Uncovered derived products are now [DA-048](#da-048) |

Earlier full text is at source `682fc3830`. DA-030, the preparation reuse mechanisms beside
Salsa, stays closed by decision on the [execution-fit page](execution-fit.md#da-030). This refresh
found several parallel premise and key mechanisms there, but no stale reuse.
