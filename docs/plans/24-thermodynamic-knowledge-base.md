---
title: Thermodynamic knowledge base — target domain model, source procurement and consolidation pipeline
status: in-progress
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_thermo-knowledge-core_2026-09-30.md]
scenario_sources: []
---

# Thermodynamic knowledge base

Authorized by the maintainer on 2026-09-30. This plan records intended design and execution
state for a standalone workstream; it is not a description of the production system and
changes no production contract.

## Context

Plan 23 gives production a typed, relational thermodynamic domain model in `.pse` packages. That
schema is deliberately small, and several of its choices would become dead ends at wider scope.
Before production grows into that scope, the maintainer wants the target state worked out and
proven against real material:

1. procure every Tier A and Tier B source listed in
   `thermodynamics_complete_conversation_handoff/thermodynamics_comprehensive_resource_index.md`
   and build the infrastructure to read them;
2. derive one unified domain model that holds what those sources represent in very different ways;
3. consolidate all of them into one PostgreSQL database whose structure is generated from that model.

The workstream lives in `thermo-knowledge/`, inside the repository but standalone: it touches no
production crate, package or relation and imports none of them. Its outputs (ontology, schemas, a
debugged intake pipeline, a populated database) wait until production is ready to adopt them. Its
second product is early warning: every place the current `.pse` language or schema cannot express
the target becomes a recorded gap.

## Decisions

| Question | Decision (maintainer, 2026-09-30) |
|---|---|
| Location | In this repository, self-contained tree, no coupling to production code |
| Authority for the model | A neutral declaration that uses `.pse`'s concepts (kind, refinement, key, relation, envelope, provenance role, function form) and may exceed today's kernel. PostgreSQL DDL, loader types, docs and a `.pse` rendering are generated from it |
| Equations | Staged. Every form is catalogued with a typed signature and all data is loaded; a form becomes *qualified* when its canonical equation reproduces the source library's numbers |
| Extra scope | Transport properties, reaction kinetics, an algorithm and capability inventory, and the FeOS and IDAES data the repository already pins |
| Licences | Recorded as rights metadata on every imported record; never a reason to skip a source. One terms-of-use exception: DDBST (below) |

No ADR is required now: this is tooling and adds no crate, relation family or contract. Adoption
by production will need one. The kernel-gap register and schema-delta report are its inputs; they
bear on ADR-0127's revisit trigger and register rows R-49 and R-50.

## Architectural drivers and scenarios

### Scope boundary

The knowledge base covers **what is known**: material identity, phases and systems, conventions,
observables, model forms, parameters, reactions, evidence, derivations, provenance and rights
(handoff dictionary B6 owners P01–P03, P06 and the evidence parts of P02 and P09). It does not
cover what is being computed at a process location: states, calculation problems, provider
execution, result publication (B6 P04, P05, P07–P10). Algorithms are inventoried as capabilities,
not ported.

The database is **open-world**: competing assertions, unresolved identities and restricted rights
coexist. A production package is **closed-world**: complete, with one selected value. The bridge
is a named export, the *resolved snapshot*, not a shared set of invariants.

"Without loss of meaning" is claimed for records and declarative structure, not for behaviour
that lives only in code (density functionals, collision integrals, PHREEQC BASIC rate blocks,
structure matching, characterisation procedures). Those are catalogued with their contract and a
typed completeness state.

### Responsibilities

| Responsibility | Owner in the tree | Consumes |
|---|---|---|
| What a source is and what was acquired | `sources/*.toml`, `sources.lock` | nothing |
| Source-faithful reading | `readers/<id>` | raw store |
| The domain model | `model/`, `forms/` | nothing; every projection derives from it |
| Identity decisions | `identity/` plus rule-derived resolution | source-faithful identity assertions |
| Source-to-canonical meaning | `mappings/<id>/` | the declaration, source-faithful tables |
| Physical database | generator output + `sql/physical.sql` | the declaration |
| Evidence that meaning survived | `verify`, `qualify` | database, oracle harnesses |

### Representative changes (design principles §E)

| Stimulus | Where it must be expressed | What must not change |
|---|---|---|
| A new residual Helmholtz term kind | one form implementing the existing term contract, plus rows | pure-fluid assembly, selection, provenance |
| A new published UNIFAC matrix with two new subgroups | one parameterization, two group rows and assignments under a new scheme revision | the UNIFAC form; other matrices |
| A mixing rule embedding an activity model not yet supported | one form implementing the mixing contract, and a named assembly | existing activity sets |
| A TDB with a four-sublattice ordered phase | phase and site-class rows; one contribution form on the constituent-array signature | the compound-energy assembly |
| A refit of a binary parameter from ThermoML data, with covariance | one fit, sets with role fitted, one selection-policy override | the previous sets, which remain |
| A carrier changes licence | one rights determination on the carrier | every mathematical record |
| A new source in an existing format family | one manifest, one mapping spec | readers of other sources, the declaration |

### Hard cases the model is tested against

