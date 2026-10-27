# Authoring language, physical semantics and identity

This page covers `pse-authoring`, `pse-modeling`, `pse-quantity`, `pse-vocabulary` and `pse-ids`.
Contracts are owned by
[§9](../../authoritative_design/sections/physical-semantics.md#section-9),
[§10–§12 and §22](../../authoritative_design/sections/models-and-composition.md#section-11), and
[§5](../../authoritative_design/sections/identity-and-publication.md#section-5). The baseline is on
the [entry page](README.md).

**System picture.** The codebase has two grammars on one lexical base.

- **Expression grammar (`dsl/`).** A token-slice recursive-descent parser with precedence
  climbing.
- **Declaration language (`language/`).** A second token cursor. It has a roughly 1k-line
  renderer and a parse→render→parse round-trip test.
- **winnow.** It handles character-level tokenizing with `LocatingSlice` offsets.
- **`ParseBudget`.** It bounds bytes and nesting depth.

Specialization, rewrite and fold operate on authored declarations and lexical environments,
before `pse-math`. They do not overlap Symbolica.

Units and dimensions are a registry-driven domain algebra: rational exponents, offsets and
physical-type contracts (ADR-0124). That is legitimately bespoke.

Identity uses blake3 derive-keys with length-prefixed framing. `pse-ids/src/document.rs` frames
any serde value without a hand-written field list. The `vocabulary!` macro derives spellings,
serde, `FromStr` and `ALL` from one table.

<a id="cbo-037"></a>

## CBO-037 — Grammar facts maintained in several hand-written forms

- **Operation and consumers.**
  - The operation: parse authored `.pse` sources into declaration rows with exact field spans,
    render rows back to canonical source for edits and round trips, walk the AST, and generate
    ASTs for property tests.
  - Consumers: `pse-admission` load, edit and reparse; workflow code; tests.
- **Current mechanism.**
  - `language/parser.rs` (about 3.7k lines) has a `Cursor` with `peek`, `eat`, `expect`, `word`
    and `error` over pre-tokenized input.
  - `dsl/parser.rs` repeats the same primitives and also builds the unit grammar by hand
    (`unit_product`, `unit_power`, `unit_exponent`).
  - The other forms are written separately:
    - renderers: `language/render.rs` and `dsl/render.rs`;
    - the walker: `dsl/walk.rs`, about 700 lines;
    - the AST generator: `dsl/arbitrary.rs`, about 550 lines;
    - field-span capture.
  - Keyword spellings are matched as strings in the parsers, the renderers and the checker,
    for example `intrinsic_callee` in `pse-modeling/src/expression.rs`.
  - A new construct touches about five forms. The round-trip test is the only thing keeping
    them in sync.
- **Why it matters.** A declarative grammar, a derived visitor and generator, or one keyword
  table would reduce both drift and the diff size per construct.
- **Patterns.** LOP-02 (primary; see the inverse-pair note in the
  [pattern output](../assessment-patterns.md#framings-considered-and-left-to-the-base-set)),
  LOP-09, LOP-01.
- **Footprint.**
  - Inspected: both parser prologues, the lexer and the grammar module.
  - Suspected: about 8k lines across parser, renderer, walker and generator code.
- **Preservation constraints.**
  - Exact byte spans per field.
  - `ParseBudget` and nesting limits.
  - Deterministic identity assignment during parsing.
  - Canonical render text.
  - `SourceSpan` errors keyed by document identity.
- **Counterevidence.**
  - The grammar is context-sensitive: kind-level binding, type-variable scope and named identity
    derivation all happen inside the parse.
  - A combinator or PEG rewrite could be a lateral move.
  - The renderer intentionally normalizes.
- **Status.** The parser is a current promise. Consolidation is a proposed addition.
- **Research questions.**
  - Can winnow's structural combinators over `Located` input replace the token cursors while
    preserving the span and error quality?
  - Do any grammar tools derive both parsing and printing?
  - Can the walker and the arbitrary generator be derived from the AST?

<a id="cbo-038"></a>

## CBO-038 — Author-facing diagnostics do not render source context

- **Operation and consumers.** Show an author where a parse or check error occurred. Consumers are
  CLI, MCP and Python users and the admission driver.
- **Current mechanism.**
  - `AuthoringError::Syntax` carries a `SourceSpan` and byte offset keyed by document identity.
  - `impl_diagnostic!` supplies a code, help text and related errors, and it forwards
    `source_code` and `labels` for wrapper types (`pse-diagnostics/src/lib.rs`).
  - A search of `pse-authoring` and `pse-modeling` found no `LabeledSpan`, `NamedSource` or
    `#[label]` use.
  - The only `labels` implementations are forwarders, in `pse-columnar/src/engine.rs` and the
    diagnostics macro.
- **Why it matters.** miette's source and label rendering is already a dependency and is already
  plumbed through the forwarding macro. It appears unused for the language's own spans, so
  authors get offsets instead of annotated excerpts.
- **Patterns.** LOP-03 (primary), LOP-12.
- **Footprint.** The authoring, check and specialize error types. More widely, every error that
  carries a `SourceSpan`.
- **Preservation constraints.**
  - Spans name documents by `SemanticId`, not by path.
  - Source text is not always available at the error site.
  - Codes follow blueprint §23.2.
  - The diagnostic wire and Arrow forms must not carry source text.
- **Counterevidence.**
  - The driver that holds the source bytes may render diagnostics elsewhere. The Python and CLI
    renderers were not inspected.
- **Status.** Proposed addition.
- **Research questions.**
  - Where would a `SemanticId`→text source map attach?
  - Can `LabeledSpan`s be derived from `SourceSpan` at the driver boundary?

<a id="cbo-039"></a>

## CBO-039 — Hand-written identity framing beside the serde-derived framer

- **Operation and consumers.**
  - The operation: content identity for resolved records, schemes, units, selections,
    envelopes and derived items.
  - Consumers: publication, replay and caches.
- **Current mechanism.**
  - `pse-ids/src/document.rs` implements a serde `Serializer` (`Framer`) so that "a field added
    to a settings type changes identity without a hand-written field list".
  - About 30 hand-written `fn frame(&self, hash)` implementations still enumerate fields. They
    are in:
    - `pse-model/src/lib.rs` (12);
    - `pse-modeling/src/derived.rs` (10);
    - `pse-modeling/src/{contextual,envelope,applicability,scientific_selection}.rs`;
    - `resolved/support.rs`;
    - `pse-quantity/src/{scheme,unit,resolved}.rs`.
  - Many more `frame` implementations are generated by the registry. Those are a derived form
    and are not part of this opportunity.
- **Why it matters.** A field added to a hand-framed type can silently fail to affect identity.
  This is the exact failure the serde framer was written to prevent.
- **Patterns.** LOP-02 (primary), LOP-03.
- **Footprint.**
  - Hand implementations were counted by search.
  - `scheme.rs` and `resolved/framing.rs` were read.
- **Preservation constraints.**
  - Frozen bytes: `pse-ids` golden vectors and every published identity. A switch must emulate
    the existing bytes or deliberately migrate them.
  - Intentional exclusions, canonical ordering and first-use ordinals must stay expressible.
  - Graph canonicalization (`resolved/framing.rs`) is legitimately bespoke.
- **Counterevidence.**
  - Many implementations deliberately frame a subset of fields or a canonical form.
  - Some variant tags differ from the serde spellings.
- **Status.** Proposed addition.
- **Research questions.**
  - Which hand implementations are full-field frames, and which are selective or canonicalized?
  - Could a derive with skip and order attributes, or serde attributes on the existing framer,
    cover the selective forms?

## Examined and not retained

- **Physical-type inference and schemes.** `pse-quantity/src/{infer,scheme}.rs` were considered
  as a unification-library candidate. The semantics are richer than generic unification:
  registered operation rules, delta types and contracts. The solver loop was not read. No
  opportunity is recorded until a later pass establishes actual unification work.
- **Package data formats.** Reference packages combine `.pse`, an 11k-line YAML file, parquet and
  TOML, decoded in `pse-admission` and projected into generated Rust. Whether any registry
  record is translated more than once was not established. A later pass should start from the
  question: how many times is a record translated between YAML, structs, Arrow and generated
  Rust?
