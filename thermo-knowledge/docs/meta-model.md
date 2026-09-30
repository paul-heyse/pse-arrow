# Declaration meta-model (v0)

This page specifies how the thermodynamic knowledge model is **declared** and how a declaration
is **projected** to PostgreSQL. It is the contract for the loader and the generator. It records
intended design; where code exists and disagrees, the code is what runs and this page is wrong.

The declaration is the single authority for the model (design principles DP-01). Generated SQL,
reified `meta` rows, Arrow schemas, reference documentation and the `.pse` rendering are derived
from it and are never edited.

## 1. Shape of a declaration

A declaration is the set of TOML documents under `model/` (structural concepts) and `forms/`
(the form catalogue). TOML is used because its parser is in the standard library, duplicate keys
are an error, and scalar types are explicit. Each document is one **module**:

```toml
module = "identity"          # unique; lowercase snake_case
schema = "tk"                # PostgreSQL schema for this module's tables: tk | prov | ev | qual
uses = ["physical"]          # modules whose names this one references; the graph is acyclic
doc = "Material identity and structure."
```

Names of kinds, relations, enums, identifier schemes, quantity types, contracts and forms are
unique across the whole declaration, so a name never needs a module qualifier. A module may
reference only names it declares or that a module in `uses` (transitively) declares.

Every construct carries `doc` (required, one or more sentences stating meaning, not
structure) and may carry `traces` (identifiers from the handoff semantic dictionary such as
`IC-13` or `SI-11`) and `pse` (section 9).

## 2. Types

An attribute, key, column or slot has a type expression:

| Type expression | Meaning | PostgreSQL |
|---|---|---|
| `Boolean`, `Integer`, `Text` | primitives | `boolean`, `bigint`, `text` |
| `Date`, `Timestamp` | calendar date; UTC instant | `date`, `timestamptz` |
| `Hash` | 32-byte content hash | domain over `bytea` with a length check |
| `Id<scheme>` | an opaque identifier in a declared scheme; never parsed | the scheme's domain over `text` |
| a quantity type name, e.g. `Temperature` | a dimensioned number in that type's storage unit | the type's domain over `double precision` |
| a quantity expression, e.g. `MolarCp / Temperature^2` | product, quotient or integer power of declared quantity types | `double precision` with a finiteness check; the unit is computed and recorded in `meta` |
| an enum name | a member of that closed vocabulary | the enum type |
| a kind name | a reference to one instance of that kind (or of any refinement) | `uuid` foreign key |
| `Record` | a reference to any registered record (section 5) | `uuid` foreign key to `prov.record` |
| `Meta<construct>` | a reference to a reified declaration row, where construct is one of `quantity_type`, `kind`, `contract`, `form`, `slot_group`, `slot`, `family`, `subform_slot` | foreign key to that `meta` table |
| `Real` | a finite number whose unit its declaration names with `unit_from`: an attribute or column of the same record of type `Meta<quantity_type>`, `Meta<slot>` or a reference to an observable (or a dotted path of references ending in one), or the literal `dimensionless`; a `Real` without `unit_from` is refused, and it is never used where a quantity type could be named | `double precision` with a finiteness check |
| `Range<Q>` | a closed interval of quantity type `Q` | a range type over `double precision`, created once in `meta` |
| `Array<T>` | an ordered list of a primitive (`Real` included), quantity or enum type | PostgreSQL array |
| `Set<K>` | an unordered set of references to kind `K` | child table |
| `SourceText` | uninterpreted text retained from a source; nothing may branch on it | `jsonb` or `text` |

`optional = true` makes a value nullable. Optional means "this instance has no such attribute";
it never means unknown-but-exists, zero or failed (DP-02). Floating-point types may not appear in
identity keys.

## 3. Structural constructs

### 3.1 Enum

A closed vocabulary that pipeline code branches on.

```toml
[enums.value_state]
doc = "Whether a slot holds a value and, if not, why."
members.known = { doc = "A value is asserted; a known zero is known." }
members.not_applicable = { doc = "The slot has no meaning for this subject." }
members.redirect = { doc = "The value is that of another parameter set." }
members.withheld = { doc = "A value exists but rights forbid storing it." }
```