Fourteen cases, each written in TK2 as a small hand-made canonical fixture that must load and
satisfy every invariant: (a) CoolProp/teqp Helmholtz fluid and a GERG-2008 binary; (b) a cubic
assembled from core, alpha function, excess-Gibbs mixing rule and volume translation; (c) SAFT
association sites and SAFT-gamma-Mie groups; (d) UNIFAC matrices and RMG Benson trees; (e) a
CALPHAD sublattice phase with piecewise parameters; (f) PHREEQC master and secondary species,
Pitzer and SIT, exchange and surface species, and ThermoFun HKF; (g) NASA-7/9, Shomate and JANAF
tables; (h) IAPWS-IF97 regions, backward equations and verification tables; (i) petroleum
pseudo-components from characterisation; (j) adsorption isotherm models, IAST and ISODB data;
(k) sigma profiles and polymer models; (l) density functionals and integral-equation inputs;
(m) kinetic-theory transport; (n) ThermoML measurements, ATcT evaluated values and ESPEI fit
lineage.

## Design

### Layout

```
thermo-knowledge/
  justfile                  standalone command surface (tk-* recipes)
  sources/<id>.toml         one declaration per source: URL, pin, licence, terms, payload globs, reader, environment
  sources.lock              what actually resolved: commit or DOI, per-file sha256, retrieval time
  model/                    the authoritative declaration (core modules)
  forms/                    form catalogue: contracts, forms, slot groups, expressions (data, not code)
  mappings/<id>/            per-source mapping specs: source construct -> canonical concept, unit, convention, declared loss
  identity/                 curated identity decisions (the only hand-maintained resolution authority)
  survey/                   per-source construct inventories and the capability inventory
  src/thermo_knowledge/     acquire, readers, resolve, mapping, generate (ddl, meta, types, docs, pse), verify, qualify
  sql/generated/            generated DDL, committed and diff-checked; sql/physical.sql is hand-written indexes
  envs/                     locked environments (uv projects and micromamba specs)
  tests/
  .store/                   gitignored: raw/ (untouched sources), staged/ (source-faithful Parquet), canonical/ (Parquet per table)
```

Committed: declarations, manifests, lock, mappings, code, generated SQL, identity decisions,
survey records, reports. **No third-party data is ever committed** (raw store, staged or canonical
Parquet, database dumps). Database: `pse_thermo` on the existing PostgreSQL 18 server, never `pse`.

### Procurement and reading

Acquisition follows the declare, acquire, record-resolved, verify split used by the shared
library-skills store. `tk-acquire` is the only networked stage. It writes an untouched copy to
`.store/raw/<id>/<resolved-pin>/` and records the resolved commit or DOI, per-file sha256 and
retrieval time in `sources.lock`; later stages refuse a store that disagrees with the lock.

Readers and oracles are separate. A reader turns a payload into source-faithful Parquet: source
vocabulary, source units, source keys, one locator per record. Most payloads are plain JSON, CSV,
TSV or YAML and are read in the core environment without the library. A library is the reader
only where the format is genuinely complex. An *oracle harness* is the installed library
answering "evaluate this model at these points"; it exists only for qualification. Readers and
oracles exchange Arrow or Parquet files, so isolated environments never leak into each other.

Environments (from declared package metadata; TK0 runs the resolvers and may split or merge):

| Environment | Tool | Hosts |
|---|---|---|
| core, Python 3.12 | uv | pipeline, plain-format readers; CoolProp, teqp, thermo, chemicals, PhasePy, Cantera, `cea`, PolyKin, flory, pyPRISM (git), pyGAPS, pycalphad, ESPEI, `neqsim` (JPype), FeOS |
| thermotools, Python 3.11, numpy < 2 | uv | thermopack, pykingas, surfpack |
| geochem | micromamba (conda-forge) | reaktoro 2.13, thermofun, gems3k, optional `phreeqcrm` |
| rmg, Python 3.11 | micromamba (conda-forge + rmg) | RMG-Py loader and oracle |
| Julia >= 1.10 | juliaup (not installed on this host yet) | Clapeyron oracle only; its CSVs are read plainly |
| JVM 21, gfortran (both present) | host | NeqSim jar; Thermochimica build |

Sources, pins and readers. Pins and payloads come from reconnaissance on 2026-09-30; the manifest
and lock, once written, are the authority and this table is not maintained against them.

