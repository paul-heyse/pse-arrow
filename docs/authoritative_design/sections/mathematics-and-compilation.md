---
title: Mathematics and compilation
status: current
---

# Mathematics and compilation

This area turns an admitted, physically typed process definition into immutable
library-owned mathematical programs and a case layout that native solvers can consume.
Authored definitions and bindings stay the model authority; Symbolica/Numerica own
arithmetic, normalization and derivatives, and faer owns sparse structure. `pse-math`
holds the narrow library integration, `pse-compiler` owns finite specialization and pure
Salsa preparation, and `pse-runtime::math` owns effectful artifact construction, retention
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

The authored function vocabulary is the `strum`-derived `pse_quantity::functions::Function`
enum; each spelling maps to exactly one implementation, never a separate support flag.
Parsing accepts only declared spellings; removed names such as `smooth_max`, `safe_log`,
`tanh`, `erf` or `weighted_mean` fail as syntax, with no numerical fallback.

| Form | Implementation | Physical rule | Original-domain obligation | Derivatives |
|---|---|---|---|---|
| `+`, `-` | library arithmetic | compatible complete types; point minus point yields a difference | — | smooth |
| `*`, `/` | library arithmetic | registered product/quotient | divisor nonzero unless a nonzero literal | smooth off the obligation |
| `^` | library power ([§7.4](#section-7-4)) | rational exponent scales dimension; symbolic exponent needs a dimensionless base | integral `n <= 0`: base nonzero; non-integral: base positive | smooth off the obligation |
| `exp`, `log`, `log10`, `sin`, `cos` | guarded library scalar | dimensionless in and out | `log`, `log10`: argument positive | smooth |
| `tan` | `sin/cos` | dimensionless | cosine nonzero | smooth off the obligation |
| `sqrt` | guarded library scalar | half dimension | value: nonnegative; first/second order: positive | `sqrt(0)` is value-valid, derivative-ineligible |
| `abs`, `min`, `max` | guarded branch selection | same complete type | — | value-only where the switching input is differentiated |
| `sum`, `prod` over a finite domain | library fold of admitted terms | declared reduction rule | — | as the terms |
| `if … then … else` | lazy branch regions | branches share one complete type | each branch keeps its own obligations | value-only where the guard input is differentiated |
| `kernel.<name>(…)` | admitted provider output | provider port contracts ([§9](physical-semantics.md#section-9)) | provider validity/envelope | capped by provider derivatives and smoothness |
| `convert`, `broadcast` | unavailable | authoring and inspection only | — | refused for execution |
| `d(…)/dt`, `integral` | not algebraic | — | — | refused by algebraic lowering; dynamics bind time derivatives through dynamic case roles ([§13](workflows-and-results.md#section-13)) |

Comparisons in guards (`==`, `!=`, `<`, `<=`, `>`, `>=`) require the same complete physical
contract and binders on both sides and select exactly, without tolerance. Finite filters in
reductions accept only Boolean structure over lexical membership (`in`, member equality).
The broader operation vocabulary in `pse-quantity` describes physical compatibility; it
does not advertise evaluator support. A new function is an extension of this enum plus
its physical rule and `BodyBuilder` constructor, with positive and negative controls.

**Source owners:** `crates/pse-quantity/src/functions.rs`,
`crates/pse-math/src/typed.rs` (`unary`, `binary`, `extremum`, `reduce`, `conditional`).

### 7.3 Operation contracts: physical admission and domain obligations

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

**Source owners:** `crates/pse-math/src/{typed,guarded,execution,error}.rs`;
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

A selected case is a `pse_math::binding::CaseStructure`: variables with declared domain
(`Continuous`, `Integer`, `Binary`, `SemiContinuous`, `SemiInteger`), fixed/free status and
optional closed bounds; ordinary parameters; instance bindings of body formals to global
sources; semantic rows with closed canonical bounds; and an optional objective with its
authored sense. A body output reaches rows or the objective through an explicit
contribution map with finite nonzero dimensionless weights. Authored `lhs == rhs`,
`<=` and `>=` equations become residual `lhs - rhs` rows with bounds `[0, 0]`,
`(-inf, 0]` or `[0, inf)`; a conditional equation selects its branch during template
specialization ([§10](models-and-composition.md#section-10)).

Mathematical class is established from the admitted program, never from authored hints:
`ProblemFacts` records admitted derivative order, bound shapes, guarded status, proved
affine rows, objective degree and the bound/value assumptions that established them.
Polynomial degree two does not establish convexity; exact or explicitly qualified
convexity evidence is separate ([§18](numerical-execution.md#section-18)). Value-dependent
facts carry the identity of the fixed/parameter values they consumed; free trial values
never establish them.

**Source owners:** `crates/pse-math/src/{binding,facts,coefficients,presolve}.rs`,
`crates/pse-runtime/src/workflow/composition/lower.rs`. Declaration columns are in the
generated [`authored.computation_models`](../../generated/relations/authored.md) reference.

### 7.6 Null, bound, and unknown semantics

| Situation | Representation |
|---|---|
| Variable to be solved | a free variable in `CaseStructure`; its fixed/free status changes the case layout, not the body |
| Starting value | every variable and parameter has a finite case value; `validate_values` refuses missing or nonfinite entries. Start selection is owned by strategies ([§17](numerical-execution.md#section-17)) |
| Unbounded variable or row side | a null bound in the case declaration, or `BoundKind::Unbounded` in template bounds; lowered to `None` or an outward infinity, never NaN |
| Case value outside declared bounds, or fixed value outside its integer domain | refused at case admission, without solver tolerance; a semi-variable's zero is admitted |
| Missing measurement | a null observation value; an included observation without a value or standard deviation is refused, and exclusion is the explicit `included` flag ([§19](workflows-and-results.md#section-19)) |
| Domain or provider failure | a typed `MathError`, never a silent null or a stale value |
| Missing versus empty input | distinct: a missing domain, group or provider is an error; an empty admitted domain is a valid finite domain |
| Undecided selected meaning | refused at admission ([§14.5](#section-14-5)), never consumed as true |

### 7.7 The expression DSL

Authored text is the expression authority; the AST is transient syntax. The same parser
serves computation-model definition sources, template equations and guards, document
editing and inspection. The compiler parses definition sources inside the admission query,
so parse failures are typed `CompileError::Syntax` diagnostics carrying the definition and
exact byte range. Parse → render → parse is the identity on the AST structure; spans are
diagnostic metadata excluded from body identity.

```ebnf
expr       := arith [ "where" ident "=" arith { "," ident "=" arith } ] ;
arith      := term { ("+" | "-") term } ;
term       := power { ("*" | "/") power } ;
power      := unary [ "^" power ] ;                  (* right-associative *)
unary      := "-" unary | primary ;                  (* "-x^2" is refused as ambiguous *)
primary    := number [ "{" unit "}" ]                (* 320{K}, 2{bar}, 1{mol/s} *)
            | "(" expr ")"
            | "if" predicate "then" expr "else" expr
            | "kernel" "." ident { "." ident } "(" [ args ] ")"
            | reduce "(" ident "in" path [ "where" predicate ] "|" expr ")"
            | "d(" expr ")/d" path
            | function "(" [ args ] ")"
            | path ;
reduce     := "sum" | "prod" | "integral" ;
path       := segment { "." segment } ;  segment := ident [ "[" expr { "," expr } "]" ] ;
predicate  := disjunct { "or" disjunct } ;  disjunct := atom { "and" atom } ;
atom       := "not" atom | "true" | "false" | "null" | "(" predicate ")"
            | arith ( "in" path | cmp arith )? ;
cmp        := "==" | "!=" | "<" | "<=" | ">" | ">=" ;
equation   := "if" predicate "then" equation "else" equation
            | expr ( "==" | "<=" | ">=" ) expr ;
function   := (* a spelling of the Function enum, §7.2 *) ;
```

Input is bounded to 65,535 bytes and nesting depth 64 (unit expressions also 64). `where`
bindings are ordered, distinct and lexically scoped; each binding is materialized once as a
shared block output instead of being substituted repeatedly. Indexed paths bind lexical
reduction members to declared group axes; a group's actual tuple must exist, so unknown
ragged members fail. Grammar forms outside [§7.2](#section-7-2) parse for authoring and
inspection but are refused by execution admission.

**Source owners:** `crates/pse-authoring/src/dsl/{parser,lexer,ast,render}.rs`; examples
and round-trip controls in `crates/pse-authoring/tests/dsl_examples.rs`.

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

| Stage | Owner | Consumes | Produces | Effects |
|---|---|---|---|---|
| Selected admission | `pse-runtime::workflow` ([§22](models-and-composition.md#section-22)) | authored relations or typed builders, physical inventory | immutable `ModelRevision` | none on compiler state |
| Template specialization (composed models only) | `pse-runtime::workflow::composition` | templates, instances, selected configuration | computation-model definitions and case rows | none |
| Input publication | `CompilerWorkspace::publish` | complete `Inputs` projected from the revision | Salsa input revisions | validated before any setter |
| Definition admission | `admitted` query → `typed_math::Request::admit` | definition source, formals, domains, groups, providers, units, physical registry | `AdmittedBody`: `BodySpec`, types, occurrences, `PreparedBody` | none |
| Case planning | `plan` query → `CasePlan::prepare` | case structure, semantic bodies | immutable `CasePlan` with faer patterns and local demands | none; no evaluators |
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

Salsa inputs carry the complete selected inventory: flowsheet declarations, quantity
registry, physical preconditions, definitions, domains, groups, provider descriptors,
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
refused; this is not a global model-size limit. `where` bindings count once, preventing
exponential growth from repeated substitution.

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

Salsa reuse is valid only when a fresh workspace given the same admitted inputs would
produce an equal product. Queries depend on exact fields, including absence; unchanged
results backdate so unrelated edits do not propagate. Durability follows semantic
stability: environment, registry and preconditions are high; definitions, domains, groups,
providers, cases and flows are medium; values are low.

| Change | Re-admitted or recomputed | Reused |
|---|---|---|
| Free-variable case value | value-reading fact queries re-check and backdate; they consume only fixed/parameter values | bodies, case plan, structure, artifacts |
| Fixed or parameter value | coefficient, presolve and class facts that consumed it | bodies, case plan, structure, artifacts |
| Fixed/free status, bounds, rows, bindings or contributions | case plan and its dependents | admitted bodies |
| One definition's source, formals, units or literal contracts | that definition's body and dependent plans | other bodies with equal keys |
| Domain membership, group tuples or provider descriptor | definitions that read them | independent definitions |
| Quantity registry or physical preconditions | every body (complete physical identity) | nothing semantic |
| Optimizer or evaluation profile | artifact requests and programs | bodies and plans |

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
Continuous-domain derivatives and integrals, general implicit or higher-index DAE, global
MINLP, JIT/SIMD evaluators, GPU and distributed execution are not admitted. Vectorized
evaluators must not call `optimize_stack()` after vectorization. Qualification basis:
[§24.2](operations-and-validation.md#section-24-2).

## Retired section identities

#### 14.2.1 Optional predicate preimages — retired

No component consumes DataFusion's `ScalarUDFImpl::preimage`; its eligibility falls under
the general library-access rule in [§3.3.1](workspace-and-dependencies.md#section-3-3-1).