An enum may declare **facets**, the properties some of its members have, and each member names the
facets it carries. A member names only facets its enum declares.

```toml
[enums.origin_role]
doc = "How the asserted content of a record came to exist."
facets.requires_derivation = { doc = "A record with this role has a producing derivation." }
facets.requires_fit = { doc = "The producing derivation is a fit." }
members.fitted = { doc = "Obtained by regression against evidence.", facets = ["requires_derivation", "requires_fit"] }
members.published = { doc = "Stated in a publication and transcribed." }
```

A rule that depends on a property of members reads the facet, never the member's name: the verify
checks of `origin_role` select the roles with `requires_derivation` and `requires_fit` from `meta`, so
giving a new member a facet puts it under the rule with no other edit.

Scientific vocabularies that grow with the science (observables, aggregations, group schemes) are
**kinds with declared entities**, not enums.

### 3.2 Identifier scheme

```toml
[identifier_schemes.inchikey]
doc = "IUPAC standard InChIKey. Opaque."
```

### 3.3 Quantity type (module `physical`)

```toml
[quantity_types.Temperature]
doc = "Absolute thermodynamic temperature."
unit = "K"                  # storage unit, a product of declared units
scale = "absolute"          # absolute | difference | ratio | dimensionless | count
production = "Temperature"  # name of the corresponding production quantity type, or omitted
```

`absolute` means non-negative: a value of zero is valid (a dipole moment of zero, a piece starting at
0 K, a mass load of zero) and a negative one is refused. Where strict positivity is physical, the kind
or relation declares it with a `positive` check on the attribute or column.

Unit strings are parsed by `pint`'s default registry, which is the unit authority; the model
declares no unit table of its own. A storage unit must be a coherent SI unit product (its factor
to base units is exactly 1), so stored numbers never carry a hidden scale. A quantity expression
resolves by unit algebra over the storage units; two types with equal dimensions remain
different types. `meta` records each type's unit and dimensionality.

### 3.4 Kind

A concept whose instances have identity.

```toml
[kinds.species]
doc = "A chemical identity: formula, charge and, where known, structure."
extends = "material_entity"       # optional refinement; the chain is acyclic
abstract = false                  # an abstract kind has no direct instances
identity = ["canonical_key"]      # attributes whose values make two instances the same
provenance = "own"                # own | inherit:<attribute> | declaration | none  (section 5)
traces = ["IC-01"]

[kinds.species.attributes]
canonical_key = { type = "Text", doc = "Structure identifier where one exists, else formula, charge and a declared discriminator." }
charge = { type = "ChargeNumber", default = 0, doc = "Net charge number." }
inchikey = { type = "Id<inchikey>", optional = true, unique = true, doc = "Asserted standard InChIKey." }

[[kinds.species.requires]]
name = "charge_matches_formula"
doc = "The charge equals the formula's charge count."
enforced = "verify"               # ddl | load | verify: where the invariant is enforced
```

Rules:

- `identity` is declared on the root of a refinement chain and inherited. A refinement adds
  content, not identity.
- An identity attribute is never optional and never a floating-point type.
- Every invariant names its enforcement point (DP-03). `ddl` means a constraint the generator
  emits from a declared `check`, a table with exactly one key from a small closed set:
  `check = { nonempty = "a" }`, `{ positive = "a" }`, `{ nonnegative = "a" }`,
  `{ ordered = ["a", "b"] }` (a <= b), `{ one_of_present = ["a", "b"] }` (exactly one is
  non-null), `{ present_iff = { column = "value", when = "state", in = ["known"] } }` (the column is
  non-null exactly when the enum column `when` holds one of the listed members; a null `when`
  satisfies the check) and `{ within = { column = "c", lower = -1, upper = 1 } }` (a closed numeric
  interval; either bound may be omitted, not both). `present_iff` governs an optional column and
  names members of the enum. `load` means the loader rejects the record (an evaluator in
  `canonical/invariants.py` decides a row); `verify` means a named check in the verify stage. An
  invariant with no enforcement point is refused.
