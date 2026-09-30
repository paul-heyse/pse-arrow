# Form expressions (v0)

This page specifies how the mathematics of a form is declared, checked and evaluated. It extends
`meta-model.md` section 4 and is the contract for the expression engine. It records intended
design; where code exists and disagrees, the code is what runs and this page is wrong.

A form with `status = "catalogued"` has no expression. A form becomes `expressed` when every
output of its contract has an expression that parses and closes dimensionally. Those are the two
statuses a declaration states. Whether a form is qualified is not declared: it follows from
recorded qualification runs, and the view `qual.form_qualification` lists, for each form and
output, the passing runs whose recorded hash equals the output's current `evaluation_hash`
(`pipeline.md` section 5). Editing the expression of a form ends its qualification without any
declaration changing.

## 1. Principles

- **The expression is data.** It is written once, in the form's declaration, and every backend
  (evaluation, `.pse`, LaTeX) is derived from the same typed tree.
- **The constructs are those of the `.pse` function language**, so rendering to `.pse` is
  mechanical. The constructs that language lacks today are marked in section 6 and feed the
  kernel-gap register.
- **No bespoke parser.** The authoring syntax is a restricted Python expression, parsed by the
  standard `ast` module and then validated node by node against the grammar below. Anything
  outside the grammar is refused; nothing is ever executed with `eval` or `exec`.
- **Dimensions close.** Every name has a quantity type. Addition and comparison need equal
  dimensions; transcendental functions need dimensionless arguments; each output's dimension
  equals its contract type's. A number has no unit until it is multiplied by `unit(...)`.
- **An expression never reads anything it does not name**: no globals, no clock, no constants
  that are not slots, convention facts or literals. The gas constant is a convention fact of the
  parameterization or an argument, never an ambient value.

## 2. What a contract declares

A contract states what an evaluation is about and what it takes and gives.

```toml
[contracts.residual_helmholtz_mixture]
doc = "Reduced residual Helmholtz energy of a mixture."

[contracts.residual_helmholtz_mixture.sets]
components = { type = "material_entity", doc = "The components of the mixture, in a fixed order." }

[contracts.residual_helmholtz_mixture.roles]
# single subjects, for contracts about one entity or one pair; empty here

[contracts.residual_helmholtz_mixture.arguments]
T = { type = "Temperature", doc = "Temperature." }
rho = { type = "MolarDensity", doc = "Molar density." }
x = { type = "Fraction", over = ["components"], basis = "mole_fraction", doc = "Composition." }

[contracts.residual_helmholtz_mixture.outputs]
alpha_r = { type = "Scalar", observable = "molar_residual_helmholtz_energy", doc = "Residual Helmholtz energy divided by RT." }
```

- An output's observable is named (`observable = "vapor_pressure"`), taken from the data
  (`observable_from_set = true`) or absent. One equation that gives the vapour pressure of a pure
  fluid and the bubble or dew pressure of a pseudo-pure one is one form: its contract output takes
  the observable from a set, and the form declares which required observable slot of which slot
  group supplies it (`[forms.F.output_observables] p = "pure.quantity"`, meta-model section 4.1).
  The expression is unchanged by this; the observable a set names must have the dimension of the
  output's declared type, which the canonical writer checks when it writes the set.
- `roles` are single subjects (a species form, a reaction, an adsorbate and a host).
- `sets` are index sets of entities the evaluation ranges over.
- An argument may be indexed `over` one or more sets and, when it is a composition, names its
  `basis`: a declared entity of the kind bound to the framework role `composition_basis`
  (meta-model sections 4.1 and 5), which `meta.contract_argument.basis` references.
- An argument may name the observable it is a value of:
  `arguments.T = { type = "Temperature", observable = "temperature", doc = "..." }`. The observable is a
  declared entity of the kind bound to the framework role `observable`, checked as an output's is and
  reified in `meta.contract_argument.observable`. The argument's type has the dimension of the
  observable's quantity type, as an output's does. A clause of a validity region limits an observable
  (meta-model section 5); the argument that names the observable is the one the clause limits, so the
  binding of a region's clause to an argument is stated here, in the contract, and a qualification
  case does not have to say it. An observable that two arguments name binds to neither. The contracts
  of the committed forms name `temperature` for `T`.
