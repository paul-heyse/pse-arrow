# Source survey records (v0)

This page specifies the construct inventory: a structured description of what each acquired
source actually contains and how it represents it. The survey is evidence for the domain model.
It is written before mappings exist, from the acquired bytes in the raw store, and it is what
the alignment step works from. It records what was found, not what the model should be.

One file per source: `survey/<source id>.toml`. The loader (`tk survey --check`, section 5) accepts
exactly this specification: a key that is not listed, a required key that is missing and a value
outside a stated vocabulary are each reported, at the key; a record that deviates is left out of
what is loaded.
Every key below is required unless it is marked optional; a table that has no records is left out.

## 1. Rules

- **Read the pinned bytes.** Everything is taken from `.store/raw/<id>/<pin>/tree/`. Nothing comes
  from memory of the library, from its website, or from a newer version.
- **Every record has a locator** that a reader can open: a path relative to the source tree,
  with a JSON pointer, line range, column name or symbol where that narrows it.
- **Describe the source in its own terms first.** `name`, `fields` and `conventions` use the
  source's vocabulary and units. The model's vocabulary appears only in `candidate`.
- **Say what you did not establish.** A unit, convention or meaning that the source does not state
  is recorded as `"not stated"`; one inferred from code or context is marked `(inferred: <from
  what>)`. Guessing silently is the one thing a survey record must not do.
- **No third-party data.** A record may quote a field name, a header line or one example value.
  It never copies tables, parameter values in bulk, or passages of source code or documentation.
- **Counts are measured**, by listing or parsing the files, not estimated.

## 2. File shape

```toml
source = "coolprop"                 # the source id: the file name without .toml
pin = "ae81610e7d23"
additional_pins = { }               # optional, see below
surveyed = "2026-09-30"             # ISO date
summary = "One or two sentences: what this source contributes and what it does not."

[[payload]]          # one per group of files that share a format and meaning
[[construct]]        # one per native construct that carries thermodynamic meaning
[[model_family]]     # one per model family the library implements
[[convention]]       # one per convention the source assumes or states
[[selection]]        # one per rule by which the source chooses among competing data
[[discrepancy]]      # one per place where the source's documentation and its code or data disagree
[[capability]]       # one per calculation the library offers, or notably does not
[[question]]         # anything the survey could not settle
```

`pin` is the pin of the source the file is named for. A record that also reads another acquired
source states each of that source's pins in the optional `additional_pins` table, keyed by the other
source's id (`additional_pins = { sigma_profiles_pyscf_code = "764075efc394" }`); a record that
reads only one pin leaves the key out.

**Not recorded.** A record written before a key existed states `"not recorded"` for that key: the
survey does not say. It is a value the specification gives only where a key below lists it (`role`,
`origin`, `read_by_source`, `offered`); it never means "none" or "not applicable", and it is not
`"not stated"` (the source does not say) or `"not established"` (the survey looked and could not
settle it). `variants` is the one array that can be not recorded: its key is left out (section
`[[model_family]]`).

### `[[payload]]`

| Key | Content |
|---|---|
| `paths` | glob(s) relative to the tree |
| `format` | the file format and its header or framing convention |
| `files`, `bytes` | measured counts |
| `records` | measured number of records (rows, objects, entries), and how it was counted |
| `reader` | how it can be parsed: the library's own loader (name the function and where it is defined in the tree), another library, or a plain parser |
| `read_by_source` | `all`, `partly`, `none` or `not recorded`: whether the source's own code reads these files; say which parts are not read in `notes` |
| `notes` | encoding quirks, headerless columns, duplicated or generated files |
| `documents` | optional: for a payload made of individual documents, an array of `{ slug, identifier, title, pages, bytes }` tables (`pages` and `bytes` integers) |

`files` and `bytes` are integers. `records` and a construct's `count` are strings, because a
count often needs its unit of counting ("21 term objects in 20 fluids, 46 rows").

### `[[construct]]`

A construct is a native unit of meaning: a JSON object type, a table, a keyword block, a record
class, a parameter-file row type.