- A declared entity's TOML key is its declared name. If its kind has an attribute called `name`
  and the entity does not set it, the key is the value. A reference-typed attribute of a declared
  entity names another declared entity of the referenced kind.
- `unique = true` on an attribute, or `[[kinds.K.unique]] attributes = [...]`, declares a
  uniqueness constraint in addition to identity.

### 3.5 Relation

An n-ary fact keyed by typed roles. It has no identity beyond its keys.

```toml
[relations.formula]
doc = "Number of atoms of an element in a species formula."
provenance = "inherit:j"
absence = { default = 0 }         # required | optional | { default = value }
value = { type = "Count" }        # a single value, or [relations.R.columns] for a row

[relations.formula.keys]
j = { type = "species" }
e = { type = "element" }
```

A relation key may be any identity-eligible type: a kind, `Record`, `Meta<...>`, `Integer`, an
enum, `Text` or `Id<scheme>`. Floating-point types are never keys.

`absence` states what a missing row means: `required` (the relation is complete over its keys and
a missing row is an error for the consumer), `optional` (missing means not asserted) or a declared
default. Missing and a stored zero are never the same state.

A relation over two roles of the same kind may declare `transposition` (section 4.3).

A relation declares invariants as a kind does, over its keys and value columns, with the same fields
and the same three enforcement points:

```toml
[[relations.datum.requires]]
name = "value_matches_state"
doc = "A value is present exactly when the state is known or censored."
enforced = "ddl"
check = { present_iff = { column = "value", when = "state", in = ["known", "censored"] } }
```

A `ddl` requirement is a `CHECK` on the relation's table and is evaluated by the canonical writer, a
`load` requirement needs an evaluator like a kind's, and a `verify` requirement needs the check file
`sql/verify/<relation>.<name>.sql`. The reified `meta.requirement` rows of kinds and relations are
alike, told apart by `owner_type`.

### 3.6 Entity

A declared instance of a kind: the members of scientific vocabularies and anything else the model
itself asserts.

```toml
[entities.aggregation.gas]
doc = "Gas or vapour."
```

Attribute values follow the kind's declaration. Declared entities have `provenance = declaration`.

## 4. Forms

Forms are declared in `forms/`. They are data: adding a form changes no code.

### 4.1 Contract

What a form consumes and produces.

```toml
[contracts.pure_vapor_pressure]
doc = "Saturation pressure of a pure species form as a function of temperature."
arguments.T = { type = "Temperature" }
outputs.p_sat = { type = "Pressure", observable = "vapor_pressure" }
```

An output has its observable in one of three ways: it names one (`observable = "vapor_pressure"`),
it takes it from the data (`observable_from_set = true`), or it has none. Naming and taking from
the data exclude each other. The second is for one equation that gives different observables for
different subjects: the vapour pressure of a pure fluid, and the bubble or dew pressure of a
pseudo-pure fluid.

```toml
[contracts.saturation_pressure]
outputs.p = { type = "Pressure", observable_from_set = true }

[forms.saturation_curve.output_observables]
p = "pure.quantity"           # <slot group>.<slot>
```

An argument that is a composition names its `basis`, a declared entity of the kind bound to the
framework role `composition_basis` (`meta.contract_argument.basis` references it). At a sub-form call
a vector passed by name has the basis of the callee's argument, or the callee's argument names none;
a comprehension passed to an argument that names a basis is written `basis('<name>', [ ... ])`, by
which the form's author asserts the basis of the vector it builds (`expressions.md` section 3).

A form implementing a contract with such an output declares, for each of them and for no other
output, the slot that supplies its observable. The slot is a required slot of a slot group (not
of a family) whose type is the kind bound to the framework role `observable`; the parameter set
that instantiates the group names the observable. The loader checks the pairing and the slot. The
observable is data, so the dimension is checked when a set is written: the canonical writer
refuses a set whose observable has a quantity type of another dimension than the output's declared
type, reporting the source locator.

### 4.2 Form, slot group, slot

