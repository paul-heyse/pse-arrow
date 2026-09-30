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
- **Resolution** assigns each source entity a target `material_entity` and a status. It is a
  derived result, recomputed on every run from the assertions and the curated decisions.

Resolution rules, applied in this order; the first that applies decides:

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
5. **Otherwise provisional.** The source entity gets a provisional `material_entity` whose
   canonical key is its own (carrier, scope, local key). Its records load and remain usable
   within that carrier; they are not joined to another carrier's records.

Statuses: `unique` (rules 1 to 4 gave one target), `ambiguous` (candidates remain; provisional
target), `unresolved` (no candidate; provisional target), `rejected` (a curated decision says the
assertion is wrong). Nothing is ever resolved by taking the first match.

A `species_form` is resolved from its species plus the aggregation and polymorph the source
states for the record, never from a suffix in a name. A source that does not state an aggregation
for a record that needs one leaves the mapping to declare it for that table, as an assumption.

Resolution reports, per carrier: counts by status and rule, and every ambiguous source entity
with its candidates.

### 1.1 The decisions file

`identity/decisions.toml` holds the curated decisions, each with a reason. An entity is named by the
manifest id of its carrier, its scope and its local key.

```toml
[[decision]]
action = "identify"      # the entity is this canonical species
entity = { carrier = "coolprop", scope = "fluids", key = "R1123" }
canonical_key = "ZWQIPCQCAOKXDI-UHFFFAOYSA-N"   # an InChIKey, or another declared canonical key
charge = 0               # optional, default 0
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

- `identify` resolves the entity to the species with that canonical key, status `unique`, rule
  `curated`, whatever its assertions say. The species' `inchikey` is the key when the key has the shape
  of a standard InChIKey.
- `reject` gives the entity status `rejected`, rule `curated` and a provisional target.
- `distinct` applies when rules 2 to 4 would resolve both entities to one species: neither is resolved
  by the rule, both are `ambiguous` with that species as their candidate, until an `identify` decision
  settles each.
- At most one `identify` or `reject` names an entity. A decision that names an entity no carrier has is
  reported in the resolution report, not applied and not an error: the file covers the whole corpus,
  and a carrier may not be mapped yet. An unknown key, a missing reason or a malformed decision refuses
  the run.

### 1.2 How the rules apply

- **Which schemes.** The structural schemes and the registry schemes are those the declared
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
  of the carrier key (`<manifest id>@<resolved pin>`), scope and local key. Its species has
  `provisional = true`, no InChIKey and the entity's stated charge.
- **Charge.** An InChIKey encodes its charge, so the structures of one key agree; the species' charge is
  the molecule's formal charge where RDKit builds the molecule, else the stated charge, else 0. An
  entity whose stated charge differs from its species' charge is `ambiguous`.
- **Species forms.** An entity that states an aggregation gets a `species_form` of its resolved (or
  provisional) species, whose canonical key is the canonical encoding of the species key, the
  aggregation name and, when stated, the polymorph; the entity's target is the form.
- **Label.** The species' `label` is never identity. It is the `name` assertion made by the most
  distinct source entities resolved to the species; ties go to a name that is the local key of one of
  those entities, then to the smaller name in code-point order. With no name it is the smallest
  `formula` assertion, else the smallest local key.
- **Origins.** A species' origins are the import records of the source entities resolved to it or
  listing it as a candidate, in the role the mapping declares for those rows; a provisional entity has
  its entity's.

### 1.3 Resolution output

`tk resolve` reads the phase-1 output of every source (section 2) and writes
`.store/canonical/_resolution/`:

| File | Holds |
|---|---|
| `tk.source_entity`, `tk.identity_assertion`, `tk.resolution_candidate` | one canonical row per source entity, per claimed identifier and per candidate of an ambiguous entity |
| `tk.material_entity`, `tk.species`, `tk.species_form` | the canonical and provisional entities |
| `prov.*` | the carrier, artifact, import-record, licence and rights rows that the origins cite, and `prov.record` and `prov.record_origin` |
| `report.json` | counts by carrier, status and rule; every ambiguous entity with its candidates; registry values mapped to several structures; unusable structural identifiers; unmatched decisions |
| `manifest.json` | the reuse key: the hash of each source's phase-1 manifest, of the decisions file and the declaration fingerprint |

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
  assumed. The conventions the source's values assume (which `convention_set`) and the scopes
  rule 4 applies to are declared here too.
- **`mapping.py`**: the structure the rules cannot express: how nested or repeated source
  structures become sets, families and nested sets; how subjects are keyed. It contains no units,
  no constants and no defaults: those are in `mapping.toml`.

Phase 1 emits source entities and identity assertions only. Phase 2 runs after resolution and
emits everything else, referring to subjects by source entity; the framework substitutes the
resolved target.

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
  required values, enum members, invariants with `enforced = "load"`, canonical orientation of
  transposable subjects (the framework reorients and applies the rule's effect on the values).
- **Coverage.** Every source-faithful row ends in exactly one state: `mapped`, `mapped_with_loss`,
  `out_of_scope`, `deferred`, `held` (its subject is ambiguous or it failed validation; with the
  reason) or `unmapped`. The counts per source table and the reasons are the coverage report.
  A row a mapping read but emitted nothing for is `unmapped`; silence is never coverage.
- **Competing assertions.** When two import records of one carrier would produce the same
  record identity with different content, the mapping is wrong about identity and the run is
  refused, naming both locators.

Output: `.store/canonical/<id>/<schema>.<table>.parquet`, one file per canonical table the mapping
emits, with a manifest carrying the reuse key (source-faithful manifest, mapping version, declaration
fingerprint, resolution result). The directory is written beside its destination and renamed into
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
doc = "What a key of this scope names."

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
"envelope.axis" = "temperature"           #   a declared entity, an enum member, text, a number,
"envelope.kind" = "fitted_range"          #   or { value = ..., unit = "..." }

[tables.<table>.partitions.derivation]    # how the records made from these rows were produced
kind = "fit"                              # a `derivation_kind` member; `fit` makes a `fit` record
outcome = "not_stated"                    # a `fit_outcome` member, for a fit only
method = "What the mapping says produced them."   # text declared here, not the source's

[tables.<table>.fields.<column>]          # one rule for every column when rows of the table map
target = "vapor_pressure_exp_series_tau.pure.T_r"
unit = "K"                                # the source unit; required for a dimensioned target
precision = "exact"                       # exact | narrower | broader | close
loss = "What the rule loses or assumes."
```