| Key | Content |
|---|---|
| `name` | the source's own name for it |
| `locator` | where it is defined or exemplified |
| `meaning` | what it represents, in the source's terms |
| `subject` | what it is about: one compound, an ordered pair, an unordered pair, a group, a group pair, a site pair, a phase with a constituent array, a reaction, a mixture, a global constant, a measurement point. State how the source keys it (name, CAS, index, formula) |
| `fields` | array of `{ name, meaning, unit, shape, role }`, written inline or as `[[construct.fields]]` sub-tables. `unit` is exactly as the source states it, `"not stated"`, or `"not applicable"` for text. `shape` is one of scalar, enum, reference, function, list, table. `role` is `input` (an authored value the source's code uses), `computed` (stored but derived from other content, such as a state computed from the equation), `unread` (present but never read by the source's own code), `metadata`, `not established`, or `not recorded` |
| `origin` | how the construct's content came to be in the source: `authored`, `transcribed` (from which publication or table), `generated` (by which tool or script, and whether its version is recorded), `computed` (from which model), `other` (none of these; say what it is), `not stated`, or `not recorded`. The value starts with the keyword as a whole word; the detail follows, in parentheses by preference. A value that combines two ("authored and transcribed") starts with the first |
| `conventions` | reference states, composition basis, temperature scale, gas constant, sign and ordering conventions that the values assume |
| `absence` | what a missing entry or an explicit zero means in this source, and whether the source distinguishes them |
| `validity` | how the source states a range of applicability, if it does |
| `provenance` | how the source attributes the content (citation fields, DOIs, comments) |
| `count` | measured number of instances |
| `example` | one locator of a representative instance |
| `candidate` | the concept(s) of the model this would map to (see `docs/meta-model.md` and the plan's concept set), or `"none"` |
| `precision` | `exact`; `narrower` (the source construct is a special case of the candidate concept); `broader` (the source construct carries more than the candidate can hold); `close` (near, with a stated difference); or `unmapped` |
| `loss` | what a mapping to the candidate would lose or have to assume; empty when exact |
| `values` | optional: for a construct that is an enumeration, an array of strings with its values exactly as the source writes them |

A construct's `name` is unique within its source: a disposition (section 4) is keyed by it.

### `[[model_family]]`

| Key | Content |
|---|---|
| `name` | the library's name for the model |
| `locator` | the module, class or file that implements it |
| `class` | one of the representation classes in section 3; `other` may be followed by what it is in parentheses (`other (numerical root finder)`) |
| `inputs` | natural variables and composition basis |
| `parameters` | the parameter names it needs, by subject (pure, pair, group, site, global) |
| `composes` | the components it embeds or accepts, as an array of `{ slot, accepts, default }`: the source's name for the slot, the abstract type or family it accepts, and the default the source uses when none is given; empty when it composes nothing |
| `variants` | optional: an array of `{ selector, selects }` tables, one per flag or version number that selects a different equation or different default data under the same name: `selector` is the flag or version as the source names it (empty when the variant has no name of its own), `selects` what it selects. `[]` when the survey looked and there are none; the key is left out when the record predates it (not recorded) |
| `equation_source` | the citation the code gives for the equations, if any |
| `closed_form` | `yes`, `implicit` (needs a solve), `procedural` (behaviour lives in code: matching, characterisation, numerical integration) or `not stated`. The value starts with the keyword as a whole word; any qualification follows |
| `data` | which `[[construct]]` supplies its parameters in this source, or `"none bundled"` |

### `[[convention]]`

`name`, `locator`, `statement` (what the source assumes or states), `scope` (which constructs it
applies to).

### `[[selection]]`

How the source chooses when several tables, methods or parameter sets could supply the same
thing: `name`, `locator` (where the rule is defined), `rule` (the order or criterion, in the
source's terms), `scope` (which properties or constructs it governs), `documented` (whether the
behaviour matches what the source documents; say how it differs if not).

### `[[discrepancy]]`

`locator`, `documented` (what the documentation, docstring or file name says), `actual` (what the
code evaluates or the data contain), `consequence` (which values or mappings it affects). Record
only what the pinned bytes show; do not judge which side is right.

### `[[capability]]`

`calculation` (a key from the `calculation` vocabulary in `model/qualification.toml`, or a new
proposed key with ` (proposed)` appended: exactly that suffix, after a key of lower-case words joined
by `_`; a key that is not in the vocabulary is proposed, and the loader refuses one without the
suffix; what a proposed key would mean is stated in `scope`), `offered` (`"yes"`, `"partial"` with
the limitation stated in `scope`, `"no"` to record that a library one might expect to offer it does
not, or `"not recorded"`; a string, never a boolean), `scope` (model families or phase kinds),
`locator`, `evidence` (`documentation` or `source_inspected`).

### `[[question]]`

`about`, `question`, `why_it_matters`.

## 3. Representation classes

| Key | Meaning |
|---|---|
| `explicit_correlation` | an empirical correlation of one property in its variables |
| `helmholtz_potential` | a fundamental Helmholtz energy function, pure or multifluid |
| `residual_or_excess_contribution` | a contribution added to an ideal or reference part |
| `group_contribution` | parameters indexed by structural groups |
| `association` | site-based association |
| `standard_state_species` | standard-state properties of a species as a function of temperature and pressure |
| `reaction_property` | an equilibrium constant or reaction property as a function of conditions |
| `electrolyte_interaction` | ion interaction, long-range and short-range electrolyte terms |
| `sublattice_gibbs` | sublattice and endmember Gibbs energy models |
| `pseudo_component_characterisation` | generation of components and parameters from assay data |
| `implicit_constitutive` | a constitutive relation defined by equations to be solved |
| `spatial_functional` | a free-energy functional of density fields |
| `adsorption` | isotherm and adsorbed-phase models |
| `transport` | transport-property models |
| `regional_piecewise` | formulations defined by regions, pieces or backward equations |
| `model_component` | a part that only has meaning inside another model: an alpha function, a mixing or combining rule, a volume translation, a reducing function |
| `wrapper` | a model that only selects, combines or delegates to other models |
| `regression` | evidence to parameters: fitting and its records |
| `kinetics` | reaction rate laws |
| `other` | none of the above; say what it is |

## 4. Dispositions

A survey record says what a source holds; it does not say what becomes of what a mapping would lose.
A **disposition** is that decision, recorded once per construct, so that every stated loss ends in
exactly one place: the declaration holds it, the mapping declares it, the declaration must change, or
it is outside the knowledge base.

### When a construct needs one

A construct needs a disposition when its `precision` is not `exact`, or when its `loss` is non-empty
after trimming and is not a bare "none" (a trailing full stop and letter case are ignored). A
construct that is exact and states no loss has none, and a disposition for it is refused. The
residue report (section 5) lists every construct that needs one and has none.

### Format

A file per source, `survey/dispositions/<source id>.toml`, with one entry per construct that needs
one. It holds only `[[disposition]]` tables, keyed by the construct's `name` exactly as the survey
writes it:

```toml
[[disposition]]
construct = "binary pair record (mixture_binary_pairs.json)"
disposition = "model_change"        # model_change | mapping_loss | out_of_scope | held_by_model
ref = "alignment-notes #4"          # what holds it, or what schedules the change
reason = "One sentence: what is kept, what is lost or assumed, or why it is out of scope."
```

| Key | Content |
|---|---|
| `construct` | the `name` of a construct of this source's survey, matched exactly; one disposition per construct |
| `disposition` | one of the four kinds below |
| `ref` | a reference, required except for `out_of_scope` (where it may be left empty); forms below |
| `reason` | required: one sentence saying which concept holds the construct, what is lost or assumed, what must change, or why it is out of scope |

| Disposition | Meaning | `reason` says |
|---|---|---|
| `held_by_model` | the current declaration holds the construct, and the stated loss does not apply or is only a difference of arrangement | which concept holds it |
| `mapping_loss` | the mapping will declare the loss or assumption | what is lost or assumed |
| `model_change` | the declaration must change | the change, and what schedules it (or that nothing does yet) |
| `out_of_scope` | outside the knowledge base | why |

### `ref`

| Form | Names | Checked |
|---|---|---|
| `kind:<name>`, `relation:<name>`, `form:<name>`, `enum:<name>` | a construct of the loaded declaration | the declaration has it |
| `mechanism:<name>` | something that is not a declared construct: a mapping rule, a pipeline stage, a convention | not checked |
| `alignment-notes #<n>` | item `n` of the numbered table of `docs/alignment-notes.md` | the item exists |
| `review F<nn>` | finding `F<nn>` of the design review of the core model (`docs/design_review/reviews/design_review_thermo-knowledge-core_2026-09-30.md`) | the finding has a heading there |
| `unscheduled` | only for `model_change`: the declaration must change and no alignment item or finding schedules it yet | |

A `model_change` is **scheduled** when its `ref` is an alignment-notes item or a review finding. The
report shows which scheduled change carries the most constructs and lists the model changes that
name none.

A missing row of an extensible vocabulary (an observable, a composition basis, an aggregation, a
naming scheme) is not a model change: the wave that maps the source declares it, and the construct
is `held_by_model` with the vocabulary's kind as `ref`. A missing quantity type, enumeration member,
kind, relation or attribute is a model change, because it changes the generated schema.

## 5. The loader and the residue report

`thermo_knowledge.survey_index` loads every `survey/*.toml` into typed `msgspec` structs, validates
each record key by key against this page and loads the dispositions. Every deviation is a
diagnostic with the file, the table, the record's name, the key, a stable code and a message; the
loader reports all of them and does not stop at the first. The calculation vocabulary and the model
constructs a `ref` names come from the loaded declaration (`thermo_knowledge.declaration`), so a
declaration that cannot be loaded is itself a diagnostic.

```text
tk survey --check                 validate the surveys and dispositions; exit 1 on any diagnostic
tk survey --report                write survey/residue-report.md and print a summary
tk survey --report --strict       exit 1 while any construct that needs a disposition has none
```

`--report` needs a clean load, so the report is never a partial count. The report is generated from
the surveys and dispositions alone: it carries no date and no environment, so regenerating it from
the same inputs is byte-identical, and its header says it is generated and from what. It has, for
each source, the constructs, their precision, how many need a disposition, how many have each kind
and how many have none; the totals; a table of the `model_change` dispositions by `ref`; and the
full list of constructs without a disposition with their precision and `loss`, grouped by source.
It is committed so that a change in the residue shows in review; edit the surveys or the
dispositions, never the report.