- A form implementing the contract binds its slot-group subject roles to the contract's roles
  and sets. A subject role binds to the contract role or set of the same name; when the names
  differ the slot group states `bind = { i = "components", j = "components" }`. The subject's
  kind and the role's or set's kind must be compatible (one refines the other). An `expressed`
  form binds every subject role; a `catalogued` form may leave them unbound.
- A form that calls a sub-form through a contract with sets passes the sets and the indexed
  arguments in the call (section 3, Calls).

## 3. What a form declares

```toml
[forms.antoine]
implements = "pure_vapor_pressure"
status = "expressed"

[forms.antoine.let]                      # local bindings, evaluated in order
theta = "T + pure.C[i]"

[forms.antoine.outputs]                  # one expression per contract output
p_sat = "exp(pure.A[i] - pure.B[i] / theta) * unit('Pa')"
```

### Names

| Written | Refers to |
|---|---|
| `T`, `x[i]` | a contract argument, subscripted by its index sets |
| `i`, `j` | a contract role, or an index variable introduced by `sum`, `prod` or a comprehension |
| `pure.A[i]` | slot `A` of slot group `pure` for the subject `i`; a pair group is `pair.k[i, j]`; a group with an empty subject is `core.omega_a` |
| `pure.piece.a1[i, n]` | slot `a1` of family `piece` for subject `i` at index `n` |
| `pure.piece(i)` | the index set of family `piece` for subject `i`, for use in `for n in ...` |
| `theta` | a local from `let`; `r[i]` for a local indexed over sets |
| `X`, `X[a]` | an unknown of an implicit block, subscripted by its sets when it has any |
| `convention.gas_constant` | a convention fact the form declares in `conventions`, of the convention set of the parameterization in force; it has the type of the convention-set attribute of that name |
| `convention.gas_constant[i]` | a convention fact the form declares in `component_conventions`: that of the parameterization that supplied the set of its slot group for the component `i` of a contract set |
| `alpha.value(i=i, T=T)` | output `value` of the form chosen for sub-form slot `alpha`, called with the accepted contract's roles, sets and arguments by keyword |
| `c.alpha_r(...) for c in residual` | each contribution of a sub-form slot with multiplicity `many` |

A transposable slot is written in either orientation; the parameter source returns the values
for the orientation as written, applying the group's transposition rule when it holds the set for
the other (section 5). A slot whose value is a nested set is called like a sub-form:
`pair.a[i, j].value(T=T)`. A slot that references a set is called the same way
(`pair.departure[i, j].alpha_r(delta=delta, tau=tau)`, and `param.term.function[i, n].value(T=T)`
for a slot of a family, subscripted by the row's index): the call follows the reference. The
called contract's roles that the call does not give are the subjects of the referenced set, bound
as its slot group binds them; a call through a nested set names them all.

A sub-form slot with multiplicity `many` or `optional` and `per = "subject"` is iterated for a
subject, `for c in terms(i=i)`; with `per = "model"` it is iterated bare, `for c in residual`.

Names in expressions are not Python keywords or grammar function names, and one name is not
reused across arguments, roles, sets, slot groups, sub-form slots, locals and unknowns.

### Convention facts

```toml
[forms.nasa7]
conventions = ["gas_constant"]

[forms.nasa7.let]
R = "convention.gas_constant"
```

A form lists in `conventions` the facts of its parameterization's convention set that it reads; each
names a quantity-typed attribute of the kind bound to the framework role `convention_set`, and an
unknown or non-quantity attribute is a located diagnostic. `convention.<fact>` has the dimension of
that attribute's type and is checked like any quantity. Reading a fact the form has not declared is
refused. `convention` is a reserved name: no argument, role, set, slot group, sub-form slot, local or
unknown may have it.

A fact the parameterizations of a mixture's components may state differently (the gas constant a multifluid
mixture averages over its components) is declared in `component_conventions = { gas_constant = "pure" }`:
the slot group, whose one subject is bound to a contract set, names where each component's fact comes from,
and `convention.gas_constant[i]` reads it for the component `i`, with the same type. A fact is read one way
or the other: `convention.gas_constant` for a fact in `conventions`, `convention.gas_constant[i]` for one in
`component_conventions`, and the wrong form is a located diagnostic.

