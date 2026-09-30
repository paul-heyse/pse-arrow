# From source-faithful tables to the canonical database (v0)

This page specifies the stages after reading: identity resolution, mapping, build, verification
and qualification. It is the contract for those stages. It records intended design; where code
exists and disagrees, the code is what runs and this page is wrong.

```
raw store ──read──> staged Parquet ──load-src──> src_<id> schemas
                                                     │
                       identity claims <── map (phase 1)
                              │
                           resolve ──> resolutions
                              │
                    canonical Parquet <── map (phase 2)
                              │
                            build ──> pse_thermo (meta, prov, tk, param, ev, qual)
                              │
                     verify, qualify, project
```

## 1. Source entities and identity resolution

A source refers to material by its own keys: a file name, a CAS number, a row name, a formula.
The model keeps that reference as what it is.

- A **source entity** is the thing a carrier means by one of its keys. Its identity is
  (carrier, scope, local key), where scope is the context in which the key is unique (a table, a
  database file). It is created for every distinct thing a source's records are about.
- An **identity assertion** is one identifier the source gives for a source entity: a CAS number,
  an InChIKey, a SMILES, a formula, a name. A source entity has any number of them.
- A source entity has a **class**, the kind of thing its source says the key names: `species`,
  `species_form`, `defined_mixture`, `material`, `pseudo_component`, `polymer_type` or
  `undetermined`. The mapping states it for each source entity (section 2.1) and resolution never
  infers it: the rules that apply to an entity are those of its class, so a blend is never
  resolved by the rules for a chemical species, and no provisional entity is a species unless its
  class is `species` (or `species_form`, which has a species).
- **Resolution** assigns each source entity a target `material_entity` and a status. It is a
  derived result, recomputed on every run from the assertions, the compositions and the curated
  decisions.

**The rules of each class.**

| Class | Resolves by | Otherwise |
|---|---|---|
| `species` | the rules of the list below, in order | a provisional `species` |
| `species_form` | its species, resolved by the same rules, and the aggregation and polymorph it states | a provisional `species` and form |
| `defined_mixture` | its composition: the mixture whose canonical key is built from its resolved components (below); unique only when every component is a unique species or species form | a provisional `defined_mixture`, and no components |
| `material` | a registry identifier in a naming scheme the declaration marks `registry = true` (rule `registry`): one identifier gives the `material` with that key, several are `ambiguous` | a provisional `material` |
| `polymer_type` | a curated decision only; its class rules come with the wave that needs them | a provisional `polymer_type` |
| `pseudo_component` | nothing yet: its class rules come with the wave that needs them, and no scope may declare the class, since a provisional pseudo-component needs the `kind` that says why it has no formula, which no source statement carries so far | |
| `undetermined` | a curated decision only, which states the class that settles it | a provisional `unclassified_entity`: an entity whose class its source does not establish, never joined across carriers |