| Source | Pin | Payload | Reader | Wave |
|---|---|---|---|---|
| chemicals | commit `7fca9acf` (v1.5.2; tag diverged from master) | 121 TSV, JSON, ChemSep XML; 69 MB; handbook-derived | plain TSV and JSON, cross-checked against `chemicals.data_reader` | 1 |
| thermo | commit `4f51c434` (v0.6.1) | 53 files, 17 MB; UNIFAC tables; ChemSep IPDB; group definitions embedded in `unifac.py` | plain files plus imported module tables | 1, 2, 3 |
| Cantera | tag v3.2.0 and the example-data submodule SHA | 24 YAML (NASA gas and condensed, mechanisms) | plain YAML; `ck2yaml` for CK inputs | 1, 7 |
| NASA CEA | tag v3.3.4 (`nasa/cea`, a 2025 rewrite) | `thermo.inp`, `trans.inp`, fixed format | `cea` library, cross-checked against Reaktoro `NasaDatabase` | 1, 7 |
| NIST-JANAF | retrieval date and per-file sha256 | about 1,800 tab-delimited tables | small parser; fetched sequentially with a delay | 1 |
| ATcT | TN version 1.222 page and sha256 | one HTML table, 3,444 species | HTML table parse; provenance pages only if needed | 1 |
| FeOS parameters | the commit cited in `crates/pse-kernels/data/README.md`, extended to the full `parameters/` tree | JSON | plain JSON | 1, 2 |
| IDAES 2.13.0 | the existing `external/idaes-pse` checkout | property parameter dictionaries in source | walk configuration dictionaries; values only, role `oracle_input` | 1, 2 |
| CoolProp | tag v8.0.0 | 138 fluid JSON, mixtures, cubics, PC-SAFT, incompressibles | plain JSON | 2, 7 |
| teqp | tag v0.23.2 | CoolProp-format JSON; GERG and SAFT tables in headers | plain JSON; embedded tables through the library | 2 |
| NIST AGA8/GERG | commit `3bdb9ab8` | constants in `GERG2008`, `DETAIL`, `GROSS` sources; 16-digit test values | extract literal assignments; verified against CoolProp's Kunz–Wagner tables and the test values | 2 |
| IAPWS | document identifier and sha256 per PDF (43 documents) | PDF coefficient and verification tables | coefficients from CoolProp and IF97 sources, accepted only if they reproduce the PDF check tables | 2 |
| Clapeyron.jl | tag v0.6.29 | 190 CSV (three-line header; suffix encodes like, unlike, assoc, groups), 42 JSON | plain CSV | 2, 3, 4 |
| ThermoPack | commit `ca75d8e0` (the v2.2.4 tag was moved) | 109 fluid and 12 binary JSON | plain JSON | 2 |
| NeqSim | tag v3.23.0, shallow fetch or sources jar | `COMP.csv`, `INTER.csv`, UNIFAC, Pitzer, reaction CSVs | plain CSV | 2, 3, 4, 8 |
| KineticGas | tag v2.3.0 | 30 fluid JSON, phase-shift tables | plain JSON | 2, 7 |
| PhasePy | tag v0.0.56 | four UNIFAC workbooks | `pandas.read_excel` | 3, 8 |
| COSMO-SAC profiles | Zenodo DOI 10.5281/zenodo.20672192 with md5; `usnistgov/COSMOSAC` commit `1b82456b` | 361,583 PySCF profiles, UD 2,261, VT-2005 1,432 | Parquet; small `.sigma` parser | 3 |
| RMG-Py and RMG-database | tags 4.0.0 | Python-syntax `entry(...)` files: groups, libraries, kinetics, solvation, transport | `RMGDatabase.load` in the rmg environment | 3, 7 |
| PHREEQC 3 | release archive 3.9.0 (`phreeqc-dev/phreeqc3`) | about 22 `.dat` databases plus third-party sets; keyword blocks | own keyword-block parser, because no library exposes Pitzer, SIT, exchange and surface blocks as data; species and log K cross-checked against Reaktoro `PhreeqcDatabase` | 4 |
| Reaktoro | tag v2.13.0 | embedded PHREEQC, NASA, SUPCRT, ThermoFun databases; Pitzer and extended UNIQUAC parameters | plain YAML and JSON; its copies of other sources are reconciled by equivalence assessment | 4 |
| ThermoFun and ThermoHub | v0.6.3; hub commit after v1.2.0 | nine JSON databases; ArangoDB export | plain JSON | 4 |
| GEMS3K | tag v4.6.1 | example input sets only | plain; the value is the solution-model inventory | 4 |
| pycalphad | tag 0.11.2 (branch `develop`) | 56 TDB and ChemSage DAT examples | `pycalphad.Database` | 5 |
| SGTE unary | `unary50.tdb` sha256 (v5.0, 2009) | one TDB | `pycalphad.Database` | 5 |
| ESPEI and ESPEI-datasets | tag 0.9.1; commit `e3b53642` | `refdata.py`; 230 dataset JSON | plain JSON, cross-checked with `load_datasets` | 5 |
| Thermochimica | commit `0c35c8d7` | 27 ChemSage DAT | pycalphad `cs_dat`; own parser as fallback | 5 |
| ThermoML | `ThermoML.v2020-09-30.tgz`, DOI 10.18434/mds2-2422, published sha256; `ThermoML.xsd` | about 11.9 thousand article files, XML and JSON | bindings generated from the XSD with `xsdata`; XML validated against the XSD | 1 (schema), 6 (data) |
| NIST ISODB | mirror `NIST-ISODB/isodb-library` commit `9e1ee472` | isotherm JSON in 3,830 DOI folders | plain JSON | 6 |
| NIST WebBook | retrieval date and per-page sha256 | per-record HTML | scoped scraper: only species and properties needed for cross-checks, at least 5 s between requests | 6 |
| pyGAPS | tag v4.6.1 | SQLite, adsorbate JSON, example isotherms | sqlite3, plain | 8 |
| PolyKin, flory, SurfPack, pyPRISM | tags v0.8.0, 0.3.1, v1.1.0, v2.0.0 | almost no data | form inventory only | 8 |

Handling that differs from a straight copy:

- **DDBST published UNIFAC pages are not retrieved.** The site's terms prohibit saving its pages
  or compiling them into a database without written permission. The same tables arrive through
  `thermo`, Clapeyron, PhasePy and NeqSim, attributed to the primary papers. DDBST is recorded as
  a source with status "not acquired; permission route open".
- **ISODB** is taken from its official GitHub mirror, because robots.txt disallows the API.
- **WebBook, JANAF, ATcT and IAPWS** have no bulk download: sequential, rate-limited,
  identity-scoped retrieval, with the raw responses kept as the snapshot.