### Grammar

| Construct | Syntax |
|---|---|
| number | `1`, `0.45724`, `1e-3` |
| unit literal | `unit('Pa')`, `8.314462618 * unit('J/(mol*K)')`; the string is a `pint` unit |
| arithmetic | `+ - * / **`, unary minus, parentheses |
| elementary functions | `exp log log10 sqrt abs sinh cosh tanh sin cos atan min max` |
| special functions | `erf`, `chebyshev_t(n, x)`, `debye(n, x)` (gap) |
| finite sum and product | `sum(expr for i in S)`, `prod(expr for i in S)`; nested `for` clauses allowed |
| indexed value | `[expr for i in S]`, `[expr for i in S for j in T]`: one element for each combination of members of contract sets; only as the value of a local or a call argument |
| asserted basis | `basis('mole_fraction', [expr for i in S])`: an indexed value whose author asserts the composition basis of the vector it builds; the name is a declared `composition_basis` entity; only where an indexed value may be written |
| index sets | a contract set, a family index set, `range(a, b)` with integer bounds |
| conditional | `a if condition else b`; comparisons `< <= > >= == !=`; `and`, `or`, `not` |
| partial derivative | `d(expr, name)` with respect to an argument or a local |
| piece selection | `at(pure.piece(i), T)`: the index of the piece whose interval contains `T` (gap) |
| definite integral | `integral(expr, z, a, b)` in one scalar variable (gap) |
| remapped call | a sub-form call whose arguments are expressions, not only names (gap) |

Integer exponents, index arithmetic and `range` bounds are dimensionless integers. A power of
a dimensioned base takes a literal integer exponent; `sqrt` of a dimensioned argument needs even
dimension exponents; the body of `prod` is dimensionless. Subjects compare with `==` and `!=`
only. Chained comparisons and `if` filters inside a `sum` generator are outside the grammar.

### Calls through contracts with sets and indexed arguments

A call gives each role, set and argument of the accepted contract by keyword, once:

```toml
a_mix = """
b * (sum(x[i] * a_pure[i] / b_pure[i] for i in components)
     - excess.gE(components=components, T=T, x=[x[i] * b_pure[i] / b for i in components]) / log(2))
"""
```

- **A set** is given as the name of a set of the calling form's contract: `components=components`.
  The kind of that set and the kind of the accepted contract's set must be compatible (one
  refines the other). The callee ranges over the caller's members in the caller's order, so its
  slot groups bound to its set read their parameters for the caller's components.
- **An indexed argument** is given as the name of an indexed argument, indexed local or unknown of
  the caller (`x=x`), or as an indexed value written in the call (`x=[...]`). The sets it ranges
  over, in order, are the caller's sets passed for the sets of the callee's argument, and its
  dimension is the callee's. `x=x` passes the composition through unchanged; a comprehension
  remaps it.
- **The basis.** When the callee's argument names a basis, the value passed is on that basis: an
  argument or a local passed by name has that basis (a local is given one by
  `basis('<name>', [...])`), and a comprehension written in the call is wrapped as
  `basis('<name>', [...])`, which the form's author writes to assert the basis of the vector it
  builds. An unwrapped comprehension, a value on another basis and a value that states none are
  refused; when the callee's argument names no basis nothing is compared. The wrapper changes no
  value: it is a statement by the form's author that the checker holds the form to, part of the
  expression's canonical text and so of its evaluation hash.
- A role or a scalar argument is given as before, a subject or a number.

An indexed value, `[expr for i in S for j in T]`, has one element `expr` for each member of `S`
and of `T`; each `for` iterates a set of the contract and its variable is scoped to `expr`. The
dimension of the value is that of `expr`. It is the value of a local, which is then subscripted
`r[i]` like an indexed argument, and can be passed by name to a call; used bare in arithmetic it
is refused. A derivative cannot be taken with respect to an element of an indexed local.

### Implicit forms

A form whose outputs are defined by equations rather than formulas declares an implicit block:

```toml
[forms.association.implicit.site_fractions]
doc = "Fraction of sites of each type not bonded."
unknowns.X = { type = "Fraction", over = ["sites"], lower = 0, upper = 1, start = 0.5 }
residuals = ["X[a] * (1 + rho * sum(x[b] * X[b] * pair.delta[a, b] for b in sites)) - 1 for a in sites"]
select = "unique"            # unique | smallest | largest | by: <expression to minimise>
```

