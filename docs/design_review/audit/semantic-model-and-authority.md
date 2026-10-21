# Semantic model and authority

Findings whose structural cause is how a meaning is represented, who owns it, and whether
consumers use that owner or reconstruct the meaning themselves. The baseline, standard
versions and coverage limits are in the [collection README](README.md). Citations are
`path:line` at source commit `a791c533e`.

## <a id="da-001"></a>DA-001 — The specialized model is carried in the authoring syntax tree, with resolved identity encoded in synthesized names

**Principles:** AP-04, DP-02, DP-04, DP-08, DP-09; AP-07 (H10, H12) for the cost consequence.

**Diagnosis.** Specialization (`pse-modeling`) hands the compiler `SpecializedModel`
equations, member expressions and contributions as `pse_authoring::dsl::Expr` /
`Equation`, which is the same type used for authored syntax. A resolved scalar reference is
represented as a single-segment path named `s_<32 hex>`. Consumers recover its identity by
stripping the prefix and parsing the hex. Dispatch grouping renames these strings to
`local_<n>`, the compiler renames them again to `input_<n>` / `member_<n>`, and body
identity frames the *rendered text* (`render_expr`) together with path strings. The type
environment is keyed by rendered names. As a result, at each checking site the whole symbol
table is rebuilt as a fresh `BTreeMap<String, Type>`.