```toml
[forms.antoine]
doc = "Antoine equation in natural-log form."
implements = "pure_vapor_pressure"
completeness = "fully_declared"   # fully_declared | structure_declared_equation_external | opaque_bundle
status = "catalogued"             # catalogued | expressed
citations = ["antoine_1888"]

[forms.antoine.slot_groups.pure]
doc = "One set of coefficients per species form."
subject.i = { type = "species_form" }      # ordered role tuple; empty for a global group

[forms.antoine.slot_groups.pure.slots]
A = { type = "Scalar", doc = "..." }
B = { type = "Temperature", doc = "..." }
C = { type = "Temperature", doc = "..." }
```

`status` states what the declaration holds: a `catalogued` form has no expression and an
`expressed` form has one for every output of its contract (`expressions.md`). Whether a form is
qualified is not declared. It is a fact about recorded qualification runs and is derived from
them: the view `qual.form_qualification` (`sql/physical.sql`) lists the passing runs made against
the form's current expression (`pipeline.md` section 5). `qualified` is refused as a status.

A **slot group** is the unit that one source fits together for one subject tuple. It is sugar for
a kind that refines the framework kind `parameter_set`: its keys are the subject roles and its
attributes are the slots. The generator expands it that way, so everything in section 3.4 applies.

A **slot** has a type (section 2) and a `presence` policy:

| `presence` | Meaning | Projection |
|---|---|---|
| `required` (default) | a known value is always present | `NOT NULL` column |
| `stateful` | the value may be `not_applicable`, `redirect` or `withheld` | nullable value column, a `value_state` column, a redirect reference, and a check tying them together |

A slot may name the `observable` it denotes (for example a critical temperature slot), which lets
a snapshot fill it from evaluated data. Uncertainty is never a slot; it is recorded in the
framework table `slot_uncertainty`.

**Value shapes.** A slot holds one of: a dimensioned scalar or count; an enum member; a reference
to an entity; a **nested set** (`accepts = "<contract>"`: the value is a parameter set of a form
implementing that contract, which is how a temperature-dependent coefficient is represented); or a
**tabulated function** (a reference to the framework kind `tabulated_function`, which has an
interpolation rule, one or more axes, each a quantity type with ascending points, and one or more
named series of values over the full grid of the axes, in row-major order).

**Indexed families.** Slots that repeat over an index (terms, orders, pieces) are declared as a
family:

```toml
[forms.nasa7.slot_groups.pure.families.piece]
doc = "One coefficient block per temperature interval."
index.n = { type = "Integer", min = 1 }
interval = { lower = "T_low", upper = "T_high" }   # optional: pieces on one axis must not overlap

[forms.nasa7.slot_groups.pure.families.piece.slots]
T_low = { type = "Temperature" }
T_high = { type = "Temperature" }
a1 = { type = "Scalar" }
```

A family projects to a child table keyed by (parameter set, index). Indices are keys inside a set,
never subjects.

### 4.3 Transposition

A slot group or relation with two roles of the same kind declares what happens when they are
swapped. The set of rules is closed:

| Rule | Meaning | Stored |
|---|---|---|
| `ordered` | (i, j) and (j, i) are independent facts | both, if asserted |
| `symmetric` | the value is the same either way | one canonical orientation |
| `parity` | the sign alternates with a named index (`by = "order"`) | one canonical orientation |
| `reciprocal` | named slots invert on swap (`slots = [...]`) | one canonical orientation |
| `linear` | the values of named slots, as a vector, are multiplied by a declared matrix on swap (`slots`, `matrix`) | one canonical orientation |
| `permutation_group` | a declared group of role permutations leaves the fact unchanged | the canonical representative |

`diagonal = "forbidden" | "allowed"` states whether both roles may name the same instance. The
canonical orientation is the one whose first role has the smaller identifier. The declaration is
one inline table on the slot group or relation:

```toml
transposition = { rule = "symmetric", roles = ["i", "j"], diagonal = "forbidden" }
transposition = { rule = "parity", roles = ["i", "j"], by = "order" }
transposition = { rule = "reciprocal", roles = ["i", "j"], slots = ["beta_T", "beta_V"] }
transposition = { rule = "linear", roles = ["i", "j"], slots = ["h0", "h1"], matrix = [[1, 1], [0, -1]] }
```