- **`unknowns`**: each has a quantity `type`, optional `over` sets of the contract, and optional
  `lower`, `upper` and `start`. A bound or start is a number, in the storage units of the type, or
  an expression, which has the dimension of the type and may use arguments, slots, sub-form calls
  and locals. One bound holds for every element of an unknown over sets. A `start` is required
  unless the unknown has both bounds, when it is their midpoint.
- **`residuals`**: expressions `g` standing for the equations `g = 0`. An expression followed by
  `for a in S` clauses states one residual for each member of `S` (for each combination when there
  are several clauses). After expansion there are as many residuals as unknown elements: each
  unknown counts once, or once for each combination of members of its sets, and a residual counts
  the same way, so `X` over `sites` needs residuals that amount to one per site.
- **Dimension of a residual.** A residual is one expression whose terms share one dimension,
  which the rule for `+` and `-` already enforces: write both sides of the equation moved to one
  side. The shared dimension is arbitrary, and different residuals may differ. A residual is
  compared with the size of its own terms, never with a fixed tolerance.
- **`select`** names the root when the equations have several. `unique` asserts the solve finds
  exactly one. `smallest`, `largest` and `by: <expression>` (the root at which the expression, a
  number, is least) apply to a block with one unknown, not over a set, that has both bounds: the
  roots in the interval are found and chosen among.

Scope. The residuals, bounds, starts and `by` expression of a block use the arguments, slots,
sub-form calls and the locals that do not depend on any unknown, directly or through other
locals. The residuals and the `by` expression also see the unknowns of their own block, and those
of no other block; bounds and starts see none. The outputs, and the locals that do depend on an
unknown, see the unknowns of every block. The locals that do not depend on an unknown are evaluated first, then the blocks, then the
locals that do, each group in declared order.

## 4. Checking

Loading a declaration with expressions refuses, with a diagnostic that names the form, the output
or local and the position in the text. Expressions are checked by dimension, not by quantity type:
two quantity types of one dimension are interchangeable inside an expression (a `Temperature` and a
`TemperatureDifference` add without complaint). Only the declared basis of a composition is compared beyond the dimension, at calls. A
slot or an output of a dependent quantity type (meta-model section 3.3) has the dimension of one opaque symbol,
`[RateConstant]` for a rate constant, times a bulk concentration to the power minus its extra order: two such
values add or compare only when they have the type and the extra order in common, and the concrete dimension
follows the reaction of each set and is checked when the set is written. The refusals:

- syntax outside the grammar; an unknown function; a call to anything else
- an unknown name, a convention fact the form does not declare included; a slot subscripted with the
  wrong number or kinds of subject
- an index variable used outside its `for`; a sum over something that is not an index set
- an argument, output or slot whose `observable` is no declared entity of the `observable` kind; an
  argument or output whose type has another dimension than the quantity type of its observable
- a dimension mismatch, stating both dimensions; a transcendental function of a dimensioned
  argument; a non-integer power of a dimensioned base
- an output whose dimension differs from the contract's
- a missing output, or an expression for an output the contract does not have
- a local that refers to a later local; a cycle among sub-form contracts
- a call that omits a set, gives a set of an unrelated kind or something that is not a set of the
  calling form's contract, gives an indexed argument a scalar, or a vector over other sets or of
  another dimension than the callee's; a bracketed comprehension anywhere but as the value of a
  local or a call argument, or over anything but the sets of the contract; an indexed local used
  bare in arithmetic
- a vector passed to an argument that names a composition basis on another basis, on none, or as an
  unwrapped comprehension; `basis(...)` with a name that is no declared `composition_basis` entity,
  with arguments other than a string and a bracketed comprehension, or anywhere but as the value of
  a local or a call argument
- an implicit block whose residuals, after expansion, do not number its unknown elements (stating
  both counts); a residual or a bound of the wrong kind or dimension; an unknown that is not a
  quantity, ranges over a set the contract lacks, or has no `start` and lacks a bound; `select`
  other than `unique`, `smallest`, `largest` or `by:` with an expression; `smallest`, `largest` or
  `by` on a block that does not have exactly one unknown, not over a set, with both bounds; a block
  that uses an unknown of another block or a local that depends on an unknown of its own block; a
  derivative of an expression that depends on an unknown with respect to a local