- **Conflicting licence statements** (SurfPack GPL-3.0 file against MIT metadata; ThermoFun
  LGPL-2.1 file against an LGPL-3.0 recipe; JANAF and ISODB site copyright against the
  data.nist.gov open licence) are recorded verbatim as competing rights determinations.
- **IDAES** values enter only the local database with role `oracle_input`; no IDAES code or text
  is copied (`docs/relationship-to-idaes.md`).
- The remote-account configuration committed in the ThermoFun repository is not used.

### Deriving the model

The model is derived from evidence in a loop:

1. **Source-faithful capture first.** Each source is loaded as-is into its own `src_<id>` schema,
   so the corpus is queryable before canonical modelling is final and every canonical record links
   to the source record it came from.
2. **Construct inventory.** One structured record per native construct of each source: meaning,
   units, conventions, cardinality, examples. Breadth-first over all 36 sources before depth on any.
3. **Alignment.** Each construct maps to a candidate concept with a typed precision (exact,
   narrower, broader, close) and its declared loss. Constructs with no home, and concepts forced
   to carry two conventions, are the signals to add, split or qualify a concept.
4. **Declare**, then regenerate DDL and types.
5. **Prove by loading.** Every source-faithful record ends as *mapped* (exactly, or with declared
   loss), *out of scope* (with a reason) or *unmapped*. Unmapped residue drives the next
   iteration; a wave closes when its residue is explained.
6. **Prove by evaluating.** Qualification shows that units, sign, ordering and conventions
   survived the mapping.
7. **Review** with the `design-review` skill and the process-simulator profile at two bounded
   checkpoints: core model v0 (TK2) and consolidation (TK9).

Concept set, v0 starting point:

| Area | Concepts |
|---|---|
| Material identity | `element`, `conserved_quantity`, `material_entity` (abstract) refined by `species`, `species_form` (species, aggregation, polymorph), `pseudo_component`, `polymer_type`, `defined_mixture`, `material`; `sample`; `identity_assertion`; `group_scheme`, `group`, `group_assignment`; `association_site`; `distributed_attribute` |
| Systems and phases | `aggregation`, `chemical_system` (with a basis-species role), `phase_definition`, `site_class` (sublattice, cage, surface site, exchanger), `composition_basis`, `speciation_map` (apparent to true) |
| Conventions | `standard_state`, `energy_reference`, `element_reference`, `convention_set` (standard states, energy reference, gas constant, temperature scale, atomic-weight edition) |
| Observables | `observable` with typed facets: subject shape, basis, relation to reference, path, adsorption kind, diffusion frame |
| Forms | `contract`, `form`, `slot_group`, `slot`, `subform_slot`, `model_assembly`, `auxiliary_relation` |
| Parameters | `parameter_set`, `parameterization`, `dependency`, `envelope`, `selection_policy`, `resolved_snapshot` |
| Reactions | `reaction` over `species_form` participants, each with its standard state; equilibrium-constant and rate-law models are forms whose subject is a reaction |
| Evidence and derivation | `dataset`, `data_point`, `derivation` (fit, estimation, characterisation, computation, evaluation, transcription, conversion), `fit` |
| Provenance and rights | `source` and `database_release`, `import_record` (carrier and locator, separate from primary attribution), `licence`, `rights_determination`, `equivalence_assessment` |

Structural decisions the rest depends on:

- A parameter set is keyed by (parameterization, slot group, subject tuple). One source's coherent
  values are never split; competing sets for one subject coexist.
- Subjects are declared role tuples, not a closed list of shapes. The closed vocabulary is the
  set of transposition rules: symmetric, ordered, parity by index, reciprocal, permutation group.
- "Binary interaction parameter" is not a concept. Each is a slot of a specific form, and values
  are never interchangeable across forms.
- Value state is a type, separate from origin: `known` (which includes a known zero),
  `not_applicable`, `redirect`, `withheld`. Missing is the absence of a set. An estimated or
  rule-derived value is a known value whose derivation names the rule.
- Five value shapes: dimensioned scalar, enumeration member, reference, nested set of a sub-form,
  tabulated function over typed axes.
- Conventions belong to the parameterization; there are no global gas constant, standard
  pressure or energy datum.
- Forms compose through typed slots that accept another form by contract. Term lists are an
  additive slot with multiplicity many; regional formulations are (predicate, form) pairs;
  implicit forms declare unknowns, residuals and a root-selection rule.
- `dependency` is first-class: a value fitted under a prerequisite is valid only with it.
- Every form carries a completeness state: `fully_declared`,
  `structure_declared_equation_external` or `opaque_bundle`.
- One import record per carrier. The same published table carried by three libraries is three
  import records and a derived equivalence assessment; rights attach to the import record.

Forms and equations. A form's expression is authored in the declaration as a typed expression
over dimensioned literals, slot references, arithmetic and elementary functions, finite sums and
products over declared index sets, conditionals, local bindings, calls through a sub-form slot,
partial derivatives and implicit blocks. These are the constructs of the `.pse` function
language, so rendering to `.pse` is mechanical. The authoring syntax is a restricted Python
expression parsed by the standard `ast` module into that typed tree. Backends are a SymPy and
NumPy evaluator for qualification, a `.pse` renderer, and LaTeX for the generated reference. Four
constructs the hard cases need and `.pse` lacks today are included and logged as kernel gaps:
piece selection by interval lookup, a call with remapped arguments, a definite integral in one
scalar, and a short list of special functions.