`linear` holds the parameters of a pair that change into one another when the pair is swapped: the
two Margules parameters of a pair become (h0 + h1, -h1). The values of the listed slots, in the
order listed, are multiplied by the matrix, row by row, when the subject is given in the
non-canonical orientation: the canonical writer applies it on writing, the evaluator when it finds
a set in the orientation other than the one asked for, and the stored values stay in the canonical
orientation. A slot the rule does not name is unchanged. The loader refuses a `linear` rule unless

- it names two roles and at least one slot, each slot exists, is named once, is required (not
  `stateful`) and has a quantity type (a number a linear combination can take);
- the matrix is square over the listed slots and holds finite numbers;
- each row combines only slots of the dimension of the slot the row gives: an entry other than zero
  links two slots of equal dimension;
- the matrix is an involution: applied twice it gives the identity, in exact rational arithmetic on
  the declared numbers (`0.1` is one tenth), so a swap applied twice returns the values.

`reciprocal` and `symmetric` are the common special cases of a swap that acts on values or leaves
them unchanged. A relation declares `linear` over its value columns in the same way.

### 4.4 Sub-form slot

```toml
[forms.cubic.subforms.alpha]
doc = "Temperature dependence of the attractive parameter."
accepts = "alpha_function"
multiplicity = "one"              # one | optional | many (additive contributions)
per = "subject"                   # model | subject
```

A `model_assembly` (a declared kind) records one choice per sub-form slot.

### 4.5 Expressions

The mathematics of a form (contract roles and sets, locals, output expressions and implicit
blocks) is specified in `expressions.md`. A form with `status = "catalogued"` needs none.

## 5. Identity, records and provenance