The distinctions between a resolved symbol reference, a lexical name, a binder and a
function parameter are therefore naming conventions rather than types. Dependency
structure is recovered by walking names: for example, the compiler's petgraph edges are
built by parsing names. Identity correctness depends on rendering staying canonical and
injective, so a presentation-level change (renderer, unit spelling) is also an identity
change. There is no separate resolved stage between authored syntax and library
mathematics. The declared design chooses this deliberately ("no intermediate IR is
produced or stored", §7.1), and that choice is the subject of this finding under DP-08.

**Consequences.**
- A change to rendering or naming silently re-keys bodies.
- An authored binder spelled `s_<hex>` would share a namespace with resolved references. The
  grammar reserves no prefix. Misresolution is a hypothesis; misses observed in code refuse
  rather than mis-bind.
- Each contribution × coordinate re-renders every symbol into a fresh map
  (`specialize.rs:2576-2582`, called inside the per-coordinate loop at `:2038-2047`). The
  same pattern recurs for annotations, forms and objectives, so specialization work grows
  with occurrences × symbols. See also [DA-029](execution-fit.md#da-029).

**Evidence.**
- Encoding and parse-back:
  - `crates/pse-modeling/src/specialize.rs:2833-2855` (`symbol_name`, `symbol_reference` → `strip_prefix("s_")`)
  - `specialize.rs:1134-1140`
  - `specialize/forms.rs:1231-1237`
  - `crates/pse-compiler/src/workspace/modeling/executable.rs:183-190`
  - `executable.rs:532-541` (dependency edges from names)
  - `executable/implicit.rs:69, 1673, 1737-1738`
- Representation: `specialize.rs:167-192`.
- Name-keyed environments: `crates/pse-modeling/src/types/environment.rs:8-16`, `specialize/value.rs:203`, `specialize/rewrite.rs:393-413`.
- An initial-state equation located by `render_path(p) == state_name`: `executable.rs:682`.
- Identity over rendered text:
  - `specialize/group.rs:16-32`, `:123`, `:150-155`
  - `executable/grouped.rs:563-590`, `:988-1023`
- About 91 production `render_*` call sites across `pse-modeling` and `pse-compiler`.

**Counterevidence and limits.**
- Lowering does not re-parse text (`Row` states this; fields are parsed once).
- Member identities themselves are well layered: `member_id` frames instance, declaration
  and coordinate identity (`specialize.rs:2815-2826`).
- Parse/render round trips are property-tested in `pse-authoring`.
- Compiler lowering refuses a missing physical admission (`crates/pse-compiler/src/typed_math.rs:689-712`).
- The cost growth is static structural evidence; it has not been measured.

**Evidence strength.** *Implemented*, from source reading by two independent slice reviews
and confirmed by the coordinator at the cited sites. The cost magnitude is a hypothesis.

**Related.** [DA-002](#da-002) (derived identities), [DA-027](execution-fit.md#da-027)
(bodies re-identified), [DA-029](execution-fit.md#da-029).

**Solution lead (proposal).** A resolved expression IR with typed symbol-reference and
binder variants, an identity-keyed type environment, and body identity framed over that IR.

## <a id="da-002"></a>DA-002 — Derived identities are minted from call-site strings, and other crates re-derive them to find relationships

**Principles:** DP-04, DP-01, DP-07; G1, G2.

**Diagnosis.** `pse_ids::named_id(parent, "<literal>")` frames its input under the
catalogued `Frame::NamedV1`. That frame is documented for named-policy reference entities
"whose qualified names are the public contract" (`crates/pse-ids/src/derive.rs:934-953`).
It is reused as a general derived-identity mint at roughly 169 production call sites
(about 85 of them in `pse-modeling`) for derived row, parameter and output identities. The
meaning that distinguishes each derivation is an uncatalogued preimage literal at the call
site, and some preimages format a Rust `Debug` value.

The frame context is catalogued, so §5.3's "no context is a literal" holds in letter. But
derivation meanings are introduced without the new frame variant that §5.3 asks for
("a new meaning or derivation version is a new variant").

`SpecializedModel` does not record the relationships these identities stand for (for
example closure → conservation row). Consumers in other crates therefore recompute the
derivation to find the related entity. When the minted string and a re-derivation differ,
lookups return `None`. At least one consumer then skips the case silently.

**Evidence.**
- Frame meaning: `crates/pse-ids/src/derive.rs:98-99`, `:934-953`.
- Conservation row:
  - Minted at `crates/pse-modeling/src/specialize.rs:2712-2713`.
  - Re-derived at:
    - `specialize/contextual.rs:78-84`
    - `crates/pse-compiler/src/workspace/modeling/executable.rs:475`, `:1180` (a per-output scan)
    - `crates/pse-runtime/src/workflow/modeling/cases.rs:2660`, `:2695-2700`. The last is a `let … else { continue; }`, which drops the numerical requirement without a diagnostic. Coordinator-confirmed.
- `conditional-boundary-value`: minted at `crates/pse-compiler/src/workspace/modeling/conditional.rs:306`, re-derived at `crates/pse-runtime/src/workflow/strategies/conditional.rs:1261`.
- `reconstructed-original-bound`: minted independently at `crates/pse-runtime/src/math/solves/derived.rs:1256` and `crates/pse-backend-native/src/derived.rs:369`.
- `Debug` in preimages: `executable/grouped.rs:393`, `:398-401`.

**Counterevidence.**
- `member_id` uses a catalogued frame.
- `ModelingOutput::row_id` centralizes output derivations, and the runtime calls it
  instead of re-deriving (`grouped.rs:378-413`; `crates/pse-runtime/src/workflow/objectives.rs:150`).
- The `continue` at `cases.rs:2697-2700` is also the legitimate path when a conservation
  row is simply not a numerical target. A derivation mismatch and a legitimate absence
  therefore look the same, which is why the silent-loss consequence remains a hypothesis.

**Evidence strength.** *Implemented* for the duplication. That a mint change silently loses
conservation requirements is a hypothesis. A probe would change the mint string and run
`just unit-package pse-runtime conservation`.

**Related.** [DA-001](#da-001).

## <a id="da-003"></a>DA-003 — Modeling expressions are stored as plain text, so each consumer restates which grammar and role a field carries

**Principles:** DP-02, DP-01, DP-16 (§E extension locality); G2.

**Diagnosis.** Every expression, predicate, equation, path and number field of
`authored.modeling_declarations` is declared as plain `Utf8`. The registry declares a
`pse.expr_dsl` extension for "authored expression text", but no relation field uses it.
Because the registry cannot say which grammar and role a field carries, that knowledge is
maintained by hand in several inventories:

1. The occurrence collector, which assigns grammar and role per field.
2. A per-arm text inventory for demand-driven source selection. It is reconciled with
   inventory 1 by comparing *text values*, and falls back lossily on parse failure.
3. About 153 production `expression_at(…, "arm.field", …)` lookups using about 70 role
   strings.
4. Two definitions of "which arms bind argument names": the checker's, and a wider one in
   canonical source selection, under a trait documented as "one lexical rule".
5. A generator special case that treats every String field of this relation as maskable.

One field can also carry two grammars. `realization.argument` is checked as a DSL
expression, but for `derived_big_m` and `hull` it is read with `str::parse::<f64>`.

**Consequence.** Adding an IR field or arm means editing the registry, the collector, the
selector inventory, the binding rule and each consumer. That is not one authoritative
addition. If the two binding rules diverge, selection and checking can resolve the same
name differently.

**Evidence.**
- `crates/pse-schema/src/catalog/modeling.rs:10-12`, `:546-571`, `:1293-1310`.
- The unused extension: `crates/pse-schema/src/model/extension.rs:151-153`, `:242`. No
  `pse.expr_dsl` use appears in `docs/generated/relations/`.
- Collector: `crates/pse-modeling/src/expression/occurrences.rs:801-880`.
- Selector: `crates/pse-modeling/src/selected_source.rs:127-138`, `:336-357`, `:409-440`, `:595-640`.
- Binding rules: `crates/pse-modeling/src/check.rs:2122-2139` versus `crates/pse-runtime/src/workflow/modeling/canonical.rs:1458-1497`.
- Generator: `crates/pse-codegen/src/codegen/rust/semantic.rs:307-316`, `:329-331`.
- Dual grammar: `occurrences.rs:837-844` versus `crates/pse-modeling/src/specialize/forms.rs:199-205`, `:237-252`.

**Counterevidence.**
- Parser field spans are derived from the generated struct through serde
  (`crates/pse-authoring/src/language/field_spans.rs:1-44`).
- Several former text vocabularies are now typed enums (§6.15.1).

**Evidence strength.** *Implemented* for the inventories. Behavioural divergence of the two
binding rules is a hypothesis. A probe would author a `response` whose argument shadows an
outer member.

## <a id="da-004"></a>DA-004 — Decision vocabularies that cross the Rust–Python boundary are spelled by hand in each layer, and their projections disagree

**Principles:** DP-01, DP-02, DP-24, AP-05; G1, G2.

**Diagnosis.** The declared design says every decision vocabulary crossing a crate, store
or language boundary is a registry enumeration with one Rust type, and that closed getters
expose generated enums (architecture-overview §0.6 "Shared vocabulary"; §4.1; §21.5).
Several vocabularies that reach Python are instead hand-spelled in each layer:

- **Scientific artifact kind** is spelled at least five ways:
  - A 3-member Rust enum, whose serde form is snake_case while `name()` is kebab-case
    (`crates/pse-runtime/src/workflow/artifacts.rs:199-217`).
  - A literal `"seed"` writer (`durable.rs:3360`).
  - A 5-member reader allow-list (`artifacts.rs:620-631`).
  - A PyO3 parser (`crates/pse-py/src/workflow.rs:1155-1162`).
  - A Python `Literal` (`python/pse/_artifacts.py:12`), with `ScientificArtifact.kind`
    returning `str`.

  The registry has machinery for scientific product completeness, `artifact_profiles`
  (`crates/pse-schema/src/catalog/products.rs:12-58`), but scientific artifacts bypass it.
  Their kinds and member relations are therefore hand lists ([DA-032](#da-032)), not
  profile declarations.
- **Terminal-class filters** arrive from Python as `Sequence[str]`
  (`python/pse/_workflow.py:252`) and are parsed by hand in the binding
  (`crates/pse-py/src/workflow.rs:1345-1353`). No Python enum is generated for them.
- **Artifact owner kind** is accepted from Python as a free `&str`. The Python
  `Literal["user","project","audit"]` (`python/pse/_artifacts.py:13`) is not enforced at
  runtime. The store-side value set is covered by [DA-007](structure-and-invariants.md#da-007).
- **Live simulation events.** `kind` and `termination` are hand strings invented in the
  adapter (`crates/pse-py/src/workflow.rs:2806-2817`, `:2880-2884`). The sibling
  `ModelingTrajectory.termination` is a generated enum.
- **Dynamics settings.** `method`, `sensitivity` and `trial_failures` cross as strings
  ([DA-005](#da-005)).

**Consequence.**
- Python callers cannot discover legal values.
- Projections of one vocabulary disagree on membership and spelling.
- Adding a member is a multi-language hand edit with no regeneration check.

**Counterevidence.**
- Source-owned vocabularies derive registry members from `ALL` / `as_str`
  (`crates/pse-schema/src/catalog/enums_platform.rs:62-100`).
- Some registry enums are parsed at the PyO3 boundary (`crates/pse-py/src/workflow/strategies.rs:30-31`).

**Evidence strength.** *Implemented*. Three independent slice reviews converged on it.

**Related.** [DA-007](structure-and-invariants.md#da-007) (store-column value sets),
[DA-032](#da-032) (relation names as strings).

## <a id="da-005"></a>DA-005 — Physical meaning does not survive the Python boundary round trip on side routes and result reads

**Principles:** PS-01, PS-12, DP-02, DP-04; G2.

**Diagnosis.** The primary typed route says outright that "bare numbers are not input
values" (`BindingAssignment` / `BindingQuantity`, which carry quantity and unit ids;
`python/pse/contracts/documents/__init__.py:2860-2883`). Around it, both directions lose
physical meaning.

**Inputs accept bare floats or solver-ordered vectors.**
- `PreparedOperation.with_primal_start(Mapping[SemanticId, float])` (`python/pse/_runs.py:235`).
- Block-initialization stages (`python/pse/_modeling.py:926-930`).
- `diagnose_samples` (`_modeling.py:722-727`).
- The `minimum` / `maximum` result filters (`python/pse/_workflow.py:311`).
- `SimulationSettings` projects `pse_backend_native::dynamics::Profile` one-to-one
  (`crates/pse-py/src/workflow.rs:2320-2400`; `crates/pse-backend-native/src/dynamics.rs:1724-1800`):
  - `atol` is positional "in normalized state order".
  - `parameter_scales` and `out_atol` are positional too.
  - Times are seconds by convention.
  - `method`, `sensitivity` and `trial_failures` are strings.

**Outputs carry opaque physical ids that Python cannot resolve.**
- `ModelingInitialization.committed_values() -> dict[SemanticId, float]`.
- `NativeAttempt` mirrors the backend `SolveReport` (`primal() -> list[tuple[str, float]]`,
  `native_status: str`; `crates/pse-py/src/workflow/strategies.rs:396-433`).
- `runtime.solve_variables` and `solve_constraints` hold a bare `Float64` value beside
  *optional*, untyped `quantity_id` / `unit_id` columns
  (`crates/pse-schema/src/catalog/native_math.rs:1175-1205`).
- The registry declares no identity domain for units, quantity types, reference states,
  symbols or rows. These are typed only inside `pse-quantity`, so about 88 physical
  references in generated rows are raw `SemanticId`.
- The public Python surface has no resolver from a quantity or unit id to unit, basis or
  reference state (`python/pse/_workflow.py:454-462`; `python/pse/_native.pyi:721-723`).

**Identities lose their kind.**
- Generated documents type about 262 identity fields as hex-pattern `str`, while relation
  rows type the same fields as NewTypes (for example `python/pse/contracts/authored.py:1639`).
  That contradicts §21.5 ("Entity identities are generated NewTypes").
- Canonical keys (run, attempt, problem, revision, analysis, artifact, owner) are bare
  `str` (`python/pse/_workflow.py:158-356`).
- The same study concept is typed in one place and not in another: `study(StudyId)` versus
  `forget_study_results(study: str)`.
- Identities of different kinds can therefore be swapped without rejection.

Minor execution note: each `SemanticId` hex encode or decode is a native call
(`python/pse/contracts/values.py:14-44`). Bulk maps such as `with_primal_start` cross once
as a dict, followed by per-element hex conversion and re-parse in Rust
(`crates/pse-py/src/workflow.rs:2663-2674`). This is linear and on side routes, and is not
raised as a separate execution-fit finding.

**Consequence.**
- A start or initialization value given in °C instead of K, or bar instead of Pa, is
  accepted silently and can steer the solve to another root. It does not reach published
  results unqualified: results are re-qualified, and initialization commits only start
  values.
- Per-state tolerances require knowing the solver's state order, which is the kind of
  solver-index knowledge PS-12 rules out.
- A result's physical meaning is recoverable only through ids the Python user cannot
  interpret.

**Counterevidence.**
- Result rows carry `target_id`, `source_id`, `path`, `quantity_id` and `unit_id`
  (`python/pse/contracts/runtime.py:2707-2719`), and in-process producers always set them.
- `solve_bound` uses the typed overlay.
- Dynamics `numerics` are keyed by id.
- The gap is in side routes and in resolution, not in the main result path.

**Evidence strength.** *Interface-checked*. The Python negative (no unit resolver) was
searched in `python/pse` (excluding generated contracts), `_native.pyi` and
`crates/pse-py/src`.

**Related.** [DA-004](#da-004), [DA-011](structure-and-invariants.md#da-011).

## <a id="da-006"></a>DA-006 — Each analysis mode lowers authored numerical annotations with its own rules

**Principles:** DP-01, AP-04, AP-03, PS-11, PS-07; G1, G2.

**Diagnosis.** Lowering authored numerical annotations (start, bounds, nominal) into
variable states and numerical requirements has no single owner. The compiler emits every
annotation as a hint without deduplicating
(`crates/pse-compiler/src/workspace/modeling/executable.rs:858-875`). Three workflow
consumers then interpret the hints with different rules:

| Mode | Duplicate annotation on one target and kind | Nominal target classes | Bounds become |
|---|---|---|---|
| Steady (`workflow/modeling/cases.rs:1324-1415`) | Refused ("ambiguous numerical annotations") | Variable, Row, Observable | Solver variable bounds |
| Nested implicit (`modeling/implicit.rs:88-121`) | Collected into a `BTreeMap`, so the last one silently wins (coordinator-confirmed) | Variable or Row only | The unknown's interval |
| Dynamic (`modeling/dynamics.rs:2068-2205`, `:3225-3385`) | Refused ("competing dynamic bounds") | Only targets in its own list; others silently dropped (`:2199`) | IDAS sign constraints |

**Consequence.**
- One authored annotation can be refused in one mode, silently resolved in another and
  dropped in a third.
- Changing what an annotation means is a coordinated edit in three modules.
- "One model serves every analysis mode" holds for structure but not for numerical policy
  such as nominal scaling.

**Counterevidence.**
- The numerical-policy resolver refuses equal-priority conflicts, which may catch some
  duplicates downstream.
- Differences between modes may be intentional, but they are not recorded anywhere.

**Evidence strength.** *Implemented*. Reachability of the divergence from authored input
is a hypothesis. A probe would author duplicate `nominal` / `bounds` annotations in an
implicit block, a steady case and a dynamic fixture.

**Related.** [DA-031](#da-031).

## <a id="da-031"></a>DA-031 — Steady case resolution restates compiler eligibility and admission rules in orchestration

**Principles:** DP-01, AP-03; G1.

**Diagnosis.** `resolve_prepared_case_with_current_view` (about 700 lines in
`crates/pse-runtime/src/workflow/modeling/cases.rs`) chooses between the original-equation
view and the reduced view.
- **Supplier predicate.** Its predicate (`cases.rs:1071-1088`) partly restates the
  compiler's promotion predicate (`crates/pse-compiler/src/workspace/modeling/executable.rs:1634-1645`;
  `conditional.rs:39-40`), but omits conditions the compiler also requires. They agree today
  only because `original_equations()` returns `None` when nothing is promoted.
- **Requirement filtering.** The same function filters numerical requirements to exclude
  inner unknowns (`cases.rs:1119-1135`).
- **Sensitivity admission.** It admits sensitivity parameters itself (`:1199-1230`),
  alongside `SensitivityRequest::admit`.

**Consequence.**
- If the compiler relaxes or tightens its promotion rule, orchestration silently keeps the
  old view choice.
- A sensitivity-admission change has two places to update.

**Evidence strength.** *Implemented* for the predicate duplication.

**Related.** [DA-006](#da-006).

## <a id="da-032"></a>DA-032 — Relation membership and handling policies name relations by literal strings and bypass the generated relation handles

**Principles:** DP-01, DP-02, AP-05, DP-16 (§E); G2.

**Diagnosis.** Several hand-maintained policies decide which relations belong to an
artifact kind, which are scalar-indexed, which get serde and how analysis rows are
dispatched. They key these decisions on qualified relation-name strings resolved at run
time:
- per-kind artifact member lists (`crates/pse-runtime/src/workflow/artifacts.rs:786-826`);
- analysis dispatch with a catch-all arm (`crates/pse-runtime/src/workflow/analyses.rs:279-299`, `:319-324`);
- the scalar-indexed relation list (`crates/pse-operations/src/canonical_execution.rs:2345-2356`);
- indexable fields and owner columns (`crates/pse-runtime/src/workflow/result_projection.rs:28-140`);
- the generator's serde list (`crates/pse-codegen/src/codegen/rust/semantic.rs:136-162`).

Generated typed handles exist (`RELATION_ID`, `NAME`;
`crates/pse-relations/src/generated/runtime/solve_variables.rs:10-17`), and some consumers
use them (`crates/pse-runtime/src/workflow/study_demand.rs:106`, `:347`).

**Consequence.** A typo or rename does not fail to compile; it silently changes
classification.
- `registry.relation(name).is_some_and(...)` is false on a bad name, so the row-coverage
  check is skipped without any error (`canonical_execution.rs:2345-2356`).
- Unknown analysis rows fall through to the catch-all arm.
- Adding a relation to a product or index edits several string lists in different crates.

**Counterevidence.** Names are resolved through the registry rather than used as free text,
and the analyses catch-all may be an intended default.

**Evidence strength.** *Interface-checked*. The silent-skip consequence is traced in source
and not *Tested*.

**Related.** [DA-004](#da-004), [DA-026](execution-fit.md#da-026).