Feedback to production is two generated reports: the **kernel-gap register** (each declaration
construct that does not render to valid `.pse`, with the hard case that needs it) and the
**schema delta** (current kinds that survive, that must generalise, and that are dead ends).

### Database and pipeline

| Stage | Input and output | Contract |
|---|---|---|
| `tk-acquire` | manifest to `.store/raw` and `sources.lock` | the only networked stage; untouched copy; hashes recorded |
| `tk-read` | raw to `.store/staged/<id>/*.parquet` and a manifest | source-faithful; every record has a locator; unread files listed with a reason |
| `tk-load-src` | staged to `src_<id>` schemas | bulk COPY from Arrow |
| `tk-resolve` | identity assertions to resolutions | rule-derived matches recomputed; curated decisions read from `identity/`; ambiguity preserved |
| `tk-map` | `src_<id>` and mapping spec to `.store/canonical/*.parquet` | each mapping states what it preserves, loses and assumes; unit conversion is a recorded transformation |
| `tk-build` | DDL and canonical Parquet to a fresh database, then swap | the database is a derived artifact rebuilt from lock, declaration and mappings; no migrations; schema fingerprint recorded |
| `tk-verify` | database to reports | constraints, semantic invariants, round trips, cross-source agreement |
| `tk-qualify` | forms and oracle harnesses to qualification records | differential evaluation within declared envelopes and tolerances |
| `tk-project` | database to `.pse` rendering, snapshot export, coverage and gap reports | read-only projections |

Reuse is keyed on the inputs of each stage: resolved pin, reader version, declaration
fingerprint, mapping version. The keys do not yet cover provider versions, framework code,
harness scripts or environment locks (core review F08).

| Schema | Holds |
|---|---|
| `meta` | the declaration reified as rows, so the database describes itself |
| `prov` | sources, releases, artifacts, import records, licences, rights determinations, derivations, ingest runs |
| `src_<id>` | source-faithful tables, one schema per source |
| `tk` | canonical entities, systems, conventions, observables, forms, parameterizations and the per-slot-group tables |
| `ev` | datasets and data points |
| `qual` | qualification runs, residuals, equivalence assessments, capability inventory |
| `api` | typed read views |

Projection rules, applied by one generator that returns a deterministic tree and has a `--check`
mode:

- Kind to table; refinement to a child table whose primary key is a foreign key to its parent.
- Slot group to one table: a typed foreign-key column per subject role, a column per slot, and
  index keys for term, interval or order children. The table corresponds one-to-one to a `.pse`
  dataset and to a Parquet data document.
- Quantity type to a PostgreSQL domain over `double precision` in the declared storage unit, with
  a finiteness check. Original values and units stay in the linked source-faithful row.
- Required slots are `NOT NULL`; a slot whose presence policy admits other states gets a typed
  state column; uncertainty is a typed side table; function-valued slots are foreign keys to
  nested sets.
- Transposable groups store one row per unordered subject with the values as the source
  asserted them and the orientation they were asserted for; the transposition rule is applied on
  reading (core review F03).
- Envelopes and piece intervals are range columns; non-overlap of pieces is an exclusion constraint.
- Closed vocabularies the pipeline acts on are enum types; scientific vocabularies are tables.
- JSONB retains only uninterpreted source text; nothing decision-relevant lives in it.
- Identifiers are deterministic functions of declared semantic keys, so a clean rebuild
  reproduces them exactly.

`tk-project snapshot` applies a selection policy to a model assembly and an entity set and emits
a closed-world package: Parquet data documents plus dataset declarations with provenance,
refusing any record whose rights forbid the use or whose dependencies are unsatisfied.

### Qualification

Form status moves `catalogued`, then `expressed` (expression present and dimensionally closed),
then `qualified` (reproduces at least one source library within a declared tolerance over a
declared domain). Checks that do not share the library's code rank above library agreement: the
IAPWS verification tables, the AGA8/GERG 16-digit test values, JANAF tables against NASA and
Shomate fits, and the same published table carried by two libraries. First target: at least one
qualified form in each of the 16 mathematical representation classes. The index lists 18;
constrained Gibbs minimisation and phase stability are problems, not representations, and appear
in the capability inventory.

## Plan

Each wave runs readers, source load, mapping, canonical load, verify, qualify and a residue
report, and adds that wave's libraries to the capability inventory.