## 5. Evaluation

The reference evaluator exists to qualify forms, not to be fast.

1. **Bind**: a concrete subject (or an ordered list of components), the parameter sets selected
   for it, and the forms chosen for its sub-form slots.
2. **Expand**: sums, products and comprehensions are expanded over the concrete index sets and the
   result is one SymPy expression per output in the argument symbols and **parameter symbols**.
   A slot reference, a family-row value and a `unit(...)` conversion factor are each a symbol
   (named in the order the expansion first reads them, and the same symbol when it reads the same
   stored value again); the stored number, in storage units, is kept beside the expression and is
   never put into it. A call to a sub-form is expanded in its own frame, bound to the sets and
   the indexed values its caller passed, reading its own source through the same symbols. An
   implicit block expands to one symbol for each scalar unknown, the residual vector, the bounds
   and starts, and its selection rule.
3. **Differentiate** symbolically where `d(...)` occurs. Through an implicit block the derivative
   of an expression with respect to an argument `u` adds, for the unknowns `y` the expression
   contains, `d expr / dy * dy/du` with `dy/du = -g_y^-1 g_u` (implicit-function theorem,
   `g` the residuals), so the unknowns are not held fixed; it applies again to a derivative of
   a derivative. A derivative with respect to a local of an expression that depends on an unknown
   is refused.
4. **Compile** with `lambdify` to NumPy and evaluate at arrays of argument values, in storage
   units, passing the parameter values beside the argument arrays. For an output that contains
   unknowns, each block is first solved at every point of the argument values.

Because stored values and arguments are in coherent SI storage units and `unit(...)` literals are
converted to the same, evaluation needs no unit handling of its own; dimensional closure was
established at load.

A missing required parameter set is an evaluation refusal that names the slot group and subject.
It is never a zero. A default exists only where the parameter source supplies one explicitly
(a named combining rule or a stated default, which a selection policy records as rule-derived);
the evaluator asks for it only after the source has found no set for the subject in any order its
transposition makes equivalent.

`at(...)` selects by half-open intervals `[lower, upper)`, with the last piece closed at its
upper bound; an argument outside every piece is refused when the expression is evaluated. That is the
default reading of the stored pieces. A binding may be given a `PiecePolicy` (`bind(..., pieces=...)`, which a
qualification case states in its `[pieces]` table): `boundary = "lower_piece"` reads the pieces as
`(lower, upper]`, with the first piece closed at its lower bound, and `outside = "nearest"` gives a point
beyond the pieces to the nearest one instead of refusing it (a point in a gap between pieces goes to the
nearer, the middle of the gap to the side the boundary rule names; a gap's middle is an expression of the
two stored bounds and is formed when the function is evaluated, never while compiling). The data is never
changed: the policy is the caller's.

**Solving a block at a point.** A block with one unknown that has both bounds has every root in
the interval found: the real roots of the residual when it is a polynomial in the unknown (from
`numpy.roots`, polished by Newton steps and kept when the residual vanishes to working accuracy; the
coefficients are the derivatives of the residual at zero over the factorial, so the expressions of the
parameters they contain are never expanded: the coefficient of a cubic whose mixing rule embeds an
activity model has hundreds of thousands of terms once expanded),
otherwise by scanning the interval (evenly, or geometrically when the upper bound is over a
hundred times the lower) and bracketing each sign change with `scipy.optimize.brentq`; a sign
change across a pole is not a root, and two roots closer than the scan spacing are not told apart.
`unique` demands exactly one root; `smallest` and `largest` take an end; `by` evaluates its
expression at each root and takes the least. Any other block is solved from its `start` with the
analytic Jacobian: `scipy.optimize.root` when it has no bound, `scipy.optimize.least_squares`
within the bounds otherwise, then Newton steps that stay inside the bounds. `unique` there means
the root reached from the start.

A block that has no root in its interval, more than one under `unique`, an empty interval, a
start outside its bounds or a solver that does not converge is an evaluation refusal naming the
form, the block and the point (the values of the arguments it depends on), never a NaN. A root
outside the bounds is never returned.