A curated decision (rule 1 below) states the class of the canonical entity it names and must agree
with the class the source states for the entity (a `species_form` entity is identified through its
species: the decision's class is `species`), unless the source states none.

**Defined mixtures.** A mapping's phase 1 claims each component of a mixture (the source entity of
the component and its fraction, section 2.2) and, for the mixture, whether its composition is by
definition or by measurement and whether its fractions are mole or mass fractions. A component
is a species or species form; it must be unique for the mixture to be, and a component listed twice
makes the mixture provisional. The canonical key of a resolved mixture is `mixture:` and the
canonical encoding of the basis (`mole` or `mass`) followed, for each component in the order of the
components' keys, by the canonical key of its resolved entity and its fraction as exact decimal text
(the shortest decimal that reads back as the source's number, with no exponent or trailing zero).
The same composition from any number of carriers, in any order, is one mixture; another fraction or
basis is another. The mixture's `mixture_component` rows are its resolved components and fractions.
Carriers that give one composition but differ on whether it is by definition or by measurement
leave it `ambiguous`: the canonical mixture is a candidate of each and none is resolved to it.
The rule of a mixture resolved this way is `composition`.

**Species and species forms** are resolved by these rules, applied in this order; the first that
applies decides:

1. **Curated decision.** `identity/decisions.toml` states that a source entity (by carrier
   manifest id, scope and local key) is a named canonical entity, or that two are distinct. A
   curated decision carries a reason. It is the only hand-maintained resolution authority.
2. **Structural identifier.** The source entity has a standard InChIKey, or an InChI or SMILES
   from which one is computed with RDKit. Source entities with the same standard InChIKey and
   charge resolve to one `species` whose canonical key is that InChIKey.
3. **Registry cross-reference.** The source entity has a registry identifier (CAS, PubChem CID)
   that the corpus maps to exactly one InChIKey: every source entity that carries both agrees.
   If the corpus maps it to more than one, the status is `ambiguous`.
4. **Formula-identified scope.** For entity classes where a formula and charge identify the
   species by convention (elements, monatomic and simple inorganic ions, and the members of a
   chemical system whose source declares its species by formula), the formula, charge and the
   chemical system resolve it. The scopes this rule applies to are declared per mapping.
5. **Otherwise provisional.** The source entity gets a provisional entity of the kind its class
   says (a `species` for a species, species form; a `defined_mixture`, a `material`, a
   `polymer_type`, or an `unclassified_entity`, see above) whose canonical key is its own
   (carrier, scope, local key). Its records load and remain usable within that carrier; they are
   not joined to another carrier's records.

Statuses: `unique` (rules 1 to 4 gave one target), `ambiguous` (candidates remain; provisional
target), `unresolved` (no candidate; provisional target), `rejected` (a curated decision says the
assertion is wrong). Nothing is ever resolved by taking the first match.

A `species_form` is resolved from its species plus the aggregation and polymorph the source
states for the record, never from a suffix in a name. A source that does not state an aggregation
for a record that needs one leaves the mapping to declare it for that table, as an assumption.

Resolution reports, per carrier and in total: counts by class, status and rule (and the provisional
entities by kind), and every ambiguous source entity with its candidates.

### 1.1 The decisions file

`identity/decisions.toml` holds the curated decisions, each with a reason. An entity is named by the
manifest id of its carrier, its scope and its local key.

```toml
[[decision]]
action = "identify"      # the entity is this canonical entity
entity = { carrier = "coolprop", scope = "fluids", key = "R1123" }
class = "species"        # the class of the canonical entity: species, defined_mixture, material or polymer_type
canonical_key = "ZWQIPCQCAOKXDI-UHFFFAOYSA-N"   # an InChIKey, or another declared canonical key
charge = 0               # optional, default 0, species only
reason = "why"

[[decision]]
action = "distinct"      # two entities that are different identities
entities = [
  { carrier = "coolprop", scope = "fluids", key = "A" },
  { carrier = "thermo", scope = "chemicals", key = "B" },
]
reason = "why"

[[decision]]
action = "reject"        # the source's identification of the entity is wrong
entity = { carrier = "coolprop", scope = "fluids", key = "C" }
reason = "why"
```

- `identify` resolves the entity to the entity of its `class` with that canonical key, status `unique`,
  rule `curated`, whatever its assertions say. The species' `inchikey` is the key when the key has the
  shape of a standard InChIKey. The class must agree with the source's (section 1, the table of the classes); `pseudo_component`
  and `undetermined` cannot be named (an entity of class `undetermined` is settled by a decision
  whose class is that of the canonical entity).
- `reject` gives the entity status `rejected`, rule `curated` and a provisional target of its class.
- `distinct` applies when rules 2 to 4 would resolve both entities to one species: neither is resolved
  by the rule, both are `ambiguous` with that species as their candidate, until an `identify` decision
  settles each.
- At most one `identify` or `reject` names an entity. A decision that names an entity no carrier has is
  reported in the resolution report, not applied and not an error: the file covers the whole corpus,
  and a carrier may not be mapped yet. An unknown key, a missing reason or a malformed decision refuses
  the run.

### 1.2 How the rules apply

- **Which schemes.** The structural and formula rules and the registry rule for species apply to
  entities of class `species` and `species_form`. The structural schemes and the registry schemes are those the declared
  `naming_scheme` entities mark `structural = true` and `registry = true` (`cas` and `pubchem_cid` are
  the registries today); resolution reads them from the declaration, and a structural scheme it cannot
  compute an InChIKey from refuses the run.
- **Structural.** An assertion of scheme `inchikey` counts when it has the shape of a standard
  InChIKey (`S` as the flag of the second block); one of scheme `inchi` when it begins `InChI=1S/`, its
  key computed with RDKit; one of scheme `smiles` when RDKit parses it, its key computed from the
  molecule. A structural assertion no key can be computed from is listed in the report and takes no part
  in the rule. An entity whose structural assertions give more than one key is `ambiguous`
  (rule `structural`, every key a candidate): the evidence contradicts itself.
- **Registry.** The corpus maps a registry value to
  the keys of the entities that carry it together with a structural key of their own (entities that
  the rule above resolved uniquely and that no decision names). An entity with no structural key
  resolves to the one key its registry values map to (rule `registry`); when the union over its registry
  values holds several keys it is `ambiguous` with them as candidates. Entities that have a structure
  keep it: a registry value mapped to two structures makes ambiguous only the entities that rely on it,
  and the report lists it.
- **Formula-identified scope.** A mapping declares such a scope with its reason and an optional
  discriminator. An entity of the scope with one `formula` assertion resolves to the species
  `formula:<formula>:<signed charge>[:<discriminator>]`, the formula text normalised to NFC and stripped
  of surrounding space and otherwise not parsed, the charge its stated charge (0 when it states none).
  Several different formulas make it `ambiguous`. Outside a declared scope a formula identifies nothing.
- **Provisional.** The provisional entity's canonical key is `provisional:` and the canonical encoding
  of the carrier key (`<manifest id>@<resolved pin>`), scope and local key, whatever its class. A
  provisional species has `provisional = true`, no InChIKey and the entity's stated charge; a
  provisional defined mixture has the definition and basis the mapping declares for its scope and
  no components.
- **Charge.** An InChIKey encodes its charge, so the structures of one key agree; the species' charge is
  the molecule's formal charge where RDKit builds the molecule, else the stated charge, else 0. An
  entity whose stated charge differs from its species' charge is `ambiguous`.
- **Species forms.** An entity of class `species_form` (the only class that states an aggregation)
  gets a `species_form` of its resolved (or provisional) species, whose canonical key is the canonical
  encoding of the species key, the aggregation name and, when stated, the polymorph; the entity's
  target is the form.
- **Label.** The species' `label` is never identity. It is the `name` assertion made by the most
  distinct source entities resolved to the species; ties go to a name that is the local key of one of
  those entities, then to the smaller name in code-point order. With no name it is the smallest
  `formula` assertion, else the smallest local key.
- **Origins.** A canonical entity's origins are the import records of the source entities resolved to it
  or listing it as a candidate, in the role the mapping declares for those rows (a mixture's also those
  of the rows that state its composition); a provisional entity has its entity's.

### 1.3 Resolution output

`tk resolve` reads the phase-1 output of every source (section 2) and writes
`.store/canonical/_resolution/`:

| File | Holds |
|---|---|
| `tk.source_entity`, `tk.identity_assertion`, `tk.resolution_candidate` | one canonical row per source entity, per claimed identifier and per candidate of an ambiguous entity |
| `tk.material_entity`, `tk.species`, `tk.species_form`, `tk.defined_mixture`, `tk.mixture_component`, `tk.material`, `tk.polymer_type`, `tk.unclassified_entity` | the canonical and provisional entities, and the components of the defined mixtures |
| `prov.*` | the carrier, artifact, import-record, licence and rights rows that the origins cite, and `prov.record` and `prov.record_origin` |
| `report.json` | counts by carrier, class, status and rule; the provisional entities by kind; every ambiguous entity with its candidates; registry values mapped to several structures; unusable structural identifiers; unmatched decisions |
| `manifest.json` | the reuse key and its inputs (section 6): the hash of each source's phase-1 manifest, of the decisions file, the declaration fingerprint, the framework files and libraries of the stage |

The result is a pure function of the phase-1 outputs and the decisions file: entities are processed in
key order, every collection is a set or sorted, and rows are written in identifier order, so running it
twice gives byte-identical files and the carriers in any order give the same result. A run whose key is
unchanged is skipped; `--force` runs it again.

## 2. Mapping

A mapping turns the source-faithful tables of one source into canonical rows. It has two parts
that live together in `mappings/<id>/`:

- **`mapping.toml`**: the declared rules. For each source table: its disposition (`mapped`,
  `out_of_scope` with a reason, or `deferred` with the wave that will map it); for each mapped
  column or field: the target (a kind attribute, a relation value, or a `form.group.slot`), the
  source unit, the precision (`exact`, `narrower`, `broader`, `close`) and what is lost or
  assumed. The class of each scope of source entities, the conventions the source's values assume
  (which `convention_set`) and the scopes rule 4 applies to are declared here too; so are the
  convention facts the values assume.
- **`mapping.py`**: the structure the rules cannot express: how nested or repeated source
  structures become sets, families and nested sets; how subjects are keyed. It contains no units,
  no constants and no defaults: those are in `mapping.toml`.

Phase 1 emits source entities (each with its class), identity assertions and the component claims
of defined mixtures, only. Phase 2 runs after resolution and emits everything else, referring to
subjects by source entity; the framework substitutes the resolved target.

What the framework does, so that no mapping re-implements it:

- **Identifiers.** Every canonical row's identifier is computed from its declared identity
  values (`thermo_knowledge.identity`). A mapping never invents an identifier.
- **Provenance.** Every emitted record is tied to the source-faithful row or rows it came from.
  The framework creates the carrier, artifact and import-record rows from the staged manifest and
  the rows' `_artifact` and `_locator`, registers the record, and links them. A record with no
  origin is refused.
- **Units.** Values are converted from the declared source unit to the slot's or attribute's
  storage unit with `pint`. The conversion factor is recorded with the mapping rule. A value
  whose source unit is `not stated` is mapped only under an explicit assumption in
  `mapping.toml`, recorded as loss.
- **Validation.** Rows are validated against the declaration before they are written: types,
  required values, enum members, invariants with `enforced = "load"`, transposable subjects stored in
  the canonical orientation (the values are kept as the source asserted them, with the arrangement
  recorded; the rule acts on reading).
- **Coverage.** Every source-faithful row ends in exactly one state: `mapped`, `mapped_with_loss`,
  `out_of_scope`, `deferred`, `held` (its subject is ambiguous or unknown, or it failed validation)
  or `unmapped`. A held or unmapped row has a typed reason (a `held_reason` member:
  `ambiguous_subject`, `unknown_subject`, `validation_failed`, `unit_not_parseable`,
  `missing_convention`, `not_a_number`, `not_an_integer`, `unusable_key`, `pattern_mismatch`,
  `unmapped_by_mapping`) and a detail in words. The counts per source table and state, and the rows
  not loaded grouped by reason, are the coverage report; nothing parses the detail. A row a mapping
  read but emitted nothing for is `unmapped`; silence is never coverage.
- **Rule use.** The context counts, for each value rule, the mapped rows it was applied to: rows
  for which the rule produced a value in a block that wrote records (a block that was held counts
  nothing). After phase 2, a value rule of a table that has mapped rows and was applied to none
  refuses the run, naming the rule: the mapping no longer emits what it declares. A rule declared
  `optional = true` (a field the source fills only sometimes) is reported instead. The count is
  `qual.mapping_rule.applied_rows`.
- **Competing assertions.** When two import records of one carrier would produce the same
  record identity with different content, the mapping is wrong about identity and the run is
  refused, naming both locators.

Output: `.store/canonical/<id>/<schema>.<table>.parquet`, one file per canonical table the mapping
emits, with a manifest carrying the reuse key and its inputs (section 6: the source-faithful manifest,
mapping version, declaration fingerprint, resolution result, framework files and libraries). The
directory is written beside its destination and renamed into
place, so a failed run leaves nothing that looks finished; `_identity/` (phase 1) is kept across a
phase-2 run.

Phase 2 refuses to run unless the phase-1 output exists and is current (its reuse key equals the key of
the present mapping, staged data and declaration) and the resolution result exists and records the hash
of exactly that phase-1 output and the present declaration. "Older" is decided by these recorded inputs,
never by a clock. Each phase is skipped when its reuse key is unchanged; `--force` runs it again.

### 2.1 `mapping.toml`

```toml
source = "coolprop"                       # the directory name and the source manifest id
doc = "What the mapping covers."

[scopes.fluids]                           # a scope of source entities: each row of `table` is one
table = "fluids"
key = "NAME"                              # the column holding the local key, verbatim
class = "species"                         # the class of every entity of the scope (section 1) ...
doc = "What a key of this scope names."

# ... or, instead of `class`, the class from a column of the row and a value-to-class table:
[scopes.fluids.class_by]
column = "CAS"
default = "species"                       # the class of a row no rule covers
[[scopes.fluids.class_by.rules]]          # the first rule a value satisfies decides
pattern = '.+\.[Pp][Pp][Ff]'              # the whole value matches (or `values = ["a", "b"]`, exactly)
class = "defined_mixture"
reason = "Why a value of this shape means this class."

[scopes.fluids.mixture]                   # exactly for a scope that has defined mixtures:
definition = "by_definition"              #   a `mixture_definition` member
basis = "mole"                            #   `mole` or `mass`: what the source's fractions are
reason = "Why these, and what is assumed where the source does not say."

[[formula_scope]]                         # optional: scopes resolution rule 4 applies to
scope = "fluids"
reason = "Why a formula and charge identify these entities."
discriminator = "aqueous"                 # optional text that joins the species key

[parameterizations.saturation_ancillaries]
key = "coolprop_saturation_ancillaries"
revision = "{pin}"                        # `{pin}` is the carrier's resolved pin
title = "CoolProp saturation ancillaries"
coherence = "independent_records"
origin_role = "published"                 # how the carrier presents the collection (an `origin_role` member)
no_convention_set = "Why the source states none."   # or: convention_set = "<name>"

[convention_sets.<name>]                  # attributes of the `convention_set` kind; a quantity is
temperature_scale = "its_90"              #   { value = 8.314, unit = "J/mol/K" }
gas_constant = { value = 1.98720425864083, unit = "cal/(mol*K)" }

[tables.<table>]
disposition = "mapped"                    # mapped | out_of_scope | deferred
reason = "..."                            # required unless mapped
wave = 2                                  # deferred only: the wave that will map it
loss = "..."                              # optional: a loss that applies to every mapped row
origin_role = "fitted"                    # required when rows are mapped: an `origin_role` member

[[tables.<table>.partitions]]             # rows matching `where` take this instead of the table's
name = "saturation_pressure"              #   disposition; partitions are disjoint
where = { ancillary = "pS", using_tau_r = true }   # column = value, or column = [values]
disposition = "mapped"                    # the same keys as a table: reason, wave, loss, origin_role

[tables.<table>.partitions.constants]     # canonical values no column holds, by `kind.attribute`
"validity_region.kind" = "fitted_range"     #   a declared entity, an enum member, text, a number,
"region_clause.observable" = "temperature"  #   or { value = ..., unit = "..." }

[tables.<table>.partitions.derivation]    # how the records made from these rows were produced
kind = "fit"                              # a `derivation_kind` member; `fit` makes a `fit` record
outcome = "not_stated"                    # a `fit_outcome` member, for a fit only
method = "What the mapping says produced them."   # text declared here, not the source's

[tables.<table>.fields.<column>]          # one rule for every column when rows of the table map
target = "vapor_pressure_exp_series_tau.pure.T_r"
unit = "K"                                # the source unit; required for a dimensioned target
precision = "exact"                       # exact | narrower | broader | close
loss = "What the rule loses or assumes."
optional = true                           # a value rule only: the source fills the field only sometimes
```

A parameterization is the carrier's collection of parameter sets, and `origin_role` (required) is how the
carrier presents that collection. The framework gives the parameterization's record, and the convention
set it names, that role as their origin role, whichever table's rows emit them; the role of a parameter set
is the role of the table or partition its own rows come from (it is the `role` of the `record_origin` rows of
the set's record, and the set has no other home for it), so a collection of `fitted` sets can itself be
`published`.

The class of a scope states what its source says its keys are, never what resolution would
conclude. A scope states exactly one of `class` and `class_by`; each class must be an `entity_class`
member with resolution rules (`pseudo_component` has none yet); a scope that has defined mixtures
(its `class` or any class of its `class_by` is `defined_mixture`) states `mixture`, which also
completes the provisional entity of a mixture whose source gives no composition. An entity states an
aggregation exactly when its class is `species_form`.

A field rule is exactly one of:

- **a value** (`target`; the rule must be applied to a mapped row of its table, or be declared
  `optional = true`, section 2): `kind.attribute`, `relation.column`, `form.group.slot` or
  `form.group.family.slot` (a family index is a target too). `unit` is required when the target is a
  quantity (or a `Real` whose unit its record fixes) and must convert to the target's storage unit;
  when the staged column states a unit the rule says the same one, and when it states none the rule's
  `loss` records the assumption. `offset` (an Integer target) is added to the source value, for an
  index the source counts from zero. An identity assertion is the target `identity_assertion.value`
  with its `scheme`; `absent` lists the values that mean "no value" in the source (a placeholder is
  never asserted); `pattern` with `otherwise_scheme` asserts a value that does not fit the scheme's
  shape under another scheme (a pseudo-CAS file name as `source_local`), and without
  `otherwise_scheme` holds the row; a component fraction of a defined mixture is the target
  `mixture_component.value` with the unit `dimensionless` (or a unit that converts to it);
- **structure** (`role = "structure"`, with a `reason`): the column keys, selects or indexes rows and
  carries no value of its own;
- **not mapped** (`disposition = "out_of_scope"` or `"deferred"`, with a `reason`, and a `wave` for
  deferred).

A `[convention_sets.<name>]` table states attributes of the `convention_set` kind as value rules do: a
dimensioned fact such as `gas_constant`, `boltzmann_constant` or `avogadro_constant` as `{ value, unit }`
with a source unit that converts to its storage unit, an enum member by name. A form may declare the convention facts it reads
(`meta-model.md` section 4.2); a mapping that emits a set of such a form under a parameterization
whose convention set does not state a declared fact is refused, naming the form and the fact, and
the run writes nothing.

The declaration is checked against `mapping.toml` before anything runs: every staged table has a
disposition (a table with none is an error), every column of a table that maps rows has a rule, every
target exists, units convert, schemes, enum members, observables and roles exist. Each table,
partition, column, constant and derivation is written to `qual.mapping_rule` (a partition as
`partition:<name>`, a constant as `constant:<target>`, a derivation as `derivation:<partition>`), a
value rule with the conversion factor of its unit (empty for a unit with an offset, and for a `Real`
target whose unit comes from its record) and with the number of mapped rows it was applied to.

### 2.2 `mapping.py`

`mapping.py` defines `identities(ctx)` (phase 1) and `records(ctx)` (phase 2) and holds structure only:
it names tables, scopes, kinds, slot groups and families, and never a unit, a number or a default.

- `ctx.rows(table, partition=None)` yields the rows the mapping maps, each with `_artifact`, `_locator`
  and its values; `ctx.group(table, *columns)` groups them.
- `ctx.quantity(row, column)` returns a value with the unit its rule declares, `ctx.value` a value
  with the rule's absent markers removed, `ctx.integer` an integer with the rule's offset;
  `ctx.slot_values(row, group)`, `ctx.family_row(row, family)` and `ctx.attributes(row, kind)` gather
  the columns whose targets lie in a slot group, a family or a kind, and the constants declared for the
  row's table or partition.
- `ctx.subject(scope, key)` (phase 2) is the material entity a source entity resolved to;
  `ctx.entity(scope, key)` (phase 1) is a source entity an earlier block of the run declared, to
  cite it as the component of a mixture.
- `with ctx.emit(row, *also) as emit:` is one unit of work whose records have these rows as origins:
  phase 1 emits `emit.source_entity(scope)` (its class comes from the scope), `emit.assertion(entity,
  column[, value])` and `emit.component(mixture, component, row, column)` (the component's fraction,
  read from `column` of `row` under a `mixture_component.value` rule, converted to a dimensionless
  number and kept as exact decimal text); phase 2
  emits `emit.parameter_set(...)`, `emit.kind(...)`, `emit.relation(...)`,
  `emit.parameterization(name)`, `emit.validity(record, region, clauses)`,
  `emit.validity_not_stated(record, region)` and `emit.derivation(outputs)` (the derivation declared for
  the block's partition, a `fit` when it is one, with a `derivation_output` row per output record and no
  `derivation_input`: a mapping links inputs only when it can name the records; a record presented in a
  role that requires a derivation, a validity region included, is the output of one). `emit.validity`
  writes one region of the validity of a parameter set, a parameterization or a model assembly with its
  clauses and the coverage row `stated` in one call (`region` is `ctx.attributes(row,
  "validity_region")`, each clause `ctx.attributes(row, "region_clause")`; regions of one kind are
  alternatives and each has its `ordinal`), and `emit.validity_not_stated` records that the source
  gives no region of that kind, with the coverage row `not_stated`. A block is atomic: if validation, an ambiguous or unknown subject or a
  value that fits no scheme refuses anything in it, none of its records is written and its rows end
  `held` with the typed reason and its detail. A refusal about identity (section 2's competing assertions) is not held: it
  ends the run.

### 2.3 Canonical rows

Every projected table has one explicit Arrow schema, read from the generator's description of the
table: the columns, their order and their nullability are those of the generated DDL, and only a
column's type is translated (`uuid` to Arrow's `uuid`, text, identifier-scheme domains and enums to
`string`, quantity domains to `float64`, a hash to `fixed_size_binary(32)`, a timestamp to
`timestamp[us, UTC]`, an array to a list). A column PostgreSQL computes (the interval of a piecewise
family) is not carried.

An identifier is carried as Arrow's canonical `uuid` extension type: 16 bytes in memory and in
Parquet (logical type `UUID`), preserved by a Parquet round trip, and ingested by the ADBC PostgreSQL
driver into a `uuid` column, whose binary `COPY` format is the same 16 bytes. A `string` is not loadable
that way. Loading inserts every table with `adbc_ingest` in one transaction with the deferrable
constraints deferred, so tables load in any order and the commit validates every constraint; a file
whose schema is not the canonical schema of its table is refused first. A PostgreSQL range has no
binary form the driver ingests, so `Range<Q>` attributes are refused for now.

The writer is the only way a mapping emits rows. An identity attribute with a declared default takes
it before the identifier is computed; a parameter set takes an optional `occurrence` (from one, in
source order, for repeated assertions for one subject within a parameterization) and one when it is
not given; a nested set keeps occurrence one, its subject key already encoding its parent. It computes identifiers from the declared identity,
writes one row per table of a kind's refinement chain, registers a record for a kind with
`provenance = "own"` and refuses a record without an origin, creates the provenance rows, and
validates before writing (types, required values, enum members, domain checks, `ddl` invariants from
their declared check, `load` invariants from a registered evaluator, units). The unit of a `Real` or
`Array<Real>` value is the one its declaration's `unit_from` gives: `dimensionless`, or a path of
references read from the record itself (a quantity type, a slot or an observable, possibly through
another record the writer wrote or the declaration holds, as `column.observable` reaches the unit of
a datum through its column); a value whose reference is absent or unknown is refused. A declared
`load` invariant with no evaluator stops the writer at start. A slot whose value is a tabulated function takes
a `TabulatedFunction`: one call writes the function, keyed by the set, the slot and the family index,
with its axes (numbered from one, each a quantity type and ascending points) and its series (each a
quantity type and values in row-major order over the axes), and refuses, naming the locator, an axis
that is not strictly ascending, a series whose number of values is not the product of the axes'
lengths, a repeated series name, and no axis or no series. For a set of a form whose
contract takes an output's observable from a set, the writer checks that the observable the set
names is declared and that its quantity type has the dimension of the output's declared type
(meta-model section 4.1). A slot that accepts a contract takes a
`NestedSet`: the writer checks that its slot group's form implements that contract and writes it as
a parameter set with `parent` and `parent_slot` set, the parameterization and origins of the
set that holds it, and `subject_key` the canonical encoding of the parent set, the slot and the
family index (empty outside a family), as `slot_uncertainty.index_key` is. A nested set may sit in a
family row and may itself hold nested sets. A slot that references a contract takes a
`SetReference` (parameterization, slot group, subjects and occurrence of a top-level set): the
writer computes the target's identifier as a set is identified, refuses a target whose slot group's
form does not implement the contract or whose subjects do not fit its group, and stores the
identifier, so any number of sets reference one set, written once by its own call. The writer also
refuses a parameter set whose subject it wrote itself (a declared entity or a record it registered)
with another kind than the role declares; a subject it has not seen, written by another stage, is
left to the foreign keys at build. For a parameter
set it stores the subjects in the canonical orientation (the order the DDL checks) and **the values
exactly as the source asserted them**: where the group's rule acts on values (`parity`,
`reciprocal`, `linear`) or keeps an order (`permutation_group`) the row also records `arrangement`,
for which arrangement of the subjects the values were asserted (0 canonical, 1 the swapped order of a
two-role rule, for a permutation group the 1-based number of the arrangement that takes the asserted
order to the canonical one). The writer applies no rule and changes no number (a reciprocal slot
holding zero is stored), so a value read in the orientation it was asserted in is the source's
number, bit for bit; `thermo_knowledge.transposition` holds the one implementation of the rule, which
a parameter source applies on reading (section 5.3). It writes the set, slot-group and family
rows. `subform_choice` writes one row of `subject_subform_choice`: the form a parameterization
chooses for a sub-form slot chosen per subject, for the subjects of the roles of the contract the
slot accepts (the writer refuses a form that does not implement it, a slot chosen per model, and an
ordinal above one for a slot that is not `many`). Provenance rows follow from the carrier (the source manifest, the lock entry and the staged
manifest) and each origin: `source` and `carrier` (key `<manifest id>@<pin>`), `artifact` (hash and
size of the file), `import_record`, a `licence` where a rights entry gives an SPDX identifier (keyed
and titled by it), a `rights_determination` per rights entry (ordinal counted per scope), and a
`record_origin` per origin. A text key that restates a reference (`site_class.host_key`,
`association_site.carrier_key`) is computed by the writer from the reference the mapping gives, and a
mapping that gives the key is refused. `citation(carrier, key, doi, year, citation)` writes the
publication a carrier's citation key denotes and the `citation` row that ties the key to it. A
publication is identified by the work: its key is `doi:` and the lower-case DOI when it has one, so
two carriers that cite one paper under different keys name one publication, and otherwise
`carrier:<manifest id>:<key>`; its `title` is its key, its `doi` the lower-case DOI and its `isbn`,
when stated, the work's. These are its identity-bound attributes, and a carrier that states them
differently is in conflict with the other, as with any record. What a carrier says about the work
(its own citation key, the year and the citation text) is per carrier and is kept on the `citation`
row, keyed by the carrier and the key, so carriers that differ in it still agree on the publication
and a build accepts them together. The load invariant `publication.key_follows_doi` states the key
rule.

#### Canonical key encodings

Every key the framework computes is text built from the canonical encoding of `thermo_knowledge.identity`:
a compact JSON array of the values in the order given, with no spaces and non-ASCII characters kept;
text, scheme values and enum members as JSON strings normalised to NFC; references as lowercase
hyphenated UUID text; integers in decimal; booleans as `true` and `false`. A floating-point number is
never an element: the encoding refuses one, and no key is built from one.

| Key | Encoding |
|---|---|
| `species.canonical_key`, structural rule | the standard InChIKey, as given or computed by RDKit from an InChI or a SMILES |
| `species.canonical_key`, registry rule | the standard InChIKey of the one structure the registry value maps to |
| `species.canonical_key`, formula rule | `formula:<formula>:<charge>` and, when the scope declares a discriminator, `:<discriminator>`; the formula is NFC and stripped of surrounding space, the charge a signed decimal integer (`+0`, `+1`, `-2`) |
| `species.canonical_key`, curated decision | the `canonical_key` of the `identify` decision, verbatim |
| `canonical_key` of a provisional entity, whatever its class | `provisional:` followed by the canonical encoding of [carrier key, scope, local key]; a carrier key is `<manifest id>@<resolved pin>` |
| `defined_mixture.canonical_key`, composition rule | `mixture:` followed by the canonical encoding of [`mole` or `mass`, then for each component in key order its canonical key and its fraction as exact decimal text] |
| `material.canonical_key`, registry rule | `material:` followed by the canonical encoding of [naming scheme, identifier] |
| `canonical_key` of an entity a curated decision identifies | the `canonical_key` of the decision, verbatim, whatever its class |
| `publication.key` | `doi:` and the DOI in lower case when it has one, else `carrier:<manifest id>:<the carrier's citation key>` |
| `species_form.canonical_key` | the canonical encoding of [species key, aggregation name], with the polymorph name appended when the entity states one; a provisional species gives its own key |
| `parameter_set.subject_key`, top-level set | the canonical encoding of the subject identifiers in the slot group's role order, in the canonical orientation; `[]` for a group with no subject |
| `parameter_set.subject_key`, nested set | the canonical encoding of [parent set identifier, `parent_slot` identifier, index key] |
| `subject_subform_choice.subject_key` | the canonical encoding of the subject identifiers of the roles of the contract the slot accepts, in that role order, as the caller names them |
| index key of a family row | the canonical encoding of the family's index values in declared order, as text; the empty text for a slot outside any family |
| `tabulated_function.key` | the canonical encoding of [holding set identifier, slot identifier, index key] |
| `site_class.host_key`, `association_site.carrier_key` | the identifier text of whichever of the two references is present |
| `derivation.key` of a mapping | `<source id>:<identifier of the first record it produced>` |
| `qualification_run.key` | as section 5.4 states |

The structural check `subject_key_matches_subjects` recomputes every `subject_key` from the rows of its
slot-group table (for a nested set, from its parent, slot and the family row that holds it), and the
verify invariants `site_class.host_key_matches_host` and `association_site.carrier_key_matches_carrier`
hold the other two text keys to their references. No stage writes the canonical key of a reaction, a
constituent array or another kind not listed here yet, and this page states no encoding for one.

### 2.4 Row states

The state of a row follows from the dispositions in `mapping.toml` and what the run recorded:

- a row of an `out_of_scope` or `deferred` table or partition is in that state;
- a row of a mapped table or partition whose block emitted records is `mapped`, or `mapped_with_loss`
  when a rule it used, or its partition or table, declares a `loss`;
- a row a block refused is `held`, with its typed reason and detail; a block refuses a row whose subject is `ambiguous`
  (an `unresolved`, `rejected` or `unique` subject loads: the records attach to its entity, which is
  provisional unless unique);
- every other row of a mapped table or partition is `unmapped`.

Phase 1 writes a ledger of the rows it emitted from or held, and phase 2 merges it with its own, so a
row mapped in either phase is covered. The counts are written to `qual.mapping_coverage`, each held or
unmapped row to `qual.held_row` (its `reason` a `held_reason` member, its `detail` the words).

## 3. Build

`tk build` replaces the canonical schemas of `pse_thermo` (`meta`, `prov`, `tk`, `param`, `ev`,
`qual`) from the declaration and every source's canonical Parquet, in one transaction. The
source-faithful `src_<id>` schemas live in the same database and are not touched, so canonical
records and the rows they came from can be queried together.

Before the database is touched, the build reads `.store/canonical/_resolution/` (when it exists) and the
records of every source, verifies each directory against its manifest, and takes the union of every
table (step 2). A manifest written against another declaration fingerprint refuses the build, and the
message says which source must be mapped again (`tk map <id>`), or that `tk resolve` and then `tk map`
must run for the resolution result. A source's carrier (manifest id, resolved pin, tree hash) is read from
the phase-1 output kept beside its records, which must be the output the records were mapped from. A
source with only phase-1 output has no records and is left out.

Everything then happens on one connection, since a transaction cannot span two:

1. Begin a transaction. Refuse when an object outside the six canonical schemas (a view in a `src_<id>`
   schema, a foreign key, a trigger, a column default) depends on an object in them, because dropping the
   schemas would remove it. Drop the canonical schemas; apply the generated DDL and `sql/physical.sql`;
   insert the reified declaration and declared entities.
2. For each canonical table, take the union of the resolution output and all sources' files. Rows with
   the same key (the table's primary key: a kind's identifier, a relation's key columns, a family row's
   set and index) and identical content are one row (the same entity, carrier or convention emitted by
   several stages; each record keeps its own origins). Rows with the same key and different content
   refuse the build, naming the table, the key, the contributing sources and the differing attributes.
   The union is columnar: the key columns are sorted once, and content is compared with Arrow kernels
   for the rows that share a key only. A table with no shared keys loads from its files as they are; one
   with identical duplicates loads from a copy without them.
3. Bulk-load with constraints deferred.
4. Record the schema fingerprint as a comment on schema `tk`, and one row per source built in
   `meta.build_source`.
5. Commit. The commit validates every foreign key, check and exclusion constraint. Any failure
   rolls the whole transaction back and leaves the previous canonical schemas exactly as they
   were; readers never see a partly built database.

`meta.build_source` is created by the build, not by the declaration, and is not part of the schema
fingerprint. One row per source built: `manifest_id` (primary key), `resolved_pin`, `tree_hash` (the
lock's), `reuse_key` (of the mapping output that was loaded) and `built_at`. The resolution result is not a
source and has no row.

`tk build --sources <id>` builds the named sources (repeat the option or comma-separate ids; the
resolution result is always loaded); without it every source with records is built. `tk build --dry-run`
performs the union of step 2 and every refusal above, reports the rows per table (and the identical
duplicates merged) and touches no database. `tk db status` reports the fingerprint the database records,
whether it equals the current declaration's, and the sources built.

Qualification outputs under `.store/canonical/_qualification/<case>/` join the union when they are
current for the values the build holds (section 5.4, Currency); each one left out is reported with its
reason and the count of those loaded and skipped is printed.

With no canonical Parquet at all, a build yields the empty schema with the reified declaration; there is
no separate schema-only path.

The database is never migrated. A build from the same lock, declaration and mappings reproduces
every identifier and every row.

## 4. Verify

`tk verify` runs the invariants the declaration marks `enforced = "verify"`, each as one named
check that returns the violating rows, plus these structural checks:

- every record has at least one origin (except the record of a declared entity), and every origin's
  carrier has at least one rights determination (which may be `not_stated`);
- every row of an abstract kind has exactly one concrete refinement row;
- reactions conserve every conserved quantity with a composition entry on a participant, the balance of
  an element counting its isotopes too (the declared invariant `reaction.conserves_declared_quantities`,
  enforced by one check);
- nested sets implement the contract their slot accepts;
- a set-reference slot holds a top-level set of a form that implements the contract the slot names
  (`referenced_set_implements_contract`, read from `meta.slot`);
- a parameterization that holds a parameter set of a form that declares convention facts has a
  convention set that states each of them (`parameterization_has_conventions`, read from
  `meta.form_convention`);
- family indices are contiguous from their minimum;
- a record presented in a role with the facet `requires_derivation` (`origin_role` marks `fitted`,
  `estimated` and `derived`) has a producing derivation (a derivation is exempt), and when the role has
  the facet `requires_fit` (`fitted`) that derivation is a fit. The roles are read from `meta`, so a
  member given the facet is covered without editing the checks; a record whose origins present it in
  different roles is covered when any of them has the facet;
- every `parameter_set.subject_key` is the canonical encoding of the subjects of its slot-group row
  (section 2.3).

The declared invariants of relations are checked the same way as those of kinds: a relation's
`requires` with `enforced = "verify"` has a check file named `<relation>.<requirement>`, and the
verify stage treats both alike. Derivation lineage and dependencies being acyclic are two of them
(`derivation_input.lineage_acyclic`, `dependency.acyclic`). A check about a property of enum members
reads the member's facets from `meta`, as the checks of `origin_role` do: the magnitudes of an
uncertainty against the facets `relative`, `factor` and `unquantified` of `uncertainty_kind`, and a column
presented against a reference state against the facet `relative_to_reference_state` of
`value_presentation`. The relation `subject_subform_choice` has two: the chosen form
implements the contract its slot accepts, and the ordinals of one slot and subject run from one
without a gap and exceed one only for multiplicity `many`.

A check is a file `sql/verify/<name>.sql`. Its first line names what it enforces, `-- invariant:
<kind or relation>.<requirement>` for a declared verify requirement or `-- structural: <name>`; its second is a
`--` comment with a one-line description. The rest is a SQL query over the built database that returns
the violating rows: an `id` column that locates the record, a `locator` column where the record has an
origin, and whatever else describes the violation. Zero rows is a pass. A table name that only the
declaration decides (the refinements of an abstract kind, the tables of the families) cannot be written
in a static query, so a check may put a line `-- query` before its query: everything above it is a setup
script (typically a function in `pg_temp` that reads `meta.kind` or `meta.family`), and every check runs
in a transaction that is rolled back, so nothing it creates remains.

`tk verify` refuses a database that records no schema fingerprint or one that differs from the current
declaration's (it must be rebuilt first). It then runs every check, prints a table (check, kind,
violations, the first offending identifiers), writes `.store/verify-report.json` beside the canonical
store (`--report` names another path) and exits non-zero when any check has violations or cannot run.
It also exits non-zero when a declared verify invariant has no check file, or a check file names an
invariant that the declaration does not declare with `enforced = "verify"`: an invariant without an
enforcement point is not allowed to exist.

## 5. Qualify

Qualification establishes that a form's canonical expression, evaluated from canonical records,
reproduces an independent answer.

- An **oracle harness** is a small script per library that evaluates the library's own model
  (section 5.2). It runs in the environment that has the library, reads a request file (which
  subjects, which library call, which points) and writes a result file (the values, and the
  library version). It never reads canonical records: the library answers from its own bundled
  data, so agreement means the mapping preserved the meaning.
- A **qualification case** is declared in `qualification/<case>.toml` (section 5.1): the form, the
  contract output, the parameterization, the subjects (all, or a declared sample), the grid of
  argument values (within each set's validity region), the comparison basis, the tolerance, the kind
  of validity region its points are counted against, and the harness.
- `tk qualify` evaluates the form from the database through the reference evaluator (section 5.3),
  runs the harness, compares, and writes `qual.qualification_run` rows (section 5.4).

Whether a form is qualified is a fact about recorded runs and is derived, never declared: a form's
status is `catalogued` or `expressed`. Each run records the hash of what evaluating its output
reads from the form's text (`expression_hash`), and `meta.form_output.evaluation_hash` holds the
current one. The view `qual.form_qualification` (`sql/physical.sql`) lists, for each form and
contract output, the passing runs whose recorded hash equals the current one, with the comparison
basis, the reference and the worst deviation; a run made against an earlier text of the form is not
listed, and a form with no row has no passing run against its current text. The hash covers the
form's own locals, the output's expression and its implicit blocks; the forms chosen for its
sub-form slots are other forms with hashes of their own.

Comparison bases rank, from strongest: a verification table published with the formulation; an
independent evaluation; a second carrier of the same published values; the source library
itself. A form qualified only against its own source library has shown that the mapping is
faithful, not that the equation is right.

### 5.1 Qualification cases

`qualification/<case>.toml`; the file's stem is the case's name. An unknown key refuses the file,
and the declaration is checked before anything runs: the form and its output, the basis, the slot
group, a grid for every argument, units that convert, and the forms chosen for sub-form slots.

```toml
doc = "What the case compares."
form = "vapor_pressure_exp_series_tau"      # the form evaluated
output = "p_sat"                            # the contract output compared
basis = "source_library"                    # a `comparison_basis` member

[parameterization]
key = "coolprop_saturation_ancillaries"
revision = "{pin}"                          # `{pin}` is the resolved pin of `carrier`
carrier = "coolprop"                        # required with `{pin}`, refused without
# occurrence = 1                            # which repeated assertion to read, from 1 (section 5.3)

[subjects]
group = "vapor_pressure_exp_series_tau.pure"   # the sets of this slot group name the subjects
select = "all"                              # all | list (with `keys`) | sample (with `size`, `seed`)
keys = ["Water"]                            # select = "list": the library's own keys
size = 10                                   # select = "sample": how many, drawn with `seed`
seed = 7

[subjects.library_key]                      # how the library's own key of a subject is obtained:
carrier = "coolprop"                        #   the local key of the source entity of this carrier
scope = "fluids"                            #   and scope whose target is the subject

[arguments.T]                               # one grid for each argument of the contract
points = 25                                 # `points` spread evenly within the one region of each set ...
within = "envelope"                         #   along the clause on the observable the argument names
envelope_kind = "fitted_range"              #   optional: when a set has regions of several kinds
inset = 0.0                                 #   optional: a fraction of the range left out at each end
# values = [298.15, 350.0]                  # ... or explicit `values`, in `unit`
# unit = "K"

[validity]                                  # optional: count the points against the regions of a kind
kind = "fitted_range"
outside = "compare"                         #   `compare` the points outside them anyway, or `exclude` them

[pieces]                                    # optional: how a piece lookup reads the stored pieces
boundary = "upper_piece"                    #   a boundary point belongs to the piece above ([low, high)) or
                                            #   to the piece below ("lower_piece": (low, high])
outside = "refuse"                          #   a point beyond the pieces is refused, or given to the "nearest" piece

[comparison]
relative_tolerance = 1e-12
absolute_tolerance = { value = 1.0, unit = "Pa" }   # optional; needs an output that denotes an observable
invalid_points = "fail"                     # fail | exclude (with `invalid_reason`)
invalid_reason = "why the library has no answer there"

[harness]
library = "coolprop"                        # oracles/<library>.py
environment = "core"                        # `core`, or a side environment of envs/
call = "saturation_pressure_ancillary"      # what the harness evaluates

[[subforms."form.slot"]]                    # only for a form with sub-form slots: the form chosen
form = "other_form"                         #   for the slot and where its own sets are read
parameterization = { key = "k", revision = "r" }
```

- **Subjects** are the top-level sets of `group` in the parameterization, each with the library's key
  of every subject role, listed in key order. A parameterization may hold several sets for one
  subject (repeated assertions, distinguished by `occurrence`): with `occurrence = <n>` the case
  reads the set of that occurrence, and a subject the parameterization holds no such set for is no
  subject of the case; without it, a subject with several occurrences blocks the run, and the note
  names the subject and the occurrences present. A subject whose key is not exactly one source entity of
  the carrier and scope blocks the run. `sample` draws with `random.Random(seed)` from the keys in
  order, so a seed gives the same subjects on every machine. A contract with index sets is refused:
  a case evaluates a form for single subjects (model assemblies come later).
- **Points.** `points` spread evenly within the one validity region of the named kind (any kind when
  none is named) of the set, along the clause of that region on the observable the contract argument
  names, from the lower bound plus the inset to the upper bound minus it (one point is the midpoint). The
  grid refuses, naming the set, and the run is blocked with `no_grid`, when the argument names no
  observable, when the set has no region of that kind or several (alternatives), or when the region has
  no clause on that observable about the whole record (none, or only clauses about a component or a
  phase), several, or one open at an end: the case must then give explicit `values` for the argument.
  A region with other clauses (a pressure limit next to the temperature limit) is gridded along the
  clause the argument follows; the other clauses are what the validity counts decide. Several arguments
  give the Cartesian product, the last varying fastest. Values are in the storage units of the
  argument's type.
- **Validity.** A case with a `[validity]` table has every point of its grid classified, for each
  subject, as inside, outside or undetermined with respect to the regions of its `kind` on the records
  the evaluation read, or as in none of these because no region of that kind is stated (section 5.3
  and `expressions.md` section 5). The classification is made whatever the grid, so explicit `values`
  beyond a range are recorded as outside. `outside = "compare"` (the default) compares the points
  outside anyway; `outside = "exclude"` leaves them out of the evaluation, the request to the harness
  and the comparison, and reports their count in the run's note. A subject the form is refused for has
  no classification. Nothing is refused because a point is outside: lying outside a region is not
  permission to extrapolate, and the case, not the evaluator, decides what to do about it.
- **Pieces.** The pieces of a family are stored as half-open intervals and stay so: what an evaluation does
  at and beyond their boundaries is a policy of the case, because the libraries read them differently and
  the case compares with one of them. `boundary` says which side a point at a boundary between two pieces
  belongs to (`upper_piece`, the default, is `[low, high)`; `lower_piece` is `(low, high]`; the outermost
  bounds belong to the outermost pieces either way), and `outside` says what a point beyond the pieces
  does: `refuse` (the default) refuses the evaluation, `nearest` gives it to the nearest piece (a point in
  a gap between pieces goes to the nearer, the middle of the gap to the side the boundary rule names). It
  applies to every piece lookup of the form and of the forms it calls, and is part of the case file, so it
  is part of the case's reuse key. The stored data never changes.
- **A point agrees** when its relative deviation from the library, `|form - library| / |library|`, is
  within `relative_tolerance`, or its absolute deviation within `absolute_tolerance` when one is
  given; a form value that is not finite never agrees. A point the library answers with NaN or an
  error is *invalid*: it is counted and listed in the report, and `invalid_points` says whether it
  fails the run or is excluded for the reason stated. A form the evaluator refuses for a subject (a
  value outside its domain) fails the subject's points, with the refusal in the report.
- **Outcome.** `passed` when every compared point agrees, `failed` otherwise, `blocked` when the run
  could not be carried out. A blocked run states its `blocked_reason`, a typed member, and exactly
  then (`qual.qualification_run` enforces it), with the detail in the run's `note`:
  `carrier_not_unique` (the `{pin}` of the case needs a carrier the database holds once),
  `parameterization_missing`, `no_subjects`, `subject_unidentified` (a subject without exactly one
  source entity to take the library's key from), `subjects_not_found` (a listed subject the
  parameterization has no set for), `sample_too_large`, `ambiguous_occurrence`, `no_grid`,
  `harness_missing` (no script), `harness_failed` (the library is absent, the harness crashed or
  timed out, or its result breaks the protocol) and `nothing_compared` (no point could be compared).
  A blocked run is recorded too and is never reused. A database built from another declaration is
  not a blocked run: the command refuses and records nothing.

### 5.2 The harness protocol

The core writes a request file, runs `oracles/<library>.py --request <file> --result <file>` and
reads the result file. The core interpreter runs a harness of a core-environment library (source
`environment = "core"`); `envs/tk-env.sh run <environment> python oracles/<library>.py ...` runs one
of a side environment (`envs/README.md`). A harness is standalone and small: it never reads canonical
records or the database and never imports `thermo_knowledge`; the library answers from its own data. A
non-zero exit, a missing script or a result that breaks this protocol blocks the run, with what the
harness said.

Request (JSON):

```json
{"protocol": 1, "library": "coolprop", "call": "saturation_pressure_ancillary",
 "arguments": {"T": "K"}, "output": {"unit": "Pa"},
 "subjects": [{"key": ["Water"], "points": 3, "arguments": {"T": [300.0, 400.0, 500.0]}}]}
```

`call` names what the harness evaluates and `arguments` and `output` state the units the values are
in (the storage units of the contract's types); a harness refuses a call or units it does not
support. `key` holds the library's own key of each subject role, in the role order of the slot group;
the argument columns are aligned, one entry per point.

Result (JSON):

```json
{"protocol": 1, "library": "CoolProp", "version": "8.0.0", "output_unit": "Pa",
 "subjects": [{"key": ["Water"], "points": [
   {"value": 3536.8, "status": "ok", "message": null},
   {"value": null, "status": "nan", "message": "above the reducing temperature"},
   {"value": null, "status": "error", "message": "the library's message"}]}]}
```

The subjects and points answer the request in order. `status` is `ok` (a finite `value`), `nan` (the
library answered NaN) or `error` (it raised; `message` says what); `value` is null unless `ok`.
`library` and `version` are what the library reports and become the run's `reference`
(`software_release`, keyed `<library>@<version>`; a run blocked before the harness reported them has
no reference). Before a run the core sends the harness a request with no subjects to learn the library and its
version: that and nothing else is what the reuse key of section 5.4 needs, and it finds a missing
library early.

### 5.3 The parameter source

The reference evaluator reads parameters through the `ParameterSource` protocol
(`expressions.md` section 5). `thermo_knowledge.qualify.source.DatabaseSource` implements it over
the built database for one parameterization or an ordered list of them: the first that holds a set
for a slot group and subjects answers.

- **Values** are the numeric slots of the `param.<form>__<group>` row, in storage units, all read
  by one query; a stateful slot that is not `known` is absent, and a `redirect` is followed to the
  same slot of the set it names. The rows of a family come from its table, one query for each
  family used. A nested set is found by following the slot's foreign key, and its values are read
  from that set alone. `prefetch` reads the sets of many subjects and their families in a number of
  queries that does not depend on the subjects.
- **Sub-form slots** are filled from an explicit mapping from slot to forms and parameterizations
  (the case's `subforms`), which takes precedence; for a slot chosen per subject that the case does
  not mention, from the relation `subject_subform_choice` in the parameterizations read (the first that
  states a choice for the subject answers; the choice names the parameterization that holds the chosen
  form's sets when it is not its own). With neither, the slot has no choices and the evaluation is
  refused, naming the slot and the subject.
- **Referenced sets.** A slot that references a set is followed like a nested one: the source returns
  the form and a source pinned to the referenced set, and the subjects of that set, from which the
  called contract's roles the call does not give are bound. A referenced set reads its sub-form
  choices and convention facts as the parameterization that holds it.
- **Orientation.** A set is found by the canonical orientation of the subjects asked for, and its
  values are returned for the order asked: the stored numbers unchanged when that is the order
  asserted (the stored subjects and `arrangement` say which it was), otherwise the group's rule applied
  once (`thermo_knowledge.transposition`: reciprocal slots inverted, the slots of a `linear` rule
  multiplied by its matrix, family rows of odd index negated under `parity`; a slot a `linear` swap
  needs and the set does not hold refuses the evaluation). The in-memory source does the same through
  the same functions when given the declaration. The evaluator applies no rule.
- **Convention facts.** The source gives the convention facts of the parameterizations that supplied the
  sets an evaluation read: the database-backed source reads them from each parameterization's convention
  set. When an evaluation draws sets from several parameterizations (within one source, or through the
  sources of sub-forms and referenced sets) the evaluator requires every fact the evaluated forms read
  to be stated by each of them and equal across them; otherwise it refuses, naming the fact, the two
  parameterizations and the two values, or, for a parameterization with no convention set or whose set
  lacks the fact, the parameterization and the fact. A parameterization that supplied no set is not
  compared. A fact a form declares per component (`component_conventions`) is read for each component from the
  parameterization that supplied the component's set, is required of each, and is not compared
  between components.
- **Validity.** `ParameterSource.validity(kind, reads)` gives, for each set read (and for the
  parameterization each belongs to), the coverage row of that record for regions of `kind` and its
  regions with their clauses (observable, component, aggregation, bounds in storage units); a record
  with neither has no coverage and no region. The database-backed source reads `tk.validity_coverage`,
  `tk.validity_region` and `tk.region_clause`, and follows a nested or referenced set to its own
  record. The evaluator turns them into where each point lies (`expressions.md` section 5). A model
  assembly is not read by an evaluation.
- **Occurrence.** A parameterization may hold several sets for one subject, distinguished by
  `occurrence`. The source takes the occurrence stated for each parameterization (the case's
  `occurrence`; a sub-form choice states its own) and reads that set; a subject that does not have
  it is missing in that parameterization, and the next parameterization answers. A subject asked
  about that has several occurrences in a parameterization with none stated is refused
  (`AmbiguousOccurrence`), naming the subject and the occurrences present: the source never picks
  one. A parameterization that answers first hides the ambiguity of a later one.
- **No defaults.** A missing set is reported as missing (`None`) and the evaluator refuses, naming
  the slot group and subject. A database built from another declaration than the one given is refused.

### 5.4 Running and recording

`tk qualify [<case>...] [--force] [--report <path>]` runs each case (every case when none is named):
it checks the database's fingerprint, selects the subjects and points, asks the harness for its
version, evaluates the form from the database, runs the harness, compares point by point, records
the run and prints a table (subjects, points, passed, failed, invalid, the worst relative deviation
and where it occurred, then the refusals, invalid points and failing subjects). It exits non-zero
when a case failed or was blocked or the declaration refused one. The JSON report, written to
`<store>/qualify-report.json` or to `--report`, holds for each case the outcome, the tolerances, the
library and version, the worst deviation, and per subject its points, passed, failed and invalid
counts, worst deviation and where, and any refusal, with every invalid point. After the cases the
command lists the blocked ones grouped by their typed reason, and the report's `blocked_by_reason`
holds the same grouping.

A run is one `qual.qualification_run` row, with a `qual.run_parameter_set` row for each set
evaluated:

| Column | Holds |
|---|---|
| `key` | `<case>/<form>/<parameterization key>@<revision>/<first 16 characters of the declaration fingerprint>` |
| `form`, `basis`, `outcome` | the form, the comparison basis, `passed`, `failed` or `blocked` |
| `blocked_reason`, `note` | for a blocked run its typed reason, and exactly then; the note is the detail of a blocked run's reason, or what the domain excluded |
| `expression_hash` | the evaluation hash of the compared output when the run was made (what `qual.form_qualification` compares with the current one); every run has one, a blocked run included |
| `reference` | the `software_release` of the library version the harness reported |
| `points` | the points compared (those that failed the run by being invalid included; the points a case excluded for lying outside its validity regions are not) |
| `validity_kind`, `inside_points`, `outside_points`, `undetermined_points` | for a case with a `[validity]` table, its kind and how many points of the grid were inside, outside and undetermined with respect to it (the points excluded for lying outside are counted as outside; points of a subject the form was refused for, and points in no region of the kind because it is stated for none, are in none of the three); the counts are absent for a run blocked before it evaluated and for a case that names no kind |
| `relative_tolerance`, `absolute_tolerance`, `observable` | as declared; the observable is the contract output's |
| `worst_relative_deviation` | the largest relative deviation seen, where it is finite |

**Persistence.** The rows are written as canonical Parquet through the canonical writer under
`<store>/canonical/_qualification/<case>/` (`qual.qualification_run`, `qual.run_parameter_set`, the
release's `prov.source` and `prov.software_release`, `manifest.json` of phase `qualification`, and
the run's `report.json`), so they survive a rebuild that drops schema `qual`. The manifest's reuse
key (section 6) covers the declaration fingerprint, the evaluation hash of the compared output, the
case file's content, the library and its version, the subjects with the identifiers of the sets
evaluated, a digest of the content hashes of every record the run read, the framework files and
libraries of the stage, the harness script and the lock of the environment the harness runs in. A
case whose key is unchanged is reported `current` and not run again (a blocked run always is), and
`--force` runs it again.

**Currency.** An identifier does not change with a value, so a run is current for a build only while
the records it read are, row for row, the ones the build holds. The manifest records a content hash
of every record the run read: each parameter set evaluated (and every set of the parameterizations a
sub-form choice reads from) with its row in the slot-group table and in the tables of its families,
the sets it nests and the sets its reference slots name (followed until no more are found), the
parameterizations they belong to with their convention sets (the convention facts an evaluation
reads) and their sub-form choices, and the validity of every set and parameterization read: its
regions, their clauses and its coverage rows. A hash is the SHA-256 of the record's table and
canonical columns with their values (`thermo_knowledge.build.currency.record_hash`); one function,
`read_records`, decides which records a run reads and hashes them, for both sides: `tk qualify`
computes the hashes from the database it reads, and `tk build` recomputes them from the canonical
Parquet it is about to load. `tk build` loads every output that is current and leaves out, and reports
with the reason, one made against another declaration fingerprint (run `tk qualify <case>` again) and
one whose hashes differ: the reason says how many records changed, are no longer in the build or are
new, and names the first of each. A mapping fix that changes a value under the same identifiers
therefore retires the earlier run, which is then absent from `qual.form_qualification`; an unchanged
rebuild keeps it. `tk qualify` also loads the run into the live database at once, by the build's
load of these tables in one transaction that replaces the rows of that case only (the runs whose key
starts with `<case>/`) and keeps a release row already there.

## 6. Reuse keys

Every stage that keeps its output (`tk read`, phases 1 and 2 of `tk map`, `tk resolve`, `tk qualify`)
records a reuse key in its manifest with the inputs the key was made from, and skips a run whose key
is unchanged. `tk load-src` keeps none of its own: it loads only staged data whose `read` key is
current. `tk build` keeps none: it replaces the database from current inputs every time.

All keys are built by one function, `thermo_knowledge.reuse.stage_key`, so a stage cannot forget
what the others cover. A key always includes:

- **the inputs the stage states**: what it reads (the hashes of the manifests upstream, the
  declaration fingerprint, the content of a mapping, a case or the decisions file) and its format
  number, which marks only a deliberate change to what a stage writes that none of the other inputs
  shows;
- **a digest of the framework source files the stage executes**, from the list declared for the stage
  in `thermo_knowledge/reuse.py` (paths of packages and modules, relative to the package; the list is
  declared, not derived from imports, and a path that does not exist is an error): the path and the
  content of every `.py` and `.toml` file;
- **the installed version of each third-party library the stage declares as result-affecting**,
  read from the installed distribution's metadata (one accessor, `installed_version`);
- **the digest of each file the stage names in addition**: the lock of the environment a side-environment
  reader runs in; for a qualification case the oracle harness script and the lock of the environment it
  runs in (the tree's `uv.lock` for `core`, else the lock files of `envs/<name>/`).

| Stage | Framework files it executes | Libraries | In addition |
|---|---|---|---|
| `read` (and the staleness check of `load-src`) | `staging`, `acquire`, `config.py` | pyarrow | the reader's own source files; the environment lock of a side reader |
| `map` (both phases) | the declaration loader, the generator, the canonical writer, `acquire`, identity and orientation rules, the pipeline contract, `mapping`, `staging` | pint, pyarrow | `mapping.toml`, `mapping.py` |
| `resolve` | the same shared packages, `resolve`, `mapping/claims.py` | rdkit, pyarrow | the decisions file |
| `qualify` | the same shared packages, `qualify`, `expression`, `build` | sympy, numpy, scipy, pint, pyarrow, and the library under test as the harness reports it | the case file, the harness script, the lock of the harness's environment, the content hashes of the records read (section 5.4) |

A change to a provider's version, to a framework file in a stage's list, to an oracle script or to
a lock therefore marks the stages that depend on it stale and no others: a stage reports `current`
only when nothing it declares has changed, and a stage whose input is another stage's output is
stale when that output was made again.

**Why stages keep their own manifests and one key builder, and do not adopt a workflow engine.**
Snakemake, DVC and doit track code, parameters, inputs and a per-rule environment, which is what
the keys above cover. Adopting one would give each stage an engine-owned cache layout beside the
manifests that already carry its inputs and the verification of its files; a stage whose output is a
database (`tk build`, the live load of a qualification run) needs a marker file for the engine to
see; and the engine's metadata would duplicate what the manifests record, leaving two places that
say whether a stage is current. The stages exchange directories with manifests, which the stages
already refuse to read when they are stale, so one small builder that puts the missing inputs in
the key completes what an engine would add. Revisit if the stage graph needs what a manifest cannot
give: running independent branches in parallel, running part of the graph on a cluster, or
sharing a cache across machines.