A parameterization is the carrier's collection of parameter sets, and `origin_role` (required) is how the
carrier presents that collection. The framework gives the parameterization's record, and the convention
set it names, that role as their origin role, whichever table's rows emit them; the role of a parameter set
is the role of the table or partition its own rows come from (it is the `role` of the `record_origin` rows of
the set's record, and the set has no other home for it), so a collection of `fitted` sets can itself be
`published`.

A field rule is exactly one of:

- **a value** (`target`): `kind.attribute`, `relation.column`, `form.group.slot` or
  `form.group.family.slot` (a family index is a target too). `unit` is required when the target is a
  quantity (or a `Real` whose unit its record fixes) and must convert to the target's storage unit;
  when the staged column states a unit the rule says the same one, and when it states none the rule's
  `loss` records the assumption. `offset` (an Integer target) is added to the source value, for an
  index the source counts from zero. An identity assertion is the target `identity_assertion.value`
  with its `scheme`; `absent` lists the values that mean "no value" in the source (a placeholder is
  never asserted); `pattern` with `otherwise_scheme` asserts a value that does not fit the scheme's
  shape under another scheme (a pseudo-CAS file name as `source_local`), and without
  `otherwise_scheme` holds the row;
- **structure** (`role = "structure"`, with a `reason`): the column keys, selects or indexes rows and
  carries no value of its own;
- **not mapped** (`disposition = "out_of_scope"` or `"deferred"`, with a `reason`, and a `wave` for
  deferred).

The declaration is checked against `mapping.toml` before anything runs: every staged table has a
disposition (a table with none is an error), every column of a table that maps rows has a rule, every
target exists, units convert, schemes, enum members, observables and roles exist. Each table,
partition, column, constant and derivation is written to `qual.mapping_rule` (a partition as
`partition:<name>`, a constant as `constant:<target>`, a derivation as `derivation:<partition>`), a
value rule with the conversion factor of its unit (empty for a unit with an offset, and for a `Real`
target whose unit comes from its record).

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
- `ctx.subject(scope, key)` (phase 2) is the material entity a source entity resolved to.
- `with ctx.emit(row, *also) as emit:` is one unit of work whose records have these rows as origins:
  phase 1 emits `emit.source_entity(scope)` and `emit.assertion(entity, column[, value])`; phase 2
  emits `emit.parameter_set(...)`, `emit.kind(...)`, `emit.relation(...)` and
  `emit.parameterization(name)` and `emit.derivation(outputs)` (the derivation declared for the block's
  partition, a `fit` when it is one, with a `derivation_output` row per output record and no
  `derivation_input`: a mapping links inputs only when it can name the records). A block is atomic: if validation, an ambiguous or unknown subject or a
  value that fits no scheme refuses anything in it, none of its records is written and its rows end
  `held` with the reason. A refusal about identity (section 2's competing assertions) is not held: it
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
family row and may itself hold nested sets. For a parameter set it stores the subjects in the
canonical orientation (the order the DDL checks), applies the swap to the values (`reciprocal` slots
invert, the slots of a `linear` rule are multiplied by its matrix, `parity` family rows of odd index
change sign; `thermo_knowledge.transposition` holds the one implementation the evaluator shares) and writes the set, slot-group and family
rows. Provenance rows follow from the carrier (the source manifest, the lock entry and the staged
manifest) and each origin: `source` and `carrier` (key `<manifest id>@<pin>`), `artifact` (hash and
size of the file), `import_record`, a `licence` where a rights entry gives an SPDX identifier (keyed
and titled by it), a `rights_determination` per rights entry (ordinal counted per scope), and a
`record_origin` per origin. A text key that restates a reference (`site_class.host_key`,
`association_site.carrier_key`) is computed by the writer from the reference the mapping gives, and a
mapping that gives the key is refused.

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
| `species.canonical_key`, provisional entity | `provisional:` followed by the canonical encoding of [carrier key, scope, local key]; a carrier key is `<manifest id>@<resolved pin>` |
| `species_form.canonical_key` | the canonical encoding of [species key, aggregation name], with the polymorph name appended when the entity states one; a provisional species gives its own key |
| `parameter_set.subject_key`, top-level set | the canonical encoding of the subject identifiers in the slot group's role order, in the canonical orientation; `[]` for a group with no subject |
| `parameter_set.subject_key`, nested set | the canonical encoding of [parent set identifier, `parent_slot` identifier, index key] |
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
- a row a block refused is `held`, with the reason; a block refuses a row whose subject is `ambiguous`
  (an `unresolved`, `rejected` or `unique` subject loads: the records attach to its entity, which is
  provisional unless unique);
- every other row of a mapped table or partition is `unmapped`.

Phase 1 writes a ledger of the rows it emitted from or held, and phase 2 merges it with its own, so a
row mapped in either phase is covered. The counts are written to `qual.mapping_coverage`, each held or
unmapped row to `qual.held_row`.

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
- reactions conserve every conserved quantity with a composition entry on a participant (the declared
  invariant `reaction.conserves_declared_quantities`, enforced by one check);
- nested sets implement the contract their slot accepts;
- family indices are contiguous from their minimum;
- derivation lineage and dependency relations are acyclic;
- a record presented in a role with the facet `requires_derivation` (`origin_role` marks `fitted`,
  `estimated` and `derived`) has a producing derivation (a derivation is exempt), and when the role has
  the facet `requires_fit` (`fitted`) that derivation is a fit. The roles are read from `meta`, so a
  member given the facet is covered without editing the checks; a record whose origins present it in
  different roles is covered when any of them has the facet;
- every `parameter_set.subject_key` is the canonical encoding of the subjects of its slot-group row
  (section 2.3).

The declared invariants of relations are checked the same way as those of kinds: a relation's
`requires` with `enforced = "verify"` has a check file named `<relation>.<requirement>`, and the
verify stage treats both alike.

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
  argument values (within each set's envelope), the comparison basis, the tolerance, and the
  harness.
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
points = 25                                 # `points` spread evenly within each set's envelope ...
within = "envelope"
observable = "temperature"                  #   on this observable
envelope_kind = "fitted_range"              #   optional: when a set has several envelopes
inset = 0.0                                 #   optional: a fraction of the range left out at each end
# values = [298.15, 350.0]                  # ... or explicit `values`, in `unit`
# unit = "K"

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
- **Points.** `points` spread evenly over the envelope of the set on the observable, from the lower
  bound plus the inset to the upper bound minus it (one point is the midpoint); a set with no envelope
  of that kind, several, or one open at an end blocks the run. Several arguments give the Cartesian
  product, the last varying fastest. Values are in the storage units of the argument's type.
- **A point agrees** when its relative deviation from the library, `|form - library| / |library|`, is
  within `relative_tolerance`, or its absolute deviation within `absolute_tolerance` when one is
  given; a form value that is not finite never agrees. A point the library answers with NaN or an
  error is *invalid*: it is counted and listed in the report, and `invalid_points` says whether it
  fails the run or is excluded for the reason stated. A form the evaluator refuses for a subject (a
  value outside its domain) fails the subject's points, with the refusal in the report.
- **Outcome.** `passed` when every compared point agrees, `failed` otherwise, `blocked` when the run
  could not be carried out (no such parameterization or subject, a grid that cannot be made, a harness
  that is missing, cannot run or answers against the protocol, no point compared); the reason is the
  run's `note`. A blocked run is recorded too and is never reused.

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
  (the case's `subforms`); a slot with none has no choices.
- **Orientation.** A transposable subject tuple is looked up exactly as asked, in the stored
  canonical orientation; the evaluator tries the others.
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
counts, worst deviation and where, and any refusal, with every invalid point.

A run is one `qual.qualification_run` row, with a `qual.run_parameter_set` row for each set
evaluated:

| Column | Holds |
|---|---|
| `key` | `<case>/<form>/<parameterization key>@<revision>/<first 16 characters of the declaration fingerprint>` |
| `form`, `basis`, `outcome`, `note` | the form, the comparison basis, `passed`, `failed` or `blocked`, and the reason or what was excluded |
| `expression_hash` | the evaluation hash of the compared output when the run was made (what `qual.form_qualification` compares with the current one); every run has one, a blocked run included |
| `reference` | the `software_release` of the library version the harness reported |
| `points` | the points compared (those that failed the run by being invalid included) |
| `relative_tolerance`, `absolute_tolerance`, `observable` | as declared; the observable is the contract output's |
| `worst_relative_deviation` | the largest relative deviation seen, where it is finite |

**Persistence.** The rows are written as canonical Parquet through the canonical writer under
`<store>/canonical/_qualification/<case>/` (`qual.qualification_run`, `qual.run_parameter_set`, the
release's `prov.source` and `prov.software_release`, `manifest.json` of phase `qualification`, and
the run's `report.json`), so they survive a rebuild that drops schema `qual`. The manifest's reuse
key covers the declaration fingerprint, the evaluation hash of the compared output, the case file's
content, the library and its version, the subjects with the identifiers of the sets evaluated, and a
digest of what the database holds for those sets and their envelopes (an identifier does not change
with a value, so the identifiers alone would keep a stale result). A case whose key is unchanged is
reported `current` and not run again (a blocked run always is), and `--force` runs it again.
`tk build` loads every output that is current: it leaves out, and reports with the reason, an output
made against another declaration fingerprint (run `tk qualify <case>` again) and one whose parameter
sets are not in the build. `tk qualify` also loads the run into the live database at once, by the
build's load of these tables in one transaction that replaces the rows of that case only (the runs
whose key starts with `<case>/`) and keeps a release row already there.

