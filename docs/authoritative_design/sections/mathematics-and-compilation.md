---
title: Mathematics and compilation
status: current
---

# Mathematics and compilation

This area turns an admitted, physically typed process definition into immutable
library-owned mathematical programs and a case layout that native solvers can consume.
Authored definitions and bindings stay the model authority; Symbolica/Numerica own
arithmetic, normalization and derivatives, and faer owns sparse structure. `pse-math`
holds the narrow library integration, `pse-modeling` owns generic checking and
specialization, and `pse-compiler` owns library lowering and pure Salsa preparation, and `pse-runtime::math` owns effectful artifact construction, retention
and attempt workers. `pse-authoring::dsl` supplies the authored expression syntax.

## 7. Library-owned process mathematics

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md)

A definition is lowered once, directly from its parsed syntax to Symbolica atoms, inside a
bounded per-specialization builder. There is no model-wide expression graph, persisted
expression relation or project-owned arithmetic interpreter. The project owns only what
libraries cannot know: physical meaning, original-domain obligations, branch selection,
fallible provider calls and source attribution.

### 7.1 Design constraints

| Constraint | Consequence |
|---|---|
| Real algebra on an admitted domain | Physical checks and domain obligations precede algebra. Equivalence is real-algebraic on the admitted domain, not ordered floating-point or bitwise identity. There is no approximate or finite-difference fallback. |
| Library algebra starts at the typed definition | `pse-compiler::typed_math` lowers the authored AST through `pse-math::typed::BodyBuilder`; no intermediate IR is produced or stored. |
| Specialization-local bodies | A body covers one definition and its consumed finite membership, never the whole model. Identical bodies are shared across instances; a changed shape can require new compilation. |
| Libraries own mathematics | Symbolica/Numerica own arithmetic, normalization, derivative composition and evaluators; faer owns sparse patterns and products; native solvers own iteration ([§18](numerical-execution.md#section-18)). |
| Explicit effects | Symbolic initialization, native compilation and evaluation are effects outside tracked Salsa queries ([§14.3](#section-14-3)); service-local retention/accounting effects preserve admitted meaning and cannot memoize transient refusal; the one process-global effect a tracked query may cause is the append-only growth of the formal-symbol pool (below). |

Unknowns are symbols bound to case variables, never nulls in a value column. Every scalar
input and output carries a complete physical type ([§8](physical-semantics.md#section-8)),
and every expression occurrence keeps a source identity for diagnostics.

`pse_math::initialize` is the single explicit entry. It supplies the optional
`SYMBOLICA_LICENSE` environment value to the library once, disables library tracing and
registers the first chunk of the formal-symbol pool (8,192 formal symbols and 8,192
function symbols for provider lifts) in a fixed order, independent of model arrival order.
It also captures the linked GMP/MPFR runtime versions, whose identity enters compiler
inventories and artifact keys.

The pool then grows lazily. The first request for a slot beyond it registers whole chunks
of 8,192 (`pse_math::library::FORMAL_CHUNK`) until the slot is covered. Growth is
append-only and serialized: a chunk is appended only once all of its symbols registered,
and chunk *k* registers its formals and then its functions, so the registration order of
any two pool symbols is the same whenever, and from whichever query, a chunk registers, and
a registered symbol never changes. The pool has no ceiling of its own. Each body's explicit
slot allowance (`BodyLimits::slots`; by default one chunk) bounds it, and a body that needs
more slots than its allowance is refused with `MathError::SlotLimit`, naming the required
and available slots (`runtime.resource_limit`; [§14.3.2](#section-14-3-2)). A tracked query
that builds a body may therefore extend the pool; otherwise tracked queries only read this
context. Fixed registration is a reproducibility control, not a claim of whole-program
bitwise identity. No license secret enters an artifact, identity or diagnostic. *Tested*
by `formal_pool_extends_to_the_declared_limit_and_refuses_beyond` and
`formal_symbols_are_stable_across_pool_extension` (`pse-math` units).

**Source owners:** `crates/pse-math/src/{lib,typed,library}.rs`,
`crates/pse-compiler/src/typed_math.rs`. Library profiles and their selected features are
explained in [math libraries](../../capability-maps/math_libraries.md); exact versions live
in the workspace manifest ([§3](workspace-and-dependencies.md#section-3)).

### 7.2 Function vocabulary and admission

> Supplement: [ADR-0144](../../adr/0144-selected-mathematical-realizations-and-square-response.md) (proposed; authorized implementation).

The admitted implicit contract distinguishes a residual relation from a selected function. Expression use requires compiler-owned selection: a justified branch, an explicit operational anchor/settings, or library-established uniqueness. Ordinary starts remain numerical aids. Realizations preserve this selection or refuse; operational determinism without established neighborhood stability supports values only. A minimum-score margin establishes a stable winner among candidates; full derivative admission additionally needs validated exclusion of every root that could beat or tie the regular winning chart across the complete alternative union. Native regime derivatives retain checked nondegenerate-affine alternatives. Nonlinear alternatives require conditional validated selection evidence as described in §7.5; unsupported alternatives remain value-only. Selected-function derivatives also require the declared unknown bounds to be inactive at their physical tolerance.

The primitive vocabulary is the `pse_quantity::functions::Function` enum backed by
library capabilities. Package functions compose those primitives with checked signatures,
lexical immutable data and explicit runtime arguments. Smoothing, hyperbolic and activation
functions are package knowledge; they do not require new primitive dispatch. Unknown
functions or unavailable external capabilities refuse at admission.

| Form | Implementation and physical contract | Original obligations |
|---|---|---|
| Arithmetic and powers | Symbolica/Numerica after complete quantity inference; exact rational exponents retained | Division and powers keep nonzero/positive requirements |
| Library elementary functions | Declared primitive with dimension/scale admission | Logarithm positivity; derivative-order-sensitive square-root validity |
| Finite sums/products and folds | Bounded lexical expansion; reductions preserve consumed entity kinds and checked prototypes | Empty/filter cases retain physical typing; fold requires nonempty membership |
| Conditionals and piecewise functions | Guarded branches; authored piecewise continuity checked at the declared order | Original guards survive; unproved switching derivatives refuse |
| Authored functions and partials | Lexical inlining and Symbolica differentiation, with shared bindings retained | Validity conditions and explicit argument independence survive derivatives |
| External functions | Explicit registered capability with quantity types, shapes and derivative contract | Typed recoverable/terminal failures and stated validity |
| Continuous derivatives/integrals | Authored realization to integrated dynamics or simultaneous schemes | Axis, physical derivative, boundary and causality requirements |
| Implicit blocks | Inline equations, nested library solve or registered acceleration | Branch eligibility/selection, regularity and derivative admissibility |

Comparisons require complete physical compatibility and select without an invented
tolerance. Static predicates resolve only admitted lexical data. Package functions are
checked before instantiation and against concrete physical operations when specialized.
A generic signature cannot authorize a physical operation absent from the admitted context.

**Source owners:** `pse-quantity::functions`, `pse-modeling::{check,specialize}`, compiler
`workspace::modeling`, and `pse-math::{typed,guarded,execution}`. The authored function and
realization contract is proposed [ADR-0100](../../adr/0100-modeling-functions-and-accounting.md).

### 7.3 Operation contracts: physical admission and domain obligations

> Decision: [ADR-0105](../../adr/0105-scip-factorable-backend.md) — obligations project as
> closed constraints of the library-neutral factorable program, never as value dependencies
> (Plan 22 G2, implemented; [§7.5](#section-7-5)).

Every constructor of `BodyBuilder` first asks `pse-quantity` inference for the complete
result type (kind, basis, reference, scale, shape, canonical unit), including registered
physical preconditions checked against the actual operands. Only an admitted operation
produces an atom. A literal is converted to its type's canonical unit before entering the
atom; bare numbers take the registry's neutral dimensionless type.

Domain obligations are `Positive`, `Nonnegative` or `Nonzero`, each tagged with the
derivative order it protects. An obligation materializes its argument as a barrier stage
and replaces it with a formal slot, so later normalization cannot cancel or rewrite the
protected expression (`x/x` does not become `1` without its nonzero check). Obligation
dependencies are recorded per output and survive simplification, cancellation and output
selection. The resulting `PreparedBody` is a sequence of stages: Symbolica blocks, guard
checks, branch regions and provider calls. Only the stages are project control flow; each
block is a library evaluator.

`PreparedBody::demanded_stages` is the stage program an evaluation of selected outputs
executes: the obligations those outputs and their recorded effects need, and only the
arithmetic they read. Evaluator compilation and every projection of a body
([§7.5](#section-7-5)) read this one program, so a projection carries exactly the
obligations the evaluator enforces. The flattened library expression of an output
(`PreparedBody::expression`) is optional: it is absent when a provider or branch prevents
flattening, and also when substituting shared stage results would exceed the flattening
bound of 16,384 operations or 1 MB, which large but factorable bodies such as Helmholtz
derivatives reach. A consumer that must not lose such outputs reads the stage program.

Branches are lazy: the unselected branch is never evaluated, so its failing obligations
cannot fire. Both branches are physically checked at construction, and a separate
physical-only pass checks reduction bodies even over empty or fully filtered domains.
A guard driven by a free coordinate makes the body nonsmooth in that coordinate; guards
that depend only on fixed or parameter inputs remain differentiable with respect to other
coordinates. Identical provider calls coalesce within one region; provider derivative
capability does not establish phase regularity ([§9](physical-semantics.md#section-9)).

Evaluation takes finite ordered inputs only. A violated obligation is a typed
`MathError::Domain` attributed to the source occurrence and, through `Instance`, to the
bound process instance; provider failures keep their typed recoverability. A failed
evaluation publishes nothing and never returns a previous trial's outputs. The compiler
owns the guarded-real interpretation; `guarded_real_policy()` is a compiler constant, and
no caller-supplied policy hash can select numerical behavior.

**Source owners:** `crates/pse-math/src/{typed,guarded,execution,factorable,error}.rs`;
physical inference in `crates/pse-quantity/src/{infer,preconditions}.rs`.

### 7.4 Normalization and exact literals

Symbolica normalizes each admitted block under real algebra. Floating operation order is
not part of the contract; admitted rewrites are those Symbolica performs within barrier
regions, and guards are never discarded by them. Integral literals in the `i16` range
enter as exact integers; other literals stay binary64 and are never guessed to be rationals.

Authored power exponents are exact facts. A literal integer, its negation, or an explicit
ratio of integer literals becomes an exact rational exponent; the builder checks that the
exponent atom agrees with that fact. Integral degrees above 1,024 in magnitude are refused.
A positive integral power materializes its base first so admission cannot construct an
enormous exact constant; a nonpositive one requires a nonzero base; any other real power
requires a positive base. Dimensioned bases with rational exponents keep their exact
dimension; a symbolic exponent requires a dimensionless base.

Body identity never hashes printed atoms or library-local symbol handles. It frames the
lowered syntax and consumed specialization facts ([§14.4](#section-14-4)); signed zero is
preserved by `pse_ids::canonical_f64_bits`
([ADR-0030](../../adr/0030-canonical-float-hashing-and-no-float-keys.md)). Library
built-in constants (for example Symbolica's `E` in normalized exponentials) are recognized
as constants, not model inputs.

**Source owners:** `crates/pse-math/src/typed.rs` (`literal`, `binary`),
`crates/pse-compiler/src/typed_math.rs` (`literal_exponent`).

### 7.5 Rows, contributions and established facts

> Decision: [ADR-0153](../../adr/0153-certify-nonlinear-regime-selection.md) (proposed; authorized kernel binding).

Nonlinear minimum-score selection has a separate validated mathematics capability,
`SelectionVerifier`, alongside the numerical `InnerSolver`. Constant authored physical
boxes and an exact shared residual/eligibility/guard projection admit conditional
First/Second preparation. At execution, the verified winning proposal requires a regular eligible root chart
uniform over an independent-input neighborhood. Complete competitive-root exclusion
covers every rival regime and the winning regime outside its chart: no eligible root
may beat or tie the winner under the authored score and tolerance. Losing regimes
need not be unique, regular or empty. Their numerical failures supply no exclusion evidence.
A numerical proposal supplies no selection authority.
Existing original residual, score-gap, regularity, physical-bound and IFT checks still apply.
No certificate upgrades residual-only selected-function export to Exact.

The native binding uses source-pinned IBEX, FILIB and bundled supported SoPlex as
static PIC libraries with hidden C++ symbols. This isolates their ABI from SCIP's
independently embedded SoPlex. Original
residual equalities become paired inequalities only for root-free covering; native HC4
and guard-admitted parametric interval Newton and certified Taylor contraction preserve that domain. Closed sequential coordinate
slabs cover the complement of a validated uniqueness box, including shared faces. Native
undefined arithmetic or incomplete covering is never empty evidence. All C1 domain premises
are established before Taylor exclusion and on the accepted root chart; strict eligibility
must have a proved margin. Opaque operations, parameter-dependent bounds, unrepresented criteria or tolerances,
competitive roots and unresolved coverage refuse derivatives. The library owns arithmetic, contraction,
chart construction and covering. Evidence carries the exact immutable alternative programs, physical domains,
selector and residual/library provenance, parameter scope and derivative order.
One worker-owned certificate may be reused only within all of those scopes; a changed
winner, higher order or changed source, bounds or parameters requires revalidation.

Competitive covering uses the library's outer Taylor relaxation with two opposite
corners and one whole-box interval Jacobian, after all C1 guards are admitted.
HC4, unknown-coordinate shaving, parametric Newton and complete native covering
retain their independent roles. A wider relaxation can require more covering work;
unresolved cells still refuse. Connected root-sheet transport first proposes the
previous existence enclosure, then the endpoint existence hull and finally the
endpoint uniqueness hull. The broader boxes supply Newton seeds only. Every proposal
must independently establish uniform existence/uniqueness, original-domain and guard
containment, and common-root bridges over a finite covering of the entire parameter
path under one deadline and proof account. Endpoint competitive exclusion does not
establish competitive exclusion at intermediate path parameters.

Cold chart construction proposes a useful parameter neighborhood before its narrow
fallbacks. A coordinate-sensitive seed uses the interval library's numerical midpoint
preconditioner and a centered parameter residual enclosure intersected with the natural
residual enclosure, after all C1 guards admit the candidate over the entire proposed
parameter box. The predicted root box supplies a numerical proposal only; midpoint
inversion establishes no interval regularity. Coordinate-sized initial radii remain
separate from the small common Newton inflation floor. A failed prediction, empty
intersection or inadmissible seed retains the scalar fallbacks. The entire proposed
neighborhood must pass uniform Newton and complete competitive exclusion; widening a proposal cannot widen an already issued certificate.
Nearby point/root refinement reuses only an actually containing certificate, with its
original source/domain/order checks and independently consumed accuracy. A competing
winner inside the proposed neighborhood prevents that broad certificate. All proposals
share the original finite proof account; larger cold-proof ceilings do not substitute
for reuse.

> Supplement: [ADR-0144](../../adr/0144-selected-mathematical-realizations-and-square-response.md) (proposed; authorized implementation).

Factorable transport retains original residuals, selection restrictions and graph fidelity. Exact means equivalent to the selected graph on its admitted domain; residual-only operational selection is Relaxed. An exact-only request refuses a relaxation. Symbolica Rational remains arbitrary precision through folding and export; binary64 conversion is an explicit backend boundary. Canonical normalized numerator/positive denominator and selection dependencies enter new identity frames; historical frames are not reinterpreted.

> Decision: [ADR-0105](../../adr/0105-scip-factorable-backend.md) — `FactorableProgram`,
> the library-neutral factorable projection with per-row fidelity, from which presolve
> derives its tapes and obligation admission (Plan 22 G2, implemented; its SCIP binding and
> runner, Plan 22 G1 and G3, are implemented,
> [§18.10.1](numerical-execution.md#section-18-10-1));
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — constraint forms
> left to native handlers are structure metadata in `CaseStructure` and `ProblemFacts` (Plan
> 22 M3 and M4, implemented);
> [ADR-0111](../../adr/0111-multi-objective-optimization.md) — several
> objectives with priority, weight and degradation tolerances (Plan 22 C3, implemented:
> members, levels, level-bound rows and the lexicographic engine,
> [§6.8](schema-and-relations.md#section-6-8)); a native lexicographic structure carries
> every level's objective (`pse.math.case-structure.v5`).
>
> Decision: [ADR-0120](../../adr/0120-provider-envelope-contract.md) — a provider output
> becomes an auxiliary bounded by its factory's declared and checked output envelope, which
> makes the dependent rows `Relaxed` (Plan 22 G4, implemented), enforced at every
> evaluation and framed into the provider configuration key (Plan 22 ENV, implemented); no
> production provider declares an envelope yet.
> [ADR-0121](../../adr/0121-convexity-compiler-facts.md) — `ProblemFacts.convexity` from a
> DCP pass over `FactorableProgram` and exact rational LDLᵀ Gram certificates, rebound with
> values (Plan 22 C5, implemented,
> [§18.10](numerical-execution.md#section-18-10)).

A selected case is a `pse_math::binding::CaseStructure`: variables with their declared
registry domain (`ModelingVariableDomain`: `continuous`, `integer`, `binary`,
`semicontinuous`, `semiinteger`; [§6.8](schema-and-relations.md#section-6-8)), fixed/free
status and optional closed bounds; ordinary parameters; instance bindings of body formals to
global sources; semantic rows with closed canonical bounds; and an optional objective with
its authored sense: the one objective level a solve optimizes
([§6.8](schema-and-relations.md#section-6-8)), whose earlier levels, when a level is
selected, become generated `LevelBound` rows over a bound parameter. A body output reaches
rows or the objective through an explicit
contribution map with finite nonzero dimensionless weights. Authored `lhs == rhs`,
`<=` and `>=` equations become residual `lhs - rhs` rows with bounds `[0, 0]`,
`(-inf, 0]` or `[0, inf)`; a conditional equation selects its branch during definition
specialization ([§10](models-and-composition.md#section-10)).

**Domain admission.** The compiler's grouped case projection carries each free variable's
declared domain; a binary variable brings the unit box, which case bounds may only narrow.
`PreparedModeling::bound_structure` ([§14.4](#section-14-4)) then admits every free discrete
variable after case binding.
Integer and semi domains need finite lower and upper bounds. Integer, binary and
semiinteger bounds are tightened exactly inward to integers (`admit_domains`: the lower
bound rounded up, the upper rounded down, a negative zero normalized to zero; semicontinuous
bounds are not tightened), and each change is recorded as a `DomainTightening` and published
as the information finding `modeling.domain.tightened` in `runtime.modeling_findings`, with
the specified and the tightened bounds (ADR-0103, Plan 22 M2a). A binary bound outside
`[0, 1]` is refused (`DomainRefusal::ConflictingBound`) rather than clamped, and bounds that
leave no value of the domain after tightening, or a semi active interval that is not
positive, are refused (`EmptyDomain`). Integer and binary variables also need a count or
indicator quantity kind ([§8.1](physical-semantics.md#section-8-1)). Each refusal names the
variable (`modeling.domain`, analysis `preparation`). A fixed variable needs no search range;
its value is checked for exact domain membership instead ([§7.6](#section-7-6)). *Tested* by
`integer_bounds_tightened_inward_and_recorded` and `binary_bounds_outside_unit_box_refused`
(compiler units). `CaseStructure::key` frames each domain by its registry spelling, every
native constraint form below and the structural requirements a lowering places on the route
(`pse.math.case-structure.v5`), such as the `l1_exact_penalty` requirement of a penalty
complementarity ([§19.7](workflows-and-results.md#section-19-7)); no route reads a
requirement yet.

**Native constraint forms.** A constraint form or disjunction realized `native` or
`indicator` ([§6.8](schema-and-relations.md#section-6-8),
[§19.7](workflows-and-results.md#section-19-7)) keeps its authored structure as
`pse_model::forms::NativeConstraint` metadata, attached by `CaseStructure::with_native`: an
indicator (the conditional row, its binary variable and the value that activates the row),
an SOS1 or SOS2 set (members with strictly increasing weights), a logic `and`, `or` or `xor`
(a binary resultant and possibly negated binary operands) or a cardinality bound (members
and the largest nonzero count). Every row and variable a form names must be selected. The
conditional row of an indicator is also an ordinary row, which an adapter without the
handler would enforce unconditionally. `CaseStructure::key` frames every form, so a native
and a linear realization of one form are different structures. `ProblemFacts.native` lists
the handlers the structure requires, sorted and unique, by the registry enum
`NativeConstraintForm` (`indicator`, `sos1`, `sos2`, `and`, `or`, `xor`, `cardinality`);
routing admits the structure only on an adapter whose capability record consumes every
listed handler ([§18.7](numerical-execution.md#section-18-7)).

Mathematical class is established from the admitted program, never from authored hints:
`ProblemFacts` records admitted derivative order, bound shapes, guarded status, proved
affine rows, objective degree and the bound/value assumptions that established them.
Polynomial degree two does not establish convexity; exact or explicitly qualified
convexity evidence is separate ([§18](numerical-execution.md#section-18)). Value-dependent
facts carry the identity of the fixed/parameter values they consumed; free trial values
never establish them. `presolve::Facts::coefficient_eligible` is the one coefficient-class
rule: every row proved affine, an objective degree of at most two and every retained
obligation discharged. Preparation, routing facts and the coefficient projection all
consume it.

**Factorable projection.** `CasePlan::factorable_program` projects each instance's demanded
stage program ([§7.3](#section-7-3)) into one shared, library-neutral DAG under fixed
consumed values. A stage result stays one node however often it is read, so the projection
never depends on the optional flattened expression. It is never an evaluator: the evaluator
and original-coordinate qualification remain the authority for every candidate.

- Nodes are `Var`, `Const`, n-ary `Sum` and `Product`, `Pow` with a constant exponent,
  `Exp`, `Log`, `Abs`, `Sin`, `Cos` and `Aux`. A constant is an exact rational when the
  library atom is rational, and otherwise a finite binary64 value, which is itself an exact
  dyadic rational. A variable exponent is exported as `exp(e * log b)`, defined on the
  positive base the evaluator requires.
- Every row and the objective carry a `Fidelity`: `Exact`; `Relaxed`, a sound relaxation
  through an auxiliary that supports bounds and infeasibility conclusions only; or
  `Unavailable`, not projected, or an objective that depends on an auxiliary without a
  finite box.
- A branch whose regions are pure arithmetic is exported exactly as
  `(X + Y)/2 - (α/2)|L - R|` when `X - Y = α(L - R)` is proved by symbolic expansion for α
  in {±1, ±2, ±½}: minimum, maximum and absolute value in either guard orientation. Any
  other branch follows the declared `BranchPolicy`. The default, `Auxiliary`, makes its
  result an auxiliary, bounded by the branch values when all of them are constant.
  `Disjunctive` is a typed refusal (`FactorableError::DisjunctiveBranch`): the exact
  mixed-integer export of a continuous branch is not provided. Authored disjunctions are
  lowered to rows at specialization
  ([§19.7](workflows-and-results.md#section-19-7)) and do not use this policy. A guard
  that is constant under the consumed values selects its region statically.
- `Require` and `Domain` stages are obligations, never value dependencies. Each becomes a
  conjunction of closed constraints, marked `strict` where the original condition excludes
  the finite bound; the conjunction is the obligation's closure. A part without a closed
  conjunctive form, such as a disjunction of branch outcomes, is dropped soundly and the
  obligation is marked unrepresented. An obligation inside a branch region is
  `Conditional`: recorded for domain analysis, never a constraint of the exported program.
- Implicit blocks export their original residual equations and declared bounds whatever
  their realization, from owner-supplied definitions
  (`AdmittedImplicit::factorable_definition`); a regime selection stays a provider output.
  Residuals are exported only where the call is unconditional, since inside a branch region
  they would constrain points where the call is never made. Other provider outputs become
  auxiliaries within the envelope their evaluation enforces, which makes the dependent rows
  `Relaxed`. Exhausting the node budget of a required projection is a typed resource
  refusal. It produces no completed partial representation and does not authorize a
  relaxed export or a different solver route; genuine opaque-provider fidelity remains
  distinct from resource exhaustion.

The SCIP route ([§18.10.1](numerical-execution.md#section-18-10-1)) projects under a request
that carries the case's implicit definitions (`factorable_definitions`, where a case bound
on an unknown replaces that endpoint's bound hint, as it does for the evaluator) and the
declared output envelope of every registered provider that has one
(`Registration::envelope`, checked against the provider's contract and enforced on every
evaluation; [§9.4](physical-semantics.md#section-9-4)). The envelopes enter the program's
identity. A
provider output without a declared envelope has no finite box, so an export with such an
output inside a nonlinear term is refused; no production provider declares an envelope yet.
*Tested* by `implicit_residual_exported_exactly` and `relaxed_rows_enclose_evaluator`
(`pse-math` units), `certify_exports_implicit_residuals_exactly` (runtime units) and
`provider_envelope_is_checked_against_the_contract` (`pse-kernels` units).

**Presolve facts.** `CasePlan::presolve_facts` derives FBBT tapes and obligation admission
from this projection, under the default request (no implicit definitions or envelopes;
auxiliary branches). Affine proofs and the objective degree still use the optional
flattened expressions. An affine row propagates from its affine proof, not from a separately
built tape, so its constants agree exactly. These facts support interval source-infeasibility
analysis independently of executable NLP transformations. The pinned binding's untracked
bound tightening is not admitted to native execution; supported affine elimination retains
its distinct source and multiplier map ([§18.5](numerical-execution.md#section-18-5)). A
large factorable body therefore keeps a complete tape, a
requirement is admitted through its exact condition on its argument, and a validity
predicate through its closed conjunction, which must be complete, over the selected
variable box. Admission follows the evaluator's obligations:

- obligations that guard only outputs no row or objective evaluates do not count;
- an obligation inside a branch region can make its instance unestablished, never
  violated;
- hard sign domains come only from obligations enforced on every evaluation: an
  unconditional requirement on a single scaled column;
- an instance whose projection hit the node budget is unestablished when its body retains
  any obligation.

**Source owners:** `crates/pse-math/src/{binding,facts,coefficients,presolve,factorable}.rs`,
`crates/pse-compiler/src/workspace/modeling/` (implicit residual definitions in
`executable/factorable.rs`). The generic declaration contract is
[`authored.modeling_declarations`](../../generated/relations/authored.md).

### 7.6 Null, bound, and unknown semantics

| Situation | Representation |
|---|---|
| Variable to be solved | a free variable in `CaseStructure`; its fixed/free status changes the case layout, not the body |
| Starting value | every variable and parameter has a finite case value; `validate_values` refuses missing or nonfinite entries. Start selection is owned by strategies ([§17](numerical-execution.md#section-17)) |
| Unbounded variable or row side | an absent authored bound; lowered to `None` or an outward infinity, never NaN |
| Case value outside declared bounds, or fixed value outside its integer domain | refused at case admission, without solver tolerance; a semi-variable's zero is admitted |
| Free discrete variable without finite bounds, or with bounds that admit no value of its domain | refused after case binding, naming the variable ([§7.5](#section-7-5)) |
| Missing measurement | a null observation value; an included observation without a value or standard deviation is refused, and exclusion is the explicit `included` flag ([§19](workflows-and-results.md#section-19)) |
| Domain or provider failure | a typed `MathError`, never a silent null or a stale value |
| Missing versus empty input | distinct: a missing domain, group or provider is an error; an empty admitted domain is a valid finite domain |
| Undecided selected meaning | refused at admission ([§14.5](#section-14-5)), never consumed as true |

### 7.7 The expression DSL

> Decision: [ADR-0142](../../adr/0142-process-state-connections-and-temporal-conservation.md)
> (checked process occurrences implemented; decision remains proposed).

Modeling documents use `pse-authoring::language`; embedded expressions use
`pse-authoring::dsl`. The registry-generated declaration IR retains each source occurrence,
identity and span. The checked package owns resolved types, visibility and physical context;
consumers cannot mutate a checked product or substitute a foreign physical revision.

Checking retains expressions, predicates, equations, logic and static-domain syntax by
actual declaration, field role and structural ordinal. Exact payload-field byte ranges
remain revision attribution. Each occurrence retains lexical/import context, physical
admission and dependencies; indexed obligations remain explicit until specialization.
All specialization and body consumers use those checked occurrences. Logic extends the
same quoted-path/count predicate grammar. Synthetic expressions are AST nodes carrying
the originating operation, rather than formatted text parsed through a second front door.

> Supplement: [ADR-0150](../../adr/0150-checked-admission-and-owned-reuse.md)
> (proposed; authorized implementation).

The language admits packages, entity kinds/entities, enumerations, sets, tables, functions,
interfaces, definitions, presets, children, equations, accumulators, contributions, ports,
connections, annotations, requirements, tests, cases and analyses, and the constraint forms,
disjunctions and realizations of [§6.8](schema-and-relations.md#section-6-8). Expressions
include physical literals, lexical bindings, indexed paths, finite reductions/folds, conditionals,
function calls, partial derivatives and continuous derivatives/integrals. Parse/render/parse
preserves declaration structure and IDs; diagnostic spans refer to original bytes.

Parser and specialization limits refuse before uncontrolled expansion. Function names
resolve in their defining lexical scope, and actual indexed arguments retain the caller's
scope. Interface defaults and overrides preserve their original declaration identity;
ambiguous inherited contracts refuse. Shared local bindings are retained in mathematical
bodies instead of flattened exponentially.

**Source owners:** `crates/pse-authoring/src/{language,dsl}/`, `pse-modeling::{check,types}`,
and compiler `workspace::modeling`. Generic declaration shape belongs to
[§6.15](schema-and-relations.md#section-6-15), not a parallel grammar catalogue here.

## 14. Compilation and preparation

> Decision: [ADR-0082](../../adr/0082-library-owned-process-mathematics.md),
> [ADR-0089](../../adr/0089-semantic-identity-projections.md)

Compilation derives immutable products from an admitted model revision. Pure semantic work
is tracked by Salsa inside `pse_compiler::workspace::CompilerWorkspace`; native program
construction, retention, provider workers and solver state are runtime effects with their
own owners. Semantic preparation (`PreparedBody`), effect-owned compilation
(`CompiledBody`) and mutable scratch (`Worker`) are separate types with separate
lifetimes, and each has a distinct identity scope
([§5](identity-and-publication.md#section-5)).

### 14.1 Preparation stages and contracts

> Supplement: [ADR-0152](../../adr/0152-demand-driven-compilation-and-contextual-routing.md) (proposed; Plan 25l functional implementation complete).

Semantic specialization is independently usable by topology. Numerical consumers select outputs with their mandatory physical/domain/effect closure before executable body admission. Value programs, conservative structural incidence, numeric derivative support and native artifacts have distinct demanded readiness; missing preparation is not mathematical incapability. The operations have explicit evidence dependencies, not an unconditional one-pass order.

> Decision: [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) —
> named lowerings of indicator, SOS, cardinality, piecewise, logic and disjunction
> declarations join generic specialization, and their derived realization parameters join
> view preparation (Plan 22 M3 and M4, implemented;
> [§19.7](workflows-and-results.md#section-19-7)), as do complementarity lowerings (M5a,
> implemented) and the `objective_bounds` transformation of
> [ADR-0111](../../adr/0111-multi-objective-optimization.md) (Plan 22 C3 authoring,
> implemented).

| Stage | Owner | Consumes | Produces | Effects |
|---|---|---|---|---|
| Package admission | runtime modeling admission ([§22](models-and-composition.md#section-22)) | exact package closure, generic declarations, physical inventory and its names | immutable checked revision | source loading and bounded admission |
| Generic specialization | `pse-modeling`, tracked by compiler modeling queries | checked package, root/instance, bindings, analysis and limits | demanded members, equations, named lowerings of constraint forms, complementarity and disjunctions, objective levels with their level bounds, lineage, checks and reports | none |
| Input publication | `CompilerWorkspace` | checked package/context and lowered case inputs | Salsa input revisions | validated before setters |
| Body admission | `modeling::executable::grouped::semantic_body` query → `typed_math::Request::admit` | complete normalized body inputs, called functions, providers, physical operations and limits | `AdmittedBody`: `BodySpec`, types, occurrences, `PreparedBody` | optional ownership/retention attachment; transient refusal is not memoized |
| Case planning | selected-view preparation → `CasePlan::prepare` | case structure, semantic bodies | immutable `CasePlan` with faer patterns and local demands | none; no evaluators |
| Modeling view and rebind | `PreparedModeling::bound_structure`, `CompilerWorkspace::prepare_modeling_view`, `PreparedCase::rebind` | bound case structure, derivation rules, case values | a `PreparedCase` view identified by `view_key`, whose value products are rebound per values ([§14.4](#section-14-4)), and on a sensitivity request its parametric program (`pse.compiler.modeling-parametric.v1`, [§15.5.1](numerical-execution.md#section-15-5-1)) | none; no evaluators |
| Structure and facts | `structure`, `coefficients`, `presolve_facts`, `problem_facts` queries | plan, consumed fixed/parameter values | `StructuralAnalysis`, coefficient/presolve/class facts | none |
| Artifact requests | `artifacts` query | plan demands, `Profile`, math environment | keyed `ArtifactRequest`s | none |
| Program construction | `pse-runtime::math` | artifact request | `CompiledBody` | native build, retained under runtime policy |
| Case assembly and attempt | `CaseAssembly`, `CaseWorker` | plan plus completed artifacts | evaluator workers with private scratch | attempt-owned |

`CasePlan` admits `z = Gx + Bp + c` gathers: formal slots bind global variables or
parameters through physically checked `SlotBinding`s. Repeated source slots are aliases,
not independent variables; unit conversion requires identical kind, basis, reference and
scale, and a composition-dependent change needs an explicit physical operation. faer fixes
the Jacobian and Lagrangian-Hessian ordering once; a mechanical refill map accumulates
values into retained buffers. When an alias lands on a global diagonal both cross-partials
accumulate, and off-diagonal entries are not doubled. The full selected row and free
variable inventories survive, including isolated rows and numerical zeros. Identical local
body/output/coordinate demands share one immutable program; each attempt owns its mutable
evaluators and provider map. Graph-only planning constructs no numeric evaluator.

Compiler products also feed structural analysis ([§15](numerical-execution.md#section-15)),
dynamic algebraic-partition matching, conditional initialization blocks and flow/tear
preparation ([§17](numerical-execution.md#section-17)). The callable surface is described in
the [native workflow guide](../../dev/native-workflow.md).

### 14.2 Relational invariants

Set-oriented integrity over relations remains in Arrow/DataFusion. Declared registry
invariant queries support explicit inspection, and declaration-owned residual obligations
serve publication admission ([§4.6](schema-and-relations.md#section-4-6)). Row predicates
are prepared under the actual immutable validation context; scalar predicates also persist
as Delta `CHECK`s. Source and physical loading each call shared Rust entity/reference and
acyclicity predicates. Their duplicate registry SQL declarations and the inactive
requirement-planner/provider-policy bridge are removed.

Generated integrity declarations carry a read-only typed `InvariantOrigin` identifying their
native producer and derivation family; authored queries cannot acquire it by naming convention.
Origin is implementation metadata and does not enter durable schema identity. Independent family
witnesses exercise key and nested-reference semantics through the real executor. Catalog controls
establish exact native bindings, and hostile admission controls exercise production enforcement.
Tests select from intact admitted packages; they do not rewrite registry obligations or authored
package declarations to make a journey executable ([ADR-0160](../../adr/0160-centralize-testing-responsibility.md), proposed; authorized implementation).

Operand-level physical prerequisites are `PhysicalPrecondition` predicates checked against
actual operands during typed inference ([§7.3](#section-7-3)). There is no production rule
engine or recursive inference executor. A future recursive-inference consumer needs its
own reviewed contract.

**Source owners:** `crates/pse-relations/src/validate/`,
`crates/pse-engine/src/validation.rs`, `crates/pse-rules/src/{references,invariants}.rs`,
`crates/pse-runtime/src/authoring_driver/p1.rs` and `workflow/physical.rs`.

### 14.3 The preparation engine and ownership

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed target). Database structural resolution supplies a semantically sufficient immutable selection to the shared admission/inference/specialization kernel. Protection covers source and interpretation inputs through off-transaction preparation, then retained product roots are admitted atomically before protection ends. Portable descriptions retain admitted mathematical specifications; reconstruction does not replay whole-package semantic compilation.

The implemented selected source boundary retains canonical revision receipts and object identities;
it resolves lexical presence and absence, imports, selected scopes and inverse suppliers before
shared scientific admission. Complete declaration inventories and source Arrow tables are explicit
exports. Physical entity/entity-kind declarations remain a genuine global physical obligation and
are projected separately for physical admission. Temporary generated columns do not become a
second canonical source representation.

Portable reconstruction consumes the original selected syntax and concrete owning-kernel
construction receipts. An opaque, non-serializable authority binds the complete compiler-issued
payload, immutable stored qualification and exact request; every restored record must belong to
that payload. Missing or incompatible receipts refuse reconstruction without an inference or
proof fallback. Controlled deployment qualification and scientific publication/replay cross
explicit audited trust assertions; privileged storage or deployment configuration modification
requires readmission. Reconstruction reserves decode scratch before parsing.

Evaluators use compact selected formal coordinates, including actual guard/provider and nested
scope requirements. Public formal identities remain stable; consumers gather through the same
recorded mapping. Derivative direction axes retain their requested order. Mutable evaluator and
solver state belongs to the attempt, while shared immutable payloads retain their own allocation
leases rather than complete source-owner ancestry.


> Proposed amendment: [ADR-0155](../../adr/0155-numerical-derived-families.md),
> [Plan 25n](../../plans/25n-automatic-simulation-solve-pipeline.md).

Automatic preparation derives separate numerical incidence, execution/validity dependencies and
objective/inequality coupling from the authored case. Value demand precedes actual conditional
derivative demand. Composite admitted reconstruction suppliers carry complete original state,
actions, selected-sheet meaning and operation-bound accuracy; structural matching alone does not
establish their regularity. Complete original reconstruction is independently assessed.

> Decision: [ADR-0155](../../adr/0155-numerical-derived-families.md) (proposed;
> independently reviewed target, implementation owned by Plan 25m).

Derived-family preparation separates immutable family structure from bound anchors and
parameters. Each family owns actual incidence, reconstruction, preserved original
bounds/guards/inequalities/objective and selection meaning, and actual derivative/action
support. Terminal identity, conditional equivalence, block coverage and approximation have
distinct permissions. Opaque oracles cannot acquire unsupported derivative order through a
wrapper. Consumed accuracy separates certified bounds, admitted numerical estimates and
unresolved evidence. Original physical/backward-error acceptance remains authoritative;
a requested forward/root/jet bound has its own consumer contract. Selected-root proof,
numerical products and connected-sheet transport have separate validity and lifetime keys.
Auxiliary success never bypasses frozen original assessment.

Consumed selected-root accuracy uses a separate fixed-parameter enclosure beneath the
retained uniform selection chart. Point refinement never replaces the chart's parameter
neighborhood or competitive exclusion. The verifier establishes a normalized interval
inverse bound over the numerical candidate/root hull; that bound includes nonlinear
variation and guides stricter physical residual controls. JVP controls translate actual
residual-jet backward-error denominators using original row scales and remaining action
allowance. Those numerical control estimates do not certify the result: the actual fixed
root/action enclosure must satisfy the consumer's product, normalization and error demand.
Each product has explicit finite refinement rounds and proof cells under the original
cancellation owner and absolute deadline. Changed controls invalidate numerical products;
compatible selection, source and sheet evidence remain separately reusable.


> Supplement: [ADR-0152](../../adr/0152-demand-driven-compilation-and-contextual-routing.md) (proposed; Plan 25l functional implementation complete).

Immutable support requests identify admitted body, ordered outputs and coordinates, order and finite construction policy. Value requests construct no numerical derivative support and First requests construct no Second support. Structural incidence is conservative all-branch dependency evidence and does not promise provider derivative evaluation. Stronger preparation cannot mutate weaker products, reset allowances or memoize transient failure; existing retention, flight and allocation owners remain authoritative.

> Supplement: [ADR-0144](../../adr/0144-selected-mathematical-realizations-and-square-response.md) (proposed; authorized implementation).

Pure derivative-requirement algebra belongs to `pse-kernels`: residual availability, output smoothness, selector-neighborhood validity, inner minimum and requested output determine residual compilation. Native adapters supply their actual minimum; runtime resolves the effective profile and supplies demand. Provider descriptors and nested native oracle contracts consume this product.

> Implemented amendment: [ADR-0140](../../adr/0140-scientific-knowledge-admission.md) and
> [ADR-0141](../../adr/0141-applicability-evidence-and-permissions.md), Plan 25b.

Scientific selection retains roots, dependency closure, conventions and applicability
references in admitted preparation. Claim and named permission identities are framed with
their actual scope. Applicability observations follow the demanded call/branch and survive
rewriting, derived arguments and differentiation; numerical-body reuse cannot erase their
attribution or use gate. Construction-local function and instance captures exclude unrelated selections; finite-call
identity is sealed after consumption. Changed preimages and generated transports receive new
versions. Claim capture references and finite values are validated independently of permission;
retained metadata and evaluation observations enter owned memory allowances.

Numeric parameters keep their case-value input role. Their authored default or constructor
argument retains its lexical source separately, and an actual demanded read carries only the
source's prerequisite effects into the returned input value. A generated function's explicit
prerequisite indices distinguish this effect transfer from ordinary unused authored locals.
Literal overrides replace the old source obligation; unused parameters and inactive branches
produce no observation. A later numerical case-value change retains declared source lineage;
result qualification for changed values belongs to §19.3 and Plan 25e/25f. Parameter-read,
finite-function, dispatch-body and typed-definition identities include the prerequisite contract.

`CompilerWorkspace` is a single-writer owner of one local Salsa database. No database
clone or Salsa handle escapes. `CompilerContext` contains the immutable quantity registry,
physical preconditions and provider declarations; the raw Inputs/case/flow front door and
its unused tracked queries are removed. Checked modeling revisions and immutable selected
bindings supply preparation. Complete body inputs include normalized AST, formals,
physical operations, transitive called functions, provider descriptors and admission limits.
A/B/A selections coexist; one mutable root slot cannot overwrite their meaning.

Tracked mathematical preparation is mathematically pure: complete immutable inputs determine
the admitted mathematics. It constructs no mutable evaluator, runs no solver and writes no
data. A service retention attachment is selected once before revision publication; its
infrastructure lookup, counters and allocation accounting do have effects. A compiler-issued
private witness binds each cacheable body to its complete semantic key and physical closure;
callbacks cannot replace that proof or mutate the admitted meaning. Owner attachment may change
accounting only. The attachment holds the service weakly; transient allocation refusal unwinds
the query as cancellation and returns its typed cause, so the same semantic inputs can retry.
Cancellation is never memoized as a result. The append-only formal-symbol pool retains its
registration-order contract (§7.1).

Returned `PreparedCase`, `CasePlan` and artifact requests own their immutable mathematics,
value bindings and current revision attribution independently of workspace generations.
No unused whole-model structural query remains: structural evidence belongs to the
consumed selected view.

> Supplement: [ADR-0150](../../adr/0150-checked-admission-and-owned-reuse.md)
> (proposed; authorized implementation).

Physical preparation consumes immutable checked products. Source functions retain
physical operation admissions by structural expression occurrence: body-relative preorder
and lexical position distinguish identical syntax, including rewritten nodes with empty
spans. Concrete products retain operand/result contracts and selected rule; generic
products retain requests and operand schemes and instantiate against the caller's admitted
substitutions before numerical construction. Specialized function bodies preserve those
products. Missing or conflicting occurrence admission refuses lowering; an expected return
type cannot supply it. Finite mathematical requests admit their actual resolved operation
before handing it to `BodyBuilder`. The builder constructs library atoms from that product
and explicit named boundary authorization, including canonical numerical scale normalization.
It does not select a second physical inference route.

Scientific response preparation also checks normalized dependence. A selected potential
must retain its declared reconstruction, and a response must retain that selected potential
or its physical partials. Abstract scientific calls are normalized through the same library
path before reduced-law implementations are substituted. Zero coefficients and cancellation
cannot masquerade as dependence; a legitimately zero selected law remains admissible.
Physical partials bind independent actual arguments, so equal numerical argument values do
not identify their differentiation coordinates. These admissions and scientific contracts
enter the typed preparation identity (§5.3). This is implemented mechanism scope; focused
check results belong to the owning packet and assembled qualification remains Plan 25k.

> Decision: [ADR-0135](../../adr/0135-physical-expression-contracts.md) (proposed) — retain admitted
> physical operations and normalized scientific witnesses through lowering.

#### 14.3.1 Artifact construction, retention and completion

`ArtifactRequest` is issued only by the compiler; its private fields prevent runtime code
from inventing keys. Its key frames the body key, math environment, source and build
identity, evaluator ABI profile, derivative order, ordered outputs and coordinates,
optimizer options and evaluation limits. Compilation settings are excluded from `BodySpec`
and included here.

`pse-runtime::math` builds programs at the effect boundary and retains them in the shared
runtime's DataFusion `DefaultCache` component, with completion-owned single flights, epoch
fences and per-entry reservations. Concurrent requests for one key share a flight; failure
and cancellation are retryable, never cached. Abandoned native work keeps its key and
reservation until the work actually exits. Storage maintenance does not evict
mathematical programs; explicit clearing fences late insertion
([ADR-0089](../../adr/0089-semantic-identity-projections.md)).

Relational operations follow the analogous completion contract in `pse-engine::operation`:
one invocation owns execution, typed finite output ports, and a shared completion; partial
streams or failed work are never promoted to a complete result, and a dropped last waiter
poisons unfinished pure work instead of letting a later consumer restart it.

**Source owners:** `crates/pse-compiler/src/workspace.rs` (`ArtifactRequest`),
`crates/pse-runtime/src/math/{artifacts,products,jobs}.rs`,
`crates/pse-engine/src/operation/`.

#### 14.3.2 Resource ownership and admission limits

Resource policies refuse excess with a typed `Limit` error; they never truncate work.
Limits are finite, configurable where they are policy, and checked before the allocation
they protect.

| Scope | Default bounds |
|---|---|
| DSL text | 65,535 bytes; nesting and unit depth 64 |
| Body construction (`BodyLimits`) | 8,192 formal slots by default (one pool chunk; the pool grows to an explicitly raised allowance); 16,384 construction occurrences; syntax, filter and conditional depth 128; integral power degree 1,024 |
| Domain/class construction (`Profile.class_proof_work`) | 1,000,000 shared conservative work units per bound product; expression storage/substitution bounds and derivative construction consume the same remaining allowance; these are not measured operation counters or byte reservations |
| Local evaluation (`EvaluationLimits`) | 4,096 derivative components; 1,000,000 scalar operations per demand; 512 MiB numeric worker scratch; 4,096 provider calls |
| Optimizer (`Optimization`) | cores 1–64 (default 1); Horner iterations up to 1,000 (default 10); common-pair rounds up to 32 (default 1); a fixed Horner-scheme budget of 8,192 variables, an optimizer budget rather than a symbol bound |
| Case (`CaseLimits`, `AssemblyLimits`) | 100,000 scalars, instances and rows; 8,192 distinct bodies; 1,000,000 slots and contributions; native index within `i32`; 2 GiB complete attempt storage (shared evaluator scratch, cloned instructions/descriptors, occurrence caches and sparse refills) |
| Workspace (`WorkspaceLimits`) | 4,096 input entries; 2 GiB input extent; 16,384 retained entries and 256 MiB known retained bytes; LRU of 64 values per expensive query |
| Structural matching | 100,000 rows on a 32 MiB stack |
| Runtime (`MathPolicy`) | 8 GiB artifact retention; 64 MiB foreign allowance per job/program; 8 GiB worker, 4 GiB workspace; 4 jobs; 128 flights |

The compiler carries the declared class-proof and typed assembly allowances through initial,
value-rebound and derivative-upgraded products. The complete consumed profile uses the v4
modeling-view identity frame, preserving historical v2/v3 hash bytes. Public preparation
documents require the assembly contribution, native-index and complete worker-byte limits;
the unchanged defaults are explicit Rust policy rather than a hidden ceiling. Changing only that policy does not
change an evaluator artifact that does not consume it. Class discovery occurs at its
actual representation consumer; a factorable export admits its own evidence rather than
waiting for an unrelated polynomial coefficient proof. Structural incidence includes all
original contributions, even when numerical evaluation selects only one objective level.
Mandatory derived-extremum and declared event guard/reset inputs remain in the executable
projection alongside requested outputs; unrelated observations remain undemanded.

Derivative admission bounds the pinned Numerica Taylor convolution and primitive expansion
from the source operation counts before vectorization, then reconciles the actual built
operations. Support admission checks the actual selected first/second cardinalities and
remaining construction allowance before allocation, including dense opaque/provider support;
there is no fixed 256-coordinate support cutoff. Shared bindings count once, and local
obligation/branch scopes retain inherited facts while copying only their assigned facts. `ModelingLimits.body_occurrences` and
`body_slots` (`pse_modeling::Limits`, also a fixture's execution policy) raise the
occurrence and slot allowances explicitly; the formal-symbol pool extends to a raised slot
allowance ([§7.1](#section-7-1)), and a body beyond its allowance is refused with
`MathError::SlotLimit`. Limits are tracked policy inputs and cannot authorize a differently
typed or truncated model.

Nonlinear selection proof work reserves all coexisting input transports and the maximum
sequential native graph/cell extent under the same escaped program allocation owner.
A worker retains at most one source- and domain-scoped winning chart within its finite
admission. The group contains at most 64 alternatives; each transport is bounded by
65,536 nodes, 1,048,576 edges, 8,192 guards and 128 coordinates. Covering allows
1,048,576 cells shared across all winning complement slabs, rival domains and chart retries. Proof execution
shares the outer deadline and cancellation. Native calls are checked before and after;
a result returned after cancellation or deadline has no proof authority. Direct library
chart calls can delay interruption until the bounded call returns. No global proof cache
or separately admitted nested worker exists.

`MathPolicy` allowances draw from the deployment memory pool, never a second budget.
Immutable prepared products, artifacts, active worker scratch, foreign allowances and
completed results are charged separately at their actual ownership lifetimes; one shared
CPU semaphore serves data and math work, and each optimizer reserves its cores once. A root
query admits its target partitions of the deployment's workers at once and is refused
(`runtime.resource_limit`), not queued, when they are taken; nested work borrows its root's
admission (`roots_admit_partition_capacity_and_nested_work_borrows_the_owner`, engine
units).
`with_worker` constructs, uses and destroys providers and evaluators on the owning thread,
and a join supervisor keeps permits through thread-local destruction after cancellation.
Preparation owns the effective thread policy
([ADR-0093](../../adr/0093-qualified-native-strategies.md)). Foreign Symbolica/GMP storage
uses a declared conservative allowance; neither reservations nor allowances claim to bound
or measure process RSS.

**Source owners:** `crates/pse-math/src/{typed,jets,library,assembly,binding}.rs`,
`crates/pse-math/src/library/admission.rs`, `crates/pse-compiler/src/workspace.rs`,
`crates/pse-runtime/src/math.rs`.

### 14.4 Incrementality

> Decision: [ADR-0164](../../adr/0164-unify-simulation-substrate.md) (proposed target). Persisted complete dependencies govern reuse across restart; Salsa is an optional local accelerator. Immutable selected layouts and native products retain their live consumers and bounded accelerator owners, without whole ancestral source bundles. Binding edits reuse structure only when those values are not consumed structural literals.

> Supplement: [ADR-0152](../../adr/0152-demand-driven-compilation-and-contextual-routing.md) (proposed; Plan 25l functional implementation complete).

Changed support/view/request preimages use new current frame versions; historical digests are not recomputed. Selected dependencies and preparation demands enter the products that consume them. Build/runtime observations belong to contextual assessment and affected requests, not unrelated symbolic body identity. Value rebinding and fresh instance attribution remain separate from shared mathematics.

> Decision: [ADR-0089](../../adr/0089-semantic-identity-projections.md) — complete identity
> projections. Plan 22 A6 (implemented) adds value-only rebind of prepared views and bounded
> package views keyed on bound structure.

Salsa reuse is valid only when a fresh workspace given the same admitted inputs would
produce an equal product. Queries depend on exact fields, including absence; unchanged
results backdate so unrelated edits do not propagate. Immutable context and checked revisions supply complete semantic inputs; binding inputs
retain each selected value independently. Salsa handles remain local.

| Change | Re-admitted or recomputed | Reused |
|---|---|---|
| Free-variable case value | value-reading fact queries re-check and backdate; they consume only fixed/parameter values | bodies, case plan, structure, artifacts |
| Fixed or parameter value | coefficient, presolve and class facts that consumed it | bodies, case plan, structure, artifacts |
| A value that a derived realization parameter consumed ([§19.7](workflows-and-results.md#section-19-7)) | that parameter, then the value products that consumed it | view, derivation rules, bodies, plan, artifacts |
| Fixed/free status, bounds, rows, bindings or contributions | case plan and its dependents | admitted bodies |
| One definition's source, formals, units or literal contracts | that definition's body and dependent plans | other bodies with equal keys |
| Domain membership, group tuples or provider descriptor | definitions that read them | independent definitions |
| Quantity registry or physical preconditions | every body (complete physical identity) | nothing semantic |
| Optimizer or evaluation profile | artifact requests and programs | bodies and plans |

**Prepared views and value-only rebind.** Solver views of a modeling analysis separate
structure from values. `PreparedModeling::bound_structure` applies the case's variable
states, admits discrete domains and tightens their bounds inward, returning the recorded
tightenings beside the structure ([§7.5](#section-7-5)), and excludes observation rows; it
reads no value. `PreparedModeling::view_key` (`pse.compiler.modeling-view.v3`) is the
complete identity of the view prepared from that structure: the bound structure key,
including native constraint forms; every admitted mathematical body; the rules
of the derived realization parameters; the derivative order; the evaluator profile; and the
physical context. Equal keys give equal plans, structural analyses, derivations, artifact
requests. Current revision attribution is rebound separately.
`CompilerWorkspace::prepare_modeling_view` prepares the view and
binds its first values.

`PreparedCase::rebind` binds later values to the same view. The plan, structural analysis,
artifact requests, source occurrences and derivation rules are shared. Derived realization
parameters are recomputed only when a value they consumed changed. The presolve projection,
coefficient snapshot and problem facts are rebuilt only when a value they consumed changed,
derived or not (`PreparedCase::values_match`: the dependencies the presolve projection
recorded and, with a coefficient snapshot, the fixed and parameter values it assumed).
Free-variable starts are never among them. The recorded fixed and parameter assumptions
always follow the new values, and every consumer evaluates with the values completed by the
derived parameters (`PreparedCase::complete`). `PreparedBlock::bind` binds a conditional
initialization block the same way: the block's plan, structural analysis and artifact
requests are its own and shared, and only the value products are built. Observation
programs are value-free: `prepare_modeling_observations` returns a plan with its artifact
requests (`PreparedFunctions`) and builds no presolve or coefficient projection, because
every evaluation binds its own values.

The shared `MathService` retains immutable admitted package snapshots, semantic bodies,
solver views, observation and parametric programs in a byte-bounded DataFusion
`DefaultCache`. Fixed-count package LRUs are removed. Keys include the consumed physical,
provider, profile and admission dependencies; package retention also binds actual registry
and engine-context owners. Mutable workspaces, evaluators and cancellation remain
attempt-owned. Exact package reconstruction obtains a fresh workspace over the admitted
snapshot; a fresh revision always rebinds its own occurrence attribution over reused math.

Cache size measures retained reachable payload with positive entry overhead. Unique
allocation leases follow shared mathematical storage, value products and attribution
owners independently of eviction. Rebinding retains unchanged provenance and only charges
new bindings; an escaped alias keeps its lease after cache clear or workspace rotation.
Job reservations partition by transfer before retention, without a release/reacquire gap.

A complete physical-inventory identity conservatively re-admits all bodies after any
registry change; a finer consumed-physical fingerprint would be an optimization, not a
correctness requirement. Registry edits also re-admit representation conversions in
instance bindings. Diagnostic spans refresh without disturbing semantic products.

Retention is bounded. After each request the workspace triggers LRU eviction; if retained
Salsa entries or known bytes still exceed `WorkspaceLimits`, it rebuilds a fresh generation
from the current inputs. Rotation discards memos, not correctness, and escaped products keep
their owners. There is no persistent Salsa store; a new process rebuilds from the admitted
revision, and durable reuse applies only to published data
([§20](identity-and-publication.md#section-20)).

### 14.5 Admission closure and refusal

A case is prepared only when every selected declaration is consumed, explicitly retained as
nonexecuting data, or refused with a source identity
([ADR-0088](../../adr/0088-selected-model-and-physical-contracts.md)). Mathematical
admission applies the same rule: an unsupported function, unresolved path, unit or group,
missing ragged tuple, unavailable provider derivative, insufficient smoothness, exceeded
limit or physically invalid operation refuses the request. Nothing substitutes a default,
drops a term or relaxes a guard to make admission succeed.

Admission closure is not numerical or physical closure. Structural matching never
certifies numerical rank ([§15](numerical-execution.md#section-15)), and solver status never
implies conservation, which remains an independent physical check
([§19](workflows-and-results.md#section-19)).

**Limits.** This area supports finite, physically typed scalar and indexed algebra with
exact first and second derivatives, value-only nonsmooth switching and explicit providers.
Continuous-axis realizations are described in [§13](workflows-and-results.md#section-13).
General implicit or higher-index integrated DAEs, JIT/SIMD evaluators, GPU and distributed
execution are not admitted; global MINLP is admitted only for factorable problems over
finite boxes on SCIP's route ([§18.10.1](numerical-execution.md#section-18-10-1)). Nested
branch derivatives are local and
refuse unproved crossing/selection behavior. Vectorized
evaluators must not call `optimize_stack()` after vectorization. Qualification basis:
[§24.2](operations-and-validation.md#section-24-2).

## Retired section identities

#### 14.2.1 Optional predicate preimages — retired

No component consumes DataFusion's `ScalarUDFImpl::preimage`; its eligibility falls under
the general library-access rule in [§3.3.1](workspace-and-dependencies.md#section-3-3-1).
