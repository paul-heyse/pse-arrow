---
title: "24a: Knowledge-base code remediation"
status: in-progress
date: 2026-09-30
adrs: []
review_sources: [docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md]
scenario_sources: [docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r1, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r2, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r3, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r4, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r5, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r6, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r7, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r8, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r9, docs/design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md#r10]
---

# 24a: Knowledge-base code remediation

## State, purpose and ownership

**In progress. Written 2026-09-30; execution of every packet through QF authorized
2026-10-01.** This is the companion plan of
[Plan 24](24-thermodynamic-knowledge-base.md). It designs and sequences the remediation of the
[code review](../design_review/reviews/design_review_thermo-knowledge-code_2026-09-30.md) of
`thermo-knowledge/`. That review found 22 defects (C01–C22) and returned the verdict Revise.

Ownership is split three ways:

- **Plan 24's [*Finding dispositions*](24-thermodynamic-knowledge-base.md#finding-dispositions)
  owns the status of C01–C22.**
- **This plan owns the design and the progress of its packets.** The status column of the
  [packet table](#packets) is the only progress record.
- **The contract pages under `thermo-knowledge/docs/` own the contracts.** Each packet updates
  the pages whose contract it changes.

Two consequences:

- **The review is evidence.** Accepting this plan resolves no finding.
- **No ADR is needed.** The tree is standalone tooling under Plan 24 and changes no
  production contract.

**Maintainer decisions (2026-09-30).**

1. **Identity is corpus-checked.** Cross-carrier identity is fixed before the W1-M1
   equivalence assessments. Resolution reads chemicals' staged structure table as a pinned
   reference corpus ([§1](#1-identity-across-carriers)).
2. **The work lives in a companion plan.** Plan 24 keeps the dispositions; 24a owns the
   packets.
3. **The typing pass is folded in.** The pyrefly pass that was in flight becomes this plan's
   first packet, [TY](#ty). Its starting point is whatever that pass has landed when execution
   begins.

## Baseline and affected foundations

**Baseline: the tree at commit `35ed87ee`, inspected 2026-09-30.** That commit moved the tree
to Python 3.14.7, added pyrefly and fixed typing errors. It resolved none of C01–C22 (*Interface-checked*,
by a read-only diff against the reviewed commit `721749df`).

The facts every packet starts from:

- **Typing (C14 is partly done).**
  - `thermo-knowledge/pyproject.toml` pins pyrefly. `pyrefly.toml` checks `src` and `tests`
    only, not `mappings/` or `oracles/`.
  - The checker is not in strict mode, does not run in CI, and is not part of the root
    `just hygiene`.
  - **Errors:** a run reported 337 errors and 17 warnings (*Measured* by the mapping pass).
    335 of the errors are in tests; 2 are in `declaration/resolve.py`.
  - **Suppressions:** 54 `type: ignore` markers in `src`, 266 in tests, 12 in mappings and
    oracles.
  - **Closed sets still typed as `str`:** `Field.shape`, `Requirement.rule`, `Form.status`,
    `ImplicitBlock.select`, `BinOp.op`, `Func.name` and `OutputCall.kind`.
  - The root `pyproject.toml` excludes the tree, with the comment that "it checks itself".
- **New code since the review.**
  - `expression/symbolic.py` wraps SymPy with typed operations. New expression code goes
    through it.
  - SQL in `qualify/` is composed with `psycopg.sql`, and `qualify/source.py` exports
    `table_identifier()`. New SQL follows that style.
- **The store is stale.** The commit edited `staging/` and `acquire/`. Both are in the `read`
  stage's reuse key (`reuse.py`, `STAGES["read"]`), and `staging` is also in the `map` key. So
  every staged source must be read again once, before any mapping.
- **Identity data** (*Measured*: read-only SQL and pyarrow over `pse_thermo` and
  `.store/staged/`).
  - **Who carries structure.** JANAF, NASA CEA and Cantera carry no CAS number and no
    structure. CoolProp, chemicals, thermo and FeOS carry CAS and structure.
  - **The reference corpus.** Chemicals' `identifiers_chemical` has 76,500 rows with CAS,
    formula, SMILES, InChI and InChIKey. Its InChI values are stored without the `InChI=1S/`
    prefix.
  - **Formula joins today.** 902 of the 1,690 formula species are reached from more than one
    carrier, and 55 formula keys duplicate an InChIKey species.
  - **How the corpus splits the 902.** Computed from structures with isotopes kept separate:
    497 have one corpus structure, 350 have none, 55 have several.
  - **Confinement alone is not enough.** Confining the formula rule to the contract's own
    classes would drop 825 of the 1,134 species forms that JANAF shares with Cantera or CEA.
- **Scale** (*Measured* in the review, slot 10):
  - JANAF mapping: 1.76 GB for 1.22 M canonical rows.
  - Build dry run (union and checks, no database): 614,600 kB peak for 1,494,807 rows. The
    largest table, `ev.datum`, is 73 bytes per row as Arrow data.
  - Database: 525 foreign keys, validated through deferred triggers.

**Foundations kept as constraints.** The review names these strengths. Every packet preserves
them:

- One declaration drives the DDL, the Arrow schemas and the `meta` rows.
- Transposition is the model for a typed closed set with one owner: a `Literal` type, one
  module, expansion in one place.
- The expression tree is closed and has visitors.
- The mapping specification is declarative and accounts for which rules it uses.
- Resolution is pure and does not depend on input order.
- Publication is never torn: a working directory, then a rename.
- The build union is columnar and reports structured conflicts.
- The harness protocol is versioned.
- Implicit solves are accepted only after an independent check.

**Foundations this plan changes, and why first.**

- **Identity:** stage currency only matters once identities can change. Identity is the
  first correction that does change them.
- **Stage lifecycle:** every later re-run relies on it.
- **Meta-model closed types:** the set read model, verify vocabulary and the cause model
  build on them.
- **Mapping operations:** the bounded writer replaces their internals.

## Target design

### 1. Identity across carriers

*Findings C01, C03, C04, C19 and C20; packets [ID1](#id1), [ID2](#id2) and [DC20](#dc20).*
*The contract is `thermo-knowledge/docs/pipeline.md` §1.*

**Rule order.** Each source entity is resolved by the first rule that applies:

1. **Curated decision.** `identify`, `reject` or `distinct`, as today.
2. **Structural.** InChIKey, InChI or SMILES, through RDKit.
3. **Registry.** CAS or PubChem CID, linked through the carriers' own entities.
4. **Listing condition** (C03; new).
5. **Corpus link** (C01; new).
6. **Conventional formula join** (C01; replaces today's unconditional formula scope).
7. **Provisional.**

Every outcome carries a typed `resolution_reason`. This is a new declared enum on
`source_entity` in `model/identity.toml`, alongside the new rule member `formula_structure`.

**Listing condition (rule 4).** A mapping's `[[formula_scope]]` gains three parameters:

| Parameter | Cantera | CEA | JANAF |
|---|---|---|---|
| `listing_unit` | `artifact` | `artifact` | `carrier` |
| `name_column` | `name` | `name` | `substance_name` |
| `name_cut` | — | — | `","` |

CEA names are not cut: cutting would merge `C3H7,n-propyl` with `C3H7,i-propyl`.

The engine groups entities by (unit, composition, charge, aggregation, polymorph), comparing
names case-folded and stripped. If one group contains several different names, every entity in
the group is `unresolved` with reason `formula_shared_in_listing`. The report lists the names.
A human `identify` decision overrides this for one entity; the other members of the group stay
blocked. This replaces the 1,478 generated rejects, `tests/identity_support.py` and
`tests/test_identity_decisions.py`.

**Keys that do not depend on position.** A Cantera source entity is keyed by the canonical
encoding of (artifact, section, name). `ScopeSpec.key` becomes a list of columns, which also
removes the hand-built key in phase 2 (C08).

**Compositions in claims.** Phase-1 entity claims carry the composition counts that the
mappings already compute. The engine builds every formula cell key from those counts with the
one function `mapping/formula.hill`, so formula strings are never parsed again. The value of an
identity assertion stays verbatim.

**Reference corpus.** It is declared in `identity/reference.toml`, which names:

- the source id (`chemicals`) and the table (`identifiers_chemical`);
- the structure column (`InChI`), with the prefix to restore;
- the key column (`InChI_key`) and the check column (`formula`);
- the aggregations excluded from linking.

The pin comes from `sources.lock`. Resolution **refuses** when the corpus is not staged, or when
the staged manifest's pin differs from the lock. A silent fallback would change identity.

`resolve/command.py` builds a `FormulaCorpus` index and passes it to the engine, which stays
pure. To build it:

1. Restore the InChI prefix and recompute the InChIKey with RDKit.
2. Index only rows whose stated key equals the recomputed one.
3. Key each cell by the composition (isotopes kept apart, D and T as declared symbols) and the
   net charge, both computed from the structure.
4. Count distinct full standard InChIKeys per cell.

Rows that do not parse, whose keys disagree, or that have no structure (spin isomers and fake
CAS numbers among them) are excluded, and counted by cause in the report.

**Corpus link (rule 5).** A cell with exactly one structure links the entity to that InChIKey
species, with rule `formula_structure`, except in three cases:

- surface aggregation (adsorbate formulas include or omit the site atoms);
- non-integral compositions (Air, Fe0.947O);
- organic odd-electron species, meaning the formula contains C and H and ΣZ·n − q is odd. The
  composition does not fix the constitution: the corpus holds only isopropyl for C3H7.

A cell with several structures yields `unresolved` with reason
`formula_has_several_structures`. A cell whose structures differ only in stereo yields
`stereo_unspecified` and waits for a human decision. Entities in a several-structure cell join
within their own listing unit through a listing-scoped formula key, never across carriers
([decision #2](#decisions)).

**Conventional join (rule 6).** A formula with no structure in the corpus keeps a formula key,
and so a cross-carrier join, only when its composition fixes the constitution:

- one atom (ions included);
- any diatomic;
- one non-hydrogen atom with only H, D or T.

In each case the composition must be integral and the species must not be on a surface.
Everything else is `unresolved` with reason `formula_not_identifying` ([decision #1](#decisions)).
*Measured* on the 350 multi-carrier "no structure" formulas, 184 keep the join and 166 become
provisional, among them Al₂Cl₆, C2H5 and Jet-A.

The electron gets a declared conventional key or a curated decision; it has no formula
assertion.

**Evidence and change diff.**

- **Provenance of a link.** A linked entity's origin cites the corpus row (`_artifact`,
  `_locator`) in the declared role `identity_reference`, so provenance carries the link and no
  new table is needed.
- **Report.** `report.json` gains:
  - a corpus block: pin, content hash, and rows indexed and excluded by cause;
  - counts by carrier × rule × reason;
  - the sorted list of links.
- **Change diff.** Before installing, `tk resolve` compares each source entity's
  (status, rule, reason, target key) with the installed `tk.source_entity`. It writes the
  differences as `changes` and prints counts. Corpus growth is not monotonic: a structure can
  create a link or break one.
- **Reuse key.** It adds the staged table's content hash, a digest of `reference.toml`, and a
  bumped format number.

**Identity checks (C04).**

- **Compositions for both kinds of species.** Today `tk.composition` holds rows only for
  formula-keyed species forms; no structure-keyed species has one, and an InChIKey species has a
  formula only as a carrier's assertion string, in that carrier's notation. Resolution therefore
  records the composition of every structure-keyed species it targets, computed with RDKit from
  the structure with isotopes kept apart. That is the same computation as the corpus cells, so no
  formula string is parsed.
- **Verify check `species.formula_duplicates_structure`.** No formula-keyed species may have the
  composition and charge of a structure-keyed species. The target is zero. The corpus comparison
  measured 55 such duplicates before the change; that figure comes from pyarrow over the
  resolution output, not from this check, which cannot run until compositions exist for
  structure-keyed species.
- **Reports, not gates:**
  - formulas left unlinked, each with its candidate structures from the corpus (the review's
    C01 verification);
  - two entities of one listing unit that reach one species form (JANAF's two HNO2 tables
    share a form today);
  - multi-carrier joins, listed with their names.

  Name overlap is too noisy to gate on: 326 of 828 one-structure links share no normalised name
  with the corpus.

**Class policy (C19).** A single table in the engine holds, for each entity class: its target
kind, whether a decision can identify it, its rules, and what it is identified through. The
specification checks and the decisions consume that table. A W8 `pseudo_component` is then one
entry plus its rule function.

**C20 decision input.** The report splits ambiguity into conflicts within one source and
conflicts across sources. That count drives [DC20](#dc20) when chemicals is resolved.

**Consequences for Plan 24.**

- **W1-M1 equivalence.**
  - Each equivalence assessment records the rule that joined its two sides.
    `formula_structure` joins are labelled as inference.
  - Pairs left uncompared because identity is unresolved are counted, not dropped.
- **W1-M2a, identity backbone.**
  - The link index is every unique structural key the resolution knows, plus the corpus.
  - Once chemicals is mapped at the same pin, a test shows that links computed from the corpus
    equal links computed from chemicals' mapped entities.
  - Chemicals' mapping asserts InChI with the prefix restored.
- **What this cannot fix.**
  - Closed-shell isomers missing from the corpus.
  - Electronic states that InChI does not encode (OH* against OH, CH2(S)), when such an entry
    stands alone in its listing unit.
  - Polymorphs that the mappings do not state (the JANAF TiO₂ tables), which is a JANAF
    mapping fix.
  - Class errors (Air, Jet-A, `(CH2)x(cr)`), which need decisions or a class.

### 2. Stage lifecycle

*Findings C02, C11 and C22; packets [ST1](#st1) and [ST2](#st2). The contracts are
`docs/pipeline.md` §3 and §6.*

**One stage module** (working name `stage.py`) replaces the lifecycle code each stage copies
today. It provides:

- **Typed manifests per phase.** A msgspec union tagged by phase replaces
  `CanonicalManifest.phase: str` and its optional per-phase fields. The staged manifest schema
  stays as it is, so staging is read again only once.
- **An `upstream` map** from artifact to manifest hash, kept apart from scalar inputs.
- **`check_upstream()`**, which every consumer calls, the build included.
- **`current()`**: whether a recorded key equals the current one.
- **`publish()`**: a working directory, then a rename. This is the only install path.

It deletes:

- the four separate key comparisons (staging, map, resolve, qualify);
- the second install path in `staging/stage.py`;
- the three discovery copies;
- `_check_resolution`.

**Resolution currency per carrier.** A carrier's records depend on a *resolution slice*: the
`_resolution` rows of that carrier's source entities, plus the canonical keys of their targets.
The map key and the build both use the hash of that slice. This has two effects:

- a re-resolve stales only the carriers whose subjects moved, which also fixes the global
  re-map in C09;
- the build refuses records whose slice no longer matches, and names the source to re-map
  (C02).

**Complete data inputs (C11).** A stage's key includes every data file it reads. For mapping,
that means the encoded source information from `sources/<id>.toml` (title, rights) in both
phase keys. This becomes part of the stage contract in `pipeline.md` §6, not a per-stage
choice. The build records every upstream hash, including each loaded `_qualification/*` run,
which `tk project` needs.

**Workspace (C22).**

- **One workspace value.** It is built at the CLI root and holds the tree, the store, the
  lock, the configuration and the database settings. Library code takes it explicitly;
  `default_factory` defaults that read `config` are removed from library code.
- **No ambient paths.** The harness working directory and the core lock come from the
  workspace.
- **Repeated work removed.** The declaration fingerprint is memoized per declaration.
- **Test marking.** pytest declares a `database` marker, which the database fixtures set.

### 3. Meta-model closed types

*Findings C14, C06, C18, C16, C17 and C05 (part); packets [TY](#ty), [MM1](#mm1) and
[MM2](#mm2). The contract is `docs/meta-model.md` §3.4, §4, §6 and §7.*

**Typing baseline (TY).**

- **Checker coverage.** pyrefly checks `src`, `tests`, `mappings` and `oracles` with zero
  errors and zero warnings. Every remaining `type: ignore` carries an error code and a reason.
- **Closed types.** These closed sets become `Literal` or `StrEnum` types: `Field.shape`,
  `Form.status`, `ImplicitBlock.select`, `BinOp.op`, `Func.name`, `UnaryOp`, `Compare`,
  `BoolOp`, `Reduce.kind` and `OutputCall.kind`. `Requirement.rule` is left to MM1, which
  replaces it.
- **Derived vocabularies.** The `generate/meta_tables.py` vocabularies are derived from those
  types (`typing.get_args` or the enum), not restated.
- **Root configuration.** The root `pyproject.toml` comment that the tree "checks itself" is
  made true. Strictness and hygiene wiring follow [decision #3](#decisions).

**Check rules (MM1).**

- **One union.** `Requirement` carries a closed `Check` union of frozen dataclasses, built by
  the resolver from the unchanged `CheckDecl` authoring syntax.
- **Exhaustive renderers.** `generate/plan.py` renders SQL and `canonical/invariants.py`
  evaluates in Python. Each uses `match` with `assert_never`, so neither has a catch-all
  branch.
- **Derived lists.** `CHECK_FORMS`, the meta vocabulary and its documentation are derived
  from the union.
- **Shared test cases.** One case table drives a Python test and a PostgreSQL test. A test
  also asserts that the table covers every variant.

**Typed DDL (MM1, C18).** `ir.Constraint` bodies become typed variants: `PrimaryKey`,
`Unique`, `Check`, `Exclude` and `ForeignKey`, rendered last. `build/keys.py` reads primary
keys from the IR, which deletes the regex.

**Slot shapes and states (MM2, C05).**

- **Typed shapes and states.** `SlotShape` and `SlotState` become typed on the model, with
  predicates (`holds_record`, `is_numeric`). The meaning of each value state is read from its
  declared facets, never from member names, as `model/parameters.toml` requires.
- **Stateful slots expanded once.** The resolver expands each stateful slot into fields plus
  `present_iff` requirements, following the `arrangement` precedent. The generator, writer,
  reader and verify SQL then consume that expansion rather than repeating it.
- **One owner for derived names.** A derived-name module in `declaration/` (the resolver mints
  these names, so `declaration/` must not import them from `generate/`) owns derived column and child-table
  names, including `__state`, `__redirect` and `Set<K>` tables.

**Parameters and bindings (MM2, C16, C17).**

- **`ContractParameter`.** Contract roles, arguments and outputs become a separate
  `ContractParameter` record, so `Field` describes columns only.
- **Framework roles.** `contract_check` requires `framework[role]` to equal the pipeline
  contract's kind for that role.
- **Value-state members.** They become an enum entry in the pipeline contract with exactly
  the declared members.

### 4. Set read model

*Finding C05; packet [MM2](#mm2). The contract is `docs/pipeline.md` §5.3 and §5.4.*

**One `SetReader`** reads a parameter set completely:

- slots with their states;
- redirect targets;
- nested and referenced sets;
- tabulated functions;
- constituent positions;
- policy defaults.

It works over a `RowSource`: the database, or the canonical Parquet. `DatabaseSource`
evaluates through it.

**Currency follows from it.** Currency records exactly what evaluation read, through a
recording `RowSource`. The run manifest stores those reads and their hashes, and the build
replays them over Parquet. The traversal in `build/currency.py` is deleted, and the snapshot in
TK9 uses the same reader. `withheld` and `not_applicable` stay distinct up to the evaluator.

### 5. Causes

*Finding C07; packets [DX1](#dx1) and [DX2](#dx2). The contracts are `docs/pipeline.md` §2.4
and the enum documentation in `model/qualification.toml`.*

**Shared diagnostic.** It generalises `declaration/diagnostics.Diagnostic` and carries a code
(a `StrEnum` per area), a locator, the subject, the detail, and `caused_by`. `RowHeld`, the
canonical writer, stage errors and `Blocked` all use it.

**Held reasons.**

- **`missing_convention` is split by cause:**
  - `aggregation_not_stated`
  - `value_not_decoded`
  - `unmapped_field_stated`
  - `unit_not_stated`

  `missing_convention` itself stays only for its declared meaning, which becomes a held path in
  place of today's `MappingError`.
- **`unknown_subject` is split from `undeclared_member`.**
- **Derived holds point at their cause.** A row held because its phase-1 entry was held
  references that cause.
- **`qual.held_row` gains `rule`, `column` and `caused_by`.** `rule` references the
  `qual.mapping_rule` key, or the name of the violated invariant.
- **Writer problems carry codes.** Value refused, missing value, unknown attribute, competing
  assertion, and the violated `owner.requirement`.

**Stage refusals.** Every stage refusal carries one of these codes: `input_missing`,
`input_stale`, `input_corrupt`, `declaration_mismatch`, `invalid_input`, `infrastructure`. A
`ReuseError` is no longer reported as `harness_failed`.

**Tests assert codes, not message text.** DX1 converts the tests for held rows. DX2 converts
the rest of the 360 `match=` assertions ([decision #4](#decisions)).

### 6. Mapping operations

*Finding C08; packet [EM](#em). The contract is `docs/pipeline.md` §2.1–2.3.*

These are a handful of operations on the mapping context and the writer, not a new layer:

- **`emit.piecewise_set(...)`** derives the validity region from the family's declared
  interval. The writer refuses a gap between pieces, or records it as a declared loss.
- **`DatasetBuilder`** builds the whole chain: dataset, component, phase, column, point, datum,
  with the standard state and the reference phase. JANAF uses it now; chemicals' point sets and
  ThermoML use it later.
- **`emit.composition(...)`** reuses the phase-1 result instead of emitting it again for each
  pair.
- **`ctx.subject_of(scope, row)`** reads the scope's declared key, now composite.
- **The writer numbers occurrences**, replacing three hand-written counting idioms.
- **One statement of the source's unit convention** per mapping or partition, for example
  `{Temperature = "kK", MolarEnergy = "kJ/mol"}`. Each coefficient's source unit follows from
  its slot's dimension, with an explicit unit as an override. The form fixes the dimension; the
  convention fixes the scale.

**Acceptance:** canonical Parquet from all four mappings is byte-identical before and after.

### 7. Qualification

*Findings C10, C12 and C13; packets [QL1](#ql1) and [QL2](#ql2). The contracts are
`docs/pipeline.md` §5.1 and §5.4, and `docs/expressions.md` §3 "Definite integrals".*

**Library key (C10).**

- **Where the key comes from.** The key for each evaluated subject comes from the set's own
  origin: the carrier's source entity in that set's artifact.
- **What the key is.** A case declares which asserted identifier the harness receives, for
  example Cantera's (file, name). It is never a positional locator.
- **What blocks.** Only selected subjects are checked, so a `list` case no longer blocks on
  unrelated ones.

**Policy module (C12).** `qualify/policy.py` is pure. Given typed facts, it does:

- **Grid:** built from `RecordValidity`. Relative clauses are resolved against their reference
  values, or blocked with `no_grid`.
- **Sampling, comparison and verdict.**

Selection returns typed facts and reuses `DatabaseSource.validities`, deleting the separate
`_regions` read in `run.py`. Recording stays separate. These policy tests need no database.

**Quadrature (C13).** `quad` is called with `full_output`. The evaluator refuses with a typed
`SolveFailure` that names the form and the point when `ier != 0` or the error estimate exceeds
the stated tolerance. This applies to both the general integral and the Debye function, and the
code goes through `expression/symbolic.py`.

### 8. Bounded resources

*Finding C09; packets [BD1](#bd1) and [BD2](#bd2). The contract is `docs/pipeline.md` §2.3 and
§3.*

**Build (BD1).** Within the one transaction, the build:

1. creates the tables without foreign keys;
2. loads them;
3. adds each foreign key with `ALTER TABLE … ADD CONSTRAINT … FOREIGN KEY`, which validates it
   in one query;
4. commits.

`SET CONSTRAINTS ALL DEFERRED` goes, and atomicity is unchanged. Today the build executes the
generated script as one text (`build/plan.py`, `build/database.py`), with each foreign key
rendered `DEFERRABLE INITIALLY DEFERRED` inside it (`generate/ir.py`). The generator therefore
gains a projection that returns the table DDL and the foreign-key statements as separate parts.
The committed `schema.sql` stays the full script, and the fingerprint keeps hashing it. The `pipeline.md` §3 statement that check
constraints are validated at commit is corrected: PostgreSQL checks them at insert.

**Mapping (BD2).**

- **Bounded reads.** Staged rows are read per artifact with `pyarrow.dataset` filtered scans.
- **Bounded writes.** The writer flushes rows whose scope is one block (`datum`, `data_point`,
  columns) to `ParquetWriter` row groups for each table, after each block.
- **What stays in memory.** An identity index, so competing assertions are still detected,
  kept only for kinds whose key is global.

DuckDB stays the candidate for the build union if one table's measured union exceeds memory.

### 9. Verify vocabulary and set-level checks

*Findings C15 and C21; packet [VF](#vf). The contract is `docs/meta-model.md` §3.4 and
`docs/pipeline.md` §4.*

**Generated set-level checks.** The `Check` union gains three set-level variants:

- `contiguous_from_one`
- `acyclic`
- `sums_to`

Their verify SQL is generated. A row-level variant, `presence_by_facet`, replaces the four
`*magnitude_matches_kind*` checks: which magnitudes are present follows the facets of the kind,
with member lists taken from the declared facets. The near-copy check files these replace are
deleted; hand-written checks remain only for rules that are genuinely bespoke.

**Checked vocabulary references.** A generated function `meta.member(enum, name)` (and its
facet form) raises on an undeclared name. Checks call it instead of writing enum and facet
names as text literals.

## Choices and alternatives

| Capability | Choice | Why | Revisit if |
|---|---|---|---|
| Static typing | pyrefly, already pinned and used at the repository root | Exhaustive `match` over closed types needs a checker | — |
| Closed sets | Frozen dataclass unions and `Literal` types built by the resolver | Keeps the TOML authoring syntax. A msgspec tagged union would force a tag into every authored check | A closed set needs third-party extension, which would justify a registry |
| Stage lifecycle | A small shared bespoke module | Workflow engines track files, not database outputs, and would keep a second record of currency beside the manifests (the reasoning in `pipeline.md` §6 stands) | The stage graph needs parallel branches, cluster runs or a shared remote cache |
| Bounded mapping | `pyarrow.dataset` filtered scans and an incremental `ParquetWriter` | Already used in staging; no new engine | — |
| Build union | pyarrow, unchanged | The dry run peaked at 614,600 kB for 1.49 M rows; `ev.datum` is 73 bytes per row as Arrow (*Measured*) | One table's measured union exceeds memory: DuckDB |
| Foreign-key validation | PostgreSQL `ADD FOREIGN KEY` after the load | Native; validates in one query; the deferred-trigger queue stays in memory and cannot spill | — |
| DDL IR | Typed constraint bodies in our own IR | SQLAlchemy Core has no construct for range types and would churn `schema.sql` | The IR grows beyond tables, constraints and domains |
| Structure keys | RDKit, already pinned and keyed | Computes keys and compositions from the corpus InChI | — |
| Harness result validation | msgspec structs with `forbid_unknown_fields` | Replaces hand checks in `qualify/harness.py`; part of QL1 | — |

## Packets

| Packet | Findings | Prerequisites | Responsibility | Status |
|---|---|---|---|---|
| <a id="ty"></a>TY Typing baseline | C14 | none (absorbs the in-flight pass) | Closed `Literal` types, derived vocabularies, pyrefly at zero over src, tests, mappings and oracles | not started |
| <a id="id1"></a>ID1 Corpus-checked identity | C01, C03, C04 | TY | Rules 4–6, the reference corpus, typed reasons, a change diff, an identity check, position-free keys | not started |
| <a id="id2"></a>ID2 Class policy | C19 | ID1 | One class-policy table consumed by spec and decisions | not started |
| <a id="ql1"></a>QL1 Qualification selection and policy | C10, C12 | TY | Library key from the set's origin; a pure policy module; relative clauses | not started |
| <a id="ql2"></a>QL2 Quadrature status | C13 | TY | Typed refusal for unconverged integrals | not started |
| <a id="st1"></a>ST1 Stage lifecycle | C02, C11 | ID1, QL1 | Typed manifests, upstream map, one currency check and install path, resolution slices, data inputs in keys | not started |
| <a id="dx1"></a>DX1 Held causes | C07 (held path) | ST1 | Shared coded diagnostic; split held reasons; `held_row` rule and cause | not started |
| <a id="em"></a>EM Mapping operations | C08 | ID1, DX1 | Piecewise sets, datasets, composition, subject lookup, occurrences, unit convention | not started |
| <a id="mm1"></a>MM1 Check-rule owner and typed DDL | C06, C18 | TY | `Check` union with exhaustive renderers; typed IR bodies | not started |
| <a id="mm2"></a>MM2 Set read model and bindings | C05, C16, C17 | MM1, QL1, EM | Slot shape and state types; stateful expansion; `SetReader`; recording currency; `ContractParameter`; binding check | not started |
| <a id="vf"></a>VF Set-level checks and verify vocabulary | C15, C21 | MM2 | Generated set-level checks; `meta.member()` | not started |
| <a id="st2"></a>ST2 Workspace and markers | C22 | ST1, MM2 | One workspace value; no ambient paths; fingerprint memo; `database` marker | not started |
| <a id="dx2"></a>DX2 Coded refusals | C07 (rest) | DX1, MM2, ST2 | Stage error codes; `Blocked` mapping; the remaining `match=` tests | not started |
| <a id="bd1"></a>BD1 Foreign keys after load | C09 (build) | MM1 | Load without foreign keys, then add them inside the transaction | not started |
| <a id="bd2"></a>BD2 Bounded mapping | C09 (mapping) | EM, MM2 | Per-artifact reads; a flushing writer | not started |
| <a id="dc20"></a>DC20 In-source ambiguity decision | C20 | ID1 report; chemicals resolved in W1-M2a | A recorded decision; no code unless precedence is chosen | not started |
| <a id="qf"></a>QF Final qualification | all | every packet above | The only integrated qualification | not started |

Each packet is accepted by its targeted tests and by the deletion of what it replaces. Formatting
and lint run in the end-of-turn hooks, and integrated tests run only in QF (AGENTS.md
*Execution rhythm*). Unless noted, test files are new and named after the acceptance they
prove. Run them with `just -f thermo-knowledge/justfile tk-test <file>`.

### TY — Typing baseline

- **Changes:**
  - `ruff format` and `ruff check` reach zero over the tree (2,000 lint errors and 130 files to
    reformat at the start, *Measured* 2026-10-01). No rule ignores are added.
  - `pyrefly.toml` includes `mappings` and `oracles`, at the default error set (decision #3).
  - The closed sets listed in [§3](#3-meta-model-closed-types) become `Literal` or
    `StrEnum`, in `declaration/model.py`, `expression/tree.py` and `expression/scope.py`.
  - `generate/meta_tables.py` derives its vocabularies from them.
  - The 335 test errors and the 2 `src` errors are fixed, and every ignore marker is either
    removed or given a code and a reason.
  - The root `pyproject.toml` comment is corrected.
  - Last, once everything above is at zero: tree recipes `tk-fmt` and `tk-lint`, and root
    wrapper recipes in `fmt` and `hygiene`. The format wrapper skips when the tree has no
    `.venv`, and uses the tree's own ruff.
- **Deletes:** the hand-restated vocabularies (`SHAPES` and its siblings) and unused ignore
  markers.
- **Acceptance:**
  - `just -f thermo-knowledge/justfile tk-types` and `tk-lint` exit 0, with no warnings, and
    the format check passes.
  - `test_meta_vocabularies_follow_model_types`
  - `tk generate --check` is unchanged.
  - Re-run RS0 (see [Sequencing](#sequencing)) leaves every table equal in Arrow content to the
    baseline built at `33261206`.
- **Docs:** `thermo-knowledge/README.md` (the `tk-types` recipe); `docs/meta-model.md` §3.1 and
  §7, where vocabularies are listed.

### ID1 — Corpus-checked identity

- **Changes:**
  - `resolve/engine.py`: rules 4–6 and typed reasons.
  - New `resolve/corpus.py`: the corpus index.
  - `resolve/command.py`: the corpus input, the reuse key and the change diff.
  - `resolve/output.py`: the corpus block, counts, links, `changes`, the reports, and the
    compositions of structure-keyed species.
  - `identity/reference.toml` (new).
  - `mapping/spec.py`: a composite `ScopeSpec.key` and the formula-scope parameters.
  - `mapping/claims.py` and `canonical/store.py`: compositions in claims, `FormulaScopeRecord`.
  - `mapping/context.py`: the hand-built keys removed.
  - `mappings/{cantera,nasa_cea,janaf}/mapping.toml` and `mappings/cantera/mapping.py`.
  - `model/identity.toml`: `resolution_reason`, `formula_structure` and the role
    `identity_reference`.
  - New `sql/verify/species.formula_duplicates_structure.sql`.
- **Deletes:**
  - The 1,478 rejects in `identity/decisions.toml`.
  - `tests/identity_support.py` and `tests/test_identity_decisions.py`.
  - Positional Cantera keys.
  - Formula identity that ignores the listing unit.
- **Acceptance.** Engine tests run over synthetic, in-memory corpora:
  - `test_c2h4o_isomers_not_joined`
  - `test_water_forms_and_coolprop_one_inchikey_species`
  - `test_oh_conventional_join_kept`
  - `test_o2_formula_merges_with_structure`
  - `test_isotopologues_keyed_apart` (D2O, HDO, CH3D)
  - `test_organic_radical_not_linked` (C3H7)
  - `test_surface_species_not_linked`
  - `test_surrogate_not_joined` (Jet-A)
  - `test_listing_names_several_species_unresolved_identify_overrides`
  - `test_cantera_reorder_keeps_keys_and_resolution`
  - `test_corpus_pin_mismatch_refused`
  - `test_corpus_hash_changes_key`
  - `test_resolution_order_independent`
  - `test_no_decision_is_reproduced_by_a_rule`: every remaining decision changes the
    resolution of its subject when removed, and carries a reason.
  - `test_unlinked_formula_reported_with_candidates`: C2H4O appears as unlinked with its corpus
    structures.
  - `test_structure_species_have_compositions`

  The new verify check gets a fixture that violates it. On the real store, RS1 must take it
  from the 55 duplicates measured beforehand to zero.
- **Docs:** `docs/pipeline.md` §1, §1.1–1.3, §2.1 and §4. Rule 4 is split into the listing
  condition, the corpus link and the conventional join. The rule-3 "corpus" (the carriers' own
  entities) is renamed so it is not confused with the reference corpus. Also `model/identity.toml`
  documentation, and Plan 24's *Open items* and *Remaining* (already linked to this packet).

### ID2 — Class policy

- **Changes:** a class-policy table in `resolve/engine.py`, consumed by `mapping/spec.py` and
  `resolve/decisions.py`.
- **Deletes:** the separate class lists (`IDENTIFIABLE_CLASSES`, `_DECISION_CLASSES`,
  `_TARGET_KINDS` and the spec's list of unsupported classes).
- **Acceptance:** `test_class_policy_drives_spec_and_decisions`, and a fixture class that is
  added through one table entry.
- **Docs:** `docs/pipeline.md` §1 (entity classes).

### QL1 — Qualification selection and policy

- **Changes:**
  - `qualify/run.py`: selection, keys and orchestration only.
  - New `qualify/policy.py` (pure): grid, sampling, comparison and verdict.
  - `qualify/source.py`: typed facts from `validities`.
  - `qualify/case.py`: the identifier a library key uses.
  - `qualify/harness.py`: results validated by msgspec.
- **Deletes:**
  - `_regions` and the lookup of source entities by target.
  - Hand validation of harness results.
  - Database-backed tests of pure policies.
- **Acceptance:**
  - `test_gri30_case_selects_h2o_harness_receives_file_and_name`
  - `test_list_case_checks_only_selected_subjects`
  - `test_grid_policy_plain_values`
  - `test_relative_clause_resolved_or_no_grid`
  - `test_harness_result_unknown_field_refused`
- **Docs:** `docs/pipeline.md` §5.1, §5.2 and §5.4. In Plan 24, the "key-prefix filter" text in
  *Remaining* is replaced by a link to this packet.

### QL2 — Quadrature status

- **Changes:** `expression/evaluate.py`, for both the general integral and the Debye function,
  through `expression/symbolic.py`. `SolveFailure` gains a quadrature cause.
- **Deletes:** `quad(...)[0]`.
- **Acceptance:** `test_unresolvable_integrand_refused_naming_form_and_point` and
  `test_converged_integral_unchanged`.
- **Docs:** `docs/expressions.md` §3 "Definite integrals" states the check, not just the
  requested tolerance.

### ST1 — Stage lifecycle

- **Changes:**
  - New `stage.py`.
  - The stages that adopt it:
    - `staging/stage.py`
    - `mapping/runner.py` (resolution slice; carrier data in both phase keys)
    - `resolve/command.py`
    - `build/inputs.py` and `build/record.py` (upstream check; all upstream hashes recorded)
    - `qualify/run.py`
    - `canonical/store.py` (manifest union)
- **Deletes:**
  - Per-stage key comparisons and discovery copies.
  - The second install path.
  - `_check_resolution`.
  - `CanonicalManifest.phase: str`.
- **Acceptance:**
  - `test_build_refuses_superseded_resolution_names_source`
  - `test_rights_edit_reruns_phase_1`
  - `test_unmoved_carrier_stays_current_after_reresolve`
  - `test_corpus_change_stales_resolution`
  - `test_one_install_path_and_currency_check`, an AST test over `src`
- **Docs:**
  - `docs/pipeline.md` §3 and §6: key inputs and currency semantics, made true.
  - `docs/acquisition.md`, if the lock's reader changes.
  - Plan 24's *Database and pipeline* paragraph on reuse.

### DX1 — Held causes

- **Changes:**
  - A shared coded diagnostic, generalised from `declaration/diagnostics.py`.
  - `mapping/context.py`: `RowHeld` and its raise sites.
  - `mapping/coverage.py`: the new `held_row` columns.
  - `canonical/values.py` and `canonical/writer.py`: coded problems.
  - `model/qualification.toml`: the split reasons and the new columns.
  - `mappings/cantera/mapping.py` and `mappings/janaf/mapping.py`.
- **Deletes:**
  - Prose problem strings.
  - The misused reasons.
  - Message-text assertions in the held-row tests.
- **Acceptance:**
  - `test_held_reason_only_for_declared_cause`
  - `test_writer_code_reaches_held_row`
  - `test_derived_hold_references_cause`
  - On the real store at RS2: `test_residue_by_reason_and_rule_reproduces_open_items`.
- **Docs:**
  - `docs/pipeline.md` §2.4.
  - The reason meanings in `model/qualification.toml`.
  - Plan 24's checkpoint sentence about unexplained rows, and its held-row *Open items*.

### EM — Mapping operations

- **Changes:**
  - `mapping/context.py`: `piecewise_set`, `composition`, `DatasetBuilder`, `subject_of`.
  - `canonical/writer.py`: gaps refused; occurrences numbered.
  - `mapping/spec.py`: the unit convention.
  - All four mappings.
- **Deletes:**
  - The validity hull, composition and occurrence code in each mapping.
  - JANAF's raw dataset assembly.
  - Per-coefficient unit lines.
- **Acceptance:**
  - `test_mapping_outputs_unchanged_by_operations` (real store, RS3: canonical Parquet
    byte-identical for all four sources).
  - `test_shomate_states_convention_once_without_t_bounds`
  - `test_piece_gap_refused_or_declared_loss`
  - `test_dataset_builder_janaf_shape`
- **Docs:** `docs/pipeline.md` §2.1–2.3.

### MM1 — Check-rule owner and typed DDL

- **Changes:**
  - `declaration/model.py`, with the `Check` union on `Requirement`.
  - `declaration/resolve.py`.
  - `generate/plan.py`, `generate/meta_tables.py`, `generate/meta_rows.py` and `generate/ir.py`.
  - `canonical/invariants.py`.
  - `verify/checks.py`, where it reads requirement rules.
  - `build/keys.py`.
- **Deletes:**
  - `Requirement.rule`.
  - Both catch-all branches.
  - The primary-key regex.
- **Acceptance:**
  - `test_check_cases_python` and `test_check_cases_postgres`, driven by one case table.
  - `test_case_table_covers_every_variant`
  - `tk generate --check` unchanged.
  - The dummy-variant pyrefly failure, recorded once in the [execution checkpoint](#execution-checkpoint).
- **Docs:** `docs/meta-model.md` §3.4, §6 and §7.

### MM2 — Set read model and bindings

- **Changes:**
  - In `declaration/`: `model.py` (`SlotShape`, `SlotState`, `ContractParameter`),
    `resolve.py` (stateful expansion), `contract_check.py`, and the new derived-name module.
  - In `generate/`: `plan.py` and `meta_rows.py`, which read the expansion and the derived
    names instead of building them.
  - `canonical/writer.py`.
  - The `SetReader`, used by `qualify/source.py`.
  - `build/currency.py` (recording `RowSource` and replay) and `build/inputs.py`.
  - `expression/check.py`.
  - `pipeline_contract.toml`.
  - `sql/verify/set_default_needs_policy_default.sql`.
- **Deletes:**
  - The currency traversal.
  - Each consumer's own `__state`, `__redirect` and `Set<K>` naming.
  - Member-name literals for value states.
- **Acceptance:**
  - `test_set_layout_names_have_one_owner`
  - `test_redirect_target_change_retires_run`
  - `test_withheld_distinct_from_not_applicable`
  - `test_framework_role_equals_contract_kind`
  - `test_contract_parameters_are_not_columns`
  - The expansion keeps today's DDL: the same columns and constraint names, so `schema.sql`
    is unchanged. The `meta` rows change only by the reified state requirements and the
    contract-parameter rows, and the RS4 comparison lists exactly those differences.
  - RS4: the canonical tables are identical; qualification runs are re-made on purpose because
    the read set grows.
- **Docs:**
  - `docs/meta-model.md` §4.1, §4.2 and *The pipeline contract*.
  - `docs/pipeline.md` §5.3–5.4.

### VF — Set-level checks and verify vocabulary

- **Changes:**
  - `Check` set-level variants (`contiguous_from_one`, `acyclic`, `sums_to`).
  - A row-level variant `presence_by_facet`: which attributes are present follows the facets of
    an enum column. Its member lists come from the declared facets at generation.
  - A verify SQL generator.
  - The generated `meta.member()`.
  - `verify/checks.py`.
  - `sql/verify/`.
- **Deletes:**
  - The near-copy `*ordinals_contiguous*` and `*acyclic*` checks, replaced by the set-level
    variants.
  - The four `*magnitude_matches_kind*` checks, replaced by `presence_by_facet`.
  - Text vocabulary literals.
- **Acceptance:**
  - `test_set_level_forms_generate_verify_sql`
  - `test_presence_by_facet_follows_declared_facets`: a new member with a facet is covered with
    no SQL edit.
  - `test_meta_member_refuses_undeclared_name`
  - Every generated check gets a fixture that violates it.
- **Docs:** `docs/meta-model.md` §3.4 and `docs/pipeline.md` §4.

### ST2 — Workspace and markers

- **Changes:**
  - `cli.py`.
  - The contexts: `acquire/runtime.py`, `staging/stage.py`, `canonical/environment.py` and the
    qualify `Context`.
  - `qualify/harness.py`, `reuse.py` and the fingerprint memo.
  - Every other library module that falls back to `config` today, which the AST acceptance
    forces: `resolve/decisions.py` (after ID2), `declaration/loader.py`,
    `staging/{reader,load,side}.py`, `qualify/{run,persist}.py`, `acquire/manifest.py`,
    `build/run.py` and `db.py`.
  - `tests/conftest.py` and the pytest `markers` setting.
- **Deletes:** `config` fallbacks in library code.
- **Acceptance:**
  - `test_harness_runs_in_injected_tree`
  - `test_library_code_has_no_config_fallback`, an AST test
  - `test_database_marker_set_by_fixture`
  - `test_fingerprint_once_per_declaration`
- **Docs:** `thermo-knowledge/README.md`.

### DX2 — Coded refusals

- **Changes:**
  - Map, canonical, resolve, build, staging and acquire errors carry codes.
  - `Blocked` maps codes, including `qualify/run.py`'s handling of `ReuseError`.
  - The remaining `match=` assertions are converted. This may be split into separate packets by
    test area ([decision #4](#decisions)).
- **Deletes:** message-text assertions.
- **Acceptance:** `test_stage_refusals_carry_codes` and `test_reuse_error_not_harness_failed`.
  Refusal tests assert through one helper that compares codes; an AST test refuses
  `pytest.raises(..., match=...)` on any exception type that carries a code.
- **Docs:** `docs/pipeline.md`, which states the refusal codes.

### BD1 — Foreign keys after load

- **Changes:** `generate/plan.py` and `generate/ir.py` (separate parts for tables and foreign
  keys); `build/plan.py` and `build/database.py`. The fingerprint and `schema.sql` are
  unchanged.
- **Deletes:** `SET CONSTRAINTS ALL DEFERRED`.
- **Acceptance:**
  - `test_fk_violation_after_load_rolls_back_build`
  - Build time and backend memory measured for both variants, on the current sources and on a
    synthetic 30 M-row load. The result is recorded in the [execution checkpoint](#execution-checkpoint).
- **Docs:** `docs/pipeline.md` §3, including step 5.

### BD2 — Bounded mapping

- **Changes:** `mapping/staged.py`, `canonical/writer.py` (flush), `canonical/store.py` and
  `mapping/runner.py`.
- **Deletes:** whole-table dict caches.
- **Acceptance:**
  - `test_janaf_tables_equal_with_bounded_writer`: equal by Arrow content, since row groups
    move.
  - JANAF mapping memory measured: it falls, and stays flat for a synthetic run ten times
    JANAF's size.
- **Docs:** `docs/pipeline.md` §2.3.

### DC20 — In-source ambiguity decision

- **Trigger:** chemicals is resolved as a carrier in W1-M2a.
- **Input:** ID1's report, with ambiguity split into in-source and cross-source.
- **Decision:** is disagreement among one source's identifiers ambiguity, or a declared
  precedence among identifier schemes?
- **Record:** in Plan 24's C20 row. If precedence is chosen, an engine change follows ID2.

### QF — Final qualification

Run once, after all functional packets:

- **Static checks:**
  - the whole `tk-test` against PostgreSQL;
  - `tk-types` at zero;
  - Ruff over the tree;
  - `tk generate --check`;
  - `tk survey --report --strict`.
- **The real-store pipeline** for the four mapped sources plus the corpus, from read to
  qualify:
  - build twice: identifiers and row counts must be identical;
  - run every verify check and every qualification case;
  - produce the residue report grouped by (reason, rule).
- **Measurements:** JANAF mapping memory (and at ten times its size), and the time of both
  build variants.

Then the Plan 24 rows C01–C22 are updated with evidence labels, and *Outcome* below is written.

## Sequencing

**Placement against Plan 24.**

| Before this Plan 24 step | These 24a packets land |
|---|---|
| W1-M1 JANAF equivalence assessments and formation reactions | TY, ID1, QL1, ST1, then RS1 |
| W1-M1 qualification cases (Cantera, CEA oracles) | QL1, QL2 |
| W1-M1 held-row decisions | DX1, then RS2 |
| W1-M2a mapping (chemicals) | EM (RS3), ID2 |
| W1 residue report | DX1 |
| W2 (including the integer slot type) | MM1, BD1, MM2, VF, ST2 (RS4), BD2 (RS5), DX2, then QF |

DC20 is decided during W1-M2a. QF runs after the last packet, before W2: BD2 runs in this pass
([decision #6](#decisions)). Chemicals' scale does not need BD2: at JANAF's measured
1.45 kB per canonical row, a few million rows fit the memory cap.

```
TY ─┬─> ID1 ──┬─> ST1 ──> RS1 ──> [W1-M1 equivalence]
    │          └─> ID2 ──> [W1-M2a]
    ├─> QL1 ───────> ST1
    │               ST1 ──> DX1 ──> RS2 ──> [W1-M1 held-row decisions, W1 residue]
    │                        └─> EM ──> RS3 ──> [W1-M2a] ──> DC20
    ├─> QL2
    └─> MM1 ─┬─> BD1 ──────────────────────────> [W2]
             └─> MM2 (after EM, QL1) ─┬─> VF ──> [W2]
                                      ├─> ST2 ──> DX2 ──> [W2]
                                      └─> BD2 ──> [W2]
QL1, QL2 ──> [W1-M1 qualification cases]
every packet ──> QF ──> [W2]
```

**Shared files.** Packets that edit the same file merge in this order:

| Files | Order |
|---|---|
| `canonical/writer.py` | TY → DX1 → EM → MM2 → BD2 |
| `mapping/context.py` | TY → ID1 → DX1 → EM |
| `mapping/spec.py` | ID1, then ID2 and EM in either order, never concurrently |
| `qualify/run.py` | TY → QL1 → ST1 → MM2 → ST2 → DX2 |
| `qualify/source.py` | QL1 → MM2 → ST2 |
| `declaration/{model,resolve}.py`, `generate/{plan,meta_tables,meta_rows,ir}.py` | TY → MM1 → BD1 → MM2 → VF (BD1 touches only `generate/{plan,ir}.py`, so it may also follow MM2) |
| `sql/generated/schema.sql`, regenerated with every declaration or projection change | ID1, DX1, MM1, MM2, VF: one at a time, each regenerating before the next starts |
| `resolve/decisions.py` | ID1 → ID2 → ST2 |
| `canonical/store.py` | ID1 → ST1 → BD2 |
| `mapping/runner.py` | ST1, then DX2 and BD2 in either order, never concurrently |
| `staging/stage.py` | ST1 → ST2 → DX2 |
| `model/*.toml` | one packet at a time, because each edit changes the fingerprint |

TY and DX2 each touch most test files, so they run alone. Packets may share
`docs/pipeline.md` as long as each edits only the sections it owns.

**Can run in parallel** (separate implementers):

- after TY: ID1, QL1, QL2 and MM1;
- EM with MM1;
- ID2 with DX1;
- VF with ST2.

**Real-store re-runs.** These run the pipeline from read to qualify, under
`scripts/memory-cap.sh`.

| Run | After | Acceptance |
|---|---|---|
| RS0 | TY | The content hashes of every staged and canonical table equal those recorded before `35ed87ee` (reuse keys change; contents do not). The CoolProp case deviations are unchanged |
| RS1 | ID1 and ST1 | **Changes on purpose.** C2H4O separated; one water species; the 55 duplicates gone; no rejects; Cantera keys renamed; new manifests. The resolution change diff explains every change by category |
| RS2 | DX1 | The declaration changes (new reason members, `held_row` columns), so the fingerprint changes and every carrier is re-mapped. The `tk`, `param`, `ev` and `prov` tables are identical; `held_row` changes as designed |
| RS3 | EM | The `tk`, `param`, `ev` and `prov` tables are identical to RS2. `qual.mapping_rule` changes where per-coefficient unit lines are replaced by the convention statement |
| RS4 | MM1, MM2, VF | Canonical tables identical. `schema.sql` identical after MM1 and MM2; `meta` rows differ only by MM2's reified state requirements and contract parameters; VF adds its generated function, checks and `presence_by_facet` constraints. Qualification runs re-made on purpose |
| RS5 | BD1, BD2 | Tables equal by Arrow content |

**No compatibility shims.** ST1 makes the existing canonical and qualification directories
unreadable, so RS1 re-makes them. Edits under `staging/`, `acquire/` or `config.py` make every
staged source stale, so they are batched and staging is re-read once per batch.

## Verification and acceptance

- **Per packet:** the targeted tests above, plus the deletion of what the packet replaces. A
  packet is not done while the replaced mechanism, its callers or its tests remain.
- **Store re-runs:** byte-identity or explained change at each re-run, as tabulated under
  [Sequencing](#sequencing).
- **Store runs RS0–RS5** are production runs of Plan 24's pipeline on the real store. Their
  comparisons gate the packets named in the re-run table; they are not qualification.
- **QF** is the only integrated qualification. Its results are reported with command, mode,
  scope and the zero baseline.
- **Planned, not run.** Nothing in this plan has been executed. The *Measured* figures in the
  baseline come from the review and from read-only investigations on 2026-09-30.
- **Contract pages** are updated in the packet that changes their contract. No static checks are
  re-run for documentation-only edits.

## Finding coverage

The status of each finding stays in [Plan 24](24-thermodynamic-knowledge-base.md#finding-dispositions).

| Finding | Primary packet | Contributing |
|---|---|---|
| C01 Formula scope joins substances | ID1 | ST1 (re-resolve safety) |
| C02 Build omits the resolution edge | ST1 | — |
| C03 Derived rejects as curated, positional data | ID1 | — |
| C04 No cross-carrier identity check | ID1 | — |
| C05 Set read model has no owner | MM2 | TY (closed types) |
| C06 Check-rule forms have no single owner | MM1 | TY |
| C07 No typed cause model | DX1, DX2 | — |
| C08 Form-level operations per mapping | EM | ID1 (composite key) |
| C09 Unbounded memory | BD1, BD2 | ST1 (resolution slice) |
| C10 Qualification library key | QL1 | ID1 (stable keys) |
| C11 Map key omits the source manifest | ST1 | — |
| C12 Relative bounds lost in grids | QL1 | — |
| C13 Quadrature status discarded | QL2 | — |
| C14 No type checker | TY | — |
| C15 Set-level checks copied as SQL | VF | MM1 |
| C16 `Field` conflates parameters | MM2 | — |
| C17 Binding channels unreconciled | MM2 | — |
| C18 IR constraint bodies as text | MM1 | — |
| C19 Class facets scattered | ID2 | — |
| C20 In-source identifier disagreement | DC20 | ID1 (report split) |
| C21 Verify vocabulary literals | VF | — |
| C22 Ambient defaults, markers, fingerprint | ST2 | — |

## Decisions

Decided by the maintainer on 2026-10-01; they were open questions when the plan was written.

1. **Conventional join (ID1).** A formula without a corpus structure keeps a cross-carrier join
   only where its composition fixes the constitution: one atom (ions included), any diatomic, or
   one non-hydrogen atom with only H, D or T; integral and not on a surface. Everything else is
   `formula_not_identifying`.
2. **Several-structure formulas (ID1).** A formula key scoped to the listing unit, so one source's
   phases of one substance still join each other; never across carriers.
3. **Typing and lint (TY).** pyrefly's default error set at zero errors and warnings over `src`,
   `tests`, `mappings` and `oracles`. The tree's ruff lint and format also reach zero, and the
   tree's format, lint and type checks are wired into the root `fmt` and `hygiene`, so the
   end-of-turn pipeline keeps them there.
4. **DX2 split.** The coordinator's call during execution: sub-commits by test area.
5. **DC20.** Still deferred: decided in W1-M2a from ID1's report.
6. **Where 24a closes.** BD2 runs in this pass, so QF runs before W2.

**Deliberate deviations from the sequencing above**, recorded before execution:

- The whole `tk-test` runs once after TY, after RS0 has made the store current: TY changes the
  entire tree, so the whole suite is its targeted check.
- ID1 and MM1 run in parallel although both touch the projection: MM1 leaves `schema.sql`
  unchanged, so only ID1 regenerates it, after MM1 is integrated.
- BD1 runs after ST1, not in parallel with it: both edit `build/plan.py` and `build/database.py`.
- Every packet after TY is implemented in a worktree and integrated into `main` as one commit,
  because once TY wires the tree into the end-of-turn hooks, a stop in `main` (from either
  session) formats the tree and may run the fixer over half-finished edits.
- Each store re-run starts from `tk read` whenever the packet edited `staging/`, `acquire/` or
  `config.py` (ST1, ST2, BD2, DX2), since the `read` key hashes them.

## Execution checkpoint

Records current state, decisions made, the measurements the packets name (BD1, BD2, MM1's dummy
variant, the RS comparisons) and the next dependency-ordered step. It is not a log of commands.

**2026-10-01, started.** Baseline: HEAD `33261206`, the tree unchanged since `35ed87ee`. The
store predates `ece085d7` and `35ed87ee`, so a fresh baseline is built at `33261206` before TY
(staged, mapped, resolved, built, verified and qualified) and snapshotted under
`.store/_baseline/33261206/`; RS0 and RS1 compare against it. Plan 24's W1 work waits for QF,
because later packets rewrite the mappings and the store is rebuilt at each RS.

Order: TY → (ID1 ‖ QL1 ‖ MM1, then QL2) → ST1 (with ID2 alongside) → RS1 → BD1 → DX1 → RS2 →
EM → RS3 → MM2 → (VF ‖ ST2) → RS4 → BD2 → RS5 → DX2 → QF.

## Outcome (recorded after implementation)

### What was built

### A mistake made and corrected

### Deviations from the plan, deliberate