| Packet | Responsibility / dependencies | Scenarios / acceptance | Replaced code / deletion | Status or status-owner link |
|---|---|---|---|---|
| TK0a Scaffold | tree, core environment and lock, standalone justfile, `pse_thermo` provisioning, stage CLI skeleton, disposable test database; shared-config touchpoints | smoke tests: CLI help, configuration resolves from the repository root, test database is created and dropped | none | done |
| TK0b Meta-model and generator | declaration meta-model, loader and validation; generator for DDL, `meta` rows and loader types with `--check`. Depends on TK0a | a minimal declaration generates DDL that builds in a test database; `--check` detects drift; invalid declarations are refused with a located error | none | done |
| TK0c Expression engine | typed expression tree, authoring syntax parsed with the standard `ast` module, dimensional closure, SymPy and NumPy evaluator, `.pse` and LaTeX renderers. Depends on TK0b | each construct parses, type-checks and evaluates; an ill-dimensioned expression is refused with a located error | none | done; the `.pse` and LaTeX renderers move to TK9 |
| TK0d Implicit blocks and set-valued sub-form calls | implicit unknowns and residuals with root selection and derivatives through the block; passing sets and composition vectors through a sub-form slot. Depends on TK0c | cubic volume roots, association site fractions and a Huron–Vidal rule calling NRTL agree with independent calculations | the not-yet-supported refusals | done |
| TK0e Side environments | locked thermotools, geochem and rmg environments with smoke tests. Depends on TK0a | each environment recreates from its lock and its smoke test makes one real call per library | none | done |
| TK1a Acquisition | manifest schema, acquisition for git, archive, single-file and scoped-web kinds, lock, store verification. Depends on TK0a | unit tests against local fixture repositories and files; a tampered store is refused | none | done |
| TK1b Procure | all 36 manifests and rights determinations; acquisition in reconnaissance order. Depends on TK1a | every source is in the lock with verified hashes; DDBST recorded as not acquired | none | in progress: 38 of 42 manifests acquired; JANAF stopped at 1,273 of 1,796 tables (resumable); ThermoML archive and ATcT open (see Open items) |
| TK3a Reader framework | `tk read` and `tk load-src`: source-faithful staged Parquet with a manifest that accounts for every payload file; in-process and side-environment readers; first reader CoolProp. Depends on TK1a | reader row counts equal independent counts; a round trip reproduces the source arrays; an unaccounted payload file is refused | none | done |
| TK3b Resolution and mapping framework | source entities, identity assertions and the resolution rules; the canonical writer with identifiers, provenance, units, validation and coverage; first mapping CoolProp identities and saturation ancillaries. Depends on TK0b, TK3a | each resolution rule on a fixture corpus; resolution is order-independent and deterministic; canonical Parquet loads with every constraint satisfied | none | done |
| TK3c Build and verify | `tk build` from every source's canonical Parquet with swap; `tk verify` and its named checks. Depends on TK3b | a second build reproduces every identifier; each verify check has a violating fixture | the schema-only build path and its tests (deleted) | done; first real build and verify of the CoolProp slice in progress |
| TK3d Qualification | database-backed parameter source, oracle harness protocol, `tk qualify`; first case the CoolProp saturation ancillary against CoolProp. Depends on TK3c | a passing run is recorded; a perturbed parameter fails | none | done |
| TK0f Evaluator passes parameters as arguments | stored values enter the compiled function as arguments, so it performs the formula's floating-point operations as written; one compilation per structure. Found by the first qualification run | a formula evaluated one unit in the last place below its reducing temperature agrees with a direct evaluation; equal structures share a compilation | value substitution into the symbolic expression | done |
| TK2a Alignment mechanisms | occurrence in a parameter set's identity; multi-axis, multi-series tabulated functions; a linear transposition rule; an output's observable given by a slot; qualification derived from recorded runs instead of a declared status. Depends on TK3d | each mechanism's tests; the slice rebuilds, verifies and qualifies | the declared `qualified` status | in progress |
| TK2 Survey and core model v0 | construct inventory for every source; core modules declared; the 14 hard-case fixtures; design review. Depends on TK0b, TK1b | fixtures load and satisfy every invariant; review findings dispositioned below | none | in progress: all 37 survey records written; `thermo-knowledge/docs/alignment-notes.md` lists the model changes identified so far, most of which are in `model/` (items 8, 10 and 15 are not), and the survey losses are not yet dispositioned construct by construct (TK2e); design review done, verdict Revise; the hard-case fixtures follow the revisions TK2b–TK2d |
| TK2b Model revision: mechanisms | core review F02, F03, F05, F06, F07, F10, F11, F12, F15, F16: referenced sets, values stored as asserted, standard states by member role, convention inputs, relation invariants, a checked pipeline contract, typed covariance and basis, enum member facets. Depends on TK2a | the verification named in each finding's disposition row | value-rewriting canonicalisation in the writer; the per-form gas-constant slot groups | not started |
| TK2c Pipeline contracts | core review F01, F04, F08, F16, F17, F18, F19: qualification currency decided on content, entity classes in resolution, one reuse-key builder, publications resolved across carriers, typed reasons, rule-use accounting. Depends on TK2b | as above | per-stage key code | not started |
| TK2d Envelopes and evidence | core review F09, F13: validity regions with clauses and a coverage state bound to contract arguments; uncertainty assessments. Depends on TK2b | as above | the single-interval envelope | not started |
| TK2e Survey dispositions | core review F14: a survey loader, one disposition per construct with a stated loss, the residue report. Depends on TK2b–TK2d for the dispositions themselves | the report lists no construct without a disposition | none | not started |
| W1 | pure-component data and standard-state thermochemistry; ThermoML schema to the observable vocabulary | residue explained; representative forms qualified | none | not started |
| W2 | equations of state: Helmholtz and multifluid, cubic, SAFT, GERG, IAPWS | same; IAPWS and AGA8/GERG verification values reproduced from canonical records | none | not started |
| W3 | activity models, group contribution, sigma profiles, RMG thermo groups | same | none | not started |
| W4 | aqueous, electrolyte, reaction equilibrium, standard-state species models | same; reactions conserve elements and charge | none | not started |
| W5 | CALPHAD, solids, fit lineage | same | none | not started |
| W6 | evidence at scale: ThermoML, ISODB, scoped WebBook | same | none | not started |
| W7 | transport and reaction kinetics | same | none | not started |
| W8 | petroleum characterisation, polymers, adsorption models, interfaces, liquid-state theory | form inventory complete; residue explained | none | not started |
| TK9 Consolidate and qualify | clean rebuild from the lock; coverage, kernel-gap and schema-delta reports; snapshot export reproducing the repository's current seed; formatting, lint and the full check set; final design review | the checks under Verification | none | not started |