### Where a point lies with respect to validity regions

The records an evaluation reads state their validity as regions (meta-model section 5): a parameter
set, and the parameterization it belongs to, for each set the expansion read through any source
(nested and referenced sets and sub-forms included). The parameter source exposes them
(`ParameterSource.validity(kind, reads)`: for each record that supplied one of the sets read, its
coverage row and its regions with their clauses), and a bound form reports, for each point of the
argument values and for one region kind asked for, where the point lies
(`BoundForm.validity(output, kind, **arguments)`, an array of `Membership` codes broadcast over the
arguments):

- a clause is **decidable** when it limits an observable that exactly one argument of the contract
  names and has a value here, and is about no component and no aggregation; it holds when the
  argument's value lies in its closed interval. A bound stated as an offset from another observable of
  the subject (`lower_relative_to`, `upper_relative_to`) is placed at that observable's value plus the
  offset, and the clause is decidable only when the evaluation has a value of the observable: one passed
  to `validity(..., references={observable: value})`, else the value of a slot that denotes the
  observable in a set the evaluation read, when the sets read give it one value. A passed value takes
  precedence. Without a value the clause cannot be decided, so the region is undetermined unless
  another clause fails;
- a region is a conjunction of its clauses, so for a point it is **outside** when a decidable clause
  fails, whatever the clauses that cannot be decided say; else **undetermined** when some clause
  cannot be decided from the arguments (a clause on a component or an aggregation, on an observable no
  argument names, or on one named by two arguments); else **inside**, all its clauses being decidable
  and holding;
- for a record that states regions of the kind, the point is **inside** when it is inside some region
  (the regions are alternatives), else **undetermined** when some region is, else **outside**; for the
  evaluation, which reads several records, it is **outside** when it is outside a record that states
  regions, else **undetermined** when it is for one, else **inside**;
- when no record read states a region of the kind (it has a `not_stated` coverage row, or none), the
  answer is **not stated**, which is neither inside nor outside.

A membership never refuses an evaluation and evaluation never consults one: lying inside a region is
independent of permission to extrapolate, and what to do outside is the caller's decision. A kind
that is not an `envelope_kind` member is refused.

### Parameters and what evaluation guarantees

What enters an expression as a symbol is a stored value; what stays in it is what the form text
writes. A number written in the text (`2`, `0.5`, `8.314462618`) is a literal, exactly as written:
an integer literal stays an integer, so `x ** 2` is an integer power. A `unit('bar')` factor is a
value, not a literal, and is a parameter like a slot value.

Decisions that depend on a stored value are made at bind time, from the values, and only their
consequence is in the expression:

- a **conditional** (`a if test else b`, `and`, `or`, `not`) whose condition depends on stored
  values and subjects alone, directly or through locals, reads only the branch it takes: the
  other is not expanded, and no stored value of it is read. A condition that involves an
  argument or an unknown stays in the expression as a piecewise function;
- the **order of the pieces** of `at(...)` is the order of their lower bounds, and overlap is
  refused, both from the values; the piece chosen is decided by the argument when the function
  is evaluated;
- a **transposition** is an operation on the stored value, made by the parameter source when it is
  read, not on the expression: the source returns the values for the order of the subjects asked
  for. In the order asserted they are the stored numbers, unchanged; in the other order the value
  passed for a slot that is inverted when the subjects are swapped is its reciprocal, the values
  passed for the slots of a `linear` rule are the product of its matrix and the vector of the
  stored values (the Margules parameters of a swapped pair are (h0 + h1, -h1), and every slot of
  the rule must be held for the pair), and the value passed for a family row whose index is odd
  under `parity` is the negated one. The evaluator applies no rule of its own, so each acts
  exactly once, and a pair asked for in either orientation has the same expression;
- a **convention fact** is a stored value like a slot value, one symbol however many forms read it,
  passed to the compiled function as an argument. Its value is settled when the expansion is
  finished, from the parameterizations that supplied the sets the forms read: every source used
  states the fact, and the values of all of them are equal, otherwise the evaluation is refused,
  naming the fact, the two parameterizations and the two values (`pipeline.md` section 5.3).
  A fact a form declares per component is one symbol for each component, settled from the
  parameterization that supplied the set of the fact's slot group for that component; the set is read
  when the fact is, and each component's parameterization states the fact, but the values are not
  compared between components.

