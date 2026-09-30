# Source survey records (v0)

This page specifies the construct inventory: a structured description of what each acquired
source actually contains and how it represents it. The survey is evidence for the domain model.
It is written before mappings exist, from the acquired bytes in the raw store, and it is what
the alignment step works from. It records what was found, not what the model should be.

One file per source: `survey/<source id>.toml`.

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
source = "coolprop"
pin = "ae81610e7d23"
surveyed = "2026-09-30"
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

### `[[payload]]`

| Key | Content |
|---|---|
| `paths` | glob(s) relative to the tree |
| `format` | the file format and its header or framing convention |
| `files`, `bytes` | measured counts |
| `records` | measured number of records (rows, objects, entries), and how it was counted |
| `reader` | how it can be parsed: the library's own loader (name the function and where it is defined in the tree), another library, or a plain parser |
| `read_by_source` | `all`, `partly` or `none`: whether the source's own code reads these files; say which parts are not read |
| `notes` | encoding quirks, headerless columns, duplicated or generated files |

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
| `fields` | array of `{ name, meaning, unit, shape, role }`, written inline or as `[[construct.fields]]` sub-tables. `unit` is exactly as the source states it, `"not stated"`, or `"not applicable"` for text. `shape` is one of scalar, enum, reference, function, list, table. `role` is `input` (an authored value the source's code uses), `computed` (stored but derived from other content, such as a state computed from the equation), `unread` (present but never read by the source's own code), `metadata`, or `not established` |
| `origin` | how the construct's content came to be in the source: `authored`, `transcribed` (from which publication or table), `generated` (by which tool or script, and whether its version is recorded), `computed` (from which model), or `not stated`. The value starts with the keyword; the detail follows in parentheses |
| `conventions` | reference states, composition basis, temperature scale, gas constant, sign and ordering conventions that the values assume |
| `absence` | what a missing entry or an explicit zero means in this source, and whether the source distinguishes them |
| `validity` | how the source states a range of applicability, if it does |
| `provenance` | how the source attributes the content (citation fields, DOIs, comments) |
| `count` | measured number of instances |
| `example` | one locator of a representative instance |
| `candidate` | the concept(s) of the model this would map to (see `docs/meta-model.md` and the plan's concept set), or `"none"` |
| `precision` | `exact`; `narrower` (the source construct is a special case of the candidate concept); `broader` (the source construct carries more than the candidate can hold); `close` (near, with a stated difference); or `unmapped` |
| `loss` | what a mapping to the candidate would lose or have to assume; empty when exact |

### `[[model_family]]`

| Key | Content |
|---|---|
| `name` | the library's name for the model |
| `locator` | the module, class or file that implements it |
| `class` | one of the representation classes in section 3 |
| `inputs` | natural variables and composition basis |
| `parameters` | the parameter names it needs, by subject (pure, pair, group, site, global) |
| `composes` | the components it embeds or accepts, as an array of `{ slot, accepts, default }`: the source's name for the slot, the abstract type or family it accepts, and the default the source uses when none is given; empty when it composes nothing |
| `variants` | flags or version numbers that select a different equation or different default data under the same name, with what each selects; empty when there are none |
| `equation_source` | the citation the code gives for the equations, if any |
| `closed_form` | `yes`, `implicit` (needs a solve), or `procedural` (behaviour lives in code: matching, characterisation, numerical integration) |
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
proposed key with ` (proposed)` appended), `offered` (`"yes"`, `"partial"` with the limitation stated in
`scope`, or `"no"` to record that a library one might expect to offer it does not), `scope` (model families or phase kinds), `locator`,
`evidence` (`documentation` or `source_inspected`).

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