## Finding dispositions

This table owns adopted finding status; link packet evidence instead of copying execution reports.

| Finding reference | Scenario reference | Disposition | Decision / work owner | Evidence or revisit trigger |
|---|---|---|---|---|
| core review F01: a stale qualification pass can be loaded by the build | S08 | scheduled | TK2c | a changed mapped value, rebuilt without re-qualifying, leaves no current run |
| core review F02: a parameter set shared by several parents cannot be represented | S06, C1, C2 | scheduled | TK2b: a slot that references an independently identified set by contract; a named model component as its subject; a per-subject sub-form choice | fixtures (a) and (e): one departure set referenced by several pairs and evaluated through them; one named function referenced by two parameters |
| core review F03: canonical orientation rewrites asserted values | S07, C1, C2 | scheduled | TK2b: values are stored as asserted with an orientation state, and the rule is applied on reading; a constituent array keeps its asserted order as part of its identity | a swapped reciprocal pair reads back the source number bitwise; an identity decision changes no stored slot value |
| core review F04: resolution yields only species | C0 | scheduled | TK2c: a scope declares the class of its entities and each class has its rules | CoolProp `Air` resolves to a defined mixture of resolved species; no provisional species for a pseudo-pure fluid |
| core review F05: standard states keyed by aggregation | C3 | scheduled | TK2b: standard states keyed by the role a member plays in its chemical system | fixture (f): pure-liquid water and one-molal solutes in one convention set |
| core review F06: conventions bind nothing and repeat facts | — | scheduled | TK2b: a form declares the convention facts it reads and reads them from the convention set; one home per fact; verify requires them; the evaluator refuses to combine differing convention sets | a NASA parameterization without a gas constant fails verify; two gas constants in one evaluation are refused |
| core review F07: a known zero of an absolute quantity is refused | — | scheduled | TK2b: `absolute` means non-negative | a zero dipole moment loads; a negative one is refused |
| core review F08: reuse keys are incomplete | S09 | scheduled | TK2c: one key builder for all stages covering framework code, provider versions, harness scripts and environment locks | a provider pin or oracle script change marks dependent stages stale |
| core review F09: the envelope is narrower than the sources and binds nothing | — | scheduled | TK2d: validity regions made of clauses on any record, with a coverage state; contract arguments name their observable; the evaluator reports inside or outside | a two-axis region and a composition clause load; points outside are recorded as outside |
| core review F10: relations cannot declare invariants | S04 | scheduled | TK2b: `requires` on relations; each stated rule becomes a declared requirement or is removed | a datum with a known state and no value is refused |
| core review F11: the declaration-to-code shape contract is undeclared | S03 | scheduled | TK2b: a declared pipeline contract the loader checks | renaming `carrier.tree_hash` is refused at load |
| core review F12: typed holes in physical semantics | — | scheduled | TK2b: covariance as typed standard uncertainties and a correlation relation; a contract argument's basis is a declared composition basis, compared at calls; the dimension-only check is a stated limit | a volume-fraction vector passed to a mole-fraction argument is refused |
| core review F13: evidence uncertainty is single and one-sided | C5 | scheduled | TK2d: uncertainty assessments as a child relation with two-sided magnitudes; censoring has a side; reference-state presentation on columns | fixture (n): a value with a standard, an expanded and a repeatability uncertainty loads as three typed rows |
| core review F14: survey losses are not dispositioned | — | scheduled | TK2e: every surveyed construct with a stated loss gets one disposition, generated into the residue report | the report lists no construct without a disposition |
| core review F15: role obligations are prose and a SQL list | — | scheduled | TK2b: enum members carry declared facets that checks read | adding a member with a lineage facet needs no SQL edit |
| core review F16: identity text without a tying invariant | C3, C6 | scheduled | TK2b (tying requirements, declared canonical encodings with exact rational coefficients); TK2c (publications resolved across carriers) | two carriers citing one DOI under different keys yield one publication |
| core review F17: reasons are prose | — | scheduled | TK2c: typed reasons for held rows and blocked runs | the residue report groups held rows without parsing text |
| core review F18: field rules are not proven applied | — | scheduled | TK2c: rule use is counted and an unused value rule is reported | a mapping that stops emitting the envelope is reported |
| core review F19: library consideration is not recorded | — | scheduled | TK2c, with F08: the reasons for the bespoke declaration language, stage reuse and acquisition are recorded in their contract pages | the three notes exist |
| core review F20: non-numeric slots cannot be evaluated | S02 | deferred | decision: a form whose mathematics needs a tabulated, enumerated or reference-valued slot stays `structure_declared_equation_external`; an interpolation construct is added when the first form is blocked only by it | trigger: a surveyed form whose only obstacle to `expressed` is a tabulated slot |
| core review F21: adoption is not yet a projection | — | scheduled | TK9: the production reference packages are read as a carrier so production entities resolve by the ordinary rules; the snapshot contract states how envelopes and convention sets render; gap marks applied | the seed-slice export equals the shipped seed values with production identifiers |

## Verification

Targeted checks accompany implementation; TK9 qualifies the applicable scope. The checks, each a
recipe in `thermo-knowledge/justfile`:

