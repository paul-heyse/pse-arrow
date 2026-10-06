---
title: "27a: Accuracy intent and contextual policy"
status: in-progress
date: 2026-10-05
adrs: [ADR-0163]
review_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md"]
scenario_sources: ["../design_review/reviews/design_review_contextual-accuracy-policy_2026-10-05.md#scenarios-that-distinguish-the-alternatives"]
---

# 27a: Accuracy intent and contextual policy

## Purpose, baseline and boundary

Supply the working semantic prerequisites for CA-F01 and CA-F02: what engineering output
or decision matters, which physical resolution applies, and how shared defaults respond to
an admitted context. The [coordinator](27-contextual-engineering-accuracy.md) owns shared
decisions. [25k](25k-integrated-qualification-and-closure.md#contextual-accuracy-review-dispositions)
owns finding status. This companion owns A0–A3 progress.

The current resolver already performs physical magnitude conversion, field precedence,
conflict rejection, ID binding and immutable provenance. Its default absolute tolerance and
continuous coordinate scale both use nominal, while relative is initially zero. Keep that
resolution machinery; replace the default engineering rationale. `term_scale` remains useful
conditioning information, including its all-zero fallback, without becoming an output-accuracy
authority. Existing explicitly authored numerical budgets retain their current meaning.

## Goal declaration and admission

Declare the semantic representation once in the registry and expose its generated authored,
Rust and Python forms. Authored goals and request additions/overrides consume the same meaning;
the compiler binds expressions and locations to the existing mathematical program. No second
expression evaluator or Python tolerance interpreter is introduced.

The conceptual `AccuracyGoal` carries:

| Field | Required meaning |
|---|---|
| Identity and target | Stable goal identity; selected scalar numerical expression/observable, or the original optimal objective |
| Observation | Steady value, admitted sample time, actual admitted endpoint, or final value of an already-declared integrated observable |
| Resolution | Optional positive physical error magnitude; required for a value-only goal |
| Criterion | Optional inclusive upper limit, inclusive lower limit or closed band; at least resolution or criterion must exist |
| Evidence strength | Estimated by default, or explicitly Certified |
| Use policy | Assess by default, or RequireSatisfied; the latter requires a criterion |
| Refinement permission | Enabled by default within the enclosing strategy/task grant; false permits assessment only |

An optimal-objective goal names the optimum value rather than the incumbent value; it requires
actual qualified original-objective bounds. Other goals concern the selected solution/output,
not every possible solution or a global optimum. Explicitly retain this subject in binding and
evidence so that a primal/dual gap cannot certify an arbitrary state variable.

Value resolution is the maximum admitted numerical error of the protected output. A
criterion-only goal instead requires sufficient evidence to resolve its decision; it does not
automatically demand an additional number of output digits. A combined goal requires both.
Default contextual solver/physical acceptance still applies in every case.

Admission validates scope, scalar numeric meaning, supported observation, positive finite
resolution, physical compatibility, ordered finite limits and evidence class. Limits for an
affine point use point conversion; errors/resolutions and differences use magnitude conversion.
A comparison of two physical points binds their legal difference and the difference's error.
No implicit molar/mass conversion, absolute-temperature-as-interval conversion or dimensionless
casting is allowed. A singleton band expresses deliberate exact equality; never synthesize
an equality tolerance from numerical resolution.

An approximate equality `q ≈ c` with engineering tolerance `h` binds `[c-h,c+h]`. `h` and
numerical resolution are independent. No general Boolean check, fixture expectation, conservation
declaration or postsolve publication request is automatically promoted into a goal.
Original check semantics remain authoritative in their own scope.

Select authored goals with the model/case and admit request-local goals before preparation.
A request may override an authored goal using its identity, preserving target/location and
recording the source; changing the target requires a new goal identity. Duplicate same-source
identities or conflicting equal-precedence declarations fail admission. Request additions do
not enable unselected expressions. A missing or unsupported target fails before native startup,
whereas a supported target with unavailable error evidence completes with an unresolved goal.

No selected goals means NotRequested, retaining ordinary qualification. Requested but missing
observations/evidence means Unresolved, never NotRequested. Unchanged historical requests may
admit an empty goal set when their supported codec is read; historical result evidence never
acquires a new goal guarantee.

## Contextual default resolution

Resolve three independently sourced inputs: typed shared physical allowance `F`, shared relative
engineering fraction `r`, and nonnegative engineering characteristic magnitude `S`. Use
`B = max(F, r*S)` for default physical acceptance; lower `B` as default absolute with relative
zero into the existing numerical resolver. All arithmetic must remain finite and in the target's
physical magnitude coordinates. This calculation is not an output-error certificate.

The initial `r` is the existing shared `1e-3`. The shared reference physical allowances remain
those in `packages/reference/domain/models/numerical-policy.pse`: 0.1 K, 1 W for each separately
typed power/transfer-rate meaning, 100 Pa, 100 J, 0.001 mol or mol/s for their distinct inventory/
flow bases, 1 mol/m³ for their distinct density/concentration meanings, and the existing
dimensionless/volume-flow slots. Keep the source's full typed declarations as their sole value
owner. These settings are provisional engineering defaults, not fitted optima. Do not invent a
mass-inventory slot or derive one from moles; when a mass-based model supplies its own meaning,
the maintainer's approximately 1 kg guideline can be an explicit shared declaration.

Declare registry-owned engineering default/scale records, reused by request policy and authored
bindings. A selected policy binds physical allowances by full quantity meaning and applicability;
same SI dimensions alone do not match a rule. The shared model-owned relative default remains
one declaration. Expose policy-level override of the fraction and individual physical defaults,
and target-specific scales or numerical overrides; do not author a new policy per fixture.

Preserve the distinction between an inherited shared default and an explicit physical number.
An authored `engineering(<shared rule>)` selector names the typed shared rule and is lowered
as a default reference, not eagerly converted into an explicit absolute override. The working
registry records that identity. Migrate the reference models' inherited reconstruction, balance,
transport, inventory and comparison slots to that selector; distinct numeric declarations keep
their override meaning. Otherwise an explicit old floor would accidentally suppress contextual
scaling everywhere. Integration's existing global local-control selectors keep their separate
native-local meaning and do not acquire output-error claims.

Reference comparisons keep their independently specified expected scientific values. Their
inherited allowance selects the shared contextual rule through the production resolver bound to
the fixture's admitted context; it is not fitted to observed output differences. A tighter
independent verification comparison remains explicit and justified. Comparison declarations do
not automatically create production goals or a second acceptance implementation.

### Engineering scale sources

| Priority | Admissible source and interpretation |
|---|---|
| 1 | Explicit analysis engineering scale |
| 2 | Selected case engineering scale |
| 3 | Explicit model engineering scale |
| 4 | Bound provider engineering scale with applicable region/meaning |
| 5 | Declared operating-range width, rated capacity or reference difference explicitly tagged for engineering accuracy |
| 6 | Quantity characteristic magnitude explicitly declared for engineering accuracy |
| Fallback | Applicable physical allowance alone; otherwise recorded canonical engineering fallback |

These ranks belong to scale resolution, not a replacement for existing numerical field ranks.
Tag a scale with its physical meaning and basis: magnitude/capacity, range width or reference
difference. A scale expression may consume fixed specifications, parameters, admitted endpoints
or capacity declarations known during preparation. A dependence on an unknown solution coordinate
cannot provide a frozen scale; record it as unavailable and use the lawful fallback. Arbitrary
numerical bounds, broad validity envelopes, raw guesses, gross additive terms and untagged
conditioning nominals are not implicitly engineering scales.

At equal precedence reject conflicting applicable declarations; never choose the largest
available magnitude. Preserve selected and overridden provenance. A declared range has finite,
ordered physical endpoints and uses their difference. A reference difference uses an admitted
physical reference. Affine point values never use their raw datum as `S`.

`S=0` is valid and yields `F`; normalization scales remain strictly positive. A cancelling net
output uses its own declared net range/characteristic scale or the shared physical allowance.
Its large opposing terms continue to help conditioning but cannot silently enlarge the output
allowance. If only an applicable `F` exists, use it without inventing a scale.

If neither a physical allowance nor a meaningful scale is available, retain the existing
canonical-unit starting allowance with an explicit canonical-engineering-fallback limitation.
The default policy permits this honestly scoped ordinary result; `strict_engineering_context`
refuses it before execution. This flag is separate from `strict_nominals`. If a meaningful `S`
exists but no `F`, use the positive `r*S`; zero without `F` requires the canonical fallback or
strict refusal. Invalid declared inputs are admission errors, not occasions for fallback.

### Overrides and lifecycle

Explicit numerical requirements still override default physical feasibility fields through
the existing per-field source ranks and `absolute + relative*nominal` semantics. They do not
implicitly set a goal, an engineering scale or an integrator's local error weights. Explicit
goal resolution may be finer than `F`: the physical allowance is a default, not a lower limit
on possible requested accuracy. Integrality, rank cutoffs, finite-difference steps, interval
rounding, KKT and gaps retain their distinct meanings.

Freeze engineering defaults, scale bindings and acceptance budgets once for each admitted
analysis. A study member with changed case/parameters admits a new context. A dynamic trajectory
retains its admitted horizon/context; native state-dependent weights do not alter outer
acceptance. Input-dependent nested producers may bind new local products at an outer trial,
but cannot loosen the enclosing frozen goal or reinterpret its scale.

Retain inputs, source identities, scale basis, selected physical units, fallback reason,
effective allowance and goal binding in request/preparation identity and persisted interpretation.
Changed context/default/criterion/strength changes admission identity. Attempt-specific tighter
work demands have their own identity and never mutate this frozen acceptance.

## Packages and local acceptance

| Package | Prerequisite | Behavior, owner, retirement and acceptance | Progress |
|---|---|---|---|
| A0 — Semantic proposal | Coordinator decisions and review S01–S06 | Coordinator settles the declaration, source selection, goal-use truth table and support boundary here; C0 supplies required decision records before production changes | done |
| A1 — Working declarations/admission | A0/C0 contract route | Schema/authoring/model owner adds registry-owned goals, engineering defaults/scales and request forms; compiles protected expressions with current scientific authority; validates independently of native startup. Retire competing hand-authored boundary forms in the same change. Targeted `accuracy_goal_admission` controls | done |
| A2 — Contextual resolution | Working A1 physical and source contracts | Model/math policy owner resolves independent scales and lowers default B; migrates reference shared slots without per-fixture tuning; retains explicit numerical interpretation and provenance. Delete nominal-as-default-engineering-scale mapping. Targeted `contextual_engineering_resolution` controls | done |
| A3 — Binding and identity | Working A1/A2 plus C0 durable inventory | Preparation/identity owner binds goal dependencies and locations, freezes context, versions relevant frames/requests and migrates their immediate consumers. Delete old keys that omit consumed meaning; regenerate changed registry products. Targeted `accuracy_context_identity` controls | done |

**Proposed controls:** admission succeeds for value-only, decision-only and combined goals;
wrong units, affine magnitude misuse, conflicting sources, unselected/non-scalar targets and
unsupported locations refuse. Independently specified upper/lower/band goals preserve limits.
Equivalent units and affine datum changes preserve the physical allowance. Zero, missing scale,
explicit trace resolution, cancelling terms and tenfold capacity changes exercise the rule and
fallback. Changing only conditioning nominal does not change contextual engineering B. Explicit
numerical relative overrides preserve their original sum. Shared default changes affect inherited
slots but not distinct overrides. Identity changes when consumed context changes and remains
stable under irrelevant declaration ordering.

Use `just check-package` and `just unit-package` for touched packages, with recipes' explicit
force-validation; run `just codegen` as part of declaration changes. These controls exercise
the actual resolver/assessment operations with semantic inputs, without a native runtime or
operational store. Assembled publication and native qualification belong to 27c/25k.

## Verification and checkpoint

**Implemented / Interface-checked:** existing resolver, magnitude projection, registry-owned
numerical rows and shared reference policy inspected at the coordinator baseline.
The A0 contract is settled through ADR-0163 and the current-standard contracts review;
the ADR remains proposed pending its decision PR. A1–A3 are implemented in the current
tree: registry-owned declarations, authored admission, contextual resolution, frozen
provenance and generated boundaries are in place. Focused resolver/admission controls
have passed, including independence from normalization and operational work context.
Selected computed-output binding consumes B3's original-program evidence. Final consumer and
assembled migration checks remain with C/25k; these local results are not qualification.

## Outcome (recorded after implementation)

Record implementation, a mistake corrected, deliberate deviations and local evidence here;
link CA-F01/CA-F02 dispositions at 25k rather than duplicating finding status.