What is and is not guaranteed about the floating-point operations:

- Parameter values are never combined before evaluation: no product, quotient or sum of two
  stored values, and no stored value and a literal, is formed while compiling. `T / T_r` is a
  division of the argument by the value, not a multiplication by a precomputed reciprocal.
- Division is performed as division, subtraction as subtraction, and an integer power as a power
  with that integer; `lambdify` is called with `cse=False`, so no subexpression is hoisted.
- The association order of a sum or a product is **SymPy's canonical order** of the commutative
  operation for the symbolic expression, not necessarily the written order, and SymPy's
  automatic simplification applies to the symbolic expression: adjacent literals are combined
  (`2 * 3` is `6`), powers of one base with symbolic exponents merge (the exponents are then
  added when the function is evaluated), repeated factors become a power. Agreement with a
  left-to-right evaluation of the written formula is therefore to the rounding of the sum or
  product, a few units in the last place of the result when the terms are well conditioned,
  not bit-for-bit. Where a quantity is the small difference of two numbers near one another,
  as `1 - T / T_r` near the reducing temperature, its operands are the written operation on
  the same doubles, so it is the same difference.
- Conditioning is the formula's own: an evaluation as written in double precision has the error
  of the formula, and the evaluator does not reduce it.

### Regional forms

A formulation defined by regions of the state (IAPWS-IF97) is a wrapper form with one sub-form slot
for the equation of each region and a conditional expression of the state for each output. The
condition involves arguments, so it stays in the expression as a piecewise function and every branch
is evaluated at every point, selected afterwards: an equation has to give a number, or a NaN that the
selection discards, at the points of the other regions, and an implicit block of a region is solved at
all of them. A condition compares arguments and stored values with each other, and is written in each
output that selects by it: a test of a local that is itself a conditional (`r == 3` for a local `r`
that holds the region) did not finish in the IF97 fixture, where it ran in SymPy's simplification of
the saturation equation. The wrapper's expansion reads the parameters of every
region, more than the 64 arrays `numpy.broadcast` takes, so the evaluator takes the shape of the result
from `numpy.broadcast_shapes`.

### Prepare once, execute many

What is compiled is the structure of an expansion. A `CompileCache` holds the compiled function
of each structure, found by the expression itself (SymPy compares expressions by structure), and
a binding given a cache (`bind(..., cache=cache)`) uses what an earlier binding compiled. The
structure is determined by the form, the sub-form choices, the sizes of the index sets and the
lengths of the families, the branch of every conditional on stored values, the order of the
pieces of `at(...)`, and the implicit blocks: those are what make two expansions the same
expression. Values are never part of the key: two subjects of equal structure with different
values share one compiled function and are evaluated with their own values. A compiled
implicit-block solver is shared the same way, by the block's expressions and selection rule.
`cache.compilations` counts the `lambdify` calls made through a cache (one for an output, one
for each distinct condition that must hold, several for a block) and `cache.reused` the lookups
that found one; without a cache a binding has one of its own and shares nothing.

## 6. Correspondence with `.pse`

| Expression construct | `.pse` today |
|---|---|
| arithmetic, elementary functions, conditionals, locals | the function language (`fn`, `where`) |
| `unit('...')` literal | `value{unit}` literal |
| `sum`, `prod` over entity sets and integer ranges | `sum(i in S | ...)` |
| slot reference by subject | attribute and table lookup (`s.c1`, `t[i, j]`) |
| sub-form call | function-typed attribute call (`s.cp(T, s)`) |
| `d(expr, name)` | `partial(...)` |
| implicit block | `implicit` block with `realize ... using` |
| indexed value `[expr for i in S]` | gap |
| piece selection `at(...)` | gap |
| `integral` | gap |
| remapped sub-form call | gap |
| `basis('name', [...])` | gap: a production quantity type carries its basis, and its checks compare it wherever it flows, not only at a call |
| expressions checked by dimension, not by quantity type | gap: production types carry basis and reference state as well as dimension, so two types of one dimension are not interchangeable there |
| convention fact `convention.x` | gap: a production quantity type carries the convention it assumes |
| set reference called like a nested set | gap |
| special functions | gap |