- `tk-acquire --check`: every file in the raw store matches `sources.lock`.
- `tk-generate --check`: generated SQL, types and docs equal a fresh regeneration.
- `tk-build` from an empty database succeeds with all constraints on; a second build yields
  identical identifiers and row counts.
- `tk-verify`: reactions conserve elements and charge; symmetric groups hold one orientation;
  piece intervals do not overlap; every record has an import record and a rights determination;
  group formulas sum to species formulas where groups carry formulas.
- Round trip on a sample: canonical to source format equals the source-faithful row.
- Coverage report per source: records read, mapped exactly, mapped with declared loss, out of
  scope, unmapped. The target is zero unexplained residue.
- `tk-qualify`: status counts per representation class; IAPWS-95, IF97 and AGA8/GERG verification
  values reproduced from canonical records.
- Snapshot export for the seed slice equals the values in `packages/reference/seed-data` and
  `packages/reference/data`.
- The query register row R-49 describes works: all parameter sets valid at a temperature for an
  observable of a species, across every source.

## Open items

- ATcT: the site answers HTTP 403 to the pipeline's descriptive User-Agent, including on
  `robots.txt`; it is not acquired and the client identity is not changed to get past the filter.
  Options are a manually saved copy, asking Argonne, or the ATcT-derived values other carriers hold.
- ThermoML archive: `data.nist.gov` did not respond on 2026-09-30; the schema was acquired from
  `trc.nist.gov` and matches its published checksum.
- JANAF: the page retrieval stopped at the background time limit with 1,273 of 1,796 tables in
  its partial directory; re-running `tk acquire janaf` fetches only what is missing.
- CoolProp's own identifiers disagree for seven fluids (its InChIKey, InChI and SMILES fields give
  different structures), so they resolve as ambiguous and their records are held; each needs a
  curated decision in `identity/decisions.toml`.

- DDBST: proceeding without retrieving its pages; written permission remains an option.
- Julia is not installed; it is needed for the Clapeyron oracle in W2.
- The environment grouping has not been through a resolver.
- Several reader API names and the IF97 verification table numbers were reported from memory by
  reconnaissance and are confirmed when each reader is written.
- Shared-configuration touchpoints outside the tree: `.gitignore`, the root `pyproject.toml`
  lint excludes, `REUSE.toml`, and the link in `docs/plans/README.md`.

## Current checkpoint

2026-09-30. The contracts the packets implement are the pages under `thermo-knowledge/docs/`:
`meta-model.md`, `expressions.md`, `acquisition.md`, `survey.md` and `pipeline.md`.

State:

- Every stage from acquisition to qualification exists with its tests: `tk acquire`, `read`,
  `load-src`, `map`, `resolve`, `build`, `verify`, `qualify`. `tk project` is the one stub.
- The first end-to-end slice is through: CoolProp fluid identities and 123 saturation-pressure
  ancillary curves are read, resolved, mapped, built into `pse_thermo`, verified (no check
  failing) and qualified against CoolProp 8.0.0 itself (3,075 points, worst relative deviation
  4.5e-12 against a tolerance of 1e-10 set from the formula's conditioning).
- The construct inventory is complete: 37 survey records under `thermo-knowledge/survey/`.
- The core model is in `model/` with the changes the surveys required; the forms declared so far
  are NASA-7, NASA-9, Shomate and one saturation-pressure curve.

Decisions made during execution:

- The declaration is TOML: its parser is in the standard library, duplicate keys are an error and
  scalar types are explicit.
- Units are parsed by `pint`; the model declares no unit table. Storage units are coherent SI.
- `prov.record` is a universal root that every source-asserted row registers in, so provenance,
  rights, derivations and equivalence assessments attach by an enforced foreign key.
- A source's own key is a *source entity*. Resolution attaches it to a canonical entity by a
  curated decision, a structural identifier, a registry cross-reference or a declared formula
  scope; otherwise to a provisional entity that is never joined across carriers.
- Structure data that a form needs (group counts, site multiplicities, formula matrices) enters
  an expression as an indexed contract argument, so expressions read only what they name.
- One equation used for several observables (a saturation curve form used for bubble and dew
  pressure) is an open modelling question; only the single-curve case is declared.
- The first end-to-end slice is CoolProp fluid identities and saturation-pressure ancillaries,
  because its reader and survey exist; it exercises resolution, mapping, build, verify and
  qualification before any wave scales out.
- WebBook acquisition waits for W6, when the species list that scopes it exists.
- `tk build` replaces the canonical schemas in one transaction instead of swapping databases,
  so the source-faithful `src_<id>` schemas stay in the same database.
- Whether a form is qualified is derived from recorded runs, not declared: a declared status
  would change the declaration fingerprint the run was made against.
- A qualification tolerance is set from the conditioning of the formula, recorded in the case
  file, and never loosened to make a run pass. The first run failed at sixteen points and
  exposed an evaluator defect (constant folding of stored values), which was fixed.

The design review of the core model (`review_sources`) returned Revise: the architecture stands
(the declaration as the one authority, the stage decomposition, the universal record root, the
open-world store with a closed-world export, forms as data, qualification derived from runs), and
a set of model concepts and four pipeline contracts must change before the hard-case fixtures and
Wave 1. Every finding is dispositioned above.

Next, in dependency order: TK2b; TK2c and TK2d; TK2e; the hard-case fixtures; then Wave 1.

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