**Identifiers are deterministic.** A kind instance's identifier is a version-5 UUID computed from
a per-kind namespace (itself derived from a fixed root namespace and the kind's root name) and the
canonical encoding of its identity values: a JSON array in declaration order, with references as
lowercase UUID strings, text normalised to NFC, integers in decimal, enum members by name,
booleans as `true`/`false`. A relation row's identifier is computed the same way from its keys. An identity attribute with a
declared default takes it when a value is not given, before the identifier is computed: a parameter
set's identity is (parameterisation, slot group, subject key, occurrence), and `occurrence`, which
numbers repeated assertions for one subject within one parameterisation from one in source order,
is one when a mapping does not state it.
A clean rebuild therefore reproduces every identifier.

**Records.** Whatever a source asserts is a *record*. A kind or relation with
`provenance = "own"` registers each instance in `prov.record` (its identifier and its kind), and
provenance, rights, derivations and equivalence assessments attach to the record. With
`provenance = "inherit:<attribute>"` an instance shares the record of the instance that attribute
references (data points share their dataset's record; family rows share their set's). With
`provenance = "declaration"` the instance originates in the declaration itself. With
`provenance = "none"` the kind is itself provenance or pipeline bookkeeping (a source, an
artifact, an import record, a qualification run): its instances are created by the pipeline and
register no record. A kind with `provenance = "own"` (or a refinement of one) may also have
declared entities; each registers its `prov.record` row at build like any other instance and
simply has no import record.

`prov.record` and the `meta` tables are the projection's own machinery and are built into the
generator. The domain concepts the generator relies on are declared in the model, not built in:
`model/manifest.toml` has a `[framework]` table binding each role (`parameter_set`,
`parameterization`, `tabulated_function`, `slot_uncertainty`) to the kind that fills it. The
roles are required only when the declaration contains forms. An optional fifth role,
`observable`, binds the kind whose declared entities are the observables: when it is bound, the
`observable` of a contract output or a slot must name a declared entity of that kind (the `meta`
rows reference it by identifier); when it is not bound, `observable` may not be used. A sixth,
`composition_basis`, binds the kind whose declared entities are the composition bases in the same
way: a contract argument's `basis` and the name in `basis('<name>', [ ... ])` must name one, and
without the role neither may be used.

### The pipeline contract

The roles above are what the generator needs. The stage code needs more: the canonical writer creates
`carrier`, `artifact`, `import_record` and `rights_determination` rows and the text keys that restate
references, resolution writes `species`, `species_form`, `source_entity`, `identity_assertion` and
`resolution_candidate`, the mapping framework writes `derivation`, `fit`, `envelope`, `mapping_rule`,
`mapping_coverage` and `held_row`, qualification writes `qualification_run` and reads `parameter_set`,
`parameterization`, `envelope` and `source_entity`. That dependency is declared in one file shipped with
the package, `src/thermo_knowledge/pipeline_contract.toml`, which names each kind, relation and enum
the code uses directly (the model is this pipeline's model) with:

- the `schema` of a kind or relation, the `identity` of a root kind and the kind another `extends`;
- each attribute (of a kind), key or value column (of a relation) the code reads or writes, with its
  declared `type` as the declaration spells it, `optional = true` where the declaration leaves it
  optional and `defaulted = true` where the code relies on a declared default;
- for an enum, the `members` the code names.

The loader checks the loaded declaration against the file and reports each mismatch as a diagnostic
with the code `pipeline-contract`, placed at the declaration of the kind or relation it concerns (at the
contract file when the construct is missing) and naming the contract entry that requires it: a missing
kind, relation, enum or member, a missing attribute, key or column, a wrong type, a wrong optionality,
a default that is missing where the code relies on it, another schema, another identity or another
parent kind. A declaration that is not this pipeline's model (a fixture of the declaration language)
loads with `contract=None`.

The code takes these names from one module, `thermo_knowledge.pipeline_contract`, which reads the
file: `CARRIER.declared` is the kind's name, `CARRIER.table` its qualified table and `CARRIER.tree_hash`
a listed attribute. A constant whose entry the file lacks fails when the module is imported, and a name
the file does not list for an entry raises `ContractError` where it is read. A test reads the code and
checks that every name taken from the module is listed, so a stage cannot depend on a name the contract
does not state. The contract covers the stages in `canonical/`, `mapping/`, `resolve/`, `build/` and
`qualify/`; a `mapping.py` names its own kinds, slot groups and families in `mapping.toml` and is
checked against the declaration when it runs.

## 6. What the loader refuses

Each refusal is a structured diagnostic with a stable code, the document path, the construct path
and a message (DP-21). At least:

- unknown key in any construct; missing `doc`
- duplicate name across the declaration; name that is not lowercase snake_case (quantity types
  and built-in types are CamelCase)
- reference to a name not reachable through `uses`; a cycle in `uses`, in `extends`, or among
  nested-set contracts
- identity declared on a refinement, missing on a root, or containing an optional or
  floating-point attribute
- an abstract kind with declared entities; an entity with an unknown or mistyped attribute
- a default whose type does not match; an enum value that is not a member
- an invariant without an enforcement point, or `ddl` enforcement with an unknown check
- a slot group subject role whose type is not a kind; transposition on roles of different kinds;
  a `linear` transposition that breaks a check of section 4.3
- a `status` other than `catalogued` and `expressed` (`qualified` included); an output that both
  names an observable and takes it from a set; a form that does not declare the slot for an output
  taken from a set, declares one for another output, or names a slot that is not a required
  observable reference of a slot group
- a quantity expression that does not resolve by unit algebra
- a member that names a facet its enum does not declare; a `present_iff` that governs a column that is
  always present, depends on a column that is not an enum or names a member the enum lacks; a
  `within` with no bound or with its lower bound above its upper
- a contract argument whose `basis` names no declared entity of the `composition_basis` kind, or is
  used without that role
- a declaration that differs from the pipeline contract (above)
- any projected PostgreSQL identifier longer than 63 bytes

## 7. Projection to PostgreSQL

The generator is a pure function from a loaded declaration to a deterministic tree of files. With
`--check` it compares that tree with the committed one and reports every difference in either
direction.

| Declaration | PostgreSQL |
|---|---|
| module `schema` | one of the schemas `tk`, `prov`, `ev`, `qual`; slot-group tables go to `param`; reified declarations go to `meta` |
| enum | `CREATE TYPE ... AS ENUM` in `meta` |
| identifier scheme | domain over `text` with a non-empty check |
| quantity type | domain over `double precision` with a finiteness check (and non-negativity for `absolute` scale) |
| root kind | table with `id uuid PRIMARY KEY`, identity attributes `NOT NULL` with a unique constraint |
| refinement | table whose `id` is both primary key and foreign key to its parent |
| abstract kind | table with no direct rows: every row must have a refinement row (checked in verify) |
| attribute | column; reference types are foreign keys; `Set<K>` is a child table |
| relation | table with a key column per role, primary key over the roles, `id uuid UNIQUE` |
| slot group | table `param.<form>__<group>`, refining the `parameter_set` table, with a foreign-key column per subject role and a column per slot |
| family | table `param.<form>__<group>__<family>` keyed by (set, index); an `interval` adds a range column and an exclusion constraint against overlap |
| `stateful` slot | value column, `<slot>__state`, `<slot>__redirect`, and a check that the value is present exactly when the state is `known` and the redirect exactly when it is `redirect` |
| symmetric, parity, reciprocal, linear transposition | a check that the first role's identifier is smaller; `diagonal = forbidden` adds inequality |
| `provenance = "own"` | foreign key from `id` to `prov.record` |
| `requires` with `enforced = "ddl"`, of a kind or a relation | named `CHECK` constraint on its table |

Every constraint is named deterministically. Every table and column carries its `doc` as a
comment. Foreign keys are deferrable so a bulk load can insert in any order inside one
transaction. No index other than those implied by constraints is generated; access paths are
hand-written in `sql/physical.sql`, which also holds the view `qual.form_qualification`
(`pipeline.md` section 5).

`meta` reifies the declaration as rows (modules, kinds, attributes, relations, the requirements of kinds
and of relations alike, enums with their members, facets and the facets of each member, identifier
schemes, units, quantity types, contracts, forms, slot groups, slots, families,
sub-form slots, transposition slots and matrices, the slot that supplies an output's observable,
and each output's evaluation hash), so the database describes itself and `parameter_set` rows reference their slot
group by foreign key. These rows and the declared entities are inserted from the declaration when
a database is built; they are not a second authority.

A schema fingerprint (SHA-256 over the generated DDL, `sql/physical.sql` and the canonical
serialisation of the reified `meta` rows and declared-entity rows exactly as inserted, so it
changes whenever the declaration does) is recorded as a comment on schema `tk`. A database whose fingerprint differs from the current declaration is
refused; it is rebuilt, never migrated.

## 8. Outputs

| Output | Location | Committed |
|---|---|---|
| DDL | `sql/generated/schema.sql` | yes, diff-checked by `tk generate --check` |
| Arrow schema per table | derived at run time from the loaded declaration | no |
| reified `meta` rows and declared entities | inserted at build | no |
| reference documentation, `.pse` rendering, kernel-gap register | later packets | yes |

## 9. Correspondence with `.pse`

The meta-model uses the concepts of the production modeling language so that a rendering is
mechanical wherever the language already has the construct. Each construct may carry
`pse = "gap:<short name>"` when it needs something the language lacks; the kernel-gap register is
generated from those marks plus the table below.

| Meta-model | `.pse` today |
|---|---|
| enum with member docs | `enum` with facets |
| identifier scheme | `identifier scheme` |
| kind, `extends`, `abstract`, identity keys | `entity kind`, `extends`, `abstract`, `key` |
| attribute with quantity type and storage unit | `attribute name: Type storage {unit}` |
| relation with typed keys, `absence`, symmetric transposition | `table t[i: K, j: K]: T missing default v` and `symmetric(i, j)` |
| declared entity | `entity kind name { ... }` |
| slot group | a refined keyed kind plus a `dataset ... from "*.parquet"` |
| family with integer index | `table t[f, k: 0..n]` |
| form contract | an abstract kind with function attributes, or an interface |
| envelope (`Range<Q>` attributes) | `envelope` |
| `parity`, `reciprocal`, `linear`, `permutation_group` transposition | gap |
| `stateful` slot (`not_applicable`, `redirect`, `withheld`) | gap |
| nested set as a slot value | gap (pair values are `Scalar` only) |
| rights determinations | gap (provenance has no rights facet) |
| `derivation` lineage beyond dataset and source | gap (register R-50) |
