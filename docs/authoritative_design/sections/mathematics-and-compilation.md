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
| Explicit effects | Symbolic initialization, native compilation and evaluation are effects outside tracked Salsa queries ([§14.3](#section-14-3)). |

Unknowns are symbols bound to case variables, never nulls in a value column. Every scalar
input and output carries a complete physical type ([§8](physical-semantics.md#section-8)),
and every expression occurrence keeps a source identity for diagnostics.

`pse_math::initialize` is the single explicit entry. It supplies the optional
`SYMBOLICA_LICENSE` environment value to the library once, disables library tracing and
registers the bounded vocabulary (4,096 formal symbols and 4,096 provider-function symbols)
in a fixed order, independent of model arrival order. It also captures the linked GMP/MPFR
runtime versions, whose identity enters compiler inventories and artifact keys. Tracked
queries only read this context. Fixed registration is a reproducibility control, not a
claim of whole-program bitwise identity. No license secret enters an artifact, identity or
diagnostic.

**Source owners:** `crates/pse-math/src/{lib,typed,library}.rs`,
`crates/pse-compiler/src/typed_math.rs`. Library profiles and their selected features are
explained in [math libraries](../../capability-maps/math_libraries.md); exact versions live
in the workspace manifest ([§3](workspace-and-dependencies.md#section-3)).

### 7.2 Function vocabulary and admission

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

> Decision: [ADR-0105](../../adr/0105-scip-factorable-backend.md) — `FactorableProgram`,
> the library-neutral factorable projection with per-row fidelity, from which presolve
> derives its tapes and obligation admission (Plan 22 G2, implemented; its SCIP binding and
> runner, Plan 22 G1 and G3, are implemented,
> [§18.10.1](numerical-execution.md#section-18-10-1));
> [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) — constraint forms
> left to native handlers are structure metadata in `CaseStructure` and `ProblemFacts` (Plan
> 22 M3 and M4, implemented);
> [ADR-0111](../../adr/0111-multi-objective-optimization.md) — several
> objectives with priority, weight and degradation tolerances (Plan 22 C3; not yet
> implemented).
>
> Decision: [ADR-0120](../../adr/0120-provider-envelope-contract.md) — a provider output
> becomes an auxiliary bounded by its factory's declared and checked output envelope, which
> makes the dependent rows `Relaxed` (Plan 22 G4, implemented); enforcement at evaluation
> is not yet implemented, and no production provider declares an envelope yet.
> [ADR-0121](../../adr/0121-convexity-compiler-facts.md) — `ProblemFacts.convexity` from a
> DCP pass over `FactorableProgram` and exact rational LDLᵀ Gram certificates, rebound with
> values (Plan 22 C5; not yet implemented).

A selected case is a `pse_math::binding::CaseStructure`: variables with their declared
registry domain (`ModelingVariableDomain`: `continuous`, `integer`, `binary`,
`semicontinuous`, `semiinteger`; [§6.8](schema-and-relations.md#section-6-8)), fixed/free
status and optional closed bounds; ordinary parameters; instance bindings of body formals to
global sources; semantic rows with closed canonical bounds; and an optional objective with
its authored sense. A body output reaches rows or the objective through an explicit
contribution map with finite nonzero dimensionless weights. Authored `lhs == rhs`,
`<=` and `>=` equations become residual `lhs - rhs` rows with bounds `[0, 0]`,
`(-inf, 0]` or `[0, inf)`; a conditional equation selects its branch during definition
specialization ([§10](models-and-composition.md#section-10)).

**Domain admission.** The compiler's grouped case projection carries each free variable's
declared domain; a binary variable brings the unit box, which case bounds may only narrow.
`PreparedModeling::bound_structure` ([§14.4](#section-14-4)) then admits every free discrete
variable after case binding.
Integer and semi domains need finite lower and upper bounds, and bounds that leave no value
of the domain are refused: a binary box containing neither 0 nor 1, an integer range with
no integer, or a semi active interval that is not positive. Integer and binary variables
also need a count or indicator quantity kind
([§8.1](physical-semantics.md#section-8-1)). Each refusal names the variable
(`modeling.domain`, analysis `preparation`). Non-integral bounds on an integer variable are
passed on unchanged: the recorded inward tightening of ADR-0103 is not implemented. A fixed
variable needs no search range; its value is checked for exact domain membership instead
([§7.6](#section-7-6)). `CaseStructure::key` frames each domain by its registry spelling,
and every native constraint form below (`pse.math.case-structure.v3`).

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
  `Relaxed`. An exhausted node budget leaves the affected rows `Unavailable` and records
  the instance as incomplete.

The SCIP route ([§18.10.1](numerical-execution.md#section-18-10-1)) projects under a request
that carries the case's implicit definitions (`factorable_definitions`, where a case bound
on an unknown replaces that endpoint's bound hint, as it does for the evaluator) and the
declared output envelope of every registered provider that has one
(`Registration::envelope`, checked against the provider's contract;
[§9.4](physical-semantics.md#section-9-4)). The envelopes enter the program's identity. A
provider output without a declared envelope has no finite box, so an export with such an
output inside a nonlinear term is refused; no production provider declares an envelope yet.
*Tested* by `implicit_residual_exported_exactly` and `relaxed_rows_enclose_evaluator`
(`pse-math` units), `certify_exports_implicit_residuals_exactly` (runtime units) and
`provider_envelope_is_checked_against_the_contract` (`pse-kernels` units).

**Presolve facts.** `CasePlan::presolve_facts` derives FBBT tapes and obligation admission
from this projection, under the default request (no implicit definitions or envelopes;
auxiliary branches). Affine proofs and the objective degree still use the optional
flattened expressions. A large factorable body therefore keeps a complete tape, a
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

Modeling documents use `pse-authoring::language`; embedded expressions use
`pse-authoring::dsl`. The registry-generated declaration IR retains each source occurrence,
identity and span. The checked package owns resolved types, visibility and physical context;
consumers cannot mutate a checked product or substitute a foreign physical revision.

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

> Decision: [ADR-0104](../../adr/0104-discrete-constraint-forms-and-realizations.md) —
> named lowerings of indicator, SOS, cardinality, piecewise, logic and disjunction
> declarations join generic specialization, and their derived realization parameters join
> view preparation (Plan 22 M3 and M4, implemented;
> [§19.7](workflows-and-results.md#section-19-7)); complementarity (M5) is not yet
> implemented.

| Stage | Owner | Consumes | Produces | Effects |
|---|---|---|---|---|
| Package admission | runtime modeling admission ([§22](models-and-composition.md#section-22)) | exact package closure, generic declarations, physical inventory and aliases | immutable checked revision | source loading and bounded admission |
| Generic specialization | `pse-modeling`, tracked by compiler modeling queries | checked package, root/instance, bindings, analysis and limits | demanded members, equations, named lowerings of constraint forms and disjunctions, lineage, checks and reports | none |
| Input publication | `CompilerWorkspace` | checked package/context and lowered case inputs | Salsa input revisions | validated before setters |
| Definition admission | `admitted` query → `typed_math::Request::admit` | definition source, formals, domains, groups, providers, units, physical registry | `AdmittedBody`: `BodySpec`, types, occurrences, `PreparedBody` | none |
| Case planning | `plan` query → `CasePlan::prepare` | case structure, semantic bodies | immutable `CasePlan` with faer patterns and local demands | none; no evaluators |
| Modeling view and rebind | `PreparedModeling::bound_structure`, `CompilerWorkspace::prepare_modeling_view`, `PreparedCase::rebind` | bound case structure, derivation rules, case values | a `PreparedCase` view identified by `view_key`, whose value products are rebound per values ([§14.4](#section-14-4)) | none; no evaluators |
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

Set-oriented integrity over relations remains in Arrow/DataFusion. The registry declares
invariants as native DataFusion SQL returning offending keys
(`pse_schema::model::invariant::InvariantSpec`) and declares residual construction
obligations ([§4.6](schema-and-relations.md#section-4-6)). `pse-relations::validate`
lowers declared row predicates and reference, ordinal, quantity and source-span obligations
to native plans; scalar predicates also persist as Delta `CHECK`s. The engine binds
registry invariants into bounded obligation plans at admission and publication
([§22.2](models-and-composition.md#section-22-2)). `pse-rules::invariants` executes the same
declarations for Python inspection requirement planning and physical-fixture validation.

There is no production rule engine or recursive inference executor. Operand-level physical
prerequisites are not relational queries: they are `PhysicalPrecondition` predicates
checked against actual operands during typed inference ([§7.3](#section-7-3)). A future
recursive-inference consumer needs its own reviewed contract.

**Source owners:** `crates/pse-schema/src/model/invariant.rs`,
`crates/pse-relations/src/validate/`, `crates/pse-engine/src/session/obligation.rs`,
`crates/pse-rules/src/invariants.rs`.

### 14.3 The preparation engine and ownership

`CompilerWorkspace` is a single-writer owner of one Salsa database. Callers serialize
access; no database clone, Salsa handle or partial input batch escapes. Tracked queries are
pure: they may read inputs, parse, admit and plan, but never construct native evaluators,
touch caches, run solvers or write data. Cancellation is checked between steps, surfaces as
`CompileError::Cancelled`, and is never memoized as a result. Errors are typed
(`Missing`, `Syntax`, `Math`, `Structure`, `Limit`, `Cancelled`) with diagnostic codes
([§23](operations-and-validation.md#section-23)).

Salsa inputs carry the complete selected inventory: checked modeling packages and bindings, quantity
registry, physical preconditions, definitions, domains, groups, external capability descriptors,
cases, fixed/parameter values (as canonical bits) and the math environment identity.
Negative lookups are tracked, so creating a previously missing name invalidates its
readers. Revision preparation publishes selected inputs under the runtime's compiler lock,
so a shared workspace cannot mix two revisions during one preparation. Returned products
(`PreparedCase`, `CasePlan`, artifact requests) are owned values that outlive a workspace
generation and keep their own allocation owner.

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
| Body construction (`BodyLimits`) | 4,096 formal slots; 16,384 construction occurrences; syntax, filter and conditional depth 128; integral power degree 1,024 |
| Local evaluation (`EvaluationLimits`) | 4,096 derivative components; 1,000,000 scalar operations per demand; 64 MiB worker scratch; 4,096 provider calls |
| Optimizer (`Optimization`) | cores 1–64 (default 1); Horner iterations up to 1,000 (default 10); common-pair rounds up to 32 (default 1) |
| Case (`CaseLimits`, `AssemblyLimits`) | 100,000 scalars, instances and rows; 1,024 bodies; 1,000,000 slots and contributions; native index within `i32`; 256 MiB aggregate worker scratch |
| Workspace (`WorkspaceLimits`) | 4,096 input entries; 2 GiB input extent; 16,384 retained entries and 256 MiB known retained bytes; LRU of 64 values per expensive query |
| Structural matching | 100,000 rows on a 32 MiB stack |
| Runtime (`MathPolicy`) | 8 GiB artifact retention; 64 MiB foreign allowance per job/program; 2 GiB worker, 4 GiB workspace; 4 jobs; 128 flights |

Derivative admission bounds the pinned Numerica Taylor convolution and primitive expansion
from the source operation counts before vectorization, then reconciles the actual built
operations. Opaque or provider derivative support wider than 256 local coordinates is
refused; this is not a global model-size limit. Shared bindings count once, preventing
exponential growth from repeated substitution. `ModelingLimits.body_occurrences` may raise
the default occurrence budget explicitly; the 4,096-slot body bound remains. Limits are
tracked policy inputs and cannot authorize a differently typed or truncated model.

`MathPolicy` allowances draw from the deployment memory pool, never a second budget.
Immutable prepared products, artifacts, active worker scratch, foreign allowances and
completed results are charged separately at their actual ownership lifetimes; one shared
CPU semaphore serves data and math work, and each optimizer reserves its cores once.
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

> Decision: [ADR-0089](../../adr/0089-semantic-identity-projections.md) — complete identity
> projections. Plan 22 A6 (implemented) adds value-only rebind of prepared views and bounded
> package views keyed on bound structure.

Salsa reuse is valid only when a fresh workspace given the same admitted inputs would
produce an equal product. Queries depend on exact fields, including absence; unchanged
results backdate so unrelated edits do not propagate. Durability follows semantic
stability: environment, registry and preconditions are high; definitions, domains, groups,
providers, cases and flows are medium; values are low.

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
states, admits discrete domains ([§7.5](#section-7-5)) and excludes observation rows; it
reads no value. `PreparedModeling::view_key` (`pse.compiler.modeling-view.v2`) is the
complete identity of the view prepared from that structure: the bound structure key,
including native constraint forms; every admitted body with its source occurrences; the rules
of the derived realization parameters; the derivative order; the evaluator profile; and the
physical context. Equal keys give equal plans, structural analyses, derivations, artifact
requests and provenance. `CompilerWorkspace::prepare_modeling_view` prepares the view and
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

The runtime keeps these products per package revision (`workflow/modeling/views.rs`): up to
16 solver views and 16 observation programs, each keyed by its view key and evicted least
recently used first. The first request for a structure prepares it; every later one rebinds
(`MathService::rebind`), which runs no job when no consumed value changed and otherwise
rebuilds only the value products on one admitted worker. A new package revision starts with
no views. A study of five points that differ only in values therefore prepares one view
(`value_only_study_prepares_once`).

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
General implicit or higher-index integrated DAEs, global MINLP, JIT/SIMD evaluators, GPU
and distributed execution are not admitted. Nested branch derivatives are local and
refuse unproved crossing/selection behavior. Vectorized
evaluators must not call `optimize_stack()` after vectorization. Qualification basis:
[§24.2](operations-and-validation.md#section-24-2).

## Retired section identities

#### 14.2.1 Optional predicate preimages — retired

No component consumes DataFusion's `ScalarUDFImpl::preimage`; its eligibility falls under
the general library-access rule in [§3.3.1](workspace-and-dependencies.md#section-3-3-1).
