# Registry, generation and the Python boundary

This page covers `pse-schema` (the registry), `pse-codegen`, `pse-py`, `pse-diagnostics`,
`pse-model` and the hand-written Python package `python/pse`. Contracts are owned by
[§4 and §6](../../authoritative_design/sections/schema-and-relations.md#section-4),
[§21.5](../../authoritative_design/sections/workflows-and-results.md#section-21-5) and
[§23](../../authoritative_design/sections/operations-and-validation.md#section-23). The baseline is
on the [entry page](README.md). Codegen is not disfavoured; each generation hop is judged on its
merits.

**System picture.** The registry catalog is the single declaration for relations, enums,
identities, documents and invariants. Codegen turns it into:
- Rust, rendered with `quote`/`syn`/`prettyplease`;
- Python attrs rows;
- msgspec document types, through a custom JSON-Schema emitter;
- JSON Schema;
- SurrealQL;
- Markdown.

`pse-py` uses PyO3 0.29 with `experimental-inspect`, pyo3-arrow (zero-copy capsule streams),
pyo3-async-runtimes and abi3. Stubs come from `pyo3_introspection`. Every error enum
implements `miette::Diagnostic` plus a typed diagnostic trait. Legitimately custom areas:
- framed identity;
- compatibility and migration semantics;
- the strict unknown-field policy;
- finite-float and null semantics at the NumPy boundary;
- drain-on-cancel.

<a id="cbo-040"></a>

## CBO-040 — Registry products baked as about 87k lines of Rust plus JSONL plus IPC

- **Operation and consumers.** `registry()` and `shared_registry()` produce the standard
  `Registry` at process start for every crate, every `pse-py` import and the codegen itself.
- **Current mechanism.**
  - `pse-codegen/src/codegen/rust/registry.rs` serializes the resolved products into Rust
    source using a pool of `#[inline(never)]` helpers.
  - It also emits a 16.7 MB JSONL file and fifteen `standard-N.arrow` IPC files.
  - The result is `pse-schema/src/generated/standard_registry.rs`: 87,267 lines, about 4.7 MB,
    ending in `hydrate()`.
  - Each `FieldContract` is rebuilt at runtime by parsing a JSONL slice.
  - The generated header explains that array-backed literals "exhausted the compilation memory
    cap". The output format follows a build limit.
  - `ResolvedRegistryProducts` is already a plain data struct.
- **Why it matters.**
  - The same products are serialized three ways.
  - The compile-memory workaround shapes the generator.
  - One embedded data authority would be smaller to compile and simpler to regenerate.
- **Patterns.** [LOP-X003](../assessment-patterns.md#lop-x003) (primary), LOP-06, LOP-07,
  LOP-11.
- **Footprint.**
  - Inspected: the generator, the head and tail of the output, and the products struct.
  - Similar but mostly typed output exists in `pse-relations/src/generated`.
- **Preservation constraints.**
  - Identical fingerprints and closure identities.
  - Hydration imports no runtime proofs.
  - Deterministic bytes for `codegen-check`.
- **Counterevidence.**
  - Static Rust gives type-checked constants.
  - It may have negligible startup cost.
  - A blob needs deterministic serde over Arrow types.
  - Hydration cost has not been measured.
- **Status.** Current mechanism; the alternative is proposed.
- **Research questions.**
  - Can the products be serialized deterministically into one embedded blob, for example with
    `postcard`, `rkyv`, `ciborium` or Arrow IPC for everything?
  - What does first-use construction cost?
- **Library assessment correction (2026-10-27).** The registry product types derive no serde; only Arrow `Field` is serialized today. A blob route must first give the model a serialization contract, including deterministic ordering for Arrow `Field` metadata, which is a `HashMap`. See [postcard](../libraries/postcard/README.md) and [ciborium](../libraries/ciborium/README.md).

<a id="cbo-041"></a>

## CBO-041 — Boundary documents transcoded through JSON, with checks in both languages

- **Operation and consumers.** Every `pse-py` call that returns or accepts a boundary document,
  and the Python wrappers around those calls.
- **Current mechanism.**
  - **Rust → Python.** `serde_json` bytes, then a Python-level call into `pse.codec.decode_json`
    (`pse-py/src/documents.rs`). That call walks `msgspec.inspect.type_info` to check version
    headers (`codec/__init__.py`).
  - **Header check, twice.** Rust repeats the header check (`check_required_header`, exposed as
    `_native.check_document_header`).
  - **Attrs rows.** These cross as attrs, then a cattrs unstructure, a finite-value check, the
    header check, `msgspec.convert`, JSON, and finally serde (`codec/_generated.py`).
  - **Per-call lookups.** Module imports and attribute lookups happen per call.
- **Why it matters.**
  - The same meaning — versioned header, finite floats, unknown-field refusal — is enforced in
    two languages.
  - Each document is transcoded several times.
- **Patterns.** LOP-06 (primary), LOP-03, LOP-02.
- **Footprint.**
  - Inspected: the boundary module, the codec and the wire module.
  - Covers about 70 boundary documents and every wrapper that uses them.
- **Preservation constraints.**
  - Refusal of unknown fields and non-finite values.
  - The exact wire form of identities.
  - Identical header diagnostics on both sides.
  - Streaming allowance limits.
  - JSON as the language-neutral contract (§21.5).
- **Counterevidence.**
  - JSON is stable and debuggable.
  - Payloads are modest.
  - msgspec is fast.
  - Nothing has been measured.
- **Status.** JSON as the wire is a current promise; unifying the checks is a proposed addition.
- **Research questions.**
  - Can the header check live on one side only, using msgspec `tag`/`Meta` or a serde tag?
  - Can `pythonize`, or msgspec conversion from attributes, avoid the double hop where no wire
    is needed?
- **Library assessment correction (2026-10-27).** The catalogue has 250 entries, not about 70: 155 FlatV1, 91 Native, 3 FlatV2 and 1 ValueV1. A probe of msgspec 0.21.1 found that a single tagged Struct does not require its tag, while a union of two or more tagged Structs does ([evidence](../../design_review/evidence/library-leverage-msgspec-tagged-decode-2026-10-27/README.md)). See [msgspec](../libraries/msgspec/README.md).

<a id="cbo-042"></a>

## CBO-042 — Document wire envelopes and directional schema surgery

- **Operation and consumers.** Document versioning, schema publication and the msgspec emitter.
- **Current mechanism.**
  - `pse-runtime/src/boundary_document_wire.rs` (678 lines) holds the following:
    - four wire shapes: `Native`, `FlatV1`, `FlatV2` and `ValueV1`;
    - `Versioned<T, N>` and a streaming `DeserializeSeed` header checker;
    - a manual `JsonSchema` implementation that rewrites schemas and examples. It comments on
      schemars 1.2.2 flatten behaviour.
  - `pse-codegen/src/codegen/documents.rs` splits input and output variants by hand
    (`directional_names`, `rename_output`). That split produces about 1.7k `*Output` names.
  - Python re-derives which field is the header.
- **Why it matters.**
  - It means fewer independent decisions.
  - schemars contract modes (`Contract::Serialize`/`Deserialize`) might produce directional
    schemas without the renaming pass.
- **Patterns.** LOP-10 (primary), LOP-02, LOP-11, [LOP-X003](../assessment-patterns.md#lop-x003).
- **Footprint.** The wire module, the renaming pass, and a table of about 70 entries.
- **Preservation constraints.**
  - Compatibility of existing flat wires.
  - Valid examples.
  - Domain fields named `version` stay legal.
- **Counterevidence.**
  - The separate header exists to protect domain `version` fields.
  - The four wires may mirror real historical contracts (ADR-0116, ADR-0151, not re-read in
    full).
- **Status.** Current mechanism.
- **Research questions.**
  - Do schemars 1.x contract modes remove the renaming pass?
  - Can a serde attribute or derive replace the hand-written header deserializer?
- **Library assessment correction (2026-10-27).** schemars contract modes are already in use. The "~1.7k `*Output` names" are 1,782 occurrences of 483 distinct classes.

<a id="cbo-043"></a>

## CBO-043 — Two Python model systems and a custom schema-to-msgspec emitter

- **Operation and consumers.** Typed Python contracts:
  - about 35k lines of msgspec document types;
  - about 12k lines of attrs relation rows;
  - cattrs and msgspec codecs.
- **Current mechanism.**
  - The `Emitter` in `codegen/documents.rs` (about 800 lines) maps JSON Schema to Python source.
  - `codegen/python/{mod,types,extensions}.rs` separately string-builds the attrs rows. They
    carry per-field validator chains and `_pse_equality_key`.
  - The codec runs two libraries with two strictness mechanisms: cattrs forbid-extra-keys and
    msgspec `forbid_unknown_fields`. Each has its own identity-type hooks.
  - ADR-0116 weighed `datamodel-code-generator` and chose byte-deterministic output.
- **Why it matters.** One registry contract family is expressed in two Python class systems,
  each with its own generator and codec.
- **Patterns.** [LOP-X003](../assessment-patterns.md#lop-x003) (primary), LOP-09, LOP-01.
- **Footprint.**
  - The codec and generated rows were inspected.
  - Outside tests, rows are used only by `codec` and `_modeling`.
- **Preservation constraints.**
  - Exact-type bool and finite-float validation.
  - Frozen values and equality keys.
  - The `typing.Any` ban.
  - Byte-deterministic generation.
- **Counterevidence.**
  - Arrow-shaped, nullable, extension-typed rows differ from documents.
  - msgspec has no exact-type validator.
  - The authoring JSON-Schema emitter is documented as intentionally outside schemars.
- **Status.** Current mechanism.
- **Research questions.**
  - Can msgspec `Struct` with `Meta` and `dec_hook` carry rows?
  - Does current `datamodel-code-generator` msgspec output meet the determinism, tagged-union
    and `StrEnum` needs?
  - Are rows needed outside the two edit documents?
- **Library assessment correction (2026-10-27).** Tested: a msgspec `Struct` validates nothing at construction (`DocV1(name=5, count='x')` succeeds), and `msgspec.convert` accepts `nan` and `inf` into `float` fields. Decode itself refuses non-finite JSON.

<a id="cbo-044"></a>

## CBO-044 — Python wrapper layer forwarding native handles accessor by accessor

- **Operation and consumers.** The public Python API: `LocalStudy`, `RunHandle`, `StudyReport`,
  `ModelingResult`, `TableStream` and related classes.
- **Current mechanism.**
  - There are about 405 `self._handle.…` forwards across:
    - `_modeling.py`
    - `_runs.py`
    - `_workflow.py`
    - `_project_operations_glue.py`
    - `diagnostics.py`
  - Some crossings are made item by item, for example
    `tuple(self._handle.attempt(i) for i in range(self._handle.attempt_count))`.
  - The wrappers exist mainly to back an import gate (`_build.py` and an ast-grep rule). Each
    accessor exists three times: as a hand-written PyO3 method (about 8k Rust lines), as a
    `_native.pyi` stub, and as a Python forward.
- **Why it matters.**
  - Three forms per accessor, with docstrings kept in step by hand.
  - Per-index loops cross the boundary once per item.
- **Patterns.** LOP-02 (primary), LOP-09, LOP-04.
- **Footprint.** Samples were read; there are 14 wrapper classes in `_modeling` alone.
- **Preservation constraints.**
  - The import gate.
  - A public naming surface separate from the extension.
  - Real behaviour in `TableStream` iteration, `RunResult` construction and identity tuple
    conversion.
- **Counterevidence.** A separate public surface can deliberately hide native names.
- **Status.** Current mechanism.
- **Research questions.**
  - Can the pyclass be the public type (with `#[pyclass(module = …)]`, introspected stubs and a
    single re-export)?
  - Are bulk getters available?
- **Library assessment correction (2026-10-27).** The 405 forwards count is exact. The stub generator strips docstrings on purpose (`xtask/src/codegen/python_stubs/validate.rs`), although pyo3-introspection would emit them.

<a id="cbo-045"></a>

## CBO-045 — Diagnostics implemented through a forwarding macro beside the miette derive

- **Operation and consumers.** Typed diagnostic codes, facts and related or source forwarding for
  every public error enum. The consumers are `DiagnosticReport` and Python.
- **Current mechanism.**
  - `impl_diagnostic!` (`pse-diagnostics/src/lib.rs`, 41 uses) implements `miette::Diagnostic`
    by hand alongside a `TypedDiagnostic` trait. Callers usually pass several `{ None }` blocks.
  - Other crates derive `thiserror::Error` together with `miette::Diagnostic`, for example
    `pse-operations/src/canonical.rs`.
  - `disposition()` maps `FailureClass` to `BoundaryClass` in a third place.
  - The `vocabulary!` macro is defined twice, once in `pse-diagnostics/src/vocabulary.rs` and
    once in `pse-vocabulary/src/lib.rs`. The copies differ only by a `description` parameter.
- **Why it matters.** Three styles serve one obligation. A variant's code and its annotations
  could sit together.
- **Patterns.** LOP-02, LOP-03, LOP-09.
- **Footprint.** 41 macro sites, both vocabulary macros, and the disposition map.
- **Preservation constraints.**
  - Codes stay the typed `DiagnosticCode`.
  - The facts and children projection.
  - Forwarding rules.
  - A §23.2 code on every public error.
- **Counterevidence.**
  - The derive cannot supply the typed facts.
  - The macro may be the thinnest way to cover both halves.
- **Status.** Current mechanism.
- **Research questions.**
  - Could `#[derive(miette::Diagnostic)]` plus a small local derive for `TypedDiagnostic` replace
    the macro?
  - Could one leaf crate own `vocabulary!`?
- **Library assessment correction (2026-10-27).** The two `vocabulary!` copies differ in more than `description`: the JsonSchema impl, String conversions, the `ALL` type and the serde rename route also differ. Derive sites such as `pse-operations/src/canonical.rs` already declare codes twice.

<a id="cbo-046"></a>

## CBO-046 — SurrealQL and Python emitted through string templates

- **Operation and consumers.** The generated SurrealQL and the Python contract modules.
- **Current mechanism.**
  - `codegen/surreal_execution.rs` chains `.replace("__INDEX_BYTES__", …)` over raw strings.
  - The `surreal*.rs` generators (about 4k lines) and `python/mod.rs` (about 60 `push_str` or
    `format!` sites) build text directly.
  - The Rust target uses `quote`/`syn` instead.
- **Why it matters.** Placeholder collisions and unescaped values go unchecked. A typed builder or
  template engine would make substitution explicit.
- **Patterns.** LOP-01, LOP-02, [LOP-X003](../assessment-patterns.md#lop-x003).
- **Footprint.** The two generator families named above.
- **Preservation constraints.** Byte-deterministic output.
- **Counterevidence.**
  - Embedded raw SurrealQL reads as written.
  - A template engine is a dependency added for a handful of placeholders.
  - This is a weak opportunity.
- **Status.** Current mechanism.
- **Research questions.**
  - Would `minijinja` or `askama` reduce risk, or only change the syntax?
  - Is there a SurrealQL AST or builder (`surrealdb-sql`, already pinned) that could render
    statements?
- **Library assessment correction (2026-10-27).** There are 18 placeholders, mostly integers. The riskier pieces are two post-generation textual rewrites over generated Python (`.replace("v.", "")`), and the guard-fragment and relation-id splices in `surreal_artifacts.rs`. `surrealdb-syn` already validates SurrealQL in xtask and `canonical_maintenance`.

